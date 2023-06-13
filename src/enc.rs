/*!
# `ChannelZ` Encoding
*/

use std::{
	io::Cursor,
	path::Path,
};



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
pub(super) fn encode(src: &Path) -> Option<(u64, u64, u64)> {
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
	let mut src = src.to_path_buf(); // Own it for later.

	// The output path.
	let gz_dst = {
		let mut tmp = src.clone();
		tmp.as_mut_os_string().push(".gz");
		tmp
	};
	let len_gz = encode_gzip(&gz_dst, &raw, &mut buf).unwrap_or(len);

	// Change the output path, then do Brotli.
	src.as_mut_os_string().push(".br");
	let len_br = encode_brotli(&src, &raw, &mut buf).unwrap_or(len);

	// Done!
	Some((len as u64, len_br as u64, len_gz as u64))
}

/// # Encode Brotli.
///
/// This will attempt to encode `raw` using Brotli, writing the result to disk
/// if it is smaller than the original.
fn encode_brotli(path: &Path, raw: &[u8], buf: &mut Vec<u8>) -> Option<usize> {
	use brotli::enc::{
		BrotliCompress,
		backward_references::BrotliEncoderParams,
	};

	buf.truncate(0);
	let config = BrotliEncoderParams::default();
	let size = BrotliCompress(&mut Cursor::new(raw), buf, &config).ok()?;
	if size != 0 && size < raw.len() {
		if write_atomic::write_file(path, &buf[..size]).is_ok() {
			return Some(size);
		}
		remove_if(path);
	}

	None
}

/// # Encode Gzip.
///
/// This will attempt to encode `raw` using Gzip, writing the result to disk
/// if it is smaller than the original.
fn encode_gzip(path: &Path, raw: &[u8], buf: &mut Vec<u8>) -> Option<usize> {
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
		if 0 < len && len < old_len && write_atomic::write_file(path, &buf[..len]).is_ok() {
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
fn remove_if(path: &Path) {
	if path.exists() {
		let _res = std::fs::remove_file(path);
	}
}
