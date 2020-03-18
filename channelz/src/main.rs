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

extern crate clap;

use clap::Shell;
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
	let mut clean = false;
	let mut exts: Vec<String> = vec![
		"css".to_string(),
		"html".to_string(),
		"js".to_string(),
		"json".to_string(),
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
			clean_dir(path, &exts);
		}
	}


	Ok(())
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

/// Clean Directory.
fn clean_dir(path: PathBuf, exts: &Vec<String>) {
	if (path.is_dir()) {
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
			if name.ends_with(i) {
				return true;
			}
		}
	}

	return false;
}

/// Come up with compressable files.
fn file_list(path: PathBuf) -> Vec<PathBuf> {
	vec![]
}
