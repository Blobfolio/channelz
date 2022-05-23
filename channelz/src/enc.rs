/*!
# `ChannelZ` Encoding
*/

use std::{
	ffi::OsStr,
	os::unix::ffi::OsStrExt,
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
	if
		channelz_brotli::encode(raw, buf) &&
		write_atomic::write_file(OsStr::from_bytes(path), buf).is_ok()
	{
		Some(buf.len())
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
