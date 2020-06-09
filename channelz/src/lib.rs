/*!
# `ChannelZ`: The Hard Bits
*/

use compu::{
	compressor::write::Compressor,
	encoder::{
		Encoder,
		EncoderOp,
		BrotliEncoder,
		ZlibEncoder,
	},
};
use fyi_witcher::Result;
use std::{
	fs::{
		self,
		File,
	},
	path::Path,
};



/// Encode!
pub trait EncodeFile {
	/// Encode File
	///
	/// Create a Brotli- and Gzip-encoded copy of `Self`, saving each as
	/// `Self.br` and `Self.gz` respectively.
	fn encode_all(&self);

	// Encode To.
	fn encode_to<E, P> (path: P, enc: E, data: &[u8]) -> Result<()>
	where E: Encoder,
	P: AsRef<Path>;
}

impl EncodeFile for Path {
	/// Encode File
	///
	/// Create a Brotli- and Gzip-encoded copy of `Self`, saving each as
	/// `Self.br` and `Self.gz` respectively.
	fn encode_all(&self) {
		// Gotta have a name.
		if let Some(stub) = self.to_str() {
			// Come up with the path names. This is rather terrible because
			// `Path` is rather terrible. Haha.
			let brp: String = stub.chars()
				.chain(".br".chars()) // .br
				.collect();

			let mut gzp: String = brp.clone();
			gzp.replace_range(stub.len().., ".gz");

			// Now make sure we have data.
			if let Ok(data) = fs::read(&self) {
				let _ = rayon::join(
					|| Self::encode_to(&brp, BrotliEncoder::default(), &data),
					|| Self::encode_to(&gzp, ZlibEncoder::default(), &data),
				);
			}
		}
	}

	// Encode To.
	fn encode_to<E, P> (path: P, enc: E, data: &[u8]) -> Result<()>
	where E: Encoder,
	P: AsRef<Path> {
		let mut output = File::create(path).map_err(|e| e.to_string())?;
		let mut writer = Compressor::new(enc, &mut output);
		writer.push(data, EncoderOp::Finish).map_err(|e| e.to_string())?;
		Ok(())
	}
}
