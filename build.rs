/*!
# `ChannelZ`
*/

use std::{
	io::Write,
	path::Path,
};



/// # Pre-Compute Extensions.
pub fn main() {
	let outbrgz = format!(
		"{} | {}",
		u16::from_le_bytes([b'b', b'r']),
		u16::from_le_bytes([b'g', b'z'])
	);

	let out2 = format!(
		"{} | {}",
		u16::from_le_bytes([b'j', b's']),
		u16::from_le_bytes([b'm', b'd'])
	);

	let mut out3 = Vec::new();
	for i in ["bmp", "css", "eot", "htc", "htm", "ico", "ics", "mjs", "otf", "rdf", "rss", "svg", "ttf", "txt", "vcs", "vtt", "xml", "xsl"] {
		let i = i.as_bytes();
		let num = u32::from_le_bytes([b'.', i[0], i[1], i[2]]);
		out3.push(num);
	}
	out3.sort_unstable();
	let out3 = out3.into_iter()
		.map(|x| x.to_string())
		.collect::<Vec<_>>()
		.join(" | ");

	let mut out4 = Vec::new();
	for i in ["atom", "html", "json", "wasm", "xhtm"] {
		let i = i.as_bytes();
		let num = u32::from_le_bytes([i[0], i[1], i[2], i[3]]);
		out4.push(num);
	}
	out4.sort_unstable();
	let out4 = out4.into_iter()
		.map(|x| x.to_string())
		.collect::<Vec<_>>()
		.join(" | ");

	let out_dir = std::fs::canonicalize(std::env::var("OUT_DIR").expect("Missing OUT_DIR."))
		.expect("Missing OUT_DIR.");

	let out = format!(
		r"
/// # Match br/gz.
pub(super) const fn match_br_gz(bytes: &[u8]) -> bool {{
	let len = bytes.len();

	if 4 < len && bytes[len - 3] == b'.' && ! matches!(bytes[len - 4], b'/' | b'\\') {{
		let ext = u16::from_le_bytes([
			bytes[len - 2].to_ascii_lowercase(),
			bytes[len - 1].to_ascii_lowercase(),
		]);
		matches!(ext, {})
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
		outbrgz,
		out2,
		out3,
		out4,
	);

	write(&out_dir.join("channelz-matchers.rs"), out.as_bytes());
}

/// # Write File.
fn write(path: &Path, data: &[u8]) {
	std::fs::File::create(path).and_then(|mut f|
		f.write_all(data).and_then(|_| f.flush())
	)
	.expect("Unable to write file.");
}
