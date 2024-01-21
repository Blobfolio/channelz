/*!
# ChannelZ: Encoding
*/

use brotli::enc::{
	BrotliCompress,
	backward_references::BrotliEncoderParams,
};
use libdeflater::{
	CompressionLvl,
	Compressor,
};
use std::{
	io::Cursor,
	path::{
		Path,
		PathBuf,
	},
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
	if 0 == len || len > u128::from(u64::MAX) { return None; }

	#[cfg(not(target_pointer_width = "128"))]
	if len == 0 { return None; }

	// A shared buffer for our encoded copies.
	let mut buf: Vec<u8> = Vec::new();

	// Start with gzip since it will likely be larger, saving us the trouble
	// of having to increase the buffer size a second time.
	let dst_gz = join_ext(src, ".gz");
	let len_gz = encode_gzip(&dst_gz, &raw, &mut buf)
		.unwrap_or_else(|| {
			remove_if(&dst_gz);
			len
		});

	// Now brotli!
	let dst_br = join_ext(src, ".br");
	let len_br = encode_brotli(&dst_br, &raw, &mut buf)
		.unwrap_or_else(|| {
			remove_if(&dst_br);
			len
		});

	// Done!
	Some((len as u64, len_br as u64, len_gz as u64))
}

/// # Encode Brotli.
///
/// This will attempt to encode `raw` using Brotli, writing the result to disk
/// if it is smaller than the original.
fn encode_brotli(path: &Path, raw: &[u8], buf: &mut Vec<u8>) -> Option<usize> {
	buf.truncate(0);
	let config = BrotliEncoderParams::default();
	let len = BrotliCompress(&mut Cursor::new(raw), buf, &config).ok()?;
	if len != 0 && len < raw.len() && write_atomic::write_file(path, &buf[..len]).is_ok() {
		Some(len)
	}
	else { None }
}

/// # Encode Gzip.
///
/// This will attempt to encode `raw` using Gzip, writing the result to disk
/// if it is smaller than the original.
fn encode_gzip(path: &Path, raw: &[u8], buf: &mut Vec<u8>) -> Option<usize> {
	// Set up the buffer/writer.
	let mut writer = Compressor::new(CompressionLvl::best());
	buf.resize(writer.gzip_compress_bound(raw.len()), 0);

	// Encode!
	let len = writer.gzip_compress(raw, buf).ok()?;
	if len != 0 && len < raw.len() && write_atomic::write_file(path, &buf[..len]).is_ok() {
		Some(len)
	}
	else { None }
}

/// # Push Extension.
///
/// Create a new path by appending .gz/.br to it.
fn join_ext(src: &Path, ext: &str) -> PathBuf {
	let mut dst = src.to_path_buf();
	dst.as_mut_os_string().push(ext);
	dst
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
