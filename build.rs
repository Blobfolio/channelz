/*!
# `ChannelZ`
*/

use argyle::{
	FlagsBuilder,
	KeyWordsBuilder,
};
use std::path::PathBuf;



/// # Pre-Compute Arguments and Extensions.
///
/// Because we know all of the target extensions in advance, we can store them
/// as primitives for faster runtime comparison (when crawling paths).
///
/// There are a few other, longer extensions that aren't worth optimizing in
/// this way. They're just dealt with inline in `ext.rs`.
fn main() {
	println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

	// CLI Arguments.
	write_cli();

	// Flags.
	write_flags();
}

/// # Output Path.
///
/// Append the sub-path to OUT_DIR and return it.
fn out_path(stub: &str) -> PathBuf {
	std::fs::canonicalize(std::env::var("OUT_DIR").expect("Missing OUT_DIR."))
		.expect("Missing OUT_DIR.")
		.join(stub)
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

/// # Write Flags.
fn write_flags() {
	FlagsBuilder::new("Flags")
		.private()
		.with_flag("Brotli", None)
		.with_flag("Gzip", None)
		.with_alias("All", ["Brotli", "Gzip"], Some("# All Encoders."))
		.with_flag("Clean", Some("# Clean Old Br/Gz First."))
		.with_complex_flag("CleanOnly", ["Clean"], Some("# Clean Old Br/Gz and Exit."))
		.with_flag("Force", Some("# Crunch All Files.\n\nIgnore the built-in extension times and crunch all the files found."))
		.save(out_path("flags.rs"));
}
