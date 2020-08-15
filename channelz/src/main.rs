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
use fyi_msg::MsgKind;
use fyi_witcher::{
	Witcher,
	WITCHING_QUIET,
	WITCHING_SUMMARIZE,
};
use std::{
	io::{
		self,
		Write,
	},
	ops::Range,
};



fn main() {
	let args = fyi_menu::parse_env_args(fyi_menu::FLAG_ALL);
	let (flags, rg, list) = parse_args(&args);

	// Put it all together!
	Witcher::default()
		.with_regex(r"(?i).+\.(css|eot|x?html?|ico|m?js|json|otf|rss|svg|ttf|txt|xml|xsl)$")
		.with(&args[rg], list)
		.into_witching()
		.with_flags(flags)
		.with_title(MsgKind::new("ChannelZ", 199).into_msg("Reticulating splines\u{2026}"))
		.run(encode_path);
}



#[allow(clippy::reversed_empty_ranges)] // Witcher will print an error for us.
#[allow(clippy::range_plus_one)] // We need a consistent return type!
/// Parse Options.
///
/// Returns a tuple containing the flags, path range, and whether or not it is
/// a list.
fn parse_args(args: &[String]) -> (u8, Range<usize>, bool) {
	let mut flags: u8 = WITCHING_QUIET | WITCHING_SUMMARIZE;
	let mut list: usize = 0;

	// Run through the arguments to see what we've got going on!
	let mut idx: usize = 0;
	let len: usize = args.len();
	while idx < len {
		match args[idx].as_str() {
			"-h" | "--help" => {
				_help();
				std::process::exit(0);
			},
			"-V" | "--version" => {
				_version();
				std::process::exit(0);
			},
			"-p" | "--progress" => {
				flags &= ! WITCHING_QUIET;
				idx += 1;
			},
			"-l" | "--list" =>
				if idx + 1 < len {
					list = idx + 1;
					idx += 2;
				}
				else { idx += 1 },
			_ => { break; }
		}
	}

	// What paths are we dealing with?
	(
		flags,
		match list {
			0 if idx < args.len() => idx..args.len(),
			0 => 0..0,
			x => x..x+1,
		},
		0 != list
	)
}



#[cfg(not(feature = "man"))]
#[cold]
/// Print Help.
fn _help() {
	io::stdout().write_fmt(format_args!(
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

{}",
		"\x1b[38;5;199mChannelZ\x1b[0;38;5;69m v",
		env!("CARGO_PKG_VERSION"),
		"\x1b[0m",
		include_str!("../misc/help.txt")
	)).unwrap();
}



#[cfg(feature = "man")]
#[cold]
/// Print Help.
///
/// This is a stripped-down version of the help screen made specifically for
/// `help2man`, which gets run during the Debian package release build task.
fn _help() {
	io::stdout().write_all(&[
		b"ChannelZ ",
		env!("CARGO_PKG_VERSION").as_bytes(),
		b"\n",
		env!("CARGO_PKG_DESCRIPTION").as_bytes(),
		b"\n\n",
		include_bytes!("../misc/help.txt"),
		b"\n",
	].concat()).unwrap();
}



#[cold]
/// Print version and exit.
fn _version() {
	io::stdout().write_all(&[
		b"ChannelZ ",
		env!("CARGO_PKG_VERSION").as_bytes(),
		b"\n"
	].concat()).unwrap();
}
