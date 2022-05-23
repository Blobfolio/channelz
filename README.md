# ChannelZ

[![Build Status](https://github.com/Blobfolio/channelz/workflows/Build/badge.svg)](https://github.com/Blobfolio/channelz/actions)
[![Dependency Status](https://deps.rs/repo/github/blobfolio/channelz/status.svg)](https://deps.rs/repo/github/blobfolio/channelz)

ChannelZ is a CLI tool for x86-64 Linux machines that simplifies the common task of encoding static web assets with Gzip and Brotli for production environments.



## Features

 * `gzip` and `brotli` are compiled into `channelz`; their binaries do not need to be separately installed;
 * The maximum compression settings are applied; the end results will often be smaller than running native `gzip` or `brotli` thanks to various optimizations;
 * It can be set against one or many files, one or many directories;
 * Paths can be specified as trailing command arguments, and/or loaded via text file (with one path per line) with the `-l` option;
 * Directory processing is recursive;
 * Processing is done in parallel with multiple threads for major speedups;
 * Appropriate file types are automatically targeted; no thinking involved!


The "appropriate" file types are:

 * appcache
 * atom
 * bmp
 * css
 * eot
 * geojson
 * htc
 * htm(l)
 * ico
 * ics
 * js
 * json
 * jsonld
 * (web)manifest
 * md
 * mjs
 * otf
 * rdf
 * rss
 * svg
 * ttf
 * txt
 * vcard
 * vcs
 * vtt
 * wasm
 * xhtm(l)
 * xml
 * xsl



## Installation

Debian and Ubuntu users can just grab the pre-built `.deb` package from the [latest release](https://github.com/Blobfolio/channelz/releases/latest).

This application is written in [Rust](https://www.rust-lang.org/) and can alternatively be built from source using [Cargo](https://github.com/rust-lang/cargo):

```bash
# Clone the source.
git clone --recurse-submodules https://github.com/Blobfolio/channelz.git

# Go to it.
cd channelz

# Build as usual. Specify additional flags as desired.
cargo build \
    --bin channelz \
    --release
```

(This should work under other 64-bit Unix environments too, like MacOS.)



## Usage

It's easy. Just run `channelz [FLAGS] [OPTIONS] <PATH(S)>…`.

The following flags and options are available:

| Short | Long | Value | Description |
| ----- | ---- | ----- | ----------- |
| | `--clean` | | Remove all existing \*.gz \*.br files before starting. |
| `-h` | `--help` | | Print help information and exit. |
| `-l` | `--list` | `<FILE>` | Read (absolute) file and/or directory paths from this text file, one entry per line. |
| `-p` | `--progress` | | Show progress bar while minifying. |
| `-V` | `--version` | | Print program version and exit. |

For example:

```bash
# Generate app.js.gz and app.js.br:
channelz /path/to/app.js

# Tackle a whole folder at once with a nice progress bar:
channelz -p /path/to/assets

# Do the same thing, but clear out any old *.gz or *.br files first:
channelz --clean -p /path/to/assets

# Or load it up with a lot of places separately:
channelz /path/to/css /path/to/js …
```


## Benchmarks

These benchmarks were performed on a Intel® Core™ i7-10610U with four discrete cores, averaging 100 runs.

    Test:  ChannelZ Documentation
    Files: 35/47
    Size:  226,456 bytes (encodable)

| Program | Time (ms) | GZ (b) | BR (b) |
| ---- | ---- | ---- | ---- |
| ChannelZ | **1,050** | **66,394** | **55,099** |
| Find + Gzip + Brotli | 3,212 | 68,946 | 55,246 |

    Test:  WordPress Core
    Files: 815/1,980
    Size:  43,988,358 bytes (encodable)

| Program | Time (s) | GZ (b) | BR (b) |
| ---- | ---- | ---- | ---- |
| ChannelZ | **10.5045** | **7,539,948** | **6,522,810** |
| Find + Gzip + Brotli | 43.9917 | 7,856,120 | 6,557,240 |



## License

See also: [CREDITS.md](CREDITS.md)

Copyright © 2022 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

This work is free. You can redistribute it and/or modify it under the terms of the Do What The Fuck You Want To Public License, Version 2.

    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    Version 2, December 2004
    
    Copyright (C) 2004 Sam Hocevar <sam@hocevar.net>
    
    Everyone is permitted to copy and distribute verbatim or modified
    copies of this license document, and changing it is allowed as long
    as the name is changed.
    
    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
    
    0. You just DO WHAT THE FUCK YOU WANT TO.
