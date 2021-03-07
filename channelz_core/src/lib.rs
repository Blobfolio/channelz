/*!
# `ChannelZ`: The Hard Bits
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

#![allow(clippy::module_name_repetitions)]



#[cfg(test)] use brunch as _;
use std::{
	ffi::OsStr,
	fs::{
		self,
		File,
	},
	io::Write,
	os::unix::ffi::OsStrExt,
	path::Path,
};



#[allow(trivial_casts)] // Trivial my arse.
/// Do the Deed!
///
/// This method generates statically-encoded Brotli and Gzip copies of a given
/// file. The raw data is read into memory once, and both it and a mutable
/// buffer are shared by the two encodings.
///
/// If for some reason the end result can't be created or winds up bigger than
/// the original, no static copy is saved to disk. (What would be the point?!)
pub fn encode_path<P>(path: P)
where P: AsRef<Path> {
	let path = path.as_ref();
	if let Some(raw) = fs::read(path).ok().filter(|r| ! r.is_empty()) {
		let mut buf: Vec<u8> = Vec::with_capacity(raw.len());

		// Make a fast byte version of the output path (starting with a .br
		// extension). This should just be a slice since it won't grow, but we
		// can't initiate a slice with a runtime-defined size.
		let mut dst = unsafe { &*(path.as_os_str() as *const OsStr as *const [u8]) }.to_vec();
		dst.extend_from_slice(b".br");

		// Brotli first.
		if 0 == encode_br(&raw, &mut buf) {
			delete_if(OsStr::from_bytes(&dst));
		}
		else {
			write_result(OsStr::from_bytes(&dst), &buf);
		}

		// Update destination path for .gz.
		let len: usize = dst.len();
		dst[len - 2..].copy_from_slice(b"gz");

		// Gzip second.
		if 0 == encode_gz(&raw, &mut buf) {
			delete_if(OsStr::from_bytes(&dst));
		}
		else {
			write_result(OsStr::from_bytes(&dst), &buf);
		}
	}
}

#[inline]
/// # Delete If.
fn delete_if<P>(path: P)
where P: AsRef<Path> {
	let path = path.as_ref();
	if path.exists() {
		let _ = std::fs::remove_file(path);
	}
}

#[must_use]
/// # Encode Brotli.
///
/// Write a Brotli-encoded copy of the raw data to the buffer using `Compu`'s
/// Brotli-C bindings.
fn encode_br(raw: &[u8], buf: &mut Vec<u8>) -> usize {
	use compu::{
		compressor::write::Compressor,
		encoder::{
			Encoder,
			EncoderOp,
			BrotliEncoder,
		},
	};

	let mut writer = Compressor::new(BrotliEncoder::default(), buf);
	writer.push(raw, EncoderOp::Finish)
		.ok()
		.filter(|&x| x < raw.len())
		.unwrap_or(0)
}

#[must_use]
/// # Encode Gzip.
///
/// Write a Gzip-encoded copy of the raw data to the buffer using the
/// `libdeflater` library. This is very nearly as fast as Cloudflare's
/// "optimized" `Zlib`, but achieves better compression.
fn encode_gz(raw: &[u8], buf: &mut Vec<u8>) -> usize {
	use libdeflater::{
		CompressionLvl,
		Compressor,
	};

	let mut writer = Compressor::new(CompressionLvl::best());
	buf.resize(writer.gzip_compress_bound(raw.len()), 0);

	writer.gzip_compress(raw, buf)
		.ok()
		.filter(|&x| x < raw.len())
		.map_or(0, |x| {
			buf.truncate(x);
			x
		})
}

#[inline]
/// # Write Result.
///
/// Write the buffer to an actual file.
///
/// The path is represented as an `OsStr` because that turns out to be the most
/// efficient medium to work with. Appending values to raw `PathBuf` objects is
/// painfully slow — much better to work with bytes — and `File::create()`
/// loads faster with an `OsStr` than `OsString`, `String`, or `str`.
fn write_result<P>(path: P, data: &[u8])
where P: AsRef<Path> {
	let _ = File::create(path)
		.and_then(|mut out| out.write_all(data).and_then(|_| out.flush()));
}
