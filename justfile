##
# Development Recipes
#
# This justfile is intended to be run from inside a Docker sandbox:
# https://github.com/Blobfolio/righteous-sandbox
#
# docker run \
#	--rm \
#	-v "{{ invocation_directory() }}":/share \
#	-it \
#	--name "righteous_sandbox" \
#	"righteous/sandbox:debian"
#
# Alternatively, you can just run cargo commands the usual way and ignore these
# recipes.
##

pkg_id      := "channelz"
pkg_name    := "ChannelZ"

cargo_dir   := "/tmp/" + pkg_id + "-cargo"
cargo_bin   := cargo_dir + "/release/" + pkg_id
data_dir    := "/tmp/bench-data"
doc_dir     := justfile_directory() + "/doc"
release_dir := justfile_directory() + "/release"

export RUSTFLAGS := "-Ctarget-cpu=x86-64-v3 -Cllvm-args=--cost-kind=throughput -Clinker-plugin-lto -Clink-arg=-fuse-ld=lld"
export CC := "clang"
export CXX := "clang++"
export CFLAGS := "-Wall -Wextra -flto -march=x86-64-v3"
export CXXFLAGS := "-Wall -Wextra -flto -march=x86-64-v3"



# Build Release!
@build:
	# First let's build the Rust bit.
	cargo build \
		--bin "{{ pkg_id }}" \
		--release \
		--target-dir "{{ cargo_dir }}"


# Build Debian package!
@build-deb: clean credits build
	# cargo-deb doesn't support target_dir flags yet.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	mv "{{ cargo_dir }}" "{{ justfile_directory() }}/target"

	# Build the deb.
	cargo-deb \
		--no-build \
		-p {{ pkg_id }} \
		-o "{{ release_dir }}"

	just _fix-chown "{{ release_dir }}"
	mv "{{ justfile_directory() }}/target" "{{ cargo_dir }}"


@clean:
	# Most things go here.
	[ ! -d "{{ cargo_dir }}" ] || rm -rf "{{ cargo_dir }}"

	# But some Cargo apps place shit in subdirectories even if
	# they place *other* shit in the designated target dir. Haha.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"


# Clippy.
@clippy:
	clear
	cargo clippy \
		--release \
		--all-features \
		--target-dir "{{ cargo_dir }}"


# Generate CREDITS.
@credits:
	# Do completions/man.
	cargo bashman -m "{{ justfile_directory() }}/Cargo.toml"
	just _fix-chown "{{ justfile_directory() }}/CREDITS.md"


# Build Docs.
@doc:
	# Make the docs.
	cargo doc \
		--release \
		--no-deps \
		--target-dir "{{ cargo_dir }}"

	# Move the docs and clean up ownership.
	[ ! -d "{{ doc_dir }}" ] || rm -rf "{{ doc_dir }}"
	mv "{{ cargo_dir }}/doc" "{{ justfile_directory() }}"
	just _fix-chown "{{ doc_dir }}"

	exit 0


# Test Run.
@run +ARGS:
	cargo run \
		--bin "{{ pkg_id }}" \
		--release \
		--target-dir "{{ cargo_dir }}" \
		-- {{ ARGS }}


# Unit tests!
@test:
	clear
	cargo test \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--release \
		--target-dir "{{ cargo_dir }}"


# Get/Set version.
version:
	#!/usr/bin/env bash

	# Current version.
	_ver1="$( toml get "{{ justfile_directory() }}/Cargo.toml" package.version | \
		sed 's/"//g' )"

	# Find out if we want to bump it.
	_ver2="$( whiptail --inputbox "Set {{ pkg_name }} version:" --title "Release Version" 0 0 "$_ver1" 3>&1 1>&2 2>&3 )"

	exitstatus=$?
	if [ $exitstatus != 0 ] || [ "$_ver1" = "$_ver2" ]; then
		exit 0
	fi

	fyi success "Setting version to $_ver2."

	# Set the release version!
	just _version "{{ justfile_directory() }}" "$_ver2"


# Set version for real.
@_version DIR VER:
	[ -f "{{ DIR }}/Cargo.toml" ] || exit 1

	# Set the release version!
	toml set "{{ DIR }}/Cargo.toml" package.version "{{ VER }}" > /tmp/Cargo.toml
	just _fix-chown "/tmp/Cargo.toml"
	mv "/tmp/Cargo.toml" "{{ DIR }}/Cargo.toml"


# Benchmark data.
_bench-init:
	#!/usr/bin/env bash

	# Make sure the data dir is set up.
	[ -d "{{ data_dir }}" ] || mkdir "{{ data_dir }}"

	# Pull some test assets.
	if [ ! -d "{{ data_dir }}/raw" ]; then
		mkdir "{{ data_dir }}/raw"

		# WP Core.
		mkdir "{{ data_dir }}/raw/wp-core"
		cd "{{ data_dir }}/raw/wp-core" && wp core download --allow-root
		cd "{{ justfile_directory() }}"

		# Build site docs.
		just doc
		cp -aR "{{ doc_dir }}" "{{ data_dir }}/raw/"
	fi

	# Fix permissions.
	just _fix-chown "{{ data_dir }}"


# Reset benchmarks.
@_bench-reset: _bench-init
	[ ! -d "{{ data_dir }}/test" ] || rm -rf "{{ data_dir }}/test"
	cp -aR "{{ data_dir }}/raw" "{{ data_dir }}/test"


# Fix file/directory permissions.
@_fix-chmod PATH:
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type f -exec chmod 0644 {} +
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type d -exec chmod 0755 {} +


# Fix file/directory ownership.
@_fix-chown PATH:
	[ ! -e "{{ PATH }}" ] || chown -R --reference="{{ justfile() }}" "{{ PATH }}"
