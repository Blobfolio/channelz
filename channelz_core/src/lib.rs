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
	fmt,
	fs::{
		self,
		File,
	},
	io::Write,
	os::unix::ffi::OsStrExt,
	path::{
		Path,
		PathBuf,
	},
};



/// # Convenience Method.
///
/// This will try to encode any path-like source. It is equivalent to
/// instantiating via [`ChannelZ::try_from`] and running [`ChannelZ::encode`].
pub fn encode_path<P>(src: P)
where P: AsRef<Path> {
	let src = src.as_ref().to_path_buf();
	if let Ok(mut enc) = ChannelZ::try_from(&src) {
		enc.encode();
	}
}



#[derive(Debug, Copy, Clone)]
/// # Error.
pub enum ChannelZError {
	/// # Empty file.
	EmptyFile,
	/// # Unable to read file.
	Read,
}

impl fmt::Display for ChannelZError {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl std::error::Error for ChannelZError {}

impl ChannelZError {
	#[must_use]
	/// # As Str.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::EmptyFile => "The file is empty.",
			Self::Read => "Unable to read the file.",
		}
	}
}



#[derive(Debug)]
/// # `ChannelZ`
///
/// This struct is used to compress a given file using Brotli and Gzip.
pub struct ChannelZ {
	raw: Box<[u8]>,
	buf: Vec<u8>,
	dst: Box<[u8]>,
	size_br: u64,
	size_gz: u64,
}

impl TryFrom<&PathBuf> for ChannelZ {
	type Error = ChannelZError;

	fn try_from(src: &PathBuf) -> Result<Self, Self::Error> {
		// Read the file.
		let raw = fs::read(src).map_err(|_| ChannelZError::Read)?.into_boxed_slice();
		if raw.is_empty() {
			return Err(ChannelZError::EmptyFile);
		}

		Ok(Self {
			buf: Vec::with_capacity(raw.len()),
			raw,
			dst: [src.as_os_str().as_bytes(), b".br"].concat().into_boxed_slice(),
			size_br: 0,
			size_gz: 0,
		})
	}
}

impl ChannelZ {
	#[inline]
	/// # Encode!
	pub fn encode(&mut self) {
		if ! self.encode_br() { self.delete_if(); }
		if ! self.encode_gz() { self.delete_if(); }
	}

	#[must_use]
	/// # Sizes.
	///
	/// Return the original size, the Brotli size, and the Gzip size. In cases
	/// where Brotli and/or Gzip didn't run, the original size will be
	/// returned in their place.
	pub const fn sizes(&self) -> (u64, u64, u64) {
		let size_src = self.raw.len() as u64;
		let size_br =
			if 0 < self.size_br { self.size_br }
			else { size_src };
		let size_gz =
			if 0 < self.size_gz { self.size_gz }
			else { size_src };

		(size_src, size_br, size_gz)
	}

	/// # Encode Brotli.
	fn encode_br(&mut self) -> bool {
		use compu::{
			compressor::write::Compressor,
			encoder::{
				Encoder,
				EncoderOp,
				BrotliEncoder,
			},
		};

		let mut writer = Compressor::new(BrotliEncoder::default(), &mut self.buf);
		if let Ok(len) = writer.push(&self.raw, EncoderOp::Finish) {
			if 0 < len && len < self.raw.len() {
				self.size_br = len as u64;
				return self.write();
			}
		}

		false
	}

	/// # Encode Gzip.
	fn encode_gz(&mut self) -> bool {
		use libdeflater::{
			CompressionLvl,
			Compressor,
		};

		let mut writer = Compressor::new(CompressionLvl::best());
		self.buf.resize(writer.gzip_compress_bound(self.raw.len()), 0);

		// Update the destination path extension.
		{
			let len: usize = self.dst.len();
			self.dst[len - 2] = b'g';
			self.dst[len - 1] = b'z';
		}

		if let Ok(len) = writer.gzip_compress(&self.raw, &mut self.buf) {
			if 0 < len && len < self.raw.len() {
				// Libdeflater does not automatically truncate the buffer to
				// the final payload size, so we need to do that before trying
				// to write the data to a file.
				self.buf.truncate(len);
				self.size_gz = len as u64;
				return self.write();
			}
		}

		false
	}

	#[cold]
	/// # Delete If (File Exists).
	///
	/// We probably don't need to explicitly check the file exists, but it is
	/// unclear how the underlying `unlink()` varies from system to system.
	///
	/// This method returns no result and suppresses any errors encountered as
	/// there's not really anything more to be done. If the file doesn't exist,
	/// it doesn't need to be deleted; if it does and can't be deleted, well,
	/// we tried.
	fn delete_if(&self) {
		let path = Path::new(OsStr::from_bytes(&self.dst));
		if path.exists() {
			let _res = std::fs::remove_file(path);
		}
	}

	/// # Write Result.
	///
	/// Write the buffer to an actual file.
	fn write(&self) -> bool {
		File::create(OsStr::from_bytes(&self.dst))
			.and_then(|mut file| file.write_all(&self.buf).and_then(|_| file.flush()))
			.is_ok()
	}
}
