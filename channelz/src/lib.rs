/*!
# `ChannelZ`: The Hard Bits
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unknown_clippy_lints)]



use std::{
	ffi::OsStr,
	fs::{
		self,
		File,
	},
	io::Write,
	os::unix::ffi::OsStrExt,
	path::PathBuf,
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
pub fn encode_path(path: &PathBuf) {
	let raw: &[u8] = &fs::read(path).unwrap_or_default();
	if ! raw.is_empty() {
		let mut buf: Vec<u8> = Vec::with_capacity(raw.len());
		let raw_path: &[u8] = unsafe { &*(path.as_os_str() as *const OsStr as *const [u8]) };

		// Brotli first.
		if 0 != encode_br(raw, &mut buf) {
			write_result(
				OsStr::from_bytes(&[raw_path, b".br"].concat()),
				&buf
			);
		}

		// Gzip second.
		if 0 != encode_gz(raw, &mut buf) {
			write_result(
				OsStr::from_bytes(&[raw_path, b".gz"].concat()),
				&buf
			);
		}
	}
}

#[must_use]
/// Encode Brotli.
///
/// Write a Brotli-encoded copy of the raw data to the buffer using `Compu`'s
/// Brotli-C bindings.
///
/// TODO: Investigate the "multi" options present in Dropbox's version of the
/// Brotli library. That plus SIMD might wind up being faster, and since 99% of
/// the total processing time is spent on Brotli operations, that could make
/// `ChannelZ` feel a lot snappier!
pub fn encode_br(raw: &[u8], buf: &mut Vec<u8>) -> usize {
	use compu::{
		compressor::write::Compressor,
		encoder::{
			Encoder,
			EncoderOp,
			BrotliEncoder,
		},
	};

	let mut writer = Compressor::new(BrotliEncoder::default(), buf);
	match writer.push(raw, EncoderOp::Finish) {
		Ok(x) if x < raw.len() => x,
		_ => 0,
	}
}

#[must_use]
/// Encode Gzip.
///
/// Write a Gzip-encoded copy of the raw data to the buffer using the
/// `libdeflater` library. This is very nearly as fast as Cloudflare's
/// "optimized" `Zlib`, but achieves better compression.
pub fn encode_gz(raw: &[u8], buf: &mut Vec<u8>) -> usize {
	use libdeflater::{
		CompressionLvl,
		Compressor,
	};

	let mut writer = Compressor::new(CompressionLvl::best());
	buf.resize(writer.gzip_compress_bound(raw.len()), 0);

	match writer.gzip_compress(raw, buf) {
		Ok(len) if len < raw.len() => {
			buf.truncate(len);
			len
		},
		_ => { 0 }
	}
}

/// Write Result.
///
/// Write the buffer to an actual file.
///
/// The path is represented as an `OsStr` because that turns out to be the most
/// efficient medium to work with. Appending values to raw `PathBuf` objects is
/// painfully slow — much better to work with bytes — and `File::create()`
/// loads faster with an `OsStr` than `OsString`, `String`, or `str`.
///
/// TODO: We should probably be using `Tempfile` for atomicity.
pub fn write_result(path: &OsStr, data: &[u8]) {
	if let Ok(mut out) = File::create(path) {
		out.write_all(data).unwrap();
		out.flush().unwrap();
	}
}
