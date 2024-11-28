/*!
# `ChannelZ`
*/

use argyle::KeyWordsBuilder;
use dactyl::{
	NiceU16,
	NiceU32,
};
use std::{
	io::Write,
	path::{
		Path,
		PathBuf,
	},
};



/// # Pre-Compute Arguments and Extensions.
///
/// Because we know all of the target extensions in advance, we can store them
/// as primitives for faster runtime comparison (when crawling paths).
///
/// There are a few other, longer extensions that aren't worth optimizing in
/// this way. They're just dealt with inline in `ext.rs`.
pub fn main() {
	println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

	// CLI Arguments.
	write_cli();

	let out = format!(
		r"
/// # Match br/gz.
pub(super) const fn match_encoded(bytes: &[u8]) -> bool {{
	if let [.., 0..=46 | 48..=91 | 93..=255, b'.', a, b] = bytes {{
		matches!(
			u16::from_le_bytes([a.to_ascii_lowercase(), b.to_ascii_lowercase()]),
			{}
		)
	}}
	else {{ false }}
}}

/// # Match 2.
const fn match2(ext: u16) -> bool {{ matches!(ext, {}) }}

/// # Match 3.
const fn match3(ext: u32) -> bool {{ matches!(ext, {}) }}

/// # Match 4.
const fn match4(ext: u32) -> bool {{ matches!(ext, {}) }}
		",
		pat16(&["br", "gz"]),
		pat16(&["js", "md"]),
		pat32(&[
			".bmp", ".css", ".csv", ".doc", ".eot", ".htc", ".htm", ".ico", ".ics",
			".mjs", ".otf", ".pdf", ".rdf", ".rss", ".svg", ".ttf", ".txt", ".vcs",
			".vtt", ".xml", ".xsl", ".xls", ".yml",
		]),
		pat32(&["atom", "docx", "html", "json", "wasm", "xhtm", "xlsx", "yaml"]),
	);

	write(&out_path("channelz-matchers.rs"), out.as_bytes());
}

/// # Output Path.
///
/// Append the sub-path to OUT_DIR and return it.
fn out_path(stub: &str) -> PathBuf {
	std::fs::canonicalize(std::env::var("OUT_DIR").expect("Missing OUT_DIR."))
		.expect("Missing OUT_DIR.")
		.join(stub)
}


/// # U16 Pattern.
///
/// Generate a match pattern of u16 values for the provided two-byte extensions.
fn pat16(ext: &[&str]) -> String {
	let mut out = Vec::new();
	for i in ext {
		let i = i.as_bytes();
		assert_eq!(i.len(), 2);
		let num = NiceU16::with_separator(
			u16::from_le_bytes([i[0], i[1]]),
			b'_',
		);
		out.push(num);
	}
	out.sort();
	out.join(" | ")
}

/// # U32 Pattern.
///
/// Generate a match pattern of u32 values for the provided four-byte
/// extensions.
///
/// Note: this is also used for three-byte extensions; they're just padded with
/// a leading dot.
fn pat32(ext: &[&str]) -> String {
	let mut out = Vec::new();
	for i in ext {
		let i = i.as_bytes();
		assert_eq!(i.len(), 4);
		let num = NiceU32::with_separator(
			u32::from_le_bytes([i[0], i[1], i[2], i[3]]),
			b'_',
		);
		out.push(num);
	}
	out.sort();
	out.join(" | ")
}

/// # Write File.
fn write(path: &Path, data: &[u8]) {
	std::fs::File::create(path).and_then(|mut f|
		f.write_all(data).and_then(|_| f.flush())
	)
	.expect("Unable to write file.");
}

/// # Write CLI Arguments.
fn write_cli() {
	let mut builder = KeyWordsBuilder::default();
	builder.push_keys([
		"--clean", "--clean-only",
		"--force",
		"--no-br",
		"--no-gz",
		"-h", "--help",
		"-p", "--progress",
		"-V", "--version",
	]);
	builder.push_keys_with_values(["-l", "--list"]);
	builder.save(out_path("argyle.rs"));
}
