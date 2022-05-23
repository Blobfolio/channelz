/*!
# `ChannelZ Brotli`

This contains a few FFI bindings to the Brotli C encoder, transcribed from what
the `build.rs` bindgen process came up with.

This borrows heavily from the [`compu`](https://crates.io/crates/compu) crate,
which unfortunately no longer builds correctly on some platforms.
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



// Linux and Windows-32 require an alignment of 8.
#[cfg(not(any(target_os = "macos", all(windows, target_pointer_width = "64"))))]
const MIN_ALIGN: usize = 8;

// Mac and Windows-64 require an alignment of 16.
#[cfg(any(target_os = "macos", all(windows, target_pointer_width = "64")))]
const MIN_ALIGN: usize = 16;

// Stick with usize.
const LAYOUT_OFFSET: usize = std::mem::size_of::<usize>();

/// # Single-Shot Encode.
pub(super) const BROTLI_OPERATION_FINISH: c_uint = 2;



/// # Alias for Encoder State.
pub(super) type BrotliEncoderState = BrotliEncoderStateStruct;

/// # Custom Allocation Callback.
type brotli_alloc_func = Option<
	unsafe extern "C" fn(
		opaque: *mut c_void,
		size: usize,
	) -> *mut c_void,
>;

/// # Custom Free Callback.
type brotli_free_func = Option<
	unsafe extern "C" fn(opaque: *mut c_void, address: *mut c_void),
>;



/// # Brotli Encoder.
///
/// This is a simple wrapper struct that allows us to enforce cleanup on
/// destruction.
pub(super) struct BrotliEncoder {
	pub(super) state: *mut BrotliEncoderState,
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
			BrotliEncoderCreateInstance(Some(custom_malloc), Some(custom_free), std::ptr::null_mut())
		};

		if state.is_null() { None }
		else { Some(Self { state }) }
	}
}



#[repr(C)]
#[derive(Debug)]
/// # Encoder State.
pub(super) struct BrotliEncoderStateStruct { _unused: [u8; 0] }



extern "C" {
	pub(super) fn BrotliEncoderCompressStream(
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
	pub(super) fn BrotliEncoderTakeOutput(state: *mut BrotliEncoderState, size: *mut usize) -> *const u8;
}

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

#[cold]
#[inline(never)]
/// # Null Pointer.
const fn unlikely_null() -> *mut c_void { std::ptr::null_mut() }
