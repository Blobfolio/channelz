/*!
# `ChannelZ`
*/

/// # Build Settings Flags.
fn main() {
	println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

	argyle::FlagsBuilder::new("Flags")
		.private()
		.with_flag("Brotli", None)
		.with_flag("Gzip", None)
		.with_alias("All", ["Brotli", "Gzip"], Some("# All Encoders."))
		.with_flag("Clean", Some("# Clean Old Br/Gz First."))
		.with_complex_flag("CleanOnly", ["Clean"], Some("# Clean Old Br/Gz and Exit."))
		.with_flag("Force", Some("# Crunch All Files.\n\nIgnore the built-in extension times and crunch all the files found."))
		.save(out_path("flags.rs"));
}

/// # Output Path.
///
/// Append the sub-path to OUT_DIR and return it.
fn out_path(stub: &str) -> std::path::PathBuf {
	std::fs::canonicalize(std::env::var("OUT_DIR").expect("Missing OUT_DIR."))
		.expect("Missing OUT_DIR.")
		.join(stub)
}
