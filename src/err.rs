/*!
# ChannelZ: Errors
*/

use fyi_ansi::{
	ansi,
	csi,
	dim,
};
use std::fmt;



/// # Help Text.
const HELP: &str = concat!(
		r"
                  ,.
                 (\(\)
 ,_              ;  o >
  (`-.          /  (_)
  `=(\`-._____/`   |
   `-( /    -=`\   |
 .==`=(  -= = _/   /`--.
(M==M=M==M=M==M==M==M==M)
 \=N=N==N=N==N=N==N=NN=/   ", csi!(199), "ChannelZ", ansi!((cornflower_blue) " v", env!("CARGO_PKG_VERSION")), r#"
  \M==M=M==M=M==M===M=/    Fast, recursive, multi-threaded
   \N=N==N=N==N=NN=N=/     static Brotli and Gzip encoding.
    \M==M==M=M==M==M/
     `-------------'

USAGE:
    channelz [FLAGS] [OPTIONS] <PATH(S)>...

FLAGS:
        --clean       Remove all existing *.gz / *.br files (of types ChannelZ
                      would encode) before starting, unless --no-gz or --no-br
                      are also set, respectively.
        --clean-only  Same as --clean, but exit immediately afterward.
        --force       Try to encode ALL files passed to ChannelZ, regardless of
                      file extension (except those already ending in .br/.gz).
                      Be careful with this!
    -h, --help        Print help information and exit.
        --no-br       Skip Brotli encoding.
        --no-gz       Skip Gzip encoding.
    -p, --progress    Show progress bar while minifying.
    -V, --version     Print version information and exit.

OPTIONS:
    -l, --list <FILE> Read (absolute) file and/or directory paths to compress
                      from this text file — or STDIN if "-" — one entry per
                      line, instead of or in addition to any trailing
                      <PATH(S)>.

ARGS:
    <PATH(S)>...      One or more file and/or directory paths to compress
                      and/or (recursively) crawl.

---

Note: static copies will only be generated for files with these extensions:

    appcache; atom; bmp; css; csv; doc(x); eot; geojson; htc; htm(l); ico; ics;
    js; json; jsonld; manifest; md; mjs; otf; pdf; rdf; rss; svg; ttf; txt;
    vcard; vcs; vtt; wasm; webmanifest; xhtm(l); xls(x); xml; xsl; y(a)ml
"#
);



#[expect(clippy::missing_docs_in_private_items, reason = "Self-explanatory.")]
#[derive(Debug, Clone, Eq, PartialEq)]
/// # Errors.
///
/// This is the binary's obligatory custom error type.
pub(super) enum ChannelZError {
	InvalidCli(String),
	Jobserver,
	Killed,
	ListFile,
	NoEncoders,
	NoFiles,
	PrintHelp,
	PrintVersion,
}

impl std::error::Error for ChannelZError {}

impl fmt::Display for ChannelZError {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let prefix = self.as_str();
		match self {
			Self::InvalidCli(s) => write!(
				f,
				concat!("{} ", dim!("{}")),
				prefix,
				s,
			),
			_ => f.write_str(prefix),
		}
	}
}

impl ChannelZError {
	/// # As String Slice.
	pub(super) const fn as_str(&self) -> &'static str {
		match self {
			Self::InvalidCli(_) => "Invalid/unknown argument:",
			Self::Jobserver => "One or more threads terminated early; please try again.",
			Self::Killed => "The process was aborted early.",
			Self::ListFile => "Invalid -l/--list text file.",
			Self::NoEncoders => "At least one encoder needs to be enabled.",
			Self::NoFiles => "No encodeable files were found.",
			Self::PrintHelp => HELP,
			Self::PrintVersion => concat!("ChannelZ v", env!("CARGO_PKG_VERSION")),
		}
	}
}
