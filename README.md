# ChannelZ

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

 * css
 * eot
 * htm(l)
 * ico
 * js
 * json
 * mjs
 * otf
 * rss
 * svg
 * ttf
 * txt
 * xhtm(l)
 * xml
 * xsl



## Installation

This application is written in [Rust](https://www.rust-lang.org/) and can be installed using [Cargo](https://github.com/rust-lang/cargo).

For stable Rust (>= `1.47.0`), run:
```bash
RUSTFLAGS="-C link-arg=-s" cargo install \
    --git https://github.com/Blobfolio/channelz.git \
    --bin channelz \
    --target x86_64-unknown-linux-gnu
```

Pre-built `.deb` packages are also added for each [release](https://github.com/Blobfolio/channelz/releases/latest). They should always work for the latest stable Debian and Ubuntu.



## Usage

It's easy. Just run `channelz [FLAGS] [OPTIONS] <PATH(S)>…`.

The following flags and options are available:
```bash
    --clean          Remove all existing *.gz *.br files before starting.
-h, --help           Prints help information
-l, --list <list>    Read file paths from this list.
-p, --progress       Show progress bar while minifying.
-V, --version        Prints version information
```

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



## Credits

| Library | License | Author |
| ---- | ---- | ---- |
| [compu](https://crates.io/crates/compu) | Apache-2.0 | Douman |
| [criterion](https://crates.io/crates/criterion) | Apache-2.0 OR MIT | Jorge Aparicio, Brook Heisler |
| [libdeflater](https://crates.io/crates/libdeflater) | Apache-2.0 | Adam Kewley |



## License

Copyright © 2021 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

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
