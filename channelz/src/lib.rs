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
use std::{
	fs::{
		self,
		File,
	},
	path::PathBuf,
};



// Do the dirty work!
pub fn encode_path(path: &PathBuf) {
	if let Some(stub) = path.to_str() {
		if let Ok(data) = &fs::read(path) {
			let _ = rayon::join(
				|| encode_br(stub, data),
				|| encode_gz(stub, data),
			);
		}
	}
}

#[allow(unused_must_use)]
pub fn encode_br(stub: &str, data: &[u8]) {
	if let Ok(mut output) = File::create([stub, ".br"].concat()) {
		let mut writer = Compressor::new(BrotliEncoder::default(), &mut output);
		writer.push(data, EncoderOp::Finish);
	}
}

#[allow(unused_must_use)]
pub fn encode_gz(stub: &str, data: &[u8]) {
	if let Ok(mut output) = File::create([stub, ".gz"].concat()) {
		let mut writer = Compressor::new(ZlibEncoder::default(), &mut output);
		writer.push(data, EncoderOp::Finish);
	}
}
