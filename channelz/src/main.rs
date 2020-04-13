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
	Error,
	Result,
	traits::path::FYIPathIO,
	Witch,
};
use std::{
	fs::File,
	path::PathBuf,
};



fn main() -> Result<()> {
	// Command line arguments.
	let opts: ArgMatches = menu::menu()
		.get_matches();

	// What path are we dealing with?
	let walk: Witch = match opts.is_present("list") {
		false => {
			let paths: Vec<PathBuf> = opts.values_of("path").unwrap()
				.into_iter()
				.map(|x| PathBuf::from(x))
				.collect();

			Witch::new(
				&paths,
				Some(r"(?i).+\.(css|x?html?|ico|m?js|json|svg|txt|xml|xsl)$".to_string())
			)
		},
		true => {
			let path = PathBuf::from(opts.value_of("list").unwrap_or(""));
			Witch::from_file(
				&path,
				Some(r"(?i).+\.(css|x?html?|ico|m?js|json|svg|txt|xml|xsl)$".to_string())
			)
		},
	};

	if walk.is_empty() {
		return Err(Error::Other("No encodable files were found.".to_string()));
	}

	// With progress.
	if opts.is_present("progress") {
		walk.progress("ChannelZ", |x| {
			let _ = x.encode().is_ok();
		});
	}
	// Without progress.
	else {
		walk.process(|ref x| {
			let _ = x.encode().is_ok();
		});
	}

	Ok(())
}

/// Encoding!
pub trait ChannelZEncode {
	/// Encode.
	fn encode(&self) -> Result<()>;
}

impl ChannelZEncode for PathBuf {
	/// Encode.
	fn encode(&self) -> Result<()> {
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
fn encode_br(data: &[u8], base: &str) -> Result<()> {
	let mut output = File::create(PathBuf::from([base, ".br"].concat()))?;

	let mut encoder = compu::compressor::write::Compressor::new(
		BrotliEncoder::default(),
		&mut output
	);

	encoder.push(&data, EncoderOp::Finish)?;

	Ok(())
}

/// Gzip business.
fn encode_gz(data: &[u8], base: &str) -> Result<()> {
	let mut output = File::create(PathBuf::from([base, ".gz"].concat()))?;

	let mut encoder = compu::compressor::write::Compressor::new(
		ZlibEncoder::default(),
		&mut output
	);

	encoder.push(&data, EncoderOp::Finish)?;

	Ok(())
}
