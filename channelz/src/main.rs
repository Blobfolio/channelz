/*!
# `ChannelZ`

Nothing but static…

Use `ChannelZ` to generate maximally-compressed Gzip- and Brotli-encoded copies
of a file or recurse a directory to do it for many files at once.
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

use channelz::encode_path;
use fyi_menu::ArgList;
use fyi_msg::MsgKind;
use fyi_witcher::Witcher;
use std::{
	io::{
		self,
		Write,
	},
	process,
};



/// -h | --help
const FLAG_HELP: u8     = 0b0001;
/// -p | --progress
const FLAG_PROGRESS: u8 = 0b0010;
/// -V | --version
const FLAG_VERSION: u8  = 0b0100;



#[allow(clippy::if_not_else)] // Code is confusing otherwise.
fn main() {
	let mut args = ArgList::default();
	args.expect();

	let mut flags: u8 = 0;
	args.pluck_flags(
		&mut flags,
		&[
			"-h", "--help",
			"-p", "--progress",
			"-V", "--version",
		],
		&[
			FLAG_HELP, FLAG_HELP,
			FLAG_PROGRESS, FLAG_PROGRESS,
			FLAG_VERSION, FLAG_VERSION,
		],
	);

	// Help.
	if 0 != flags & FLAG_HELP {
		_help();
	}
	// Version.
	else if 0 != flags & FLAG_VERSION {
		_version();
	}
	// Actual stuff!
	else {
		// What path are we dealing with?
		let walk = match args.pluck_opt(|x| x == "-l" || x == "--list") {
			Some(p) => Witcher::from_file(
				p,
				r"(?i).+\.(css|eot|x?html?|ico|m?js|json|otf|rss|svg|ttf|txt|xml|xsl)$"
			),
			None => Witcher::new(
				&args.expect_args(),
				r"(?i).+\.(css|eot|x?html?|ico|m?js|json|otf|rss|svg|ttf|txt|xml|xsl)$"
			),
		};

		if walk.is_empty() {
			MsgKind::Error.as_msg("No encodable files were found.").eprintln();
			process::exit(1);
		}
		// Without progress.
		else if 0 == flags & FLAG_PROGRESS {
			walk.process(encode_path);
		}
		// With progress.
		else {
			walk.progress("ChannelZ", encode_path);
		}
	}
}

#[cfg(not(feature = "man"))]
#[cold]
/// Print Help.
fn _help() {
	io::stdout().write_all({
		let mut s = String::with_capacity(1024);
		s.push_str(&format!(
			r"
                  ,.
                 (\(\)
 ,_              ;  o >
  (`-.          /  (_)
  `=(\`-._____/`   |
   `-( /    -=`\   |
 .==`=(  -= = _/   /`--.
(M==M=M==M=M==M==M==M==M)
 \=N=N==N=N==N=N==N=NN=/   {}{}{}
  \M==M=M==M=M==M===M=/    Fast, recursive, multi-threaded
   \N=N==N=N==N=NN=N=/     static Brotli and Gzip encoding.
    \M==M==M=M==M==M/
     `-------------'

",
			"\x1b[38;5;199mChannelZ\x1b[0;38;5;69m v",
			env!("CARGO_PKG_VERSION"),
			"\x1b[0m"
		));
		s.push_str(include_str!("../misc/help.txt"));
		s.push('\n');
		s
	}.as_bytes()).unwrap();
}

#[cfg(feature = "man")]
#[cold]
/// Print Help.
///
/// This is a stripped-down version of the help screen made specifically for
/// `help2man`, which gets run during the Debian package release build task.
fn _help() {
	io::stdout().write_all({
		let mut s = String::with_capacity(1024);
		s.push_str("HTMinL ");
		s.push_str(env!("CARGO_PKG_VERSION"));
		s.push('\n');
		s.push_str(env!("CARGO_PKG_DESCRIPTION"));
		s.push('\n');
		s.push('\n');
		s.push_str(include_str!("../misc/help.txt"));
		s.push('\n');
		s
	}.as_bytes()).unwrap();
}

#[cold]
/// Print version and exit.
fn _version() {
	io::stdout().write_all({
		let mut s = String::from("ChannelZ ");
		s.push_str(env!("CARGO_PKG_VERSION"));
		s.push('\n');
		s
	}.as_bytes()).unwrap();
}
