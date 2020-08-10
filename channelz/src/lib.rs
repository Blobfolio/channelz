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



use std::{
	ffi::{
		OsStr,
		OsString,
	},
	fs::{
		self,
		File,
	},
	io::Write,
	os::unix::ffi::{
		OsStrExt,
		OsStringExt,
	},
	path::PathBuf,
	sync::Arc,
};



/// Do the dirty work!
pub fn encode_path(path: &PathBuf) {
	let a_data: Arc<Vec<u8>> = Arc::from(fs::read(path).unwrap_or_default());
	if ! a_data.is_empty() {
		rayon::join(
			|| encode_br(path, &a_data),
			|| encode_gz(path, &a_data),
		);
	}
}

#[allow(trivial_casts)] // It is better this way.
#[allow(unused_must_use)] // We don't care.
/// Encode `Brotli`.
pub fn encode_br(path: &PathBuf, data: &Arc<Vec<u8>>) {
	use compu::{
		compressor::write::Compressor,
		encoder::{
			Encoder,
			EncoderOp,
			BrotliEncoder,
		},
	};

	// Calculate the output path.
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

		// Stream-write to the file. If it fails, we'll need to delete the file
		// we just created.
		if 0 == writer.push(data, EncoderOp::Finish).unwrap_or_default() {
			drop(output);
			fs::remove_file(out_path);
		}
	}
}

#[allow(trivial_casts)] // It is better this way.
/// Encode `GZip`.
pub fn encode_gz(path: &PathBuf, data: &Arc<Vec<u8>>) {
	use libdeflater::{
		CompressionLvl,
		Compressor,
	};

	// This compresses to memory.
	let compressed: Vec<u8> = {
		let mut writer = Compressor::new(CompressionLvl::best());
		let mut tmp = Vec::new();
		tmp.resize(writer.gzip_compress_bound(data.len()), 0);

		match writer.gzip_compress(data, &mut tmp) {
			Ok(len) if len > 0 => {
				tmp.resize(len, 0);
				tmp
			},
			_ => { return; }
		}
	};

	// Write what needs writing.
	if let Ok(mut output) = File::create(unsafe {
		OsStr::from_bytes(
			&[
				&*(path.as_os_str() as *const OsStr as *const [u8]),
				b".gz",
			].concat()
		)
	}) {
		output.write_all(&compressed).unwrap();
		output.flush().unwrap();
	}
}
