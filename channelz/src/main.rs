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
use compu::encoder::{Encoder, EncoderOp, BrotliEncoder, ZlibEncoder};
use fyi_core::{
	Progress,
	progress_arc,
	witcher
};
use fyi_core::witcher::mass::FYIMassOps;
use fyi_core::witcher::ops::FYIOps;
use rayon::prelude::*;
use std::fs::File;
use std::path::PathBuf;



fn main() -> Result<(), String> {
	// Command line arguments.
	let opts: ArgMatches = menu::menu()
		.get_matches();

	// What path are we dealing with?
	let mut paths: Vec<PathBuf> = opts.values_of("path").unwrap()
		.into_iter()
		.filter_map(|x| Some(PathBuf::from(x)))
		.collect();

	let pattern = witcher::pattern_to_regex(r"(?i)\.(css|x?html?|ico|m?js|json|svg|txt|xml|xsl)$");
	paths.fyi_walk_filtered_mut(&pattern);

	if paths.is_empty() {
		return Err("No HTML files were found.".to_string());
	}

	// Do it with progress.
	if opts.is_present("progress") {
		let bar = Progress::new("", paths.len() as u64, 0);

		paths.into_par_iter().for_each(|ref x| {
			let _ = x.encode().is_ok();
			progress_arc::set_path(bar.clone(), &x);
			progress_arc::increment(bar.clone(), 1);
			progress_arc::tick(bar.clone());
		});

		// Finish progress bar if applicable.
		progress_arc::finish(bar.clone());
	}
	// Do it without.
	else {
		paths.into_par_iter().for_each(|ref x| {
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

		// The base name won't be changing, so let's grab that too.
		let base = self.to_str().unwrap_or("");

		{
			// Brotli business.
			let mut output = File::create(PathBuf::from(format!("{}.br", &base)))
				.map_err(|e| e.to_string())?;

			let mut encoder = compu::compressor::write::Compressor::new(
				BrotliEncoder::default(),
				&mut output
			);

			encoder.push(&data, EncoderOp::Finish)
				.map_err(|e| e.to_string())?;
		}

		{
			// Gzip business.
			let mut output = File::create(PathBuf::from(format!("{}.gz", &base)))
				.map_err(|e| e.to_string())?;

			let mut encoder = compu::compressor::write::Compressor::new(
				ZlibEncoder::default(),
				&mut output
			);

			encoder.push(&data, EncoderOp::Finish)
				.map_err(|e| e.to_string())?;
		}

		Ok(())
	}
}
