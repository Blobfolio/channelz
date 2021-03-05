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



use argyle::{
	Argue,
	ArgyleError,
	FLAG_HELP,
	FLAG_REQUIRED,
	FLAG_VERSION,
};
use channelz_core::encode_path;
use dowser::Dowser;
use fyi_msg::{
	Msg,
	MsgKind,
	Progless,
};
use rayon::iter::{
	IntoParallelRefIterator,
	ParallelIterator,
};
use std::{
	convert::TryFrom,
	ffi::OsStr,
	os::unix::ffi::OsStrExt,
	path::PathBuf,
};



/// Main.
fn main() {
	match _main() {
		Ok(_) => {},
		Err(ArgyleError::WantsVersion) => {
			println!(concat!("ChannelZ v", env!("CARGO_PKG_VERSION")));
		},
		Err(ArgyleError::WantsHelp) => {
			helper();
		},
		Err(e) => {
			Msg::error(e).die(1);
		},
	}
}

#[inline]
/// Actual Main.
fn _main() -> Result<(), ArgyleError> {
	// Parse CLI arguments.
	let args = Argue::new(FLAG_HELP | FLAG_REQUIRED | FLAG_VERSION)?
		.with_list();

	let paths: Vec<PathBuf> = args.args()
		.iter()
		.map(|x| PathBuf::from(OsStr::from_bytes(x.as_ref())))
		.collect();

	// Cleaning?
	if args.switch(b"--clean") {
		clean(&paths);
	}

	// Put it all together!
	let paths = Vec::<PathBuf>::try_from(
		Dowser::default()
			.with_regex(r"(?i)[^/]+\.((geo)?json|atom|bmp|css|eot|htc|ico|ics|m?js|manifest|md|otf|rdf|rss|svg|ttf|txt|vcard|vcs|vtt|wasm|x?html?|xml|xsl)$")
			.with_paths(&paths)
	).map_err(|_| ArgyleError::Custom("No encodeable files were found."))?;

	// Sexy run-through.
	if args.switch2(b"-p", b"--progress") {
		let len: u32 = u32::try_from(paths.len())
			.map_err(|_| ArgyleError::Custom("Only 4,294,967,295 files can be crunched at one time."))?;

		// Boot up a progress bar.
		let progress = Progless::steady(len)
			.ok_or(ArgyleError::Custom("No encodeable files were found."))?
			.with_title(Some(Msg::custom("ChannelZ", 199, "Reticulating splines\u{2026}")));

		// Process!
		paths.par_iter().for_each(|x| {
			let tmp = x.to_string_lossy();
			progress.add(&tmp);
			encode_path(x);
			progress.remove(&tmp);
		});

		// Finish up.
		let _ = progress.finish();
		progress.summary(MsgKind::Crunched, "file", "files").print();
	}
	else {
		paths.par_iter().for_each(|x| {
			encode_path(x);
		});
	}

	Ok(())
}

/// Clean.
///
/// This will run a separate search over the specified paths with the sole
/// purpose of removing `*.gz` and `*.br` files.
fn clean(paths: &[PathBuf]) {
	Dowser::default()
		.with_regex(r"(?i)[^/]+\.((geo)?json|atom|bmp|css|eot|htc|ico|ics|m?js|manifest|md|otf|rdf|rss|svg|ttf|txt|vcard|vcs|vtt|wasm|x?html?|xml|xsl)\.(br|gz)$")
		.with_paths(paths)
		.build()
		.par_iter()
		.for_each(|x| {
			let _ = std::fs::remove_file(x);
		});
}

#[cold]
/// Print Help.
fn helper() {
	println!(concat!(
		r"
                  ,.
                 (\(\)
 ,_              ;  o >
  (`-.          /  (_)
  `=(\`-._____/`   |
   `-( /    -=`\   |
 .==`=(  -= = _/   /`--.
(M==M=M==M=M==M==M==M==M)
 \=N=N==N=N==N=N==N=NN=/   ", "\x1b[38;5;199mChannelZ\x1b[0;38;5;69m v", env!("CARGO_PKG_VERSION"), "\x1b[0m", r"
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
"
	));
}
