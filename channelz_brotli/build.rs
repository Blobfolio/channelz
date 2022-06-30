/*!
# `ChannelZ Brotli`
*/



pub fn main() {
	println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
	println!("cargo:rerun-if-changed=./vendor/");

	build();
}

/// # Build Brotli.
fn build() {
	let vendor_dir = std::fs::canonicalize("vendor/c").expect("Missing brotli source.");

	cc::Build::new()
		.include(&vendor_dir.join("include"))
		.files(&[
			vendor_dir.join("common/constants.c"),
			vendor_dir.join("common/context.c"),
			vendor_dir.join("common/dictionary.c"),
			vendor_dir.join("common/platform.c"),
			vendor_dir.join("common/shared_dictionary.c"),
			vendor_dir.join("common/transform.c"),
			vendor_dir.join("dec/bit_reader.c"),
			vendor_dir.join("dec/decode.c"),
			vendor_dir.join("dec/huffman.c"),
			vendor_dir.join("dec/state.c"),
			vendor_dir.join("enc/backward_references.c"),
			vendor_dir.join("enc/backward_references_hq.c"),
			vendor_dir.join("enc/bit_cost.c"),
			vendor_dir.join("enc/block_splitter.c"),
			vendor_dir.join("enc/brotli_bit_stream.c"),
			vendor_dir.join("enc/cluster.c"),
			vendor_dir.join("enc/command.c"),
			vendor_dir.join("enc/compound_dictionary.c"),
			vendor_dir.join("enc/compress_fragment.c"),
			vendor_dir.join("enc/compress_fragment_two_pass.c"),
			vendor_dir.join("enc/dictionary_hash.c"),
			vendor_dir.join("enc/encode.c"),
			vendor_dir.join("enc/encoder_dict.c"),
			vendor_dir.join("enc/entropy_encode.c"),
			vendor_dir.join("enc/fast_log.c"),
			vendor_dir.join("enc/histogram.c"),
			vendor_dir.join("enc/literal_cost.c"),
			vendor_dir.join("enc/memory.c"),
			vendor_dir.join("enc/metablock.c"),
			vendor_dir.join("enc/static_dict.c"),
			vendor_dir.join("enc/utf8_util.c"),
		])
		.define("BROTLI_BUILD_ENC_EXTRA_API", None)
		.compile("libbrotli.a");

	// bindings(&vendor_dir);
}
/*
/// # Generate Bindings.
///
/// These are manually transcribed within the library, but this got us going.
fn bindings(vendor_dir: &std::path::Path) {
	let bindings = bindgen::Builder::default()
		.header(vendor_dir.join("include/brotli/encode.h").to_string_lossy())
		.generate_comments(false)
		.size_t_is_usize(true)
		.allowlist_function("BrotliEncoderCompressStream")
		.allowlist_function("BrotliEncoderCreateInstance")
		.allowlist_function("BrotliEncoderDestroyInstance")
		.allowlist_function("BrotliEncoderTakeOutput")
		.allowlist_type("BrotliEncoderOperation")
		.clang_arg(format!("-I{}", vendor_dir.to_string_lossy()))
		.generate()
		.expect("Unable to generate bindings.");

	let out_path = std::fs::canonicalize(std::env::var("OUT_DIR").expect("Missing OUT_DIR."))
		.expect("Missing OUT_DIR.")
		.join("channelz-brotli.rs");

	bindings
		.write_to_file(&out_path)
		.expect("Couldn't write bindings!");
}
*/
