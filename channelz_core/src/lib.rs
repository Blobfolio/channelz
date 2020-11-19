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

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::map_err_ignore)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



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
	if let Some(raw) = fs::read(path).ok().filter(|r| ! r.is_empty()) {
		let mut buf: Vec<u8> = Vec::with_capacity(raw.len());
		let raw_path: &[u8] = unsafe { &*(path.as_os_str() as *const OsStr as *const [u8]) };

		// Brotli first.
		if 0 != encode_br(&raw, &mut buf) {
			write_result(OsStr::from_bytes(&[raw_path, b".br"].concat()), &buf);
		}

		// Gzip second.
		if 0 != encode_gz(&raw, &mut buf) {
			write_result(OsStr::from_bytes(&[raw_path, b".gz"].concat()), &buf);
		}
	}
}

#[must_use]
/// Encode Brotli.
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
/// Encode Gzip.
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

/// Write Result.
///
/// Write the buffer to an actual file.
///
/// The path is represented as an `OsStr` because that turns out to be the most
/// efficient medium to work with. Appending values to raw `PathBuf` objects is
/// painfully slow — much better to work with bytes — and `File::create()`
/// loads faster with an `OsStr` than `OsString`, `String`, or `str`.
fn write_result(path: &OsStr, data: &[u8]) {
	let _ = File::create(path)
		.and_then(|mut out| out.write_all(data).and_then(|_| out.flush()));
}
