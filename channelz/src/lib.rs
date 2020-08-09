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
	ffi::{
		OsStr,
		OsString,
	},
	fs::{
		self,
		File,
	},
	os::unix::ffi::OsStringExt,
	path::PathBuf,
};



/// Do the dirty work!
pub fn encode_path(path: &PathBuf) {
	let _ = rayon::join(
		|| encode_br(path),
		|| encode_gz(path),
	);
}

#[allow(unused_must_use)]
/// Encode Brotli.
pub fn encode_br(path: &PathBuf) {
	// It is more efficient to calculate the output path from OsString than
	// Path or PathBuf since every goddamn Path join/concat-type method adds a
	// separator.
	let out_path: OsString = unsafe {
		OsString::from_vec(
			[
				&*(path.as_os_str() as *const OsStr as *const [u8]),
				b".br",
			].concat()
		)
	};

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
/// Encode GZip.
pub fn encode_gz(path: &PathBuf) {
	// It is more efficient to calculate the output path from OsString than
	// Path or PathBuf since every goddamn Path join/concat-type method adds a
	// separator.
	let out_path: OsString = unsafe {
		OsString::from_vec(
			[
				&*(path.as_os_str() as *const OsStr as *const [u8]),
				b".gz",
			].concat()
		)
	};

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
