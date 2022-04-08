/*!
# `ChannelZ`: Encoder
*/

use std::{
	ffi::OsStr,
	fs::File,
	io::{
		Read,
		Write,
	},
	os::unix::ffi::OsStrExt,
	path::Path,
};



#[derive(Debug, Clone, Default)]
/// # Encoder.
pub(super) struct Encoder {
	raw: Vec<u8>,
	path: Vec<u8>,
	buf: Vec<u8>,
}

impl Encoder {
	/// # New Instance.
	///
	/// This will return `None` if the file cannot be read, is empty, or too
	/// big. Otherwise it will return a new `Encoder`.
	pub(super) fn new(src: &Path) -> Option<Self> {
		let raw = std::fs::read(src).ok()?;
		if raw.is_empty() { None }
		else {
			#[cfg(target_pointer_width = "128")]
			if raw.len() > 18_446_744_073_709_551_615 { return None; }

			Some(Self {
				raw,
				path: [src.as_os_str().as_bytes(), b".gz"].concat(),
				buf: Vec::new(),
			})
		}
	}

	/// # Encode.
	///
	/// Encode the file with Brotli and Gzip, and return all the lengths
	/// (raw, brotli, gzip).
	///
	/// If either Brotli or Gzip do not work out, their "size" will be
	/// represented using the original size to indicate no savings.
	pub(super) fn encode(&mut self) -> (u64, u64, u64) {
		let len = self.raw.len();

		// Try to encode Gzip.
		let len_gz = self.encode_gzip().unwrap_or(len);

		// Update the path for Brotli.
		let path_len = self.path.len();
		self.path[path_len - 2..].copy_from_slice(b"br");

		// Try to encode Brotli.
		let len_br = self.encode_brotli().unwrap_or(len);

		// Done!
		(len as u64, len_br as u64, len_gz as u64)
	}

	/// # Encode With.
	///
	/// This works just like `Encode`, but recycles the instance's existing
	/// buffers, (hopefully) reducing the number of allocations being made.
	pub(super) fn encode_with(&mut self, src: &Path) -> Option<(u64, u64, u64)> {
		if let Ok(mut f) = File::open(src) {
			self.raw.truncate(0);
			if let Ok(len) = f.read_to_end(&mut self.raw) {
				#[cfg(target_pointer_width = "128")]
				if 0 == len || len > 18_446_744_073_709_551_615 { return None; }

				#[cfg(not(target_pointer_width = "128"))]
				if len == 0 { return None; }

				// Set up the path for Gzip.
				self.path.truncate(0);
				self.path.extend_from_slice(src.as_os_str().as_bytes());
				self.path.extend_from_slice(b".gz");

				// Encode!
				return Some(self.encode());
			}
		}

		None
	}

	/// # Encode Brotli.
	fn encode_brotli(&mut self) -> Option<usize> {
		use compu::{
			compressor::write::Compressor,
			encoder::{
				Encoder,
				EncoderOp,
				BrotliEncoder,
			},
		};

		// Set up the buffer/writer.
		self.buf.truncate(0);
		let mut writer = Compressor::new(BrotliEncoder::default(), &mut self.buf);

		// Encode!
		if let Ok(len) = writer.push(&self.raw, EncoderOp::Finish) {
			// Save it?
			if 0 < len && len < self.raw.len() && self.write() {
				return Some(len);
			}
		}

		// Clean up.
		self.remove_if();
		None
	}

	/// # Encode Gzip.
	fn encode_gzip(&mut self) -> Option<usize> {
		use libdeflater::{
			CompressionLvl,
			Compressor,
		};

		// Set up the buffer/writer.
		let old_len = self.raw.len();
		let mut writer = Compressor::new(CompressionLvl::best());
		self.buf.resize(writer.gzip_compress_bound(old_len), 0);

		// Encode!
		if let Ok(len) = writer.gzip_compress(&self.raw, &mut self.buf) {
			if 0 < len && len < old_len {
				self.buf.truncate(len);
				if self.write() {
					return Some(len);
				}
			}
		}

		// Clean up.
		self.remove_if();
		None
	}

	/// # Remove If It Exists.
	///
	/// This method is used to clean up previously-encoded copies of a file when
	/// the current encoding operation fails.
	///
	/// We can't do anything if deletion fails, but at least we can say we tried.
	fn remove_if(&self) {
		let path = Path::new(OsStr::from_bytes(&self.path));
		if path.exists() {
			let _res = std::fs::remove_file(path);
		}
	}

	/// # Write Result.
	///
	/// Write the buffer to an actual file.
	fn write(&self) -> bool {
		File::create(OsStr::from_bytes(&self.path))
			.and_then(|mut file| file.write_all(&self.buf).and_then(|_| file.flush()))
			.is_ok()
	}
}
