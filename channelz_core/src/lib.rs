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

#![allow(clippy::module_name_repetitions)]



#[cfg(test)] use brunch as _;
use std::{
	convert::TryFrom,
	ffi::OsStr,
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



#[derive(Debug)]
/// # `ChannelZ`
///
/// This struct is used to compress a given file using Brotli and Gzip.
pub struct ChannelZ {
	raw: Box<[u8]>,
	buf: Vec<u8>,
	dst: Box<[u8]>,
}

impl TryFrom<&PathBuf> for ChannelZ {
	type Error = ();

	#[allow(trivial_casts)] // This is how `std::path::PathBuf` does it.
	fn try_from(src: &PathBuf) -> Result<Self, Self::Error> {
		// Read the file.
		if let Ok(raw) = fs::read(src) {
			let len = raw.len();
			if 0 != len {
				return Ok(Self {
					raw: raw.into_boxed_slice(),
					buf: Vec::with_capacity(len),
					dst: [
						unsafe { &*(src.as_os_str() as *const OsStr as *const [u8]) },
						b".br",
					].concat().into_boxed_slice(),
				});
			}
		}

		Err(())
	}
}

impl ChannelZ {
	#[inline]
	/// # Encode!
	pub fn encode(&mut self) {
		self.encode_br();
		self.encode_gz();
	}

	/// # Encode Brotli.
	fn encode_br(&mut self) {
		use compu::{
			compressor::write::Compressor,
			encoder::{
				Encoder,
				EncoderOp,
				BrotliEncoder,
			},
		};

		let mut writer = Compressor::new(BrotliEncoder::default(), &mut self.buf);
		if let Ok(x) = writer.push(&self.raw, EncoderOp::Finish) {
			if 0 < x && x < self.raw.len() {
				self.write();
				return;
			}
		}

		self.delete_if();
	}

	/// # Encode Gzip.
	fn encode_gz(&mut self) {
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

		if let Ok(x) = writer.gzip_compress(&self.raw, &mut self.buf) {
			if 0 < x && x < self.raw.len() {
				// Libdeflater does not automatically truncate the buffer to
				// the payload size.
				self.buf.truncate(x);
				self.write();
				return;
			}
		}

		self.delete_if();
	}

	#[cold]
	/// # Delete If (File Exists).
	///
	/// We probably don't need to explicitly check the file exists, but it is
	/// unclear how the underlying `unlink()` varies from system to system.
	fn delete_if(&self) {
		let path = PathBuf::from(OsStr::from_bytes(&self.dst));
		if path.exists() {
			let _res = std::fs::remove_file(path);
		}
	}

	/// # Write Result.
	///
	/// Write the buffer to an actual file.
	fn write(&self) {
		// The result doesn't matter. It'll work or it won't.
		let _res = File::create(OsStr::from_bytes(&self.dst))
			.and_then(|mut file| file.write_all(&self.buf).and_then(|_| file.flush()));
	}
}
