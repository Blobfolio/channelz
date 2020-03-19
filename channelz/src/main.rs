/*!
# ChannelZ

Generate static Brotli and Gzip encodings of a file or directory.
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
extern crate regex;
extern crate walkdir;

use brotli::enc::backward_references::BrotliEncoderParams;
use clap::Shell;
use flate2::Compression;
use flate2::write::GzEncoder;
use rayon::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::{copy, BufReader, SeekFrom};
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use regex::Regex;



fn main() -> Result<(), String> {
	let opts: clap::ArgMatches = menu()
		.get_matches();

	if opts.is_present("completions") {
		menu().gen_completions_to(
			"channelz",
			Shell::Bash,
			&mut io::stdout()
		);
		std::process::exit(0);
	}

	let path = PathBuf::from(opts.value_of("path").unwrap_or("/none"));
	let mut exts: Vec<String> = vec![
		".css".to_string(),
		".html".to_string(),
		".ico".to_string(),
		".js".to_string(),
		".json".to_string(),
		".mjs".to_string(),
		".svg".to_string(),
		".xml".to_string(),
	];

	if path.is_dir() {
		if let Some(x) = opts.values_of("ext") {
			let mut raw: Vec<String> = x.filter_map(|y| {
				let z: String = y.to_string()
					.trim()
					.to_lowercase()
					.trim_start_matches(".")
					.to_string();

				if 0 < z.len() {
					Some(format!(".{}", z))
				}
				else {
					None
				}
			}).collect();

			if 0 < raw.len() {
				if 1 < raw.len() {
					raw.sort();
					raw.dedup();
				}
				exts = raw;
			}
		}

		// Go ahead and clean.
		if opts.is_present("clean") {
			clean_dir(path.to_path_buf(), &exts);
		}
	}

	let exts: Regex = ext_regex(&exts);
	if let Ok(paths) = file_list(path.to_path_buf(), &exts) {
		let params = BrotliEncoderParams::default();

		paths.into_par_iter().for_each(|x| {
			if let Err(_) = encode(x.to_path_buf(), &params) {}
		});
	}
	else {
		return Err("No files were compressed.".to_string());
	}

	Ok(())
}

/// Clean Directory.
fn clean_dir(path: PathBuf, exts: &Vec<String>) {
	if path.is_dir() {
		// Make Brotli/Gzip versions of the extensions provided.
		let mut exts2: Vec<String> = vec![];
		for i in exts {
			exts2.push(format!("{}.br", i));
			exts2.push(format!("{}.gz", i));
		}

		let exts2: Regex = ext_regex(&exts2);
		for i in WalkDir::new(&path)
			.follow_links(true)
			.into_iter() {
				if let Ok(path) = i {
					let path = path.path();
					if true == path.channelz_ext_one_of(&exts2) {
						path.channelz_unlink();
					}
				}
			}
	}
}

/// Encode file!
fn encode(path: PathBuf, params: &BrotliEncoderParams) -> Result<(), String> {
	// We can re-use the input.
	let mut input = File::open(path.to_path_buf()).map_err(|e| e.to_string())?;

	// Brotli business.
	let mut output = path.channelz_create("br".to_string());
	brotli::BrotliCompress(&mut input, &mut output, &params).map_err(|e| e.to_string())?;

	// Rewind.
	input.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
	let mut input = BufReader::new(&input);

	// Gzip business.
	let output = path.channelz_create("gz".to_string());
    let mut encoder = GzEncoder::new(output, Compression::new(9));
    copy(&mut input, &mut encoder).map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())?;

	Ok(())
}

/// Extension Regex.
fn ext_regex(exts: &Vec<String>) -> Regex {
	let re: Regex = Regex::new(r"^[a-z\.]+$").unwrap();

	let esc: Vec<String> = exts.into_iter().filter_map(|ref x| {
		if re.is_match(x) {
			Some(x.replace(".", "\\."))
		}
		else {
			None
		}
	}).collect();

	let esc = &format!("(?i)({})$", esc.join("|"));
	Regex::new(esc).unwrap()
}

/// Come up with compressable files.
fn file_list(path: PathBuf, exts: &Regex) -> Result<Vec<PathBuf>, String> {
	// One file requires no recursion.
	if path.is_file() {
		if true == path.channelz_ext_one_of(&exts) {
			return Ok(vec![path.to_path_buf()]);
		}
		else {
			return Err("Invalid path.".into());
		}
	}
	// If it's a directory, let's peek!
	else if path.is_dir() {
		let paths: Vec<PathBuf> = WalkDir::new(&path)
			.follow_links(true)
			.into_iter()
			.filter_map(|x| match x {
				Ok(path) => {
					let path = path.path();
					match path.channelz_ext_one_of(&exts) {
						true => Some(path.to_path_buf()),
						false => None,
					}
				},
				_ => None,
			})
			.collect();

		if 0 < paths.len() {
			return Ok(paths);
		}
	}

	Err("Invalid path.".into())
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
	/// Append extension (correctly).
	fn channelz_append_ext(&self, ext: String, clean: bool) -> PathBuf;

	/// Create file.
	fn channelz_create(&self, ext: String) -> File;

	/// Has One Of Exts.
	fn channelz_ext_one_of(&self, exts: &Regex) -> bool;

	/// Unlink.
	fn channelz_unlink(&self) -> bool;
}

impl PathFuckery for Path {
	/// Append extension (correctly).
	fn channelz_append_ext(&self, ext: String, clean: bool) -> PathBuf {
		let src = self.to_str().unwrap_or("");
		let out = format!("{}.{}", src, ext);
		let out = PathBuf::from(out);

		if true == clean && out.is_file() {
			out.channelz_unlink();
		}

		return out;
	}

	/// Create file.
	fn channelz_create(&self, ext: String) -> File {
		let out = self.channelz_append_ext(ext, true);
		File::create(out).expect("That didn't work!")
	}

	/// Has One Of Exts.
	fn channelz_ext_one_of(&self, exts: &Regex) -> bool {
		if self.is_file() {
			if let Some(name) = self.file_name() {
				return exts.is_match(name.to_str().unwrap_or(""));
			}
		}

		return false;
	}

	/// Unlink.
	fn channelz_unlink(&self) -> bool {
		if self.is_file() {
			std::fs::remove_file(&self).is_ok()
		}
		else {
			false
		}
	}
}
