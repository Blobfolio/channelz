/*!
# `ChannelZ Brotli`

This contains a few FFI bindings to the Brotli C encoder. Refer to `build.rs`
for the (commented-out) bindgen settings that were used as a foundation.
*/

#![allow(non_camel_case_types, non_upper_case_globals)]

use std::{
	alloc::Layout,
	os::raw::{
		c_int,
		c_uint,
		c_void,
	},
};



/// # Allocation Handling.
///
/// This borrows heavily from the [`compu`](https://crates.io/crates/compu) crate.
mod alloc {
	use super::{
		Layout,
		c_void,
	};

	// Linux and Windows-32 require an alignment of 8.
	#[cfg(not(any(target_os = "macos", all(windows, target_pointer_width = "64"))))]
	const MIN_ALIGN: usize = 8;

	// Mac and Windows-64 require an alignment of 16.
	#[cfg(any(target_os = "macos", all(windows, target_pointer_width = "64")))]
	const MIN_ALIGN: usize = 16;

	// Stick with usize.
	const LAYOUT_OFFSET: usize = std::mem::size_of::<usize>();

	#[allow(clippy::cast_ptr_alignment, unsafe_code)]
	/// # Custom Malloc.
	///
	/// This tells Brotli how to allocate memory for us.
	pub(super) unsafe extern "C" fn custom_malloc(_: *mut c_void, size: usize) -> *mut c_void {
		if let Ok(layout) = Layout::from_size_align(size + LAYOUT_OFFSET, MIN_ALIGN) {
			let mem = std::alloc::alloc(layout);
			if ! mem.is_null() {
				std::ptr::write(mem.cast::<usize>(), size);
				return mem.add(LAYOUT_OFFSET).cast();
			}
		}

		std::ptr::null_mut()
	}

	#[allow(clippy::cast_ptr_alignment, unsafe_code)]
	/// # Custom Free.
	///
	/// This tells Brotli how to free memory allocated for us.
	pub(super) unsafe extern "C" fn custom_free(_: *mut c_void, mem: *mut c_void) {
		if ! mem.is_null() {
			let mem = mem.cast::<u8>().sub(LAYOUT_OFFSET);
			let size = std::ptr::read(mem.cast::<usize>());
			let layout = Layout::from_size_align_unchecked(size, MIN_ALIGN);
			std::alloc::dealloc(mem, layout);
		}
	}
}



/// # Brotli Encoder.
///
/// This is a simple wrapper struct that allows us to enforce cleanup on
/// destruction.
pub(super) struct BrotliEncoder {
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
	pub(super) fn new() -> Option<Self> {
		// Safety: the pointer will be null if the operation fails.
		let state = unsafe {
			BrotliEncoderCreateInstance(
				Some(alloc::custom_malloc),
				Some(alloc::custom_free),
				std::ptr::null_mut()
			)
		};

		if state.is_null() { None }
		else { Some(Self { state }) }
	}

	#[allow(unsafe_code)]
	/// # Encode.
	pub(super) fn encode(&self, src: &[u8]) -> bool {
		let mut avail_in = src.len();
		let mut avail_out = 0;

		// Encode it!
		// Safety: a zero response indicates an error.
		0 != avail_in && 0 != unsafe {
			BrotliEncoderCompressStream(
				self.state,
				2, // FINISH
				&mut avail_in,
				&mut src.as_ptr(),
				&mut avail_out,
				std::ptr::null_mut(),
				std::ptr::null_mut()
			)
		}
	}

	#[allow(unsafe_code)]
	/// # Write Result.
	pub(super) fn write_to(&self, len: usize, buf: &mut Vec<u8>) -> bool {
		// Let's try to extract the slice we just encoded.
		// Safety: result will be null if the operation fails.
		let mut size = 0;
		let result = unsafe { BrotliEncoderTakeOutput(self.state, &mut size) };

		// Let's save it if it's worth saving.
		if result.is_null() || size == 0 || len <= size { false }
		else {
			buf.truncate(0);
			buf.extend_from_slice(unsafe {
				std::slice::from_raw_parts(result, size)
			});
			true
		}
	}
}



/// # Allocation Callback.
type brotli_alloc_func = Option<unsafe extern "C" fn(opaque: *mut c_void, size: usize) -> *mut c_void>;

/// # Free Callback.
type brotli_free_func = Option<unsafe extern "C" fn(opaque: *mut c_void, address: *mut c_void)>;

#[repr(C)]
/// # Encoder State.
struct BrotliEncoderState {
	_unused: [u8; 0],
	_marker: std::marker::PhantomData<(*mut u8, std::marker::PhantomPinned)>,
}

extern "C" {
	fn BrotliEncoderCompressStream(
		state: *mut BrotliEncoderState,
		op: c_uint,
		available_in: *mut usize,
		next_in: *mut *const u8,
		available_out: *mut usize,
		next_out: *mut *mut u8,
		total_out: *mut usize,
	) -> c_int;
}

extern "C" {
	fn BrotliEncoderCreateInstance(
		alloc_func: brotli_alloc_func,
		free_func: brotli_free_func,
		opaque: *mut c_void,
	) -> *mut BrotliEncoderState;
}

extern "C" {
	fn BrotliEncoderDestroyInstance(state: *mut BrotliEncoderState);
}

extern "C" {
	fn BrotliEncoderTakeOutput(state: *mut BrotliEncoderState, size: *mut usize) -> *const u8;
}
