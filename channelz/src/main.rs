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

For stable Rust (>= `1.51.0`), run:
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

#![forbid(unsafe_code)]

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
use channelz_core::ChannelZ;
use dactyl::{
	NiceU64,
	NicePercent,
};
use dowser::{
	Dowser,
	Extension,
};
use fyi_msg::{
	Msg,
	MsgKind,
	Progless,
};
use rayon::iter::{
	IntoParallelRefIterator,
	ParallelIterator,
};
use regex::bytes::Regex;
use std::{
	ffi::OsStr,
	os::unix::ffi::OsStrExt,
	path::{
		Path,
		PathBuf,
	},
	sync::atomic::{
		AtomicU64,
		Ordering::SeqCst,
	},
};



/// # Main.
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
/// # Actual Main.
fn _main() -> Result<(), ArgyleError> {
	// Parse CLI arguments.
	let args = Argue::new(FLAG_HELP | FLAG_REQUIRED | FLAG_VERSION)?
		.with_list();

	// Clean first?
	if args.switch(b"--clean") {
		clean(args.args().iter().map(|x| OsStr::from_bytes(x.as_ref())));
	}

	// Put it all together!
	let paths: Vec<PathBuf> =
		if args.switch(b"--force") {
			const E_BR: Extension = Extension::new2(*b"br");
			const E_GZ: Extension = Extension::new2(*b"gz");

			Dowser::default()
				.with_paths(args.args().iter().map(|x| OsStr::from_bytes(x)))
				.filter(|p|
					Extension::try_from2(p).map_or(true, |e| e != E_BR && e != E_GZ)
				)
				.collect()
		}
		else {
			let re = Regex::new(r"(?i)[^/]+\.((geo)?json|atom|bmp|css|eot|htc|ico|ics|m?js|manifest|md|otf|rdf|rss|svg|ttf|txt|vcard|vcs|vtt|wasm|x?html?|xml|xsl)$").unwrap();
			Dowser::default()
				.with_paths(args.args().iter().map(|x| OsStr::from_bytes(x)))
				.filter(|p| re.is_match(p.as_os_str().as_bytes()))
				.collect()
		};

	if paths.is_empty() {
		return Err(ArgyleError::Custom("No encodeable files were found."));
	}

	// Sexy run-through.
	if args.switch2(b"-p", b"--progress") {
		// Boot up a progress bar.
		let progress = Progless::try_from(paths.len())
			.map_err(|_| ArgyleError::Custom("Progress can only be displayed for up to 4,294,967,295 files. Try again with fewer files or without the -p/--progress flag."))?
			.with_title(Some(Msg::custom("ChannelZ", 199, "Reticulating splines\u{2026}")));

		let size_src = AtomicU64::new(0);
		let size_br = AtomicU64::new(0);
		let size_gz = AtomicU64::new(0);

		// Process!
		paths.par_iter().for_each(|x| {
			if let Ok(mut enc) = ChannelZ::try_from(x) {
				let tmp = x.to_string_lossy();
				progress.add(&tmp);
				enc.encode();

				// Update the size totals.
				let (a, b, c) = enc.sizes();
				size_src.fetch_add(a, SeqCst);
				size_br.fetch_add(b, SeqCst);
				size_gz.fetch_add(c, SeqCst);

				progress.remove(&tmp);
			}
			else {
				progress.increment();
			}
		});

		// Finish up.
		progress.finish();
		progress.summary(MsgKind::Crunched, "file", "files").print();
		size_chart(size_src.load(SeqCst), size_br.load(SeqCst), size_gz.load(SeqCst));
	}
	else {
		paths.par_iter().for_each(|x|
			if let Ok(mut x) = ChannelZ::try_from(x) { x.encode(); }
		);
	}

	Ok(())
}

/// # Clean.
///
/// This will run a separate search over the specified paths with the sole
/// purpose of removing `*.gz` and `*.br` files.
fn clean<P, I>(paths: I)
where P: AsRef<Path>, I: IntoIterator<Item=P> {
	let re = Regex::new(r"(?i)[^/]+\.((geo)?json|atom|bmp|css|eot|htc|ico|ics|m?js|manifest|md|otf|rdf|rss|svg|ttf|txt|vcard|vcs|vtt|wasm|x?html?|xml|xsl)\.(br|gz)$").unwrap();
	for p in Dowser::default().with_paths(paths) {
		if re.is_match(p.as_os_str().as_bytes()) {
			let _res = std::fs::remove_file(p);
		}
	}
}

#[cold]
/// # Print Help.
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
        --force       Try to encode ALL files passed to ChannelZ, regardless of
                      file extension (except those already ending in .br/.gz).
                      Be careful with this!
    -h, --help        Print help information and exit.
    -p, --progress    Show progress bar while minifying.
    -V, --version     Print version information and exit.

OPTIONS:
    -l, --list <FILE> Read (absolute) file and/or directory paths from this
                      text file, one entry per line.

ARGS:
    <PATH(S)>...      One or more file and/or directory paths to compress
                      and/or (recursively) crawl.

---

Note: static copies will only be generated for files with these extensions:

    atom; bmp; css; eot; (geo)json; htc; htm(l); ico; ics; js; manifest; md;
    mjs; otf; rdf; rss; svg; ttf; txt; vcard; vcs; vtt; wasm; xhtm(l); xml; xsl
"
	));
}

/// # Summarize Output Sizes.
///
/// This compares the original sources against their Brotli and Gzip
/// counterparts.
fn size_chart(src: u64, br: u64, gz: u64) {
	// Add commas to the numbers.
	let nice_src = NiceU64::from(src);
	let nice_br = NiceU64::from(br);
	let nice_gz = NiceU64::from(gz);

	// Find the maximum byte length so we can pad nicely.
	let len = usize::max(usize::max(nice_src.len(), nice_br.len()), nice_gz.len());

	// Figure out relative savings, if any.
	let per_br: String = dactyl::int_div_float(br, src).map_or_else(
			String::new,
			|x| format!(" \x1b[2m(Saved {}.)\x1b[0m", NicePercent::from(1.0 - x).as_str())
	);

	let per_gz: String = dactyl::int_div_float(gz, src).map_or_else(
			String::new,
			|x| format!(" \x1b[2m(Saved {}.)\x1b[0m", NicePercent::from(1.0 - x).as_str())
	);

	// Print the totals!
	Msg::custom("  Source", 13, &format!("{}{} bytes", " ".repeat(len - nice_src.len()), nice_src.as_str()))
		.with_newline(true)
		.print();

	Msg::custom("  Brotli", 13, &format!("{}{} bytes", " ".repeat(len - nice_br.len()), nice_br.as_str()))
		.with_suffix(per_br)
		.with_newline(true)
		.print();

	Msg::custom("    Gzip", 13, &format!("{}{} bytes", " ".repeat(len - nice_gz.len()), nice_gz.as_str()))
		.with_suffix(per_gz)
		.with_newline(true)
		.print();
}
