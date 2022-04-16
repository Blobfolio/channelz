/*!
# `ChannelZ`: Extensions
*/


// See build.rs.
include!(concat!(env!("OUT_DIR"), "/channelz-matchers.rs"));



#[allow(clippy::option_if_let_else)] // No.
/// # Match Extension.
pub(super) fn match_extension(ext: &[u8]) -> bool {
	if let Some(dot) = ext.iter().rposition(|b| b'.'.eq(b)) {
		if
			// There's stuff before the dot.
			0 < dot &&
			// The dot leaves room for our shortest extension afterward.
			dot + 2 < ext.len() &&
			// Safety: we tested 0 < dot, so the subtraction won't overflow.
			! matches!(unsafe { *(ext.get_unchecked(dot - 1)) }, b'/' | b'\\')
		{
			// Safety: we tested dot+2 earlier, so dot+1 is fine.
			let ext = unsafe { ext.get_unchecked(dot + 1..) };
			match ext.len() {
				2 => match2(u16::from_le_bytes([
					ext[0].to_ascii_lowercase(),
					ext[1].to_ascii_lowercase(),
				])),
				3 => match3(u32::from_le_bytes([
					b'.',
					ext[0].to_ascii_lowercase(),
					ext[1].to_ascii_lowercase(),
					ext[2].to_ascii_lowercase(),
				])),
				4 => match4(u32::from_le_bytes([
					ext[0].to_ascii_lowercase(),
					ext[1].to_ascii_lowercase(),
					ext[2].to_ascii_lowercase(),
					ext[3].to_ascii_lowercase(),
				])),
				5 => ext.eq_ignore_ascii_case(b"vcard") || ext.eq_ignore_ascii_case(b"xhtml"),
				6 => ext.eq_ignore_ascii_case(b"jsonld"),
				7 => ext.eq_ignore_ascii_case(b"geojson"),
				8 => ext.eq_ignore_ascii_case(b"appcache") || ext.eq_ignore_ascii_case(b"manifest"),
				11 => ext.eq_ignore_ascii_case(b"webmanifest"),
				_ => false,
			}
		}
		else { false }
	}
	else { false }
}



#[cfg(test)]
mod tests {
	use super::*;

	const BASE: &[u8] = b"/foo/bar/";
	const FILE: &[u8] = b"file.";
	const EXTS: [&[u8]; 32] = [
		b"js", b"md",
		b"bmp", b"css", b"eot", b"htc", b"htm", b"ico", b"ics", b"mjs",
		b"otf", b"rdf", b"rss", b"svg", b"ttf", b"txt", b"vcs", b"vtt",
		b"xml", b"xsl",
		b"atom", b"html", b"json", b"wasm", b"xhtm",
		b"vcard", b"xhtml",
		b"jsonld",
		b"geojson",
		b"appcache", b"manifest",
		b"webmanifest",
	];

	#[test]
	/// # Test Extension Finding.
	fn t_ext() {
		for ext in EXTS {
			let mut fine = [BASE, FILE, ext].concat();
			assert!(match_extension(&fine));

			fine.make_ascii_uppercase();
			assert!(match_extension(&fine));

			// This should fail without the file bit.
			let bad = [BASE, ext].concat();
			assert!(! match_extension(&bad));
		}

		// These should not match.
		assert!(! match_extension(b"/foo/file.jss"));
		assert!(! match_extension(b"/foo/file.js.br"));
		assert!(! match_extension(b"/foo/.js"));
		assert!(! match_extension(b"/foo/file.xxx"));
		assert!(! match_extension(b"/foo/.bmp"));
		assert!(! match_extension(b"/foo/.atom"));
		assert!(! match_extension(b"/foo/y.xxxx"));
		assert!(! match_extension(b"/foo/bar"));
		assert!(! match_extension(b""));
	}

	#[test]
	/// # Test Extension Finding.
	fn t_br_gz() {
		for ext in EXTS {
			let mut fine = [BASE, FILE, ext, b".gz"].concat();
			assert!(match_br_gz(&fine));

			fine.make_ascii_uppercase();
			assert!(match_br_gz(&fine));

			let len = fine.len();
			fine[len - 2] = b'B';
			fine[len - 1] = b'R';
			assert!(match_br_gz(&fine));

			fine.make_ascii_lowercase();
			assert!(match_br_gz(&fine));

			// This should fail without the file bit.
			let bad = [BASE, b".gz"].concat();
			assert!(! match_br_gz(&bad));
		}

		assert!(! match_br_gz(b"/foo/foo.js"));
	}
}
