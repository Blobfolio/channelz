/*!
# ChannelZ

Nothing but static…

Use ChannelZ to generate maximally-compressed Gzip- and Brotli-encoded copies
of a file or recurse a directory to do it for many files at once.
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

extern crate clap;
extern crate compu;
extern crate fyi_core;
extern crate rayon;

mod menu;

use clap::ArgMatches;
use compu::encoder::{
	Encoder,
	EncoderOp,
	BrotliEncoder,
	ZlibEncoder,
};
use fyi_core::{
	Msg,
	Prefix,
	Progress,
	progress_arc,
	PROGRESS_CLEAR_ON_FINISH,
};
use fyi_core::witcher::{
	self,
	mass::FYIMassOps,
	ops::FYIOps,
};
use rayon::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use std::collections::HashSet;



fn main() -> Result<(), String> {
	// Command line arguments.
	let opts: ArgMatches = menu::menu()
		.get_matches();

	let pattern = witcher::pattern_to_regex(r"(?i).+\.(css|x?html?|ico|m?js|json|svg|txt|xml|xsl)$");

	// What path are we dealing with?
	let paths: HashSet<PathBuf> = match opts.is_present("list") {
		false => opts.values_of("path").unwrap()
			.into_iter()
			.filter_map(|x| Some(PathBuf::from(x)))
			.collect::<HashSet<PathBuf>>()
			.fyi_walk_filtered(&pattern),
		true => PathBuf::from(opts.value_of("list").unwrap_or(""))
			.fyi_walk_file_lines(Some(pattern))
			.into_iter()
			.collect::<HashSet<PathBuf>>(),
	};

	if paths.is_empty() {
		return Err("No encodable files were found.".to_string());
	}

	// With progress.
	if opts.is_present("progress") {
		let time = Instant::now();
		let found: u64 = paths.len() as u64;

		{
			let bar = Progress::new(
				Msg::new("Reticulating splines…")
					.with_prefix(Prefix::Custom("ChannelZ", 199))
					.to_string(),
				found,
				PROGRESS_CLEAR_ON_FINISH
			);
			let looper = progress_arc::looper(&bar, 60);
			paths.par_iter().for_each(|ref x| {
				progress_arc::add_working(&bar, &x);
				let _ = x.encode().is_ok();
				progress_arc::update(&bar, 1, None, Some(x.to_path_buf()));
			});
			progress_arc::finish(&bar);
			looper.join().unwrap();
		}

		Msg::msg_crunched_in(found, time, None)
			.print();
	}
	// Without progress.
	else {
		paths.par_iter().for_each(|ref x| {
			let _ = x.encode().is_ok();
		});
	}

	Ok(())
}

/// Encoding!
pub trait ChannelZEncode {
	/// Encode.
	fn encode(&self) -> Result<(), String>;
}

impl ChannelZEncode for PathBuf {
	/// Encode.
	fn encode(&self) -> Result<(), String> {
		// Load the full file contents as we'll need to reference it twice.
		let data = self.fyi_read()?;
		if false == data.is_empty() {
			// The base name won't be changing, so let's grab that too.
			let base = self.to_str().unwrap_or("");

			// MORE PARALLEL!
			let _ = rayon::join(
				|| encode_br(&data, &base),
				|| encode_gz(&data, &base),
			);
		}

		Ok(())
	}
}

/// Brotli business.
fn encode_br(data: &[u8], base: &str) -> Result<(), String> {
	let mut output = File::create(PathBuf::from(format!("{}.br", &base)))
		.map_err(|e| e.to_string())?;

	let mut encoder = compu::compressor::write::Compressor::new(
		BrotliEncoder::default(),
		&mut output
	);

	encoder.push(&data, EncoderOp::Finish)
		.map_err(|e| e.to_string())?;

	Ok(())
}

/// Gzip business.
fn encode_gz(data: &[u8], base: &str) -> Result<(), String> {
	let mut output = File::create(PathBuf::from(format!("{}.gz", &base)))
		.map_err(|e| e.to_string())?;

	let mut encoder = compu::compressor::write::Compressor::new(
		ZlibEncoder::default(),
		&mut output
	);

	encoder.push(&data, EncoderOp::Finish)
		.map_err(|e| e.to_string())?;

	Ok(())
}
