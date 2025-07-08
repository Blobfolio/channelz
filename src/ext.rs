/*!
# ChannelZ: Extensions
*/

use dowser::Extension;



/// # Helper: Define Extensions.
macro_rules! ext {
	($($v:ident $ext:literal),+ $(,)?) => (
		$(
			#[doc = concat!("# Extension (`", $ext, "`).")]
			const $v: Extension = Extension::new($ext).unwrap();
		)+

		/// # Match Extension.
		///
		/// This checks that the path (as a byte slice) ends with one of the
		/// supported extensions.
		pub(super) const fn match_extension(ext: &[u8]) -> bool {
			if let Some(ext) = Extension::from_path_slice(ext) {
				matches!(ext, $($v) |+)
			}
			else {
				13 <= ext.len() &&
				matches!(
					ext,
					[
						.., 0..=46 | 48..=91 | 93..=255, b'.',
						b'W' | b'w',
						b'E' | b'e',
						b'B' | b'b',
						b'M' | b'm',
						b'A' | b'a',
						b'N' | b'n',
						b'I' | b'i',
						b'F' | b'f',
						b'E' | b'e',
						b'S' | b's',
						b'T' | b't',
					]
				)
			}
		}
	);
}

/// # Extension (`br`).
const EXT_BR: Extension = Extension::new("br").unwrap();

/// # Extension (`gz`).
const EXT_GZ: Extension = Extension::new("gz").unwrap();

ext!{
	EXT_JS       "js",
	EXT_MD       "md",
	EXT_BMP      "bmp",
	EXT_CSS      "css",
	EXT_EOT      "eot",
	EXT_HTC      "htc",
	EXT_HTM      "htm",
	EXT_ICO      "ico",
	EXT_ICS      "ics",
	EXT_MJS      "mjs",
	EXT_OTF      "otf",
	EXT_RDF      "rdf",
	EXT_RSS      "rss",
	EXT_SVG      "svg",
	EXT_TTF      "ttf",
	EXT_TXT      "txt",
	EXT_VCS      "vcs",
	EXT_VTT      "vtt",
	EXT_XML      "xml",
	EXT_XSL      "xsl",
	EXT_ATOM     "atom",
	EXT_HTML     "html",
	EXT_JSON     "json",
	EXT_WASM     "wasm",
	EXT_XHTM     "xhtm",
	EXT_VCARD    "vcard",
	EXT_XHTML    "xhtml",
	EXT_JSONLD   "jsonld",
	EXT_GEOJSON  "geojson",
	EXT_APPCACHE "appcache",
	EXT_MANIFEST "manifest",
}



/// # Match br/gz.
pub(super) const fn match_encoded(bytes: &[u8]) -> bool {
	if let [.., 0..=46 | 48..=91 | 93..=255, b'.', a, b] = bytes {
		matches!(Extension::new_slice(&[*a, *b]), Some(EXT_BR | EXT_GZ))
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
			assert!(match_encoded(&fine));

			fine.make_ascii_uppercase();
			assert!(match_encoded(&fine));

			let len = fine.len();
			fine[len - 2] = b'B';
			fine[len - 1] = b'R';
			assert!(match_encoded(&fine));

			fine.make_ascii_lowercase();
			assert!(match_encoded(&fine));

			// This should fail without the file bit.
			let bad = [BASE, b".gz"].concat();
			assert!(! match_encoded(&bad));

			let bad = [BASE, b".br"].concat();
			assert!(! match_encoded(&bad));
		}

		assert!(! match_encoded(b"/foo/foo.js"));
	}
}
