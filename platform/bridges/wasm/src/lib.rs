//! The six functions as raw WebAssembly exports, plus the allocator and panic handler a module
//! with no std needs. They carry bytes and hold no rules.

#![cfg_attr(target_arch = "wasm32", no_std)]

pub mod heap;

#[cfg(target_arch = "wasm32")]
pub mod exports {
    extern crate alloc;

    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use core::arch::wasm32::{memory_grow, unreachable};
    use core::panic::PanicInfo;
    use core::{ptr, slice, str};

    #[cfg(not(feature = "layer"))]
    use core_rules::Core as Machine;
    use core_rules::step_json;
    #[cfg(feature = "layer")]
    use offline::Machine;

    use crate::heap::{Grow, Heap, PAGE};

    /// The number `core_abi` gives. It stays frozen; what changes lives inside the messages.
    pub const ABI: u32 = 1;
    /// The step ran and the out pointers hold its message.
    pub const OK: i32 = 0;
    /// The in message did not read. Nothing changed.
    pub const BAD_MESSAGE: i32 = 1;
    /// Never given here, because a panic traps. Kept so the numbers match the header.
    pub const PANICKED: i32 = 2;
    /// A pointer was null where one is needed. Nothing changed.
    pub const BAD_CALL: i32 = 3;

    /// One running core. Opaque to a host.
    pub struct CoreHandle {
        machine: Machine,
    }

    struct Memory;

    impl Grow for Memory {
        fn grow(&self, pages: usize) -> Option<usize> {
            let old = memory_grow(0, pages);
            (old != usize::MAX).then_some(old * PAGE)
        }
    }

    #[global_allocator]
    static HEAP: Heap<Memory> = Heap::new(Memory);

    #[panic_handler]
    fn panic(_: &PanicInfo<'_>) -> ! {
        unreachable()
    }

    /// Bytes at a host's pointer. Address 0 reads as empty only with a length of 0.
    unsafe fn bytes_at<'a>(at: *const u8, len: usize) -> Option<&'a [u8]> {
        if len == 0 {
            return Some(&[]);
        }
        // SAFETY: the host promises `len` readable bytes at this address.
        (!at.is_null()).then(|| unsafe { slice::from_raw_parts(at, len) })
    }

    /// The ABI number.
    #[unsafe(no_mangle)]
    pub extern "C" fn core_abi() -> u32 {
        ABI
    }

    /// Room for `len` bytes, or 0 when there is none.
    #[unsafe(no_mangle)]
    pub extern "C" fn core_alloc(len: usize) -> *mut u8 {
        let mut bytes: Vec<u8> = Vec::new();
        if bytes.try_reserve_exact(len).is_err() {
            return ptr::null_mut();
        }
        bytes.resize(len, 0);
        Box::into_raw(bytes.into_boxed_slice()).cast()
    }

    /// Give back bytes from `core_alloc` or `core_step`.
    ///
    /// # Safety
    /// `ptr` and `len` are one such buffer, given back once, or `ptr` is 0.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn core_free(ptr: *mut u8, len: usize) {
        if !ptr.is_null() {
            // SAFETY: this buffer began as a boxed slice of this length.
            drop(unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(ptr, len)) });
        }
    }

    /// Open a core from a config, or 0 when it does not read.
    ///
    /// # Safety
    /// `cfg_ptr` holds `cfg_len` readable bytes.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn core_open(cfg_ptr: *const u8, cfg_len: usize) -> *mut CoreHandle {
        // SAFETY: passed on from this function's own promise.
        let bytes = unsafe { bytes_at(cfg_ptr, cfg_len) };
        let text = bytes.and_then(|bytes| str::from_utf8(bytes).ok());
        match text.map(Machine::open_json) {
            Some(Ok(machine)) => Box::into_raw(Box::new(CoreHandle { machine })),
            _ => ptr::null_mut(),
        }
    }

    /// One step. On `OK` the out addresses hold a message for `core_free`, otherwise 0 and 0.
    ///
    /// # Safety
    /// `handle` came from `core_open` and is open. `in_ptr` holds `in_len` readable bytes. The out
    /// addresses can be written.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn core_step(
        handle: *mut CoreHandle,
        in_ptr: *const u8,
        in_len: usize,
        out_ptr_out: *mut *mut u8,
        out_len_out: *mut usize,
    ) -> i32 {
        if handle.is_null() || out_ptr_out.is_null() || out_len_out.is_null() {
            return BAD_CALL;
        }
        // SAFETY: passed on from this function's own promise, and the pointers are not 0.
        let (open, input) = unsafe {
            out_ptr_out.write(ptr::null_mut());
            out_len_out.write(0);
            (&mut *handle, bytes_at(in_ptr, in_len))
        };
        let Some(input) = input else {
            return BAD_CALL;
        };
        let Ok(text) = str::from_utf8(input) else {
            return BAD_MESSAGE;
        };
        let Ok(output) = step_json(&mut open.machine, text) else {
            return BAD_MESSAGE;
        };
        let output = output.into_bytes().into_boxed_slice();
        let len = output.len();
        // SAFETY: both out addresses were checked above.
        unsafe {
            out_ptr_out.write(Box::into_raw(output).cast());
            out_len_out.write(len);
        }
        OK
    }

    /// Close a core.
    ///
    /// # Safety
    /// `handle` came from `core_open`, closed once, or is 0.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn core_close(handle: *mut CoreHandle) {
        if !handle.is_null() {
            // SAFETY: the handle began as this box.
            drop(unsafe { Box::from_raw(handle) });
        }
    }
}
