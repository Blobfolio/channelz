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
use fyi_menu::Argue;
use fyi_msg::{
	Msg,
	MsgKind,
};
use fyi_witcher::{
	Witcher,
	WITCHING_QUIET,
	WITCHING_SUMMARIZE,
};



fn main() {
	// Parse CLI arguments.
	let args = Argue::new()
		.with_any()
		.with_version(b"ChannelZ", env!("CARGO_PKG_VERSION").as_bytes())
		.with_help(helper)
		.with_list();

	let mut flags: u8 = WITCHING_QUIET | WITCHING_SUMMARIZE;
	if args.switch2("-p", "--progress") {
		flags &= ! WITCHING_QUIET;
	}

	// Put it all together!
	Witcher::default()
		.with_regex(r"(?i).+\.(css|eot|x?html?|ico|m?js|json|otf|rss|svg|ttf|txt|xml|xsl)$")
		.with_paths(args.args())
		.into_witching()
		.with_flags(flags)
		.with_title(MsgKind::new("ChannelZ", 199).into_msg("Reticulating splines\u{2026}"))
		.run(encode_path);
}

#[cfg(not(feature = "man"))]
#[cold]
/// Print Help.
fn helper(_: Option<&str>) {
	Msg::from(format!(
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
	)).print()
}

#[cfg(feature = "man")]
#[cold]
/// Print Help.
///
/// This is a stripped-down version of the help screen made specifically for
/// `help2man`, which gets run during the Debian package release build task.
fn helper(_: Option<&str>) {
	Msg::from([
		b"ChannelZ ",
		env!("CARGO_PKG_VERSION").as_bytes(),
		b"\n",
		env!("CARGO_PKG_DESCRIPTION").as_bytes(),
		b"\n\n",
		include_bytes!("../misc/help.txt"),
	].concat())
		.print();
}
