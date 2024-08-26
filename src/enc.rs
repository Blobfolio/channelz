/*!
# ChannelZ: Encoding
*/

// Performance is a lot better when the encoding pieces are kept together. The
// caller is #[inline(never)], so the trail won't be too long.
#![allow(clippy::inline_always)]

use brotli::enc::{
	backward_references::BrotliEncoderParams,
	BrotliCompress,
};
use libdeflater::{
	CompressionLvl,
	Compressor,
};
use std::{
	ffi::OsStr,
	fs::File,
	io::Cursor,
	num::NonZeroU64,
	os::unix::ffi::OsStrExt,
	path::Path,
};



/// # Encoder.
///
/// This re-usable (per-thread) structure holds the uncompressed source data,
/// a buffer for encoding, and output paths for the encoded versions.
pub(super) struct Encoder {
	src: Vec<u8>,      // Buffer for source data.
	dst_buf: Vec<u8>,  // Buffer for encoded data.
	dst_br: Vec<u8>,   // Output paths for encoded versions.
	dst_gz: Vec<u8>,
}

impl Encoder {
	/// # New.
	///
	/// This returns a new empty instance.
	pub(super) const fn new() -> Self {
		Self {
			src: Vec::new(),
			dst_buf: Vec::new(),
			dst_br: Vec::new(),
			dst_gz: Vec::new(),
		}
	}

	#[inline(always)]
	/// # Encode.
	///
	/// This method attempts to read `src` and re-encode it with gzip and
	/// brotli, saving each copy if they offer any improvement, or removing
	/// previous instances if not.
	///
	/// So long as the file is readable and non-empty, this will return the
	/// uncompressed size and, if brotli and/or gzip copies get saved, their
	/// sizes too.
	///
	/// If an encoding fails, the source size will be returned in its place
	/// (regardless of how big the encoded version wound up).
	pub(super) fn encode(&mut self, src: &Path)
	-> Option<(NonZeroU64, NonZeroU64, NonZeroU64)> {
		// First, let's update the destination paths.
		self.set_dst_paths(src);

		// Now try to read the source.
		let Some(len_src) = self.read_source(src) else {
			self.remove_br();
			self.remove_gz();
			return None;
		};

		// Try to encode it with gzip! This version is done first because it
		// will likely be bigger, saving brotli the trouble of reallocating.
		let len_gz = self.gzip().unwrap_or_else(|| {
			self.remove_gz();
			len_src
		});

		// And now do the same with brotli… (Note: this method updates the
		// destination path accordingly.)
		let len_br = self.brotli().unwrap_or_else(|| {
			self.remove_br();
			len_src
		});

		// Done!
		Some((len_src, len_br, len_gz))
	}
}

impl Encoder {
	#[inline(always)]
	/// # Encode With Brotli.
	///
	/// This will return `None` if encoding fails, the output winds up _larger_,
	/// or the result cannot be written to disk, otherwise the length of the
	/// encoded copy.
	fn brotli(&mut self) -> Option<NonZeroU64> {
		// Sliceify the source to make life easier.
		let raw = self.src.as_slice();

		// Reset the buffer and encode!
		self.dst_buf.truncate(0);
		let config = BrotliEncoderParams {
			size_hint: raw.len(),
			..BrotliEncoderParams::default()
		};
		let len = BrotliCompress(&mut Cursor::new(raw), &mut self.dst_buf, &config).ok()?;

		// We're good so long as the result didn't get bigger.
		if len <= raw.len() {
			let len = NonZeroU64::new(len as u64)?;

			// Write the contents and return the length.
			if write_atomic::write_file(self.brotli_path(), &self.dst_buf).is_ok() { Some(len) }
			else { None }
		}
		else { None }
	}

	#[inline(always)]
	/// # Brotli Path.
	fn brotli_path(&self) -> &OsStr { OsStr::from_bytes(&self.dst_br) }

	#[inline(always)]
	/// # Encode With Gzip.
	///
	/// This will return `None` if encoding fails, the output winds up _larger_,
	/// or the result cannot be written to disk, otherwise the length of the
	/// encoded copy.
	fn gzip(&mut self) -> Option<NonZeroU64> {
		// Sliceify the source to make life easier.
		let raw = self.src.as_slice();

		// Reset the buffer and encode!
		let mut writer = Compressor::new(CompressionLvl::best());
		self.dst_buf.resize(writer.gzip_compress_bound(raw.len()), 0);
		let len = writer.gzip_compress(raw, &mut self.dst_buf).ok()?;

		// We're good so long as the result didn't get bigger.
		if len <= raw.len() {
			self.dst_buf.truncate(len); // Libdeflater doesn't trim to fit.
			let len = NonZeroU64::new(len as u64)?;

			// Write the contents and return the length.
			if write_atomic::write_file(self.gzip_path(), &self.dst_buf).is_ok() { Some(len) }
			else { None }
		}
		else { None }
	}

	#[inline(always)]
	/// # Gzip Path.
	fn gzip_path(&self) -> &OsStr { OsStr::from_bytes(&self.dst_gz) }
}

impl Encoder {
	#[allow(clippy::cast_possible_truncation)]
	#[inline(always)]
	/// # Read Source.
	///
	/// This is basically `std::fs::read`, except the data is copied into our
	/// existing buffer to reduce the number of runtime allocations.
	///
	/// If everything works and the file is non-empty, its size is returned,
	/// otherwise `None`.
	fn read_source(&mut self, raw: &Path) -> Option<NonZeroU64> {
		use std::io::Read;

		let Ok(mut file) = File::open(raw) else { return None; };
		let Ok(meta) = file.metadata() else { return None; };

		self.src.truncate(0);
		let len = meta.len();
		if len == 0 || self.src.try_reserve_exact(len as usize).is_err() { return None; }

		if file.read_to_end(&mut self.src).is_ok() {
			NonZeroU64::new(self.src.len() as u64)
		}
		else { None }
	}

	#[cold]
	/// # Remove Brotli Copy (if it exists)
	///
	/// In cases where encoding can't be run or failed, this method is called
	/// to remove any previously-generated copy of the encoded content.
	fn remove_br(&self) {
		let path: &Path = self.brotli_path().as_ref();
		if path.exists() {
			let _res = std::fs::remove_file(path);
		}
	}

	#[cold]
	/// # Remove Gzip Copy (if it exists)
	///
	/// In cases where encoding can't be run or failed, this method is called
	/// to remove any previously-generated copy of the encoded content.
	fn remove_gz(&self) {
		let path: &Path = self.gzip_path().as_ref();
		if path.exists() {
			let _res = std::fs::remove_file(path);
		}
	}

	#[inline(always)]
	/// # Set Destination Paths.
	///
	/// This rebuilds the struct's destination paths to match the source path
	/// (with the extra `.br`/`.gz` suffixes).
	fn set_dst_paths(&mut self, src: &Path) {
		// Working with bytes is a little strange, but noticeably more
		// performant than sticking with PathBuf. As this app only supports
		// Unix platforms, we have the option, so might as well take it.
		let raw_bytes = src.as_os_str().as_bytes();

		self.dst_br.truncate(0);
		self.dst_br.extend_from_slice(raw_bytes);
		self.dst_br.extend_from_slice(b".br");

		self.dst_gz.truncate(0);
		self.dst_gz.extend_from_slice(raw_bytes);
		self.dst_gz.extend_from_slice(b".gz");
	}
}



#[cfg(test)]
mod test {
	use super::*;
	use std::path::PathBuf;

	const RAW: &str = r#"Björk Guðmundsdóttir OTF (/bjɜːrk/ BYURK, Icelandic: [pjœr̥k ˈkvʏðmʏntsˌtouhtɪr̥] ⓘ; born 21 November 1965) is an Icelandic singer, songwriter, composer, record producer, and actress. Noted for her distinct voice, three-octave vocal range, and sometimes eccentric public persona, she has developed an eclectic musical style over a career spanning four decades, drawing on electronic, pop, experimental, trip hop, classical, and avant-garde music."#;
	const NAME_RAW: &str = "channelz.txt";
	const NAME_BR: &str = "channelz.txt.br";
	const NAME_GZ: &str = "channelz.txt.gz";

	/// # Temporary Path.
	///
	/// This returns a path we can use for the source file.
	fn tmp_path() -> Option<PathBuf> {
		let path = std::env::temp_dir();
		if path.is_dir() { Some(path.join(NAME_RAW)) }
		else { None }
	}

	/// # Decode Brotli.
	fn decode_brotli(src: &Path) {
		use std::io::Read;

		// Load the encoded content.
		let enc = std::fs::read(src).expect("Missing brotli copy.");

		// Decode it.
		let mut dec = Vec::new();
		let mut r = brotli::Decompressor::new(enc.as_slice(), 4096);
		r.read_to_end(&mut dec).expect("Brotli decoding failed.");
		let dec = String::from_utf8(dec)
			.expect("Brotli decoding is invalid UTF-8.");

		assert_eq!(dec, RAW, "Brotli enc/dec doesn't match input.");
	}

	/// # Decode Gzip.
	fn decode_gzip(src: &Path) {
		// Load the encoded content.
		let enc = std::fs::read(src).expect("Missing gzip copy.");
		let len = enc.len();
		assert!(10 < len, "Gzip encoding is too small!");

		let gz_isize = {
			let mut ret = u32::from(enc[len - 4]);
			ret |= u32::from(enc[len - 3]) << 8;
			ret |= u32::from(enc[len - 2]) << 16;
			ret |= u32::from(enc[len - 1]) << 24;
			ret as usize
		};

		// Decode it.
		let mut r = libdeflater::Decompressor::new();
		let mut dec = Vec::new();
		dec.resize(gz_isize, 0);
		r.gzip_decompress(&enc, &mut dec).expect("Gzip decoding failed.");
		let dec = String::from_utf8(dec)
			.expect("Gzip decoding is invalid UTF-8.");

		assert_eq!(dec, RAW, "Gzip enc/dec doesn't match input.");
	}

	#[test]
	fn t_encode() {
		// Save an uncompressed source to work with.
		let Some(src) = tmp_path() else { return; };
		let src_br = src.with_file_name(NAME_BR);
		let src_gz = src.with_file_name(NAME_GZ);
		write_atomic::write_file(&src, RAW.as_bytes()).expect("Unable to save source file.");

		// Encode it!
		let mut encoder = Encoder::new();
		encoder.encode(&src).expect("Encoding failed!");

		// Check the paths.
		assert_eq!(src_br, encoder.brotli_path());
		assert_eq!(src_gz, encoder.gzip_path());

		// Decode both encoded copies and compare them to the original.
		decode_brotli(&src_br);
		decode_gzip(&src_gz);

		// Clean up.
		let _res = std::fs::remove_file(&src);
		let _res = std::fs::remove_file(&src_br);
		let _res = std::fs::remove_file(&src_gz);
	}
}
