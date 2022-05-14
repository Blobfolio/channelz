/*!
# `ChannelZ`: Brotli

This is a slimmed-down rewrite of the wrappers provided by the [`compu`](https://crates.io/crates/compu) crate.
We only need a fraction of what that provides, so by doing it ourselves we can
depend on `compu_brotli_sys` directly.
*/

use compu_brotli_sys::{
	BrotliEncoderCompressStream,
	BrotliEncoderCreateInstance,
	BrotliEncoderDestroyInstance,
	BrotliEncoderOperation_BROTLI_OPERATION_FINISH,
	BrotliEncoderState,
	BrotliEncoderTakeOutput,
};
use std::{
	alloc::Layout,
	os::raw::c_void,
};



// Linux and Windows-32 require an alignment of 8, while Mac and Windows-64
// require an alignment of 16.
#[cfg(not(any(target_os = "macos", all(windows, target_pointer_width = "64"))))]
const MIN_ALIGN: usize = 8;

#[cfg(any(target_os = "macos", all(windows, target_pointer_width = "64")))]
const MIN_ALIGN: usize = 16;

const LAYOUT_OFFSET: usize = std::mem::size_of::<usize>();



#[allow(unsafe_code)]
/// # Encode.
///
/// Try to encode the contents of a slice with Brotli, writing the result to
/// the provided output buffer if it comes out smaller than the source.
///
/// This returns the number of bytes written. Zero means it didn't work.
pub(super) fn encode(src: &[u8], buf: &mut Vec<u8>) -> usize {
	// Start an encoder instance.
	let enc = match BrotliEncoder::new() {
		Some(x) => x,
		_ => return 0,
	};

	let mut avail_in = src.len();
	let mut avail_out = 0;

	// Encode it!
	// Safety: a zero response indicates an error.
	if 0 == unsafe {
		BrotliEncoderCompressStream(
			enc.state,
			BrotliEncoderOperation_BROTLI_OPERATION_FINISH,
			&mut avail_in,
			&mut src.as_ptr(),
			&mut avail_out,
			std::ptr::null_mut(),
			std::ptr::null_mut()
		)
	} { return 0; }

	// Let's try to extract the slice we just encoded.
	// Safety: result will be null if the operation fails.
	let mut size = 0;
	let result = unsafe { BrotliEncoderTakeOutput(enc.state, &mut size) };

	// Let's save it if it's worth saving.
	if result.is_null() || size == 0 || src.len() <= size { 0 }
	else {
		buf.truncate(0);
		buf.extend_from_slice(unsafe {
			std::slice::from_raw_parts(result, size)
		});
		buf.len()
	}
}



/// # Brotli Encoder.
///
/// This is a simple wrapper struct that allows us to enforce cleanup on
/// destruction.
struct BrotliEncoder {
	state: *mut BrotliEncoderState,
}

impl Drop for BrotliEncoder {
	#[allow(unsafe_code)]
	fn drop(&mut self) {
		// Safety: let Brotli run its memory cleanup.
		unsafe { BrotliEncoderDestroyInstance(self.state); }
	}
}

impl BrotliEncoder {
	#[allow(unsafe_code)]
	/// # New Instance.
	///
	/// This shouldn't fail, but it technically _can_, so the return value is
	/// wrapped in an option.
	fn new() -> Option<Self> {
		// Safety: the pointer will be null if the operation fails.
		let state = unsafe {
			BrotliEncoderCreateInstance(Some(custom_malloc), Some(custom_free), std::ptr::null_mut())
		};

		if state.is_null() { None }
		else { Some(Self { state }) }
	}
}



#[cold]
#[inline(never)]
const fn unlikely_null() -> *mut c_void { std::ptr::null_mut() }

#[allow(clippy::cast_ptr_alignment, unsafe_code)]
/// # Custom Malloc.
///
/// This tells Brotli how to allocate memory for us.
unsafe extern "C" fn custom_malloc(_: *mut c_void, size: usize) -> *mut c_void {
	let layout = match Layout::from_size_align(size + LAYOUT_OFFSET, MIN_ALIGN) {
		Ok(layout) => layout,
		_ => return unlikely_null(),
	};

	let mem = std::alloc::alloc(layout);
	if mem.is_null() { return unlikely_null(); }

	std::ptr::write(mem.cast::<usize>(), size);
	mem.add(LAYOUT_OFFSET).cast()
}

#[allow(clippy::cast_possible_wrap, clippy::cast_ptr_alignment, unsafe_code)]
/// # Custom Free.
///
/// This tells Brotli how to free memory allocated for us.
unsafe extern "C" fn custom_free(_: *mut c_void, mem: *mut c_void) {
	if ! mem.is_null() {
		let mem = mem.cast::<u8>().offset(-(LAYOUT_OFFSET as isize));
		let size = std::ptr::read(mem.cast::<usize>());
		let layout = Layout::from_size_align_unchecked(size, MIN_ALIGN);
		std::alloc::dealloc(mem, layout);
	}
}
