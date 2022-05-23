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



#[allow(unsafe_code)]
/// # Encode With Brotli.
///
/// Try to encode the contents of a slice with brotli using the strongest
/// compression settings, and write the results to the provided buffer _if_ the
/// brotlified version comes out _smaller_ than the original.
///
/// This returns `true` on success, `false` on failure.
///
/// The output buffer does not need to be sized or cleared in advance; it will
/// be truncated/extended as needed. When compressing multiple files, it is
/// recommended you reuse the same buffer to minimize the number of
/// allocations.
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
	// Start an encoder instance.
	let enc = match ffi::BrotliEncoder::new() {
		Some(x) => x,
		_ => return false,
	};

	let mut avail_in = src.len();
	let mut avail_out = 0;

	// Encode it!
	// Safety: a zero response indicates an error.
	if 0 == unsafe {
		ffi::BrotliEncoderCompressStream(
			enc.state,
			ffi::BROTLI_OPERATION_FINISH,
			&mut avail_in,
			&mut src.as_ptr(),
			&mut avail_out,
			std::ptr::null_mut(),
			std::ptr::null_mut()
		)
	} { return false; }

	// Let's try to extract the slice we just encoded.
	// Safety: result will be null if the operation fails.
	let mut size = 0;
	let result = unsafe { ffi::BrotliEncoderTakeOutput(enc.state, &mut size) };

	// Let's save it if it's worth saving.
	if result.is_null() || size == 0 || src.len() <= size { false }
	else {
		buf.truncate(0);
		buf.extend_from_slice(unsafe {
			std::slice::from_raw_parts(result, size)
		});

		true
	}
}
