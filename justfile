##
# Development Recipes
#
# This requires Just: https://github.com/casey/just
#
# To see possible tasks, run:
# just --list
##

pkg_id      := "channelz"
pkg_name    := "ChannelZ"
pkg_dir1    := justfile_directory() + "/channelz"

cargo_dir   := "/tmp/" + pkg_id + "-cargo"
data_dir    := "/tmp/bench-data"
release_dir := justfile_directory() + "/release"



# Compare performance with native gzip/brotli.
bench: _bench-init build
	#!/usr/bin/env bash

	clear

	fyi notice "Pausing 5s before next run."
	just _bench-reset
	sleep 5s

	fyi print -p Method "(Find + Parallel + Brotli) + (Find + Parallel + Gzip)"
	time just _bench-fp
	echo ""

	fyi notice "Pausing 5s before next run."
	just _bench-reset
	sleep 5s

	fyi print -p Method "ChannelZ"
	time "{{ cargo_dir }}/release/channelz" "{{ data_dir }}/test/flags"


# Self benchmark.
bench-self: _bench-init build
	#!/usr/bin/env bash

	clear

	just _bench-reset
	fyi notice "Pausing 5s before running."
	sleep 5s

	"{{ cargo_dir }}/release/channelz" -p "{{ data_dir }}/test"


# Build Release!
@build:
	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo build \
		--release \
		--target-dir "{{ cargo_dir }}"


# Build Debian package!
@build-deb: build-man
	[ $( command -v cargo-deb ) ] || cargo install cargo-deb

	# cargo-deb doesn't support target_dir flags yet.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	mv "{{ cargo_dir }}" "{{ justfile_directory() }}/target"

	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo-deb \
		--no-build \
		-p {{ pkg_id }} \
		-o "{{ justfile_directory() }}/release"

	just _fix-chown "{{ release_dir }}"
	mv "{{ justfile_directory() }}/target" "{{ cargo_dir }}"


# Build Man.
@build-man: build
	# Pre-clean.
	rm "{{ release_dir }}/man"/*

	# Use help2man to make a crappy MAN page.
	help2man -o "{{ release_dir }}/man/{{ pkg_id }}.1" \
		-N "{{ cargo_dir }}/release/{{ pkg_id }}"

	# Strip some ugly out.
	sd '{{ pkg_name }} [0-9.]+\nBlobfolio, LLC. <hello@blobfolio.com>\n' \
		'' \
		"{{ release_dir }}/man/{{ pkg_id }}.1"

	# Gzip it and reset ownership.
	gzip -k -f -9 "{{ release_dir }}/man/{{ pkg_id }}.1"
	just _fix-chown "{{ release_dir }}/man"


# Check Release!
@check:
	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo check \
		--bin "{{ pkg_id }}" \
		--release \
		--target-dir "{{ cargo_dir }}"


@clean:
	# Most things go here.
	[ ! -d "{{ cargo_dir }}" ] || rm -rf "{{ cargo_dir }}"
	[ ! -d "{{ pgo_dir }}" ] || rm -rf "{{ pgo_dir }}"

	# But some Cargo apps place shit in subdirectories even if
	# they place *other* shit in the designated target dir. Haha.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	[ ! -d "{{ pkg_dir1 }}/target" ] || rm -rf "{{ pkg_dir1 }}/target"


# Clippy.
@clippy:
	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo clippy \
		--release \
		--all-features \
		--target-dir "{{ cargo_dir }}"


# Get/Set version.
version:
	#!/usr/bin/env bash

	# Current version.
	_ver1="$( toml get "{{ pkg_dir1 }}/Cargo.toml" package.version | \
		sed 's/"//g' )"

	# Find out if we want to bump it.
	_ver2="$( whiptail --inputbox "Set {{ pkg_name }} version:" --title "Release Version" 0 0 "$_ver1" 3>&1 1>&2 2>&3 )"

	exitstatus=$?
	if [ $exitstatus != 0 ] || [ "$_ver1" = "$_ver2" ]; then
		exit 0
	fi

	fyi success "Setting version to $_ver2."

	# Set the release version!
	just _version "{{ pkg_dir1 }}" "$_ver2"


# Set version for real.
@_version DIR VER:
	[ -f "{{ DIR }}/Cargo.toml" ] || exit 1

	# Set the release version!
	toml set "{{ DIR }}/Cargo.toml" package.version "{{ VER }}" > /tmp/Cargo.toml
	just _fix-chown "/tmp/Cargo.toml"
	mv "/tmp/Cargo.toml" "{{ DIR }}/Cargo.toml"


# Benchmark Find + Parallel
@_bench-fp:
	find "{{ data_dir }}/test/flags" \
		\( -iname '*.css' -o -iname '*.htm' -o -iname '*.html' -o -iname '*.ico' -o -iname '*.js' -o -iname '*.json' -o -iname '*.mjs' -o -iname '*.svg' -o -iname '*.txt' -o -iname '*.xhtm' -o -iname '*.xhtml' -o -iname '*.xml' -o -iname '*.xsl' \) \
		-type f \
		-print0 | \
		parallel -0 brotli -q 11

	find "{{ data_dir }}/test/flags" \
		\( -iname '*.css' -o -iname '*.htm' -o -iname '*.html' -o -iname '*.ico' -o -iname '*.js' -o -iname '*.json' -o -iname '*.mjs' -o -iname '*.svg' -o -iname '*.txt' -o -iname '*.xhtm' -o -iname '*.xhtml' -o -iname '*.xml' -o -iname '*.xsl' \) \
		-type f \
		-print0 | \
		parallel -0 gzip -k -9


# Benchmark data.
_bench-init:
	#!/usr/bin/env bash

	[ -d "{{ data_dir }}" ] || mkdir "{{ data_dir }}"

	if [ ! -f "{{ data_dir }}/list.csv" ]; then
		wget -q -O "{{ data_dir }}/list.csv" "https://moz.com/top-500/download/?table=top500Domains"
		sed -i 1d "{{ data_dir }}/list.csv"
	fi

	if [ ! -d "{{ data_dir }}/raw" ]; then
		fyi info "Gathering Top 500 Sites."
		mkdir -p "{{ data_dir }}/raw/html"
		echo "" > "{{ data_dir }}/raw.txt"

		# Fake a user agent.
		_user="\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/74.0.3729.169 Safari/537.36\""

		# Download everything.
		cat "{{ data_dir }}/list.csv" | rargs \
			-p '^"(?P<id>\d+)","(?P<url>[^"]+)"' \
			-j 50 \
			wget -q -T5 -t1 -U "$_user" -O "{{ data_dir }}/raw/html/{url}.html" "https://{url}"

		fyi info "Grabbing SVG samples."
		git clone -q https://github.com/hjnilsson/country-flags.git "{{ data_dir }}/raw/flags"

		fyi info "Grabbing JS samples."
		git clone -q https://github.com/lodash/lodash.git "{{ data_dir }}/raw/lodash"

		find "{{ data_dir }}/raw" \( -iname "*.br" -o -iname "*.gz" \) -type f -delete
	fi

	exit 0


# Reset benchmarks.
@_bench-reset: _bench-init
	[ ! -d "{{ data_dir }}/test" ] || rm -rf "{{ data_dir }}/test"
	cp -aR "{{ data_dir }}/raw" "{{ data_dir }}/test"


# Init dependencies.
@_init:
	[ ! -f "{{ justfile_directory() }}/Cargo.lock" ] || rm "{{ justfile_directory() }}/Cargo.lock"
	cargo update


# Fix file/directory permissions.
@_fix-chmod PATH:
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type f -exec chmod 0644 {} +
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type d -exec chmod 0755 {} +


# Fix file/directory ownership.
@_fix-chown PATH:
	[ ! -e "{{ PATH }}" ] || chown -R --reference="{{ justfile() }}" "{{ PATH }}"
