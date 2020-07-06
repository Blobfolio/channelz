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
	let _ = rayon::join(
		|| encode_br(path),
		|| encode_gz(path),
	);
}

#[allow(unused_must_use)]
pub fn encode_br(path: &PathBuf) {
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
		if 0 == writer.push(&fs::read(path).unwrap_or_default(), EncoderOp::Finish).unwrap_or_default() {
			drop(output);
			fs::remove_file(out_path);
		}
	}
}

#[allow(unused_must_use)]
pub fn encode_gz(path: &PathBuf) {
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
		if 0 == writer.push(&fs::read(path).unwrap_or_default(), EncoderOp::Finish).unwrap_or_default() {
			drop(output);
			fs::remove_file(out_path);
		}
	}
}
