##
# Development Recipes
#
# This requires Just: https://github.com/casey/just
#
# To see possible tasks, run:
# just --list
##

cargo_dir     := "/tmp/channelz-cargo"
data_dir      := "/tmp/bench-data"
debian_dir    := "/tmp/channelz-release/channelz"
release_dir   := justfile_directory() + "/release"

build_ver     := "1"


# Benchmark Directory Comparisons.
bench: _bench-init build
	#!/usr/bin/env bash

	clear

	fyi notice "Pausing 5s before next run."
	just _bench_reset
	sleep 5s

	fyi print -p Method "(Find + Parallel + Brotli) + (Find + Parallel + Gzip)"
	time just _bench-fp
	echo ""

	fyi notice "Pausing 5s before next run."
	just _bench_reset
	sleep 5s

	fyi print -p Method "ChannelZ"
	time "{{ cargo_dir }}/release/channelz" "{{ data_dir }}/test"


# Benchmark Self.
just bench-self: _bench-init build
	#!/usr/bin/env bash

	clear

	just _bench_reset
	fyi notice "Pausing 5s before running."
	sleep 5s

	fyi print -p Method "ChannelZ w/ Progress"
	"{{ cargo_dir }}/release/channelz" -p "{{ data_dir }}/test"


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
		mkdir "{{ data_dir }}/raw"
		echo "" > "{{ data_dir }}/raw.txt"

		# Fake a user agent.
		_user="\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/74.0.3729.169 Safari/537.36\""

		# Download everything.
		cat "{{ data_dir }}/list.csv" | rargs \
			-p '^"(?P<id>\d+)","(?P<url>[^"]+)"' \
			-j 50 \
			wget -q -T5 -t1 -U "$_user" -O "{{ data_dir }}/raw/{url}.html" "https://{url}"

		fyi info "Grabbing SVG samples."
		git clone -q https://github.com/hjnilsson/country-flags.git "{{ data_dir }}/raw/flags"

		fyi info "Grabbing JS samples."
		git clone -q https://github.com/lodash/lodash.git "{{ data_dir }}/raw/lodash"

		find "{{ data_dir }}/raw" \( -iname "*.br" -o -iname "*.gz" \) -type f -delete
	fi

	exit 0


# Reset benchmarks.
@_bench_reset: _bench-init
	[ ! -d "{{ data_dir }}/test" ] || rm -rf "{{ data_dir }}/test"
	cp -aR "{{ data_dir }}/raw" "{{ data_dir }}/test"


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


# Build Release!
@build:
	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo build \
		--release \
		--target-dir "{{ cargo_dir }}"


# Build Debian Package.
@build-debian: build
	[ ! -e "{{ debian_dir }}" ] || rm -rf "{{ debian_dir }}"
	mkdir -p "{{ debian_dir }}/DEBIAN"
	mkdir -p "{{ debian_dir }}/etc/bash_completion.d"
	mkdir -p "{{ debian_dir }}/usr/bin"
	mkdir -p "{{ debian_dir }}/usr/share/man/man1"

	# Steal the version from Cargo.toml really quick.
	cat "{{ justfile_directory() }}/channelz/Cargo.toml" | grep version | head -n 1 | sed 's/[^0-9\.]//g' > "/tmp/VERSION"

	# Copy the application.
	cp -a "{{ cargo_dir }}/release/channelz" "{{ debian_dir }}/usr/bin"
	chmod 755 "{{ debian_dir }}/usr/bin/channelz"
	strip "{{ debian_dir }}/usr/bin/channelz"

	# Generate completions.
	cp -a "{{ cargo_dir }}/channelz.bash" "{{ debian_dir }}/etc/bash_completion.d"
	chmod 644 "{{ debian_dir }}/etc/bash_completion.d/channelz.bash"

	# Set up the control file.
	cp -a "{{ release_dir }}/skel/control" "{{ debian_dir }}/DEBIAN"
	sed -i "s/VERSION/$( cat "/tmp/VERSION" )-{{ build_ver }}/g" "{{ debian_dir }}/DEBIAN/control"
	sed -i "s/SIZE/$( du -scb "{{ debian_dir }}/usr" | tail -n 1 | awk '{print $1}' )/g" "{{ debian_dir }}/DEBIAN/control"

	# Generate the manual.
	just _build-man

	# Build the Debian package.
	chown -R root:root "{{ debian_dir }}"
	cd "$( dirname "{{ debian_dir }}" )" && dpkg-deb --build channelz
	chown --reference="{{ justfile() }}" "$( dirname "{{ debian_dir }}" )/channelz.deb"

	# And a touch of clean-up.
	mv "$( dirname "{{ debian_dir }}" )/channelz.deb" "{{ release_dir }}/channelz_$( cat "/tmp/VERSION" )-{{ build_ver }}.deb"
	rm -rf "/tmp/VERSION" "{{ debian_dir }}"


# Build MAN page.
@_build-man:
	# Most of it can come straight from the help screen.
	help2man -N \
		"{{ debian_dir }}/usr/bin/channelz" > "{{ debian_dir }}/usr/share/man/man1/channelz.1"

	# Fix a few formatting quirks.
	sed -i -e ':a' -e 'N' -e '$!ba' -Ee \
		"s#ChannelZ [0-9\.]+[\n]Blobfolio, LLC. <hello@blobfolio.com>[\n]##g" \
		"{{ debian_dir }}/usr/share/man/man1/channelz.1"

	# Wrap up by gzipping to save some space.
	gzip -9 "{{ debian_dir }}/usr/share/man/man1/channelz.1"


# Get/Set ChannelZ version.
version:
	#!/usr/bin/env bash

	# Current version.
	_ver1="$( cat "{{ justfile_directory() }}/channelz/Cargo.toml" | \
		grep version | \
		head -n 1 | \
		sed 's/[^0-9\.]//g' )"

	# Find out if we want to bump it.
	_ver2="$( whiptail --inputbox "Set ChannelZ version:" --title "Release Version" 0 0 "$_ver1" 3>&1 1>&2 2>&3 )"

	exitstatus=$?
	if [ $exitstatus != 0 ] || [ "$_ver1" = "$_ver2" ]; then
		exit 0
	fi

	fyi success "Setting plugin version to $_ver2."

	# Set the release version!
	just _version "{{ justfile_directory() }}/channelz/Cargo.toml" "$_ver2" >/dev/null 2>&1


# Truly set version.
_version TOML VER:
	#!/usr/bin/env php
	<?php
	if (! is_file("{{ TOML }}") || ! preg_match('/^\d+.\d+.\d+$/', "{{ VER }}")) {
		exit(1);
	}

	$content = file_get_contents("{{ TOML }}");
	$content = explode("\n", $content);
	$section = null;

	foreach ($content as $k=>$v) {
		if (\preg_match('/^\[[^\]]+\]$/', $v)) {
			$section = $v;
			continue;
		}
		elseif ('[package]' === $section && 0 === \strpos($v, 'version')) {
			$content[$k] = \sprintf(
				'version = "%s"',
				"{{ VER }}"
			);
			break;
		}
	}

	$content = implode("\n", $content);
	file_put_contents("{{ TOML }}", $content);


# Init dependencies.
@_init:
	echo ""
