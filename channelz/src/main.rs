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
	Witch,
};
use std::{
	borrow::Cow,
	fs::{
		self,
		File,
	},
	path::PathBuf,
};



fn main() -> Result<()> {
	// Command line arguments.
	let opts: ArgMatches = menu::menu()
		.get_matches();

	// What path are we dealing with?
	let walk: Witch = match opts.is_present("list") {
		false => Witch::new(
			&opts.values_of("path")
				.unwrap()
				.collect::<Vec<&str>>(),
			Some(r"(?i).+\.(css|x?html?|ico|m?js|json|svg|txt|xml|xsl)$".to_string()),
		),
		true => Witch::from_file(
			opts.value_of("list").unwrap_or(""),
			Some(r"(?i).+\.(css|x?html?|ico|m?js|json|svg|txt|xml|xsl)$".to_string()),
		),
	};

	if walk.is_empty() {
		return Err(Error::NoPaths("encodable files".into()));
	}

	// With progress.
	if opts.is_present("progress") {
		walk.progress("ChannelZ", |ref x| {
			let _ = encode(x).is_ok();
		});
	}
	// Without progress.
	else {
		walk.process(|ref x| {
			let _ = encode(x).is_ok();
		});
	}

	Ok(())
}

/// Encode.
fn encode(path: &PathBuf) -> Result<()> {
	// Load the full file contents as we'll need to reference it twice.
	let data: Cow<[u8]> = Cow::Owned(fs::read(&path)?);
	if false == data.is_empty() {
		// The base name won't be changing, so let's grab that too.
		let stub: Cow<str> = Cow::Borrowed(path.to_str().unwrap_or(""));

		// Handle Brotli and Gzip in their own threads.
		let _ = rayon::join(
			|| encode_br(&stub, &data),
			|| encode_gz(&stub, &data),
		);
	}

	Ok(())
}

/// Encode.
fn encode_br(stub: &Cow<str>, data: &Cow<[u8]>) -> Result<()> {
	let mut output = File::create({
		let mut p: String = String::with_capacity(stub.len() + 3);
		p.push_str(&stub);
		p.push_str(".br");
		p
	})?;

	let mut encoder = compu::compressor::write::Compressor::new(
		BrotliEncoder::default(),
		&mut output
	);

	encoder.push(&data, EncoderOp::Finish)?;
	Ok(())
}

/// Encode.
fn encode_gz(stub: &Cow<str>, data: &Cow<[u8]>) -> Result<()> {
	let mut output = File::create({
		let mut p: String = String::with_capacity(stub.len() + 3);
		p.push_str(&stub);
		p.push_str(".gz");
		p
	})?;

	let mut encoder = compu::compressor::write::Compressor::new(
		ZlibEncoder::default(),
		&mut output
	);

	encoder.push(&data, EncoderOp::Finish)?;
	Ok(())
}
