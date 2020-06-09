/*!
# `ChannelZ`

Nothing but static…

Use `ChannelZ` to generate maximally-compressed Gzip- and Brotli-encoded copies
of a file or recurse a directory to do it for many files at once.
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod menu;

use clap::ArgMatches;
use channelz::encode_path;
use fyi_witcher::{
	Result,
	Witcher,
};



fn main() -> Result<()> {
	// Command line arguments.
	let opts: ArgMatches = menu::menu()
		.get_matches();

	// What path are we dealing with?
	let walk = if opts.is_present("list") {
		Witcher::from_file(
			opts.value_of("list").unwrap_or(""),
			r"(?i).+\.(css|x?html?|ico|m?js|json|svg|txt|xml|xsl)$"
		)
	}
	else {
		Witcher::new(
			&opts.values_of("path")
				.unwrap()
				.collect::<Vec<&str>>(),
			r"(?i).+\.(css|x?html?|ico|m?js|json|svg|txt|xml|xsl)$"
		)
	};

	if walk.is_empty() {
		return Err("No encodable files were found.".to_string());
	}

	// With progress.
	if opts.is_present("progress") {
		walk.progress("ChannelZ", encode_path);
	}
	// Without progress.
	else {
		walk.process(encode_path);
	}

	Ok(())
}
