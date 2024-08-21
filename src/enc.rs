/*!
# ChannelZ: Encoding
*/

use brotli::enc::{
	backward_references::BrotliEncoderParams,
	BrotliCompress,
};
use libdeflater::{
	CompressionLvl,
	Compressor,
};
use std::{
	io::Cursor,
	num::NonZeroU64,
	path::{
		Path,
		PathBuf,
	},
};



#[inline]
/// # Encode File.
///
/// This will attempt to encode the given file with both Brotli and Gzip, and
/// return all three sizes (original, br, gz).
///
/// This returns `None` if the file is unreadable or empty, otherwise a triple
/// containing the original, brotli, and gzip sizes (in that order).
///
/// Note that if an encoder fails or produces larger output, the disk copy will
/// be deleted (if any) and its size will be adjusted to match the source to
/// emphasize the lack of savings.
pub(super) fn encode(src: &Path, buf: &mut Vec<u8>) -> Option<(NonZeroU64, NonZeroU64, NonZeroU64)> {
	// First things first, read the file and make sure its length is non-zero.
	let raw = std::fs::read(src).ok()?;
	let dst_gz = join_ext(src, ".gz");
	let dst_br = join_ext(src, ".br");
	let Some(len) = NonZeroU64::new(raw.len() as u64) else {
		remove_if(&dst_gz);
		remove_if(&dst_br);
		return None;
	};

	// Start with gzip since it will likely be larger, saving us the trouble
	// of having to increase the buffer size a second time.
	let len_gz = encode_gzip(&raw, buf)
		.filter(|_| write_atomic::write_file(&dst_gz, buf).is_ok())
		.unwrap_or_else(|| {
			remove_if(&dst_gz);
			len
		});

	// Now brotli!
	let len_br = encode_brotli(&raw, buf)
		.filter(|_| write_atomic::write_file(&dst_br, buf).is_ok())
		.unwrap_or_else(|| {
			remove_if(&dst_br);
			len
		});

	// Done!
	Some((len, len_br, len_gz))
}

#[inline]
/// # Encode Brotli.
///
/// Encode `raw` with Brotli and write the data into `buf`.
///
/// If there are problems or the result winds up bigger, `None` is
/// returned.
fn encode_brotli(raw: &[u8], buf: &mut Vec<u8>) -> Option<NonZeroU64> {
	buf.truncate(0);
	let config = BrotliEncoderParams {
		size_hint: raw.len(),
		..BrotliEncoderParams::default()
	};
	let len = BrotliCompress(&mut Cursor::new(raw), buf, &config).ok()?;

	// The brotli encoder is supposed to handle resizing.
	debug_assert_eq!(len, buf.len(), "Brotli buffer doesn't match length written.");

	// We're good if the result is smaller.
	if len <= raw.len() { NonZeroU64::new(len as u64) }
	else { None }
}

#[inline]
/// # Encode Gzip.
///
/// Encode `raw` with Gzip and write the data into `buf`.
///
/// If there are problems or the result winds up bigger, `None` is
/// returned.
fn encode_gzip(raw: &[u8], buf: &mut Vec<u8>) -> Option<NonZeroU64> {
	let mut writer = Compressor::new(CompressionLvl::best());
	buf.resize(writer.gzip_compress_bound(raw.len()), 0);
	let len = writer.gzip_compress(raw, buf).ok()?;

	// We're good if the result is smaller.
	if len <= raw.len() {
		// Trim to what was actually written.
		buf.truncate(len);
		NonZeroU64::new(len as u64)
	}
	else { None }
}

#[inline]
/// # Push Extension.
///
/// Create a new path by appending .gz/.br to it.
fn join_ext(src: &Path, ext: &str) -> PathBuf {
	let mut dst = src.to_path_buf();
	dst.as_mut_os_string().push(ext);
	dst
}

#[inline(never)]
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



#[cfg(test)]
mod test {
	use super::*;

	const RAW: &str = r#"Björk Guðmundsdóttir OTF (/bjɜːrk/ BYURK, Icelandic: [pjœr̥k ˈkvʏðmʏntsˌtouhtɪr̥] ⓘ; born 21 November 1965) is an Icelandic singer, songwriter, composer, record producer, and actress. Noted for her distinct voice, three-octave vocal range, and sometimes eccentric public persona, she has developed an eclectic musical style over a career spanning four decades, drawing on electronic, pop, experimental, trip hop, classical, and avant-garde music."#;

	#[test]
	fn t_brotli() {
		use std::io::Read;

		let mut enc = Vec::new();
		encode_brotli(RAW.as_bytes(), &mut enc).expect("Brotli encoding failed.");

		let mut dec = Vec::new();
		let mut r = brotli::Decompressor::new(enc.as_slice(), 4096);
		r.read_to_end(&mut dec).expect("Brotli decoding failed.");
		let dec = String::from_utf8(dec)
			.expect("Brotli decoding is invalid UTF-8.");

		assert_eq!(dec, RAW, "Brotli enc/dec doesn't match input.");
	}

	#[test]
	fn t_gzip() {
		let mut enc = Vec::new();
		encode_gzip(RAW.as_bytes(), &mut enc)
			.expect("Gzip encoding failed.");
		let len = enc.len();
		assert!(10 < len, "Gzip encoding is too small!");

		let gz_isize = {
			let mut ret = u32::from(enc[len - 4]);
			ret |= u32::from(enc[len - 3]) << 8;
			ret |= u32::from(enc[len - 2]) << 16;
			ret |= u32::from(enc[len - 1]) << 24;
			ret as usize
		};

		let mut r = libdeflater::Decompressor::new();
		let mut dec = Vec::new();
		dec.resize(gz_isize, 0);
		r.gzip_decompress(&enc, &mut dec).expect("Gzip decoding failed.");
		let dec = String::from_utf8(dec)
			.expect("Gzip decoding is invalid UTF-8.");

		assert_eq!(dec, RAW, "Gzip enc/dec doesn't match input.");
	}
}
