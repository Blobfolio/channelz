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



mod brotli;
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
	ffi::OsStr,
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
			Ordering::SeqCst,
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
		Err(ArgyleError::WantsHelp) => {
			helper();
		},
		Err(e) => {
			Msg::error(e).die(1);
		},
	}
}

#[inline]
#[allow(clippy::cast_possible_truncation)] // It fits.
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
	let paths: Vec<PathBuf> =
		// Pull anything that isn't already br/gz.
		if args.switch(b"--force") {
			Dowser::default()
				.with_paths(args.args_os())
				.into_vec(|p| ! ext::match_br_gz(p.as_os_str().as_bytes()))
		}
		// Pull only supported files.
		else {
			Dowser::default()
				.with_paths(args.args_os())
				.into_vec(|p| ext::match_extension(p.as_os_str().as_bytes()))
		};

	if paths.is_empty() {
		return Err(ArgyleError::Custom("No encodeable files were found."));
	}

	// Should we show progress?
	let progress =
		if args.switch2(b"-p", b"--progress") {
			if paths.len() <= Progless::MAX_TOTAL { true }
			else {
				Msg::warning(Progless::MAX_TOTAL_ERROR).print();
				false
			}
		}
		else { false };

	// Watch for SIGINT so we can shut down cleanly.
	let killed = Arc::from(AtomicBool::new(false));
	let _res = signal_hook::flag::register_conditional_shutdown(
		signal_hook::consts::SIGINT,
		1,
		Arc::clone(&killed)
	).and_then(|_| signal_hook::flag::register(
		signal_hook::consts::SIGINT,
		Arc::clone(&killed)
	));

	// Sexy run-through.
	if progress {
		// Boot up a progress bar.
		let progress = Progless::try_from(paths.len() as u32)
			.unwrap()
			.with_reticulating_splines("ChannelZ");

		let size_src = AtomicU64::new(0);
		let size_br = AtomicU64::new(0);
		let size_gz = AtomicU64::new(0);

		// Process!
		paths.par_iter().for_each(|x| {
			if killed.load(SeqCst) { progress.sigint(); }
			else {
				let tmp = x.to_string_lossy();
				progress.add(&tmp);

				if let Some((a, b, c)) = encode(x) {
					size_src.fetch_add(a, SeqCst);
					size_br.fetch_add(b, SeqCst);
					size_gz.fetch_add(c, SeqCst);
				}

				progress.remove(&tmp);
			}
		});

		// Finish up.
		progress.finish();
		progress.summary(MsgKind::Crunched, "file", "files").print();
		size_chart(size_src.load(SeqCst), size_br.load(SeqCst), size_gz.load(SeqCst));
	}
	// Silent run-through.
	else {
		paths.par_iter().for_each(|x| {
			if ! killed.load(SeqCst) { let _res = encode(x); }
		});
	}

	// Early abort?
	if killed.load(SeqCst) { Err(ArgyleError::Custom("The process was aborted early.")) }
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

/// # Encode File.
///
/// This will attempt to encode the given file with both Brotli and Gzip, and
/// return all three sizes (original, br, gz).
///
/// If the file is unreadable, empty, or too big to represent as `u64`, `None`
/// will be returned. If either Gzip or Brotli fail (or result in larger
/// output), their "sizes" will actually represent the original input size.
/// (We're looking for savings, and if we can't encode as .gz or whatever,
/// there are effectively no savings.)
fn encode(src: &Path) -> Option<(u64, u64, u64)> {
	// First things first, read the file and make sure its length is non-zero
	// and fits within `u64`.
	let raw = std::fs::read(src).ok()?;
	let len = raw.len();

	#[cfg(target_pointer_width = "128")]
	if 0 == len || len > 18_446_744_073_709_551_615 { return None; }

	#[cfg(not(target_pointer_width = "128"))]
	if len == 0 { return None; }

	// Do Gzip first because it will likely be bigger than Brotli, saving us
	// the trouble of allocating additional buffer space down the road.
	let mut buf: Vec<u8> = Vec::new();
	let mut src: Vec<u8> = [src.as_os_str().as_bytes(), b".gz"].concat();
	let len_gz = encode_gzip(&src, &raw, &mut buf).unwrap_or(len);

	// Change the output path, then do Brotli.
	let src_len = src.len();
	src[src_len - 2] = b'b';
	src[src_len - 1] = b'r';
	let len_br = encode_brotli(&src, &raw, &mut buf).unwrap_or(len);

	// Done!
	Some((len as u64, len_br as u64, len_gz as u64))
}

/// # Encode Brotli.
///
/// This will attempt to encode `raw` using Brotli, writing the result to disk
/// if it is smaller than the original.
fn encode_brotli(path: &[u8], raw: &[u8], buf: &mut Vec<u8>) -> Option<usize> {
	let size = brotli::encode(raw, buf);
	if 0 < size && write_atomic::write_file(OsStr::from_bytes(path), buf).is_ok() {
		Some(size)
	}
	else {
		// Clean up.
		remove_if(path);
		None
	}
}

/// # Encode Gzip.
///
/// This will attempt to encode `raw` using Gzip, writing the result to disk
/// if it is smaller than the original.
fn encode_gzip(path: &[u8], raw: &[u8], buf: &mut Vec<u8>) -> Option<usize> {
	use libdeflater::{
		CompressionLvl,
		Compressor,
	};

	// Set up the buffer/writer.
	let old_len = raw.len();
	let mut writer = Compressor::new(CompressionLvl::best());
	buf.resize(writer.gzip_compress_bound(old_len), 0);

	// Encode!
	if let Ok(len) = writer.gzip_compress(raw, buf) {
		if 0 < len && len < old_len && write_atomic::write_file(OsStr::from_bytes(path), &buf[..len]).is_ok() {
			return Some(len);
		}
	}

	// Clean up.
	remove_if(path);
	None
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

/// # Remove If It Exists.
///
/// This method is used to clean up previously-encoded copies of a file when
/// the current encoding operation fails.
///
/// We can't do anything if deletion fails, but at least we can say we tried.
fn remove_if(path: &[u8]) {
	let path = Path::new(OsStr::from_bytes(path));
	if path.exists() {
		let _res = std::fs::remove_file(path);
	}
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
