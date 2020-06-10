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
cargo_bin   := cargo_dir + "/x86_64-unknown-linux-gnu/release/" + pkg_id
data_dir    := "/tmp/bench-data"
pgo_dir     := "/tmp/pgo-data"
release_dir := justfile_directory() + "/release"

rustflags   := "-Clinker-plugin-lto -Clinker=clang-9 -Clink-args=-fuse-ld=lld-9 -C link-arg=-s"



# Benchmark Rust functions.
bench BENCH="" FILTER="":
	#!/usr/bin/env bash

	clear

	find "{{ justfile_directory() }}/test/assets" \( -iname "*.br" -o -iname "*.gz" \) -type f -delete

	if [ -z "{{ BENCH }}" ]; then
		cargo bench \
			-q \
			--workspace \
			--all-features \
			--target x86_64-unknown-linux-gnu \
			--target-dir "{{ cargo_dir }}" -- "{{ FILTER }}"
	else
		cargo bench \
			-q \
			--bench "{{ BENCH }}" \
			--workspace \
			--all-features \
			--target x86_64-unknown-linux-gnu \
			--target-dir "{{ cargo_dir }}" -- "{{ FILTER }}"
	fi

	find "{{ justfile_directory() }}/test/assets" \( -iname "*.br" -o -iname "*.gz" \) -type f -delete

	exit 0


# Self benchmark.
bench-self: _bench-init build
	#!/usr/bin/env bash

	clear

	just _bench-reset
	fyi notice "Pausing 5s before running."
	sleep 5s

	"{{ cargo_bin }}" -p "{{ data_dir }}/test"


# Build Release!
@build:
	# First let's build the Rust bit.
	RUSTFLAGS="{{ rustflags }}" cargo build \
		--bin "{{ pkg_id }}" \
		--release \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Build Debian package!
@build-deb: build-man
	[ $( command -v cargo-deb ) ] || cargo install cargo-deb

	# cargo-deb doesn't support target_dir flags yet.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	mv "{{ cargo_dir }}" "{{ justfile_directory() }}/target"

	# First let's build the Rust bit.
	cargo-deb \
		--no-build \
		-p {{ pkg_id }} \
		-o "{{ justfile_directory() }}/release"

	just _fix-chown "{{ release_dir }}"
	mv "{{ justfile_directory() }}/target" "{{ cargo_dir }}"


# Build Man.
@build-man: build-pgo
	# Pre-clean.
	find "{{ release_dir }}/man" -type f -delete

	# Use help2man to make a crappy MAN page.
	help2man -o "{{ release_dir }}/man/{{ pkg_id }}.1" \
		-N "{{ cargo_bin }}"

	# Strip some ugly out.
	sd '{{ pkg_name }} [0-9.]+\nBlobfolio, LLC. <hello@blobfolio.com>\n' \
		'' \
		"{{ release_dir }}/man/{{ pkg_id }}.1"

	# Gzip it and reset ownership.
	gzip -k -f -9 "{{ release_dir }}/man/{{ pkg_id }}.1"
	just _fix-chown "{{ release_dir }}/man"


# Build PGO.
@build-pgo: clean _bench-init
	# First let's build the Rust bit.
	RUSTFLAGS="{{ rustflags }} -Cprofile-generate={{ pgo_dir }}" \
		cargo build \
			--bin "{{ pkg_id }}" \
			--release \
			--target x86_64-unknown-linux-gnu \
			--target-dir "{{ cargo_dir }}"

	clear

	# Instrument a few tests.
	just _bench-reset
	"{{ cargo_bin }}" "{{ data_dir }}/test"

	# Do them again with the UI.
	just _bench-reset
	"{{ cargo_bin }}" -p "{{ data_dir }}/test"

	# Lots of paths and files!.
	just _bench-reset
	cp -aR "{{ justfile_directory() }}/test/assets" "{{ data_dir }}"
	"{{ cargo_bin }}" -p "{{ data_dir }}/assets" "{{ data_dir }}/test" "{{ data_dir }}/test2"
	rm -rf "{{ data_dir }}/assets"

	# Do a file.
	just _bench-reset
	echo "{{ data_dir }}/test/css" > "/tmp/pgo-list.txt"
	echo "{{ data_dir }}/test/js" >> "/tmp/pgo-list.txt"
	echo "{{ data_dir }}/test/page" >> "/tmp/pgo-list.txt"
	echo "" >> "/tmp/pgo-list.txt"
	"{{ cargo_bin }}" -p -l "/tmp/pgo-list.txt"
	rm "/tmp/pgo-list.txt"

	# A bunk path.
	"{{ cargo_bin }}" "/nowhere/blankety" || true

	# And some CLI screens.
	"{{ cargo_bin }}" -V
	"{{ cargo_bin }}" -h

	clear

	# Merge the data back in.
	llvm-profdata-9 \
		merge -o "{{ pgo_dir }}/merged.profdata" "{{ pgo_dir }}"

	RUSTFLAGS="{{ rustflags }} -Cprofile-use={{ pgo_dir }}/merged.profdata" \
		cargo build \
			--bin "{{ pkg_id }}" \
			--release \
			--target x86_64-unknown-linux-gnu \
			--target-dir "{{ cargo_dir }}"


# Check Release!
@check:
	# First let's build the Rust bit.
	RUSTFLAGS="{{ rustflags }}" cargo check \
		--workspace \
		--release \
		--target x86_64-unknown-linux-gnu \
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
	clear
	RUSTFLAGS="{{ rustflags }}" cargo clippy \
		--workspace \
		--release \
		--all-features \
		--target x86_64-unknown-linux-gnu \
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
	find "{{ data_dir }}/test" \
		\( -iname '*.css' -o -iname '*.htm' -o -iname '*.html' -o -iname '*.ico' -o -iname '*.js' -o -iname '*.json' -o -iname '*.mjs' -o -iname '*.svg' -o -iname '*.txt' -o -iname '*.xhtm' -o -iname '*.xhtml' -o -iname '*.xml' -o -iname '*.xsl' \) \
		-type f \
		-print0 | \
		parallel -0 brotli -q 11

	find "{{ data_dir }}/test" \
		\( -iname '*.css' -o -iname '*.htm' -o -iname '*.html' -o -iname '*.ico' -o -iname '*.js' -o -iname '*.json' -o -iname '*.mjs' -o -iname '*.svg' -o -iname '*.txt' -o -iname '*.xhtm' -o -iname '*.xhtml' -o -iname '*.xml' -o -iname '*.xsl' \) \
		-type f \
		-print0 | \
		parallel -0 gzip -k -9


# Benchmark data.
_bench-init:
	#!/usr/bin/env bash

	[ -d "{{ data_dir }}" ] || mkdir "{{ data_dir }}"

	# The Vue web site has a nice mixture of encodable assets.
	if [ ! -d "{{ data_dir }}/raw" ]; then
		git clone \
			--single-branch \
			-b master \
			https://github.com/vuejs/vuejs.org.git \
			"{{ data_dir }}/raw"

		cd "{{ data_dir }}/raw"
		npm i
		npm run -s build
	fi


# Reset benchmarks.
@_bench-reset: _bench-init
	[ ! -d "{{ data_dir }}/test" ] || rm -rf "{{ data_dir }}/test"
	[ ! -d "{{ data_dir }}/test2" ] || rm -rf "{{ data_dir }}/test2"

	cp -aR "{{ data_dir }}/raw" "{{ data_dir }}/test2"
	cp -aR "{{ data_dir }}/raw/public" "{{ data_dir }}/test"


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
