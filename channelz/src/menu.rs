use clap::{App, Arg};



/// CLI Menu.
pub fn menu() -> App<'static, 'static> {
	App::new("ChannelZ")
		.version(env!("CARGO_PKG_VERSION"))
		.author("Blobfolio, LLC. <hello@blobfolio.com>")
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg(Arg::with_name("progress")
			.short("p")
			.long("progress")
			.help("Show progress bar while minifying.")
		)
		.arg(Arg::with_name("path")
			.index(1)
			.help("One or more files or directories to compress.")
			.multiple(true)
			.required(true)
			.value_name("PATH(S)")
			.use_delimiter(false)
		)
		.after_help("Note: static copies will only be generated for files with these extensions:
css; htm(l); ico; js; json; mjs; svg; txt; xhtm(l); xml; xsl")
}
