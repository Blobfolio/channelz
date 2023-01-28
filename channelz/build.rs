/*!
# `ChannelZ`
*/

use dactyl::{
	NiceU16,
	NiceU32,
};
use std::{
	io::Write,
	path::Path,
};



/// # Pre-Compute Extensions.
///
/// Because we know all of the target extensions in advance, we can store them
/// as primitives for faster runtime comparison (when crawling paths).
///
/// There are a few other, longer extensions that aren't worth optimizing in
/// this way. They're just dealt with inline in `ext.rs`.
pub fn main() {
	let out = format!(
		r"
/// # Match br/gz.
pub(super) const fn match_br_gz(bytes: &[u8]) -> bool {{
	if let [.., x, b'.', a, b] = bytes {{
		! matches!(*x, b'/' | b'\\') &&
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
			".bmp", ".css", ".eot", ".htc", ".htm", ".ico", ".ics", ".mjs", ".otf",
			".rdf", ".rss", ".svg", ".ttf", ".txt", ".vcs", ".vtt", ".xml", ".xsl",
		]),
		pat32(&["atom", "html", "json", "wasm", "xhtm"]),
	);

	let out_path = std::fs::canonicalize(std::env::var("OUT_DIR").expect("Missing OUT_DIR."))
		.expect("Missing OUT_DIR.")
		.join("channelz-matchers.rs");

	write(&out_path, out.as_bytes());
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
