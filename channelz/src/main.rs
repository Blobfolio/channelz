/*!
# `ChannelZ`
*/

#![deny(unsafe_code)]

#![warn(
	clippy::filetype_is_file,
	clippy::integer_division,
	clippy::needless_borrow,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::suboptimal_flops,
	clippy::unneeded_field_pattern,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]

#![allow(clippy::redundant_pub_crate)]



mod enc;
mod ext;



use argyle::{
	Argue,
	ArgyleError,
	FLAG_HELP,
	FLAG_REQUIRED,
	FLAG_VERSION,
};
use dactyl::{
	NiceU64,
	NicePercent,
};
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
	os::unix::ffi::OsStrExt,
	path::{
		Path,
		PathBuf,
	},
	sync::{
		Arc,
		atomic::{
			AtomicBool,
			AtomicU64,
			Ordering::{
				Acquire,
				Relaxed,
				SeqCst,
			},
		},
	},
};



/// # Main.
fn main() {
	match _main() {
		Ok(_) => {},
		Err(ArgyleError::WantsVersion) => {
			println!(concat!("ChannelZ v", env!("CARGO_PKG_VERSION")));
		},
		Err(ArgyleError::WantsHelp) => { helper(); },
		Err(e) => { Msg::error(e).die(1); },
	}
}

#[inline]
/// # Actual Main.
fn _main() -> Result<(), ArgyleError> {
	// Parse CLI arguments.
	let args = Argue::new(FLAG_HELP | FLAG_REQUIRED | FLAG_VERSION)?
		.with_list();

	// Clean first?
	if args.switch2(b"--clean", b"--clean-only") {
		clean(args.args_os());
		if args.switch(b"--clean-only") { return Ok(()); }
	}

	// Put it all together!
	let paths: Vec<PathBuf> = Dowser::default()
		.with_paths(args.args_os())
		.into_vec_filtered(
			if args.switch(b"--force") { find_all }
			else { find_default }
		);

	if paths.is_empty() {
		return Err(ArgyleError::Custom("No encodeable files were found."));
	}

	// Watch for SIGINT so we can shut down cleanly.
	let killed = Arc::from(AtomicBool::new(false));

	// Sexy run-through.
	if args.switch2(b"-p", b"--progress") {
		// Boot up a progress bar.
		let progress = Progless::try_from(paths.len())
			.map_err(|e| ArgyleError::Custom(e.as_str()))?
			.with_reticulating_splines("ChannelZ");

		let size_src = AtomicU64::new(0);
		let size_br = AtomicU64::new(0);
		let size_gz = AtomicU64::new(0);

		// Process!
		sigint(Arc::clone(&killed), Some(progress.clone()));
		paths.par_iter().for_each(|x| {
			if ! killed.load(Acquire) {
				let tmp = x.to_string_lossy();
				progress.add(&tmp);

				if let Some((a, b, c)) = enc::encode(x) {
					size_src.fetch_add(a, Relaxed);
					size_br.fetch_add(b, Relaxed);
					size_gz.fetch_add(c, Relaxed);
				}

				progress.remove(&tmp);
			}
		});

		// Finish up.
		progress.finish();
		progress.summary(MsgKind::Crunched, "file", "files").print();
		size_chart(size_src.into_inner(), size_br.into_inner(), size_gz.into_inner());
	}
	// Silent run-through.
	else {
		sigint(Arc::clone(&killed), None);
		paths.par_iter().for_each(|x| if ! killed.load(Acquire) {
			let _res = enc::encode(x);
		});
	}

	// Early abort?
	if killed.load(Acquire) { Err(ArgyleError::Custom("The process was aborted early.")) }
	else { Ok(()) }
}

/// # Clean.
///
/// This will run a separate search over the specified paths with the sole
/// purpose of removing `*.gz` and `*.br` files.
fn clean<P, I>(paths: I)
where P: AsRef<Path>, I: IntoIterator<Item=P> {
	for p in Dowser::default().with_paths(paths) {
		let bytes = p.as_os_str().as_bytes();
		if ext::match_br_gz(bytes) {
			let len = bytes.len();
			if ext::match_extension(&bytes[..len - 3]) && std::fs::remove_file(&p).is_err() {
				Msg::warning(format!("Unable to delete {:?}", p)).print();
			}
		}
	}
}

#[cold]
/// # Find Non-GZ/BR.
fn find_all(p: &Path) -> bool { ! ext::match_br_gz(p.as_os_str().as_bytes()) }

/// # Find Default.
fn find_default(p: &Path) -> bool { ext::match_extension(p.as_os_str().as_bytes()) }

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
        --clean       Remove all existing *.gz *.br files (of types ChannelZ
                      would encode) before starting.
        --clean-only  Same as --clean, but exit immediately afterward.
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

    appcache; atom; bmp; css; eot; geojson; htc; htm(l); ico; ics; js; json;
    jsonld; manifest; md; mjs; otf; rdf; rss; svg; ttf; txt; vcard; vcs; vtt;
    wasm; webmanifest; xhtm(l); xml; xsl
"
	));
}

/// # Hook Up CTRL+C.
///
/// Once stops processing new items, twice forces immediate shutdown.
fn sigint(killed: Arc<AtomicBool>, progress: Option<Progless>) {
	let _res = ctrlc::set_handler(move ||
		if killed.compare_exchange(false, true, SeqCst, Relaxed).is_ok() {
			if let Some(p) = &progress { p.sigint(); }
		}
		else { std::process::exit(1); }
	);
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
