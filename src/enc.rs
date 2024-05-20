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
	let len_gz = encode_gzip(&raw, &mut buf)
		.and_then(|()| write_atomic::write_file(&dst_gz, &buf).ok())
		.map_or_else(
			|| {
				remove_if(&dst_gz);
				len
			},
			|()| buf.len(),
		);

	// Now brotli!
	let dst_br = join_ext(src, ".br");
	let len_br = encode_brotli(&raw, &mut buf)
		.and_then(|()| write_atomic::write_file(&dst_br, &buf).ok())
		.map_or_else(
			|| {
				remove_if(&dst_br);
				len
			},
			|()| buf.len(),
		);

	// Done!
	Some((len as u64, len_br as u64, len_gz as u64))
}

#[inline]
/// # Encode Brotli.
///
/// Encode `raw` with Brotli and write the data into `buf`.
///
/// If there are problems or the result winds up bigger, `None` is
/// returned.
fn encode_brotli(raw: &[u8], buf: &mut Vec<u8>) -> Option<()> {
	buf.truncate(0);
	let config = BrotliEncoderParams {
		size_hint: raw.len(),
		..BrotliEncoderParams::default()
	};
	let len = BrotliCompress(&mut Cursor::new(raw), buf, &config).ok()?;

	// The brotli encoder is supposed to handle resizing.
	debug_assert_eq!(len, buf.len(), "Brotli buffer doesn't match length written.");

	// We're good if the result is smaller.
	if len == 0 || raw.len() < len { None }
	else { Some(()) }
}

#[inline]
/// # Encode Gzip.
///
/// Encode `raw` with Gzip and write the data into `buf`.
///
/// If there are problems or the result winds up bigger, `None` is
/// returned.
fn encode_gzip(raw: &[u8], buf: &mut Vec<u8>) -> Option<()> {
	let mut writer = Compressor::new(CompressionLvl::best());
	buf.resize(writer.gzip_compress_bound(raw.len()), 0);
	let len = writer.gzip_compress(raw, buf).ok()?;

	// We're good if the result is smaller.
	if len == 0 || raw.len() < len { None }
	else {
		// The gzip writer doesn't handle resizing, so let's trim any excess.
		buf.truncate(len);
		Some(())
	}
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
