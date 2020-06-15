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
	ffi::OsString,
	fs::{
		self,
		File,
	},
	path::PathBuf,
};



// Do the dirty work!
pub fn encode_path(path: &PathBuf) {
	if let Ok(data) = fs::read(path).as_ref() {
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
	// It is more efficient to calculate the output path from OsString than
	// Path or PathBuf since every goddamn Path join/concat-type method adds a
	// separator.
	let mut out_path: OsString = OsString::from(path);
	out_path.reserve_exact(3);
	out_path.push(".br");

	// Create the output file.
	if let Ok(mut output) = File::create(&out_path) {
		let mut writer = Compressor::new(BrotliEncoder::default(), &mut output);

		// Write the data! If nothing is written because of failure or general
		// emptiness, try to delete the file we just created.
		if 0 == writer.push(data, EncoderOp::Finish).unwrap_or_default() {
			drop(output);
			fs::remove_file(out_path);
		}
	}
}

#[allow(unused_must_use)]
pub fn encode_gz(path: &PathBuf, data: &[u8]) {
	// It is more efficient to calculate the output path from OsString than
	// Path or PathBuf since every goddamn Path join/concat-type method adds a
	// separator.
	let mut out_path: OsString = OsString::from(path);
	out_path.reserve_exact(3);
	out_path.push(".gz");

	// Create the output file.
	if let Ok(mut output) = File::create(&out_path) {
		let mut writer = Compressor::new(ZlibEncoder::default(), &mut output);

		// Write the data! If nothing is written because of failure or general
		// emptiness, try to delete the file we just created.
		if 0 == writer.push(data, EncoderOp::Finish).unwrap_or_default() {
			drop(output);
			fs::remove_file(out_path);
		}
	}
}
