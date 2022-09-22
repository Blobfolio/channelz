/*!
# `ChannelZ Brotli`

[![docs.rs](https://img.shields.io/docsrs/channelz_brotli.svg?style=flat-square&label=docs.rs)](https://docs.rs/channelz_brotli/)
<br>
[![crates.io](https://img.shields.io/crates/v/channelz_brotli.svg?style=flat-square&label=crates.io)](https://crates.io/crates/channelz_brotli)
[![ci](https://img.shields.io/github/workflow/status/Blobfolio/channelz/Build.svg?style=flat-square&label=ci)](https://github.com/Blobfolio/channelz/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/channelz/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/channelz)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/channelz/issues)

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

#![allow(clippy::redundant_pub_crate)]



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
