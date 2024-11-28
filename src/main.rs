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



mod abacus;
mod enc;
mod err;
mod ext;



use abacus::{
	EncoderTotals,
	ThreadTotals,
};
use argyle::Argument;
use crossbeam_channel::Receiver;
use dactyl::NiceU64;
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
			Ordering::{
				Acquire,
				Relaxed,
				SeqCst,
			},
		},
	},
	thread,
};



/// # Flag: Brotli Enabled.
const FLAG_BR: u8 =         0b0000_0001;

/// # Flag: Gzip Enabled.
const FLAG_GZ: u8 =         0b0000_0010;

/// # Flag: All Encoders Enabled.
const FLAG_ALL: u8 =        0b0000_0011;

/// # Flag: Clean.
const FLAG_CLEAN: u8 =      0b0100_0000;

/// # Flag: Clean (Only).
const FLAG_CLEAN_ONLY: u8 = 0b1100_0000;

/// # Extension: Brotli.
const EXT_BR: u16 = u16::from_le_bytes([b'b', b'r']);

/// # Extension: Gzip.
const EXT_GZ: u16 = u16::from_le_bytes([b'g', b'z']);



/// # Main.
fn main() {
	match main__() {
		Ok(()) => {},
		Err(e @ (ChannelZError::PrintHelp | ChannelZError::PrintVersion)) => {
			println!("{e}");
		},
		Err(e) => { Msg::error(e.as_str()).die(1); },
	}
}

#[inline]
/// # Actual Main.
fn main__() -> Result<(), ChannelZError> {
	let args = argyle::args()
		.with_keywords(include!(concat!(env!("OUT_DIR"), "/argyle.rs")));

	let mut force = false;
	let mut kinds: u8 = FLAG_ALL;
	let mut paths = Dowser::default();
	let mut progress = false;
	for arg in args {
		match arg {
			Argument::Key("--clean") => { kinds |= FLAG_CLEAN; },
			Argument::Key("--clean-only") => { kinds |= FLAG_CLEAN_ONLY; },
			Argument::Key("--force") => { force = true; },
			Argument::Key("--no-br") => { kinds &= ! FLAG_BR; },
			Argument::Key("--no-gz") => { kinds &= ! FLAG_GZ; },
			Argument::Key("-p" | "--progress") => { progress = true; },

			Argument::Key("-h" | "--help") => return Err(ChannelZError::PrintHelp),
			Argument::Key("-V" | "--version") => return Err(ChannelZError::PrintVersion),

			Argument::KeyWithValue("-l" | "--list", s) => {
				paths.read_paths_from_file(s).map_err(|_| ChannelZError::ListFile)?;
			},

			// Assume paths.
			Argument::Other(s) => { paths = paths.with_path(s); },
			Argument::InvalidUtf8(s) => { paths = paths.with_path(s); },

			// Nothing else is expected.
			_ => {},
		}
	}

	// Nothing?
	if 0 == kinds & FLAG_ALL { return Err(ChannelZError::NoEncoders); }

	// Clean first?
	if FLAG_CLEAN == kinds & FLAG_CLEAN {
		clean(paths.clone(), progress, kinds);
		if FLAG_CLEAN_ONLY == kinds & FLAG_CLEAN_ONLY { return Ok(()); }
	}

	// Put it all together!
	let mut paths: Vec<PathBuf> = paths.into_vec_filtered(
		if force { find_all }
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
		if progress {
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
	let len = thread::scope(#[inline(always)] |s| {
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

		// Sum the totals as each thread finishes.
		// TODO: prefer try_reduce() when stable.
		workers.into_iter()
			.try_fold(ThreadTotals::new(), |acc, worker|
				worker.join().map(|len2| acc + len2)
			)
			.map_err(|_| ChannelZError::Jobserver)
	})?;
	drop(rx);

	// Summarize?
	if let Some(progress) = progress {
		progress.finish();
		progress.summary(MsgKind::Crunched, "file", "files").print();
		len.summarize(kinds);
	}

	// Early abort?
	if killed.load(Acquire) { Err(ChannelZError::Killed) }
	else { Ok(()) }
}

/// # Clean.
///
/// This will run a separate search over the specified paths with the sole
/// purpose of removing `*.gz` and/or `*.br` files.
fn clean(paths: Dowser, summary: bool, kinds: u8) {
	let has_br = FLAG_BR == kinds & FLAG_BR;
	let has_gz = FLAG_GZ == kinds & FLAG_GZ;

	let mut cleaned = 0_u64;
	for p in paths {
		let [rest @ .., b'.', y, z] = p.as_os_str().as_bytes() else { continue; };
		let ext = u16::from_le_bytes([y.to_ascii_lowercase(), z.to_ascii_lowercase()]);
		if
			((has_br && ext == EXT_BR) || (has_gz && ext == EXT_GZ)) &&
			ext::match_extension(rest)
		{
			if std::fs::remove_file(&p).is_ok() { cleaned += 1; }
			else {
				Msg::warning(format!("Unable to delete {p:?}")).eprint();
			}
		}
	}

	if summary {
		if cleaned == 0 { Msg::info("There was nothing to clean.") }
		else {
			Msg::success(format!(
				"Removed {} old {}-encoded {}.",
				NiceU64::from(cleaned),
				if has_br && has_gz { "br/gz" }
				else if has_br { "br" }
				else { "gz" },
				if cleaned == 1 { "copy" } else { "copies" },
			))
		}
		.print();
	}
}

#[inline(never)]
/// # Worker Callback (Pretty).
///
/// This is the worker callback for pretty crunching. It listens for "new"
/// file paths and crunches them — and updates the progress bar, etc. —
/// then quits when the work has dried up.
fn crunch_pretty(rx: &Receiver::<&Path>, kinds: u8, progress: &Progless) -> ThreadTotals {
	let mut enc = enc::Encoder::new(kinds);

	let mut len = ThreadTotals::new();
	while let Ok(p) = rx.recv() {
		let name = p.to_string_lossy();
		progress.add(&name);

		if let Some(len2) = enc.encode(p) { len += len2; }

		progress.remove(&name);
	}

	len
}

#[inline(never)]
/// # Worker Callback (Quiet).
///
/// This is the worker callback for quiet crunching. It listens for "new"
/// file paths and crunches them, then quits when the work has dried up.
fn crunch_quiet(rx: &Receiver::<&Path>, kinds: u8) -> ThreadTotals {
	let mut enc = enc::Encoder::new(kinds);
	while let Ok(p) = rx.recv() { let _res = enc.encode(p); }
	ThreadTotals::new() // We aren't keeping track in this mode.
}

#[cold]
/// # Find Non-GZ/BR.
///
/// This is a callback for `Dowser`. That library ensures the paths passed will
/// be valid, canonical _files_; all we need to do is check the extensions.
///
/// For this variation, everything is fair game so long as it isn't already
/// `gz`/`br`-encoded.
fn find_all(p: &Path) -> bool { ! ext::match_encoded(p.as_os_str().as_bytes()) }

/// # Find Default.
///
/// This is a callback for `Dowser`. That library ensures the paths passed will
/// be valid, canonical _files_; all we need to do is check the extensions.
///
/// For this variation, we're looking for all the hard-coded "default" types.
/// Refer to the main documentation or help screen for that list.
fn find_default(p: &Path) -> bool { ext::match_extension(p.as_os_str().as_bytes()) }

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
