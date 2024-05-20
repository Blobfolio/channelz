/*!
# ChannelZ - Parallelism!
*/

use argyle::ArgyleError;
use crate::enc::encode;
use dactyl::{
	NiceU64,
	NicePercent,
	traits::IntDivFloat,
};
use fyi_msg::{
	Msg,
	MsgKind,
	Progless,
};
use std::{
	num::NonZeroUsize,
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



#[inline(never)]
/// # Crunch Everything!
///
/// This crunches each of the `files` in parallel.
pub(super) fn exec(files: &[PathBuf]) -> Result<(), ArgyleError> {
	// Sort out the threads and job server.
	let total = NonZeroUsize::new(files.len()).ok_or(ERROR_NO_FILES)?;
	let threads = thread::available_parallelism().map_or(
		NonZeroUsize::MIN,
		|t| t.min(total),
	);

	// Set up the killswitch.
	let killed = Arc::new(AtomicBool::new(false));
	sigint(Arc::clone(&killed), None);

	// Thread business!
	let (tx, rx) = crossbeam_channel::bounded::<&Path>(threads.get());
	thread::scope(#[inline(always)] |s| {
		// Set up the worker threads.
		let mut workers = Vec::with_capacity(threads.get());
		for _ in 0..threads.get() {
			workers.push(s.spawn(#[inline(always)] ||
				while let Ok(p) = rx.recv() {
					let _res = encode(p);
				}
			));
		}

		// Push all the files to it, then drop the sender to disconnect.
		for file in files {
			if killed.load(Acquire) || tx.send(file).is_err() { break; }
		}
		drop(tx);

		// Wait for the threads to finish!
		for worker in workers { let _res = worker.join(); }
	});
	drop(rx);

	// Early abort?
	if killed.load(Acquire) { Err(ERROR_KILLED) }
	else { Ok(()) }
}

#[inline(never)]
/// # Crunch Everything (with Progress)!
///
/// This is the same as `exec`, but includes a progress bar and summary.
pub(super) fn exec_pretty(files: &[PathBuf]) -> Result<(), ArgyleError> {
	// Sort out the threads and job server.
	let total = NonZeroUsize::new(files.len()).ok_or(ERROR_NO_FILES)?;
	let threads = thread::available_parallelism().map_or(
		NonZeroUsize::MIN,
		|t| t.min(total),
	);

	// Boot up a progress bar.
	let progress = Progless::try_from(total.get())
		.map_err(|e| ArgyleError::Custom(e.as_str()))?
		.with_reticulating_splines("ChannelZ");

	// Set up the killswitch.
	let killed = Arc::new(AtomicBool::new(false));
	sigint(Arc::clone(&killed), Some(progress.clone()));

	// Thread business!
	let (tx, rx) = crossbeam_channel::bounded::<&Path>(threads.get());
	thread::scope(#[inline(always)] |s| {
		// Set up the worker threads.
		let mut workers = Vec::with_capacity(threads.get());
		for _ in 0..threads.get() {
			workers.push(s.spawn(#[inline(always)] ||
				while let Ok(p) = rx.recv() {
					let name = p.to_string_lossy();
					progress.add(&name);

					if let Some((a, b, c)) = encode(p) {
						SIZE_RAW.fetch_add(a, Relaxed);
						SIZE_BR.fetch_add(b, Relaxed);
						SIZE_GZ.fetch_add(c, Relaxed);
					}

					progress.remove(&name);
				}
			));
		}

		// Push all the files to it, then drop the sender to disconnect.
		for file in files {
			if killed.load(Acquire) || tx.send(file).is_err() { break; }
		}
		drop(tx);

		// Wait for the threads to finish!
		for worker in workers { let _res = worker.join(); }
	});
	drop(rx);

	// Finish up.
	progress.finish();
	progress.summary(MsgKind::Crunched, "file", "files").print();
	size_chart();

	// Early abort?
	if killed.load(Acquire) { Err(ERROR_KILLED) }
	else { Ok(()) }
}



#[inline(never)]
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

#[inline]
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
