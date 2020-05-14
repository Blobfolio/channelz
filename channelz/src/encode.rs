/*!
# `ChannelZ`: Encoding

This mod contains the encoding wrappers.
*/

use compu::encoder::{
	Encoder,
	EncoderOp,
	BrotliEncoder,
	ZlibEncoder,
};
use fyi_witcher::Result;
use std::{
	fs::{
		self,
		File,
	},
	path::PathBuf,
};



/// Encode.
pub fn encode(path: &PathBuf) {
	// Load the full file contents as we'll need to reference it twice.
	if let Ok(data) = fs::read(&path) {
		if ! data.is_empty() {
			if let Some(stub) = path.to_str() {
				// Handle Brotli and Gzip in their own threads.
				let _ = rayon::join(
					|| encode_br(stub, &data),
					|| encode_gz(stub, &data),
				);
			}
		}
	}
}

/// Encode.
pub fn encode_br(stub: &str, data: &[u8]) -> Result<()> {
	let mut output = File::create([stub, ".br"].concat())
		.map_err(|e| e.to_string())?;

	let mut encoder = compu::compressor::write::Compressor::new(
		BrotliEncoder::default(),
		&mut output
	);

	encoder.push(data, EncoderOp::Finish)
		.map_err(|e| e.to_string())?;

	Ok(())
}

/// Encode.
pub fn encode_gz(stub: &str, data: &[u8]) -> Result<()> {
	let mut output = File::create([stub, ".gz"].concat())
		.map_err(|e| e.to_string())?;

	let mut encoder = compu::compressor::write::Compressor::new(
		ZlibEncoder::default(),
		&mut output
	);

	encoder.push(data, EncoderOp::Finish)
		.map_err(|e| e.to_string())?;

	Ok(())
}
