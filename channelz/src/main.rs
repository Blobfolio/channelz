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

extern crate brotli;
extern crate clap;
extern crate flate2;
extern crate rayon;
extern crate regex;
extern crate walkdir;

use brotli::enc::backward_references::BrotliEncoderParams;
use clap::Shell;
use flate2::Compression;
use flate2::write::GzEncoder;
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::io::stdout;
use std::path::{Path, PathBuf};
use std::process::exit;
use walkdir::WalkDir;



fn main() -> Result<(), String> {
	// Command line arguments.
	let opts: clap::ArgMatches = menu()
		.get_matches();

	// Generate completions and exit.
	if opts.is_present("completions") {
		menu().gen_completions_to(
			"channelz",
			Shell::Bash,
			&mut stdout()
		);
		exit(0);
	}

	// What path are we dealing with?
	let path: PathBuf = PathBuf::from(opts.value_of("path").expect("A path is required."));

	// Recurse a directory.
	if path.is_dir() {
		let (exts, exts_clean) = init_exts(&opts);

		// Go ahead and clean.
		if opts.is_present("clean") {
			path.channelz_clean(&exts_clean);
		}

		// Loop and compress!
		if let Ok(paths) = path.channelz_find(&exts) {
			paths.into_par_iter().for_each(|ref x| {
				if let Err(_) = x.channelz_encode() {}
			});
		}
	}
	// Just hit one file.
	else if path.is_file() {
		if false == path.channelz_encode().is_ok() {
			return Err("No files were compressed.".to_string());
		}
	}
	// There's nothing to do!
	else {
		return Err("No files were compressed.".to_string());
	}

	Ok(())
}

/// Initialize extension patterns.
fn init_exts(opts: &clap::ArgMatches) -> (Regex, Regex) {
	// Default patterns.
	let mut exts: Regex = Regex::new(r"(?i)\.(css|html|ico|js|json|mjs|svg|xml)$").unwrap();
	let mut exts_clean: Regex = Regex::new(r"(?i)\.(css|html|ico|js|json|mjs|svg|xml)\.(br|gz)$").unwrap();

	// User-supplied patterns.
	if let Some(x) = opts.values_of("ext") {
		let ext: Regex = Regex::new(r"^[A-Za-z]+$").unwrap();

		let mut raw: Vec<String> = x.filter_map(|y| {
			match ext.is_match(y) {
				true => Some(y.to_string().to_lowercase()),
				false => None,
			}
		}).collect();

		if false == raw.is_empty() {
			let inner: String = if 1 < raw.len() {
				raw.par_sort();
				raw.dedup();

				raw.join("|")
			}
			else {
				raw[0].clone()
			};

			exts = Regex::new(&format!("(?i)\\.({})$", inner)).unwrap();
			exts_clean = Regex::new(&format!("(?i)\\.({})\\.(br|gz)$", inner)).unwrap();
		}
	}

	return (exts, exts_clean);
}

/// CLI Menu.
fn menu() -> clap::App<'static, 'static> {
	clap::App::new("ChannelZ")
		.version(env!("CARGO_PKG_VERSION"))
		.author("Blobfolio, LLC. <hello@blobfolio.com>")
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg(clap::Arg::with_name("completions")
			.long("completions")
			.hidden(true)
			.takes_value(false)
		)
		.arg(clap::Arg::with_name("clean")
			.long("clean")
			.takes_value(false)
			.help("Delete any existing .br/.gz files before starting. (Directory mode only.)")
		)
		.arg(clap::Arg::with_name("ext")
			.short("e")
			.long("ext")
			.alias("extensions")
			.takes_value(true)
			.multiple(true)
			.use_delimiter(true)
			.help("Only compress files with these extensions. (Directory mode only.)")
		)
		.arg(clap::Arg::with_name("path")
			.index(1)
			.help("File or directory to compress.")
			.multiple(false)
			.required_unless_one(&["completions"])
			.value_name("PATH")
			.use_delimiter(false)
		)
}

/// Path Helpers
pub trait PathFuckery {
	/// Clean directory.
	fn channelz_clean(&self, exts: &Regex);

	/// Create file.
	fn channelz_create(&self, ext: String) -> File;

	/// Encode file!
	fn channelz_encode(&self) -> Result<(), String>;

	/// Find files.
	fn channelz_find(&self, exts: &Regex) -> Result<Vec<PathBuf>, String>;

	/// Unlink.
	fn channelz_unlink(&self) -> bool;
}

impl PathFuckery for Path {
	/// Clean Directory.
	///
	/// Recurse a directory to remove all existing *.br and *.gz files
	/// matching the extension pattern.
	fn channelz_clean(&self, exts: &Regex) {
		if let Ok(paths) = self.channelz_find(&exts) {
			paths.into_par_iter().for_each(|ref x| {
				x.channelz_unlink();
			});
		}
	}

	/// Create file.
	fn channelz_create(&self, ext: String) -> File {
		let out: PathBuf = PathBuf::from(format!(
			"{}.{}",
			self.to_str().expect("Unable to create path"),
			&ext
		));

		if out.is_file() {
			out.channelz_unlink();
		}

		File::create(out).expect("That didn't work!")
	}

	/// Encode file!
	///
	/// Generate Brotli and Gzip versions of a given file.
	fn channelz_encode(&self) -> Result<(), String> {
		// Load the full file contents as we'll need to reference it twice.
		let data = std::fs::read(&self).expect("Unable to read file.");

		// Brotli business.
		let mut output = self.channelz_create("br".to_string());
		let mut encoder = brotli::CompressorWriter::with_params(
			&mut output,
			4096,
			&BrotliEncoderParams::default()
		);
		encoder.write_all(&data).unwrap();
		encoder.flush().unwrap();

		// Gzip business.
		let output = self.channelz_create("gz".to_string());
		let mut encoder = GzEncoder::new(output, Compression::new(9));
		encoder.write_all(&data).unwrap();
		encoder.finish().unwrap();

		Ok(())
	}

	/// Find files.
	fn channelz_find(&self, exts: &Regex) -> Result<Vec<PathBuf>, String> {
		let paths: Vec<PathBuf> = WalkDir::new(&self)
			.follow_links(true)
			.into_iter()
			.filter_map(|x| match x {
				Ok(path) => {
					if
						path.file_type().is_file() &&
						exts.is_match(path.file_name().to_str().unwrap_or(""))
					{
						Some(path.path().to_path_buf())
					}
					else {
						None
					}
				},
				_ => None,
			})
			.collect();

		match paths.is_empty() {
			false => Ok(paths),
			true => Err("Invalid path.".into())
		}
	}

	/// Unlink.
	fn channelz_unlink(&self) -> bool {
		std::fs::remove_file(&self).is_ok()
	}
}
