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
extern crate walkdir;
extern crate num_format;

use brotli::enc::backward_references::BrotliEncoderParams;
use clap::Shell;
use flate2::Compression;
use flate2::write::GzEncoder;
use num_format::{Locale, ToFormattedString};
use rayon::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::{copy, BufReader, SeekFrom};
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;



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
		"css".to_string(),
		"html".to_string(),
		"ico".to_string(),
		"js".to_string(),
		"json".to_string(),
		"mjs".to_string(),
		"svg".to_string(),
		"xml".to_string(),
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
					Some(z)
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

	if let Ok(paths) = file_list(path.to_path_buf(), &exts) {
		let params = BrotliEncoderParams::default();

		paths.clone().into_par_iter().for_each(|x| {
			if let Err(_) = encode(x.to_path_buf(), &params) {}
		});

		// Add up all the sizes.
		if opts.is_present("summarize") {
			let mut raw = 0;
			let mut br = 0;
			let mut gz = 0;

			for i in paths.into_iter() {
				if let Ok(meta) = std::fs::metadata(i.to_path_buf()) {
					raw = raw + meta.len();

					let tmp = append_ext(i.to_path_buf(), "br".to_string(), false);
					if let Ok(meta) = std::fs::metadata(tmp) {
						br = br + meta.len();
					}

					let tmp = append_ext(i.to_path_buf(), "gz".to_string(), false);
					if let Ok(meta) = std::fs::metadata(tmp) {
						gz = gz + meta.len();
					}
				}
			}

			let raw: String = raw.to_formatted_string(&Locale::en);
			let br: String = pad_left(br.to_formatted_string(&Locale::en), raw.len(), b' ');
			let gz: String = pad_left(gz.to_formatted_string(&Locale::en), raw.len(), b' ');

			println!("Plain:  {}", raw);
			println!("Gzip:   {}", gz);
			println!("Brotli: {}", br);
		}
	}
	else {
		return Err("No files were compressed.".to_string());
	}

	Ok(())
}

/// Append extension.
fn append_ext(path: PathBuf, ext: String, clean: bool) -> PathBuf {
	let src = path.to_str().unwrap_or("");
	let out = format!("{}.{}", src, ext);
	let out = PathBuf::from(out);

	if true == clean && out.is_file() {
		if let Err(_) = std::fs::remove_file(out.to_path_buf()) {}
	}

	return out;
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

		for i in WalkDir::new(&path)
			.follow_links(true)
			.into_iter() {
				if let Ok(path) = i {
					let path = path.path();
					if true == matches_exts(path.to_path_buf(), &exts2) {
						if let Err(_) = std::fs::remove_file(path) {
						}
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
	let path_out: PathBuf = append_ext(path.to_path_buf(), "br".to_string(), true);
	let mut output = File::create(path_out).map_err(|e| e.to_string())?;
	brotli::BrotliCompress(&mut input, &mut output, &params).map_err(|e| e.to_string())?;

	// Rewind.
	input.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
	let mut input = BufReader::new(input);

	// Gzip business.
	let path_out: PathBuf = append_ext(path.to_path_buf(), "gz".to_string(), true);
	let output = File::create(path_out.to_path_buf()).map_err(|e| e.to_string())?;
    let mut encoder = GzEncoder::new(output, Compression::new(9));
    copy(&mut input, &mut encoder).map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())?;

	Ok(())
}

/// Come up with compressable files.
fn file_list(path: PathBuf, exts: &Vec<String>) -> Result<Vec<PathBuf>, String> {
	// One file requires no recursion.
	if path.is_file() {
		if true == matches_exts(path.to_path_buf(), &exts) {
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
					match matches_exts(path.to_path_buf(), &exts) {
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

/// Matches Exts.
fn matches_exts(path: PathBuf, exts: &Vec<String>) -> bool {
	if false == path.is_file() {
		return false;
	}

	if let Some(name) = path.file_name() {
		let name: String = name.to_str()
			.unwrap_or("")
			.to_string()
			.to_lowercase();

		for i in exts {
			if name.ends_with(&format!(".{}", i)) {
				return true;
			}
		}
	}

	return false;
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
		.arg(clap::Arg::with_name("summarize")
			.short("s")
			.long("summarize")
			.alias("summary")
			.takes_value(false)
			.help("Print a summary at the end.")
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

/// Pad String On Left.
pub fn pad_left<S>(text: S, pad_len: usize, pad_fill: u8) -> String
where S: Into<String> {
	let text = text.into();
	let text_len: usize = text.len();

	// Prop it up!
	if text_len < pad_len {
		format!(
			"{}{}",
			String::from_utf8(vec![pad_fill; pad_len - text_len]).unwrap_or("".to_string()),
			text,
		)
	}
	// No padding needed.
	else {
		text
	}
}
