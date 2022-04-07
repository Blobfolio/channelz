/*!
# `ChannelZ`: The Hard Bits
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

#![allow(clippy::module_name_repetitions)]



#[cfg(test)] use brunch as _;
use std::{
	ffi::OsStr,
	fs::File,
	io::Write,
	os::unix::ffi::OsStrExt,
	path::Path,
};



#[must_use]
/// # Encode File.
///
/// This will attempt to encode the given file with both Brotli and Gzip, and
/// return all three sizes (original, br, gz).
pub fn encode(src: &Path) -> Option<(u64, u64, u64)> {
	// First things first, read the file and make sure its length is non-zero
	// and fits within `u64`.
	let raw = std::fs::read(src).ok()?;
	let len = raw.len();
	if len == 0 { return None; }

	// Usize should normally be <= u64, but on 128-bit systems we have to
	// check!
	#[cfg(target_pointer_width = "128")]
	u64::try_from(len).ok()?;

	// Do Gzip first because it will likely be bigger than Brotli, saving us
	// the trouble of allocating additional buffer space down the road.
	let mut buf: Vec<u8> = Vec::new();
	let mut src: Vec<u8> = [src.as_os_str().as_bytes(), b".gz"].concat();
	let len_gz = encode_gzip(&src, &raw, &mut buf).unwrap_or(len);

	// Change the output path, then do Brotli.
	let src_len = src.len();
	src[src_len - 2] = b'b';
	src[src_len - 1] = b'r';
	let len_br = encode_brotli(&src, &raw, buf).unwrap_or(len);

	// Done!
	Some((len as u64, len_br as u64, len_gz as u64))
}



/// # Encode Brotli.
///
/// This will attempt to encode `raw` using Brotli, writing the result to disk
/// if it is smaller than the original.
fn encode_brotli(path: &[u8], raw: &[u8], mut buf: Vec<u8>) -> Option<usize> {
	use compu::{
		compressor::write::Compressor,
		encoder::{
			Encoder,
			EncoderOp,
			BrotliEncoder,
		},
	};

	// Set up the buffer/writer.
	buf.truncate(0);
	let mut writer = Compressor::new(BrotliEncoder::default(), &mut buf);

	// Encode!
	if let Ok(len) = writer.push(raw, EncoderOp::Finish) {
		// Save it?
		if 0 < len && len < raw.len() && write(OsStr::from_bytes(path), &buf) {
			return Some(len);
		}
	}

	// Clean up.
	remove_if(path);
	None
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
		if 0 < len && len < old_len && write(OsStr::from_bytes(path), &buf[..len]) {
			return Some(len);
		}
	}

	// Clean up.
	remove_if(path);
	None
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

/// # Write Result.
///
/// Write the buffer to an actual file.
fn write(path: &OsStr, data: &[u8]) -> bool {
	File::create(path)
		.and_then(|mut file| file.write_all(data).and_then(|_| file.flush()))
		.is_ok()
}
