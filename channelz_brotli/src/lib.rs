/*!
# `ChannelZ Brotli`

This library exposes brotli (FFI C) encoding via the single-shot [`encode`] method.

That's it!

Refer to the documentation for usage details.
*/

#![deny(unsafe_code)]

#![warn(
	clippy::filetype_is_file,
	clippy::integer_division,
	clippy::needless_borrow,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::suboptimal_flops,
	clippy::unneeded_field_pattern,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]



mod ffi;



/// # Encode With Brotli.
///
/// This method will encode the contents of a slice with brotli — using the
/// strongest possible settings — and write the results to the provided buffer
/// if brotlification brought any savings.
///
/// (If the result winds up larger, it will not be saved.)
///
/// The return value is a simple bool indicating whether or not the buffer was
/// written to.
///
/// Note: the output buffer does not need to be pre-sized; it will be truncated
/// and/or extended as necessary.
///
/// ## Examples
///
/// ```
/// // This slice should be compressable!
/// let raw: &[u8] = b"One One One Two Two Two Three Three Three!";
/// let mut out = Vec::new();
/// assert!(channelz_brotli::encode(raw, &mut out)); // True is happy.
///
/// // Not everything will compress, though. The following slice is too small.
/// let raw: &[u8] = b"I'm already small.";
/// assert!(! channelz_brotli::encode(raw, &mut out)); // False is sad.
/// ```
pub fn encode(src: &[u8], buf: &mut Vec<u8>) -> bool {
	ffi::BrotliEncoder::encode(src).map_or(false, |enc| enc.write_to(buf))
}
