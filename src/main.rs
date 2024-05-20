/*!
# ChannelZ
*/

#![forbid(unsafe_code)]

#![warn(
	clippy::filetype_is_file,
	clippy::integer_division,
	clippy::needless_borrow,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::suboptimal_flops,
	clippy::unneeded_field_pattern,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]

#![allow(
	clippy::doc_markdown,
	clippy::redundant_pub_crate,
)]



mod enc;
mod ext;
mod jobs;



use argyle::{
	Argue,
	ArgyleError,
	FLAG_HELP,
	FLAG_REQUIRED,
	FLAG_VERSION,
};
use dowser::Dowser;
use fyi_msg::Msg;
use std::{
	os::unix::ffi::OsStrExt,
	path::{
		Path,
		PathBuf,
	},
};



/// # Main.
fn main() {
	match _main() {
		Ok(()) => {},
		Err(ArgyleError::WantsVersion) => {
			println!(concat!("ChannelZ v", env!("CARGO_PKG_VERSION")));
		},
		Err(ArgyleError::WantsHelp) => { helper(); },
		Err(e) => { Msg::error(e).die(1); },
	}
}

#[inline]
/// # Actual Main.
fn _main() -> Result<(), ArgyleError> {
	// Parse CLI arguments.
	let args = Argue::new(FLAG_HELP | FLAG_REQUIRED | FLAG_VERSION)?
		.with_list();

	// Clean first?
	if args.switch2(b"--clean", b"--clean-only") {
		clean(args.args_os());
		if args.switch(b"--clean-only") { return Ok(()); }
	}

	// Put it all together!
	let mut paths: Vec<PathBuf> = Dowser::default()
		.with_paths(args.args_os())
		.into_vec_filtered(
			if args.switch(b"--force") { find_all }
			else { find_default }
		);
	paths.sort();

	// Sexy run-through.
	if args.switch2(b"-p", b"--progress") {
		jobs::exec_pretty(&paths)
	}
	else { jobs::exec(&paths) }
}

/// # Clean.
///
/// This will run a separate search over the specified paths with the sole
/// purpose of removing `*.gz` and `*.br` files.
fn clean<P, I>(paths: I)
where P: AsRef<Path>, I: IntoIterator<Item=P> {
	for p in Dowser::default().with_paths(paths) {
		let bytes = p.as_os_str().as_bytes();
		if ext::match_br_gz(bytes) {
			let len = bytes.len();
			if ext::match_extension(&bytes[..len - 3]) && std::fs::remove_file(&p).is_err() {
				Msg::warning(format!("Unable to delete {p:?}")).print();
			}
		}
	}
}

#[cold]
/// # Find Non-GZ/BR.
fn find_all(p: &Path) -> bool { ! ext::match_br_gz(p.as_os_str().as_bytes()) }

/// # Find Default.
fn find_default(p: &Path) -> bool { ext::match_extension(p.as_os_str().as_bytes()) }

#[cold]
/// # Print Help.
fn helper() {
	println!(concat!(
		r"
                  ,.
                 (\(\)
 ,_              ;  o >
  (`-.          /  (_)
  `=(\`-._____/`   |
   `-( /    -=`\   |
 .==`=(  -= = _/   /`--.
(M==M=M==M=M==M==M==M==M)
 \=N=N==N=N==N=N==N=NN=/   ", "\x1b[38;5;199mChannelZ\x1b[0;38;5;69m v", env!("CARGO_PKG_VERSION"), "\x1b[0m", r#"
  \M==M=M==M=M==M===M=/    Fast, recursive, multi-threaded
   \N=N==N=N==N=NN=N=/     static Brotli and Gzip encoding.
    \M==M==M=M==M==M/
     `-------------'

USAGE:
    channelz [FLAGS] [OPTIONS] <PATH(S)>...

FLAGS:
        --clean       Remove all existing *.gz *.br files (of types ChannelZ
                      would encode) before starting.
        --clean-only  Same as --clean, but exit immediately afterward.
        --force       Try to encode ALL files passed to ChannelZ, regardless of
                      file extension (except those already ending in .br/.gz).
                      Be careful with this!
    -h, --help        Print help information and exit.
    -p, --progress    Show progress bar while minifying.
    -V, --version     Print version information and exit.

OPTIONS:
    -l, --list <FILE> Read (absolute) file and/or directory paths to compress
                      from this text file — or STDIN if "-" — one entry per
                      line, instead of or in addition to specifying <PATH(S)>
                      directly at the end of the command.

ARGS:
    <PATH(S)>...      One or more file and/or directory paths to compress
                      and/or (recursively) crawl.

---

Note: static copies will only be generated for files with these extensions:

    appcache; atom; bmp; css; csv; doc(x); eot; geojson; htc; htm(l); ico; ics;
    js; json; jsonld; manifest; md; mjs; otf; pdf; rdf; rss; svg; ttf; txt;
    vcard; vcs; vtt; wasm; webmanifest; xhtm(l); xls(x); xml; xsl; y(a)ml
"#
	));
}
