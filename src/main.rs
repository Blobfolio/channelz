/*!
# ChannelZ
*/

#![forbid(unsafe_code)]

#![deny(
	clippy::allow_attributes_without_reason,
	clippy::correctness,
	unreachable_pub,
)]

#![warn(
	clippy::complexity,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::style,

	clippy::allow_attributes,
	clippy::clone_on_ref_ptr,
	clippy::create_dir,
	clippy::filetype_is_file,
	clippy::format_push_string,
	clippy::get_unwrap,
	clippy::impl_trait_in_params,
	clippy::lossy_float_literal,
	clippy::missing_assert_message,
	clippy::missing_docs_in_private_items,
	clippy::needless_raw_strings,
	clippy::panic_in_result_fn,
	clippy::pub_without_shorthand,
	clippy::rest_pat_in_fully_bound_structs,
	clippy::semicolon_inside_block,
	clippy::str_to_string,
	clippy::string_to_string,
	clippy::todo,
	clippy::undocumented_unsafe_blocks,
	clippy::unneeded_field_pattern,
	clippy::unseparated_literal_suffix,
	clippy::unwrap_in_result,

	macro_use_extern_crate,
	missing_copy_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]

#![expect(clippy::doc_markdown, reason = "`ChannelZ` makes this annoying.")]
#![expect(clippy::redundant_pub_crate, reason = "Unresolvable.")]



mod enc;
mod err;
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
use err::ChannelZError;
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



/// # Progress Counters: Total Uncompressed Size.
static SIZE_RAW: AtomicU64 = AtomicU64::new(0);

/// # Progress Counters: Total Brotli Size.
static SIZE_BR: AtomicU64 = AtomicU64::new(0);

/// # Progress Counters: Total Gzip Size.
static SIZE_GZ: AtomicU64 = AtomicU64::new(0);

/// # Flag: Brotli Enabled.
const FLAG_BR: u8 =  0b0001;

/// # Flag: Gzip Enabled.
const FLAG_GZ: u8 =  0b0010;

/// # Flag: All Encoders Enabled.
const FLAG_ALL: u8 = 0b0011;

/// # Extension: Brotli.
const EXT_BR: u16 = u16::from_le_bytes([b'b', b'r']);

/// # Extension: Gzip.
const EXT_GZ: u16 = u16::from_le_bytes([b'g', b'z']);



/// # Main.
fn main() {
	match _main() {
		Ok(()) => {},
		Err(ChannelZError::Argue(ArgyleError::WantsVersion)) => {
			println!(concat!("ChannelZ v", env!("CARGO_PKG_VERSION")));
		},
		Err(ChannelZError::Argue(ArgyleError::WantsHelp)) => { helper(); },
		Err(e) => { Msg::error(e.as_str()).die(1); },
	}
}

#[inline]
/// # Actual Main.
fn _main() -> Result<(), ChannelZError> {
	// Parse CLI arguments.
	let args = Argue::new(FLAG_HELP | FLAG_REQUIRED | FLAG_VERSION)?
		.with_list();

	let kinds: u8 = match (args.switch2(b"--no-br", b"--no-brotli"), args.switch2(b"--no-gz", b"--no-gzip")) {
		(false, false) => FLAG_ALL,
		(false, true) => FLAG_BR,
		(true, false) => FLAG_GZ,
		(true, true) => return Err(ChannelZError::NoEncoders),
	};

	// Clean first?
	if args.switch2(b"--clean", b"--clean-only") {
		clean(args.args_os(), kinds);
		if args.switch(b"--clean-only") { return Ok(()); }
	}

	// Put it all together!
	let mut paths: Vec<PathBuf> = Dowser::default()
		.with_paths(args.args_os())
		.into_vec_filtered(
			if args.switch(b"--force") { find_all }
			else { find_default }
		);
	let total = NonZeroUsize::new(paths.len()).ok_or(ChannelZError::NoFiles)?;
	paths.sort();

	// How many threads?
	let threads = thread::available_parallelism().map_or(
		NonZeroUsize::MIN,
		|t| NonZeroUsize::min(t, total),
	);

	// Boot up a progress bar, if desired.
	let progress =
		if args.switch2(b"-p", b"--progress") {
			Progless::try_from(total)
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
				workers.push(s.spawn(#[inline(always)] || crunch_pretty(&rx, kinds, p)));
			}
		}
		else {
			for _ in 0..threads.get() {
				workers.push(s.spawn(#[inline(always)] || crunch_quiet(&rx, kinds)));
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
		size_chart(kinds);
	}

	// Early abort?
	if killed.load(Acquire) { Err(ChannelZError::Killed) }
	else { Ok(()) }
}

/// # Clean.
///
/// This will run a separate search over the specified paths with the sole
/// purpose of removing `*.gz` and/or `*.br` files.
fn clean<P, I>(paths: I, kinds: u8)
where P: AsRef<Path>, I: IntoIterator<Item=P> {
	let has_br = FLAG_BR == kinds & FLAG_BR;
	let has_gz = FLAG_GZ == kinds & FLAG_GZ;

	for p in Dowser::default().with_paths(paths) {
		let [rest @ .., b'.', y, z] = p.as_os_str().as_bytes() else { continue; };
		let ext = u16::from_le_bytes([y.to_ascii_lowercase(), z.to_ascii_lowercase()]);
		if
			((has_br && ext == EXT_BR) || (has_gz && ext == EXT_GZ)) &&
			ext::match_extension(rest) &&
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
fn crunch_pretty(rx: &Receiver::<&Path>, kinds: u8, progress: &Progless) {
	let mut enc = enc::Encoder::new(kinds);
	while let Ok(p) = rx.recv() {
		let name = p.to_string_lossy();
		progress.add(&name);

		if let Some([a, b, c]) = enc.encode(p) {
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
fn crunch_quiet(rx: &Receiver::<&Path>, kinds: u8) {
	let mut enc = enc::Encoder::new(kinds);
	while let Ok(p) = rx.recv() { let _res = enc.encode(p); }
}

#[cold]
/// # Find Non-GZ/BR.
///
/// This is a callback for `Dowser`. That library ensures the paths passed will
/// be valid, canonical _files_; all we need to do is check the extensions.
///
/// For this variation, everything is fair game so long as it isn't already
/// `gz`/`br`-encoded.
fn find_all(p: &Path) -> bool { ! ext::match_br_gz(p.as_os_str().as_bytes()) }

/// # Find Default.
///
/// This is a callback for `Dowser`. That library ensures the paths passed will
/// be valid, canonical _files_; all we need to do is check the extensions.
///
/// For this variation, we're looking for all the hard-coded "default" types.
/// Refer to the main documentation or help screen for that list.
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
        --clean       Remove all existing *.gz / *.br files (of types ChannelZ
                      would encode) before starting, unless --no-gz or --no-br
                      are also set, respectively.
        --clean-only  Same as --clean, but exit immediately afterward.
        --force       Try to encode ALL files passed to ChannelZ, regardless of
                      file extension (except those already ending in .br/.gz).
                      Be careful with this!
    -h, --help        Print help information and exit.
        --no-br       Skip Brotli encoding.
        --no-gz       Skip Gzip encoding.
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
fn size_chart(kinds: u8) {
	// What formats were we doing?
	let has_br = FLAG_BR == kinds & FLAG_BR;
	let has_gz = FLAG_GZ == kinds & FLAG_GZ;

	// Grab the totals.
	let src = SIZE_RAW.load(Acquire);
	let br = if has_br { SIZE_BR.load(Acquire) } else { 0 };
	let gz = if has_gz { SIZE_GZ.load(Acquire) } else { 0 };

	// Add commas to the numbers.
	let nice_src = NiceU64::from(src);
	let nice_br = NiceU64::from(br);
	let nice_gz = NiceU64::from(gz);

	// Find the maximum byte length so we can pad nicely.
	let len = usize::max(usize::max(nice_src.len(), nice_br.len()), nice_gz.len());

	// Figure out relative savings, if any.
	let per_br: String = if has_br {
		br.div_float(src).map_or_else(
			String::new,
			|x| format!(" \x1b[2m(Saved {}.)\x1b[0m", NicePercent::from(1.0 - x).as_str())
		)
	}
	else { String::new() };

	let per_gz: String = if has_gz {
		gz.div_float(src).map_or_else(
			String::new,
			|x| format!(" \x1b[2m(Saved {}.)\x1b[0m", NicePercent::from(1.0 - x).as_str())
		)
	}
	else { String::new() };

	// Print the totals!
	Msg::custom("  Source", 13, &format!("{}{} bytes", " ".repeat(len - nice_src.len()), nice_src.as_str()))
		.with_newline(true)
		.print();

	if has_br {
		Msg::custom("  Brotli", 13, &format!("{}{} bytes", " ".repeat(len - nice_br.len()), nice_br.as_str()))
			.with_suffix(per_br)
			.with_newline(true)
			.print();
	}

	if has_gz {
		Msg::custom("    Gzip", 13, &format!("{}{} bytes", " ".repeat(len - nice_gz.len()), nice_gz.as_str()))
			.with_suffix(per_gz)
			.with_newline(true)
			.print();
	}
}
