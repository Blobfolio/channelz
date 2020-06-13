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
	if let Ok(data) = &fs::read(path) {
		if ! data.is_empty() {
			let _ = rayon::join(
				|| encode_br(path, data),
				|| encode_gz(path, data),
			);
		}
	}
}

#[allow(unused_must_use)]
pub fn encode_br(path: &PathBuf, data: &[u8]) {
	if let Ok(mut output) = File::create({
		let mut p = path.to_path_buf().into_os_string();
		p.reserve_exact(3);
		p.push(".br");
		p
	}) {
		let mut writer = Compressor::new(BrotliEncoder::default(), &mut output);
		writer.push(data, EncoderOp::Finish);
	}
}

#[allow(unused_must_use)]
pub fn encode_gz(path: &PathBuf, data: &[u8]) {
	if let Ok(mut output) = File::create({
		let mut p = path.to_path_buf().into_os_string();
		p.reserve_exact(3);
		p.push(".gz");
		p
	}) {
		let mut writer = Compressor::new(ZlibEncoder::default(), &mut output);
		writer.push(data, EncoderOp::Finish);
	}
}
