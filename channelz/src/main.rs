/*!
# `ChannelZ`

`ChannelZ` is a CLI tool for x86-64 Linux machines that simplifies the common task of encoding static web assets with Gzip and Brotli for production environments.



## Features

 * `gzip` and `brotli` are compiled into `channelz`; their binaries do not need to be separately installed;
 * The maximum compression settings are applied; the end results will often be smaller than running native `gzip` or `brotli` thanks to various optimizations;
 * It can be set against one or many files, one or many directories;
 * Paths can be specified as trailing command arguments, and/or loaded via text file (with one path per line) with the `-l` option;
 * Directory processing is recursive;
 * Processing is done in parallel with multiple threads for major speedups;
 * Appropriate file types are automatically targeted; no thinking involved!


The "appropriate" file types are:

 * atom
 * bmp
 * css
 * eot
 * (geo)json
 * htc
 * htm(l)
 * ico
 * ics
 * js
 * manifest
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
-h, --help           Prints help information.
-l, --list <list>    Read file paths from this list.
-p, --progress       Show progress bar while minifying.
-V, --version        Prints version information.
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



## License

Copyright © 2020 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

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

*/

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(macro_use_extern_crate)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::map_err_ignore)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



use fyi_menu::{
	Argue,
	FLAG_REQUIRED,
};
use fyi_msg::Msg;
use fyi_witcher::{
	Witcher,
	WITCHING_QUIET,
	WITCHING_SUMMARIZE,
};
use std::path::PathBuf;



/// Main.
fn main() {
	// Parse CLI arguments.
	let args = Argue::new(FLAG_REQUIRED)
		.with_version("ChannelZ", env!("CARGO_PKG_VERSION"))
		.with_help(helper)
		.with_list();

	// Cleaning?
	if args.switch("--clean") {
		clean(args.args());
	}

	let flags: u8 =
		if args.switch2("-p", "--progress") { WITCHING_SUMMARIZE }
		else { WITCHING_QUIET | WITCHING_SUMMARIZE };

	// Put it all together!
	Witcher::default()
		.with_regex(r"(?i).+\.((geo)?json|atom|bmp|css|eot|htc|ico|ics|m?js|manifest|md|otf|rdf|rss|svg|ttf|txt|vcard|vcs|vtt|wasm|x?html?|xml|xsl)$")
		.with_paths(args.args())
		.into_witching()
		.with_flags(flags)
		.with_title(Msg::custom("ChannelZ", 199, "Reticulating splines\u{2026}"))
		.run(channelz_core::encode_path);
}

/// Clean.
///
/// This will run a separate search over the specified paths with the sole
/// purpose of removing `*.gz` and `*.br` files.
fn clean(paths: &[String]) {
	Witcher::default()
		.with_regex(r"(?i).+\.((geo)?json|atom|bmp|css|eot|htc|ico|ics|m?js|manifest|md|otf|rdf|rss|svg|ttf|txt|vcard|vcs|vtt|wasm|x?html?|xml|xsl)\.(br|gz)$")
		.with_paths(paths)
		.into_witching()
		.with_flags(WITCHING_QUIET)
		.run(|p: &PathBuf| {
			let _ = std::fs::remove_file(p).is_ok();
		});
}

#[cold]
/// Print Help.
fn helper(_: Option<&str>) {
	Msg::plain(format!(
		r"
                  ,.
                 (\(\)
 ,_              ;  o >
  (`-.          /  (_)
  `=(\`-._____/`   |
   `-( /    -=`\   |
 .==`=(  -= = _/   /`--.
(M==M=M==M=M==M==M==M==M)
 \=N=N==N=N==N=N==N=NN=/   {}{}{}
  \M==M=M==M=M==M===M=/    Fast, recursive, multi-threaded
   \N=N==N=N==N=NN=N=/     static Brotli and Gzip encoding.
    \M==M==M=M==M==M/
     `-------------'

USAGE:
    channelz [FLAGS] [OPTIONS] <PATH(S)>...

FLAGS:
        --clean       Remove all existing *.gz *.br files before starting.
    -h, --help        Prints help information.
    -p, --progress    Show progress bar while minifying.
    -V, --version     Prints version information.

OPTIONS:
    -l, --list <list>    Read file paths from this list.

ARGS:
    <PATH(S)>...    One or more files or directories to compress.

---

Note: static copies will only be generated for files with these extensions:

    atom; bmp; css; eot; (geo)json; htc; htm(l); ico; ics; js; manifest; md;
    mjs; otf; rdf; rss; svg; ttf; txt; vcard; vcs; vtt; wasm; xhtm(l); xml; xsl

",
		"\x1b[38;5;199mChannelZ\x1b[0;38;5;69m v",
		env!("CARGO_PKG_VERSION"),
		"\x1b[0m",
	)).print()
}
