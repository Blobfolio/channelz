/*!
# ChannelZ
*/

#![forbid(unsafe_code)]

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

#![allow(
	clippy::doc_markdown,
	clippy::redundant_pub_crate,
)]



mod enc;
mod ext;



use argyle::{
	Argue,
	ArgyleError,
	FLAG_HELP,
	FLAG_REQUIRED,
	FLAG_VERSION,
};
use crossbeam_channel::Receiver;
use dactyl::{
	NiceU64,
	NicePercent,
	traits::IntDivFloat,
};
use dowser::Dowser;
use fyi_msg::{
	Msg,
	MsgKind,
	Progless,
};
use std::{
	num::NonZeroUsize,
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
	thread,
};



/// # Progress Counters.
static SIZE_RAW: AtomicU64 = AtomicU64::new(0);
static SIZE_BR: AtomicU64 = AtomicU64::new(0);
static SIZE_GZ: AtomicU64 = AtomicU64::new(0);

/// # Error Constants.
const ERROR_KILLED: ArgyleError = ArgyleError::Custom("The process was aborted early.");
const ERROR_NO_FILES: ArgyleError = ArgyleError::Custom("No encodeable files were found.");



/// # Main.
fn main() {
	match _main() {
		Ok(()) => {},
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
	let mut paths: Vec<PathBuf> = Dowser::default()
		.with_paths(args.args_os())
		.into_vec_filtered(
			if args.switch(b"--force") { find_all }
			else { find_default }
		);
	let total = NonZeroUsize::new(paths.len()).ok_or(ERROR_NO_FILES)?;
	paths.sort();

	// How many threads?
	let threads = thread::available_parallelism().map_or(
		NonZeroUsize::MIN,
		|t| NonZeroUsize::min(t, total),
	);

	// Boot up a progress bar, if desired.
	let progress =
		if args.switch2(b"-p", b"--progress") {
			Progless::try_from(total.get())
				.ok()
				.map(|p| p.with_reticulating_splines("ChannelZ"))
		}
		else { None };

	// Set up the killswitch.
	let killed = Arc::new(AtomicBool::new(false));
	sigint(Arc::clone(&killed), progress.clone());

	// Thread business!
	let (tx, rx) = crossbeam_channel::bounded::<&Path>(threads.get());
	thread::scope(#[inline(always)] |s| {
		// Set up the worker threads.
		let mut workers = Vec::with_capacity(threads.get());
		if let Some(p) = progress.as_ref() {
			for _ in 0..threads.get() {
				workers.push(s.spawn(#[inline(always)] || crunch_pretty(&rx, p)));
			}
		}
		else {
			for _ in 0..threads.get() {
				workers.push(s.spawn(#[inline(always)] || crunch_quiet(&rx)));
			}
		}

		// Push all the files to it, then drop the sender to disconnect.
		for path in &paths {
			if killed.load(Acquire) || tx.send(path).is_err() { break; }
		}
		drop(tx);

		// Wait for the threads to finish!
		for worker in workers { let _res = worker.join(); }
	});
	drop(rx);

	// Summarize?
	if let Some(progress) = progress {
		progress.finish();
		progress.summary(MsgKind::Crunched, "file", "files").print();
		size_chart();
	}

	// Early abort?
	if killed.load(Acquire) { Err(ERROR_KILLED) }
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
		let len = bytes.len();
		if
			3 < len &&
			ext::match_br_gz(bytes) &&
			ext::match_extension(&bytes[..len - 3]) &&
			std::fs::remove_file(&p).is_err()
		{
			Msg::warning(format!("Unable to delete {p:?}")).eprint();
		}
	}
}

#[inline(never)]
/// # Worker Callback (Pretty).
///
/// This is the worker callback for pretty crunching. It listens for "new"
/// file paths and crunches them — and updates the progress bar, etc. —
/// then quits when the work has dried up.
fn crunch_pretty(rx: &Receiver::<&Path>, progress: &Progless) {
	let mut buf = Vec::new();
	while let Ok(p) = rx.recv() {
		let name = p.to_string_lossy();
		progress.add(&name);

		if let Some((a, b, c)) = enc::encode(p, &mut buf) {
			SIZE_RAW.fetch_add(a.get(), Relaxed);
			SIZE_BR.fetch_add(b.get(), Relaxed);
			SIZE_GZ.fetch_add(c.get(), Relaxed);
		}

		progress.remove(&name);
	}
}

#[inline(never)]
/// # Worker Callback (Quiet).
///
/// This is the worker callback for quiet crunching. It listens for "new"
/// file paths and crunches them, then quits when the work has dried up.
fn crunch_quiet(rx: &Receiver::<&Path>) {
	let mut buf = Vec::new();
	while let Ok(p) = rx.recv() { let _res = enc::encode(p, &mut buf); }
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
 \=N=N==N=N==N=N==N=NN=/   ", "\x1b[38;5;199mChannelZ\x1b[0;38;5;69m v", env!("CARGO_PKG_VERSION"), "\x1b[0m", r#"
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
    -l, --list <FILE> Read (absolute) file and/or directory paths to compress
                      from this text file — or STDIN if "-" — one entry per
                      line, instead of or in addition to specifying <PATH(S)>
                      directly at the end of the command.

ARGS:
    <PATH(S)>...      One or more file and/or directory paths to compress
                      and/or (recursively) crawl.

---

Note: static copies will only be generated for files with these extensions:

    appcache; atom; bmp; css; csv; doc(x); eot; geojson; htc; htm(l); ico; ics;
    js; json; jsonld; manifest; md; mjs; otf; pdf; rdf; rss; svg; ttf; txt;
    vcard; vcs; vtt; wasm; webmanifest; xhtm(l); xls(x); xml; xsl; y(a)ml
"#
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
fn size_chart() {
	// Grab the totals.
	let src = SIZE_RAW.load(Acquire);
	let br = SIZE_BR.load(Acquire);
	let gz = SIZE_GZ.load(Acquire);

	// Add commas to the numbers.
	let nice_src = NiceU64::from(src);
	let nice_br = NiceU64::from(br);
	let nice_gz = NiceU64::from(gz);

	// Find the maximum byte length so we can pad nicely.
	let len = usize::max(usize::max(nice_src.len(), nice_br.len()), nice_gz.len());

	// Figure out relative savings, if any.
	let per_br: String = br.div_float(src).map_or_else(
		String::new,
		|x| format!(" \x1b[2m(Saved {}.)\x1b[0m", NicePercent::from(1.0 - x).as_str())
	);

	let per_gz: String = gz.div_float(src).map_or_else(
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
