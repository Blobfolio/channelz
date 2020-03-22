use clap::{App, Arg};



/// CLI Menu.
pub fn menu() -> App<'static, 'static> {
	App::new("ChannelZ")
		.version(env!("CARGO_PKG_VERSION"))
		.author("Blobfolio, LLC. <hello@blobfolio.com>")
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg(Arg::with_name("clean")
			.long("clean")
			.takes_value(false)
			.help("Delete any existing *.br/gz files before starting. (Directory mode only.)")
		)
		.arg(Arg::with_name("path")
			.index(1)
			.help("File or directory to compress.")
			.multiple(false)
			.value_name("PATH")
			.use_delimiter(false)
		)
		.after_help("Note: In directory mode, static copies will only be generated for files with these extensions:
css; htm(l); ico; js; json; mjs; svg; txt; xhtm(l); xml; xsl")
}
