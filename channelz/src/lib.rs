/*!
# `ChannelZ`: The Hard Bits
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unknown_clippy_lints)]

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

#[allow(trivial_casts)] // It is better this way.
#[allow(unused_must_use)] // We don't care.
/// Encode `Brotli`.
pub fn encode_br(path: &PathBuf) {
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

#[allow(trivial_casts)] // It is better this way.
#[allow(unused_must_use)] // We don't care.
/// Encode `GZip`.
pub fn encode_gz(path: &PathBuf) {
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
