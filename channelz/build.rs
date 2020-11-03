#[cfg(not(feature = "man"))]
/// # Do Nothing.
///
/// We only need to rebuild stuff for new releases. The "man" feature is
/// basically used to figure that out.
fn main() {}

#[cfg(feature = "man")]
/// # Build.
fn main() {
	make_bash();
	make_man();
}



#[cfg(feature = "man")]
/// # Build BASH Completions.
fn make_bash() {
	use fyi_menu::Basher;
	use std::{
		env,
		path::PathBuf,
	};

	// We're going to shove this in "channelz/misc/channelz.bash". If we used
	// `OUT_DIR` like Cargo suggests, we'd never be able to find it to shove
	// it into the `.deb` package.
	let mut path: PathBuf = env::var("CARGO_MANIFEST_DIR")
		.ok()
		.and_then(|x| std::fs::canonicalize(x).ok())
		.expect("Missing completion script directory.");

	path.push("misc");
	path.push("channelz.bash");

	// All of our options.
	let b = Basher::new("channelz")
		.with_option(Some("-l"), Some("--list"))
		.with_switch(None, Some("--clean"))
		.with_switch(Some("-h"), Some("--help"))
		.with_switch(Some("-p"), Some("--progress"))
		.with_switch(Some("-V"), Some("--version"));

	// Write it!
	b.write(&path)
		.unwrap_or_else(|_| panic!("Unable to write completion script: {:?}", path));
}



#[cfg(feature = "man")]
/// # Build MAN Page.
fn make_man() {
	use fyi_menu::{
		Man,
		ManSection,
		ManSectionItem,
	};
	use std::{
		env,
		path::PathBuf,
	};

	// Build the output path.
	let mut path: PathBuf = env::var("CARGO_MANIFEST_DIR")
		.ok()
		.and_then(|x| std::fs::canonicalize(x).ok())
		.expect("Missing completion script directory.");

	path.push("misc");
	path.push("channelz.1");

	// Build the manual!
	let m = Man::new("ChannelZ", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
		.with_text(
			"DESCRIPTION",
			env!("CARGO_PKG_DESCRIPTION"),
			false
		)
		.with_text(
			"USAGE:",
			"channelz [FLAGS] [OPTIONS] <PATH(s)…>",
			true
		)
		.with_section(
			ManSection::list("FLAGS:")
				.with_item(
					ManSectionItem::new("Remove all existing *.gz *.br files before starting.")
						.with_key("--clean")
				)
				.with_item(
					ManSectionItem::new("Print help information.")
						.with_key("-h")
						.with_key("--help")
				)
				.with_item(
					ManSectionItem::new("Show progress bar while minifying.")
						.with_key("-p")
						.with_key("--progress")
				)
				.with_item(
					ManSectionItem::new("Print version information.")
						.with_key("-V")
						.with_key("--version")
				)
		)
		.with_section(
			ManSection::list("OPTIONS:")
				.with_item(
					ManSectionItem::new("Read file paths from this text file.")
						.with_key("-l")
						.with_key("--list")
						.with_value("<FILE>")
				)
		)
		.with_text(
			"<PATH(s)…>:",
			"Any number of files or directories to crawl and crunch.",
			true
		)
		.with_text(
			"NOTE",
			"Static copies will only be generated for files with these extensions:\n.RE\ncss; eot; htm(l); ico; js; json; mjs; otf; rss; svg; ttf; txt; xhtm(l); xml; xsl",
			false
		);

	// Write it!
	m.write(&path)
		.unwrap_or_else(|_| panic!("Unable to write MAN script: {:?}", path));
}
