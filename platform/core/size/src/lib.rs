//! One export per codec, over the real core, so the linker keeps both paths and
//! the difference between two builds is the codec and nothing else.
//!
//! The allocator and the panic handler come from `bridge-wasm`, which this crate
//! depends on, so the module measured here is the shipped module plus these
//! exports. A build with no features and a build with `--features proto` differ by
//! the proto path alone.
#![no_std]
#![cfg_attr(not(target_arch = "wasm32"), allow(unused_extern_crates))]

extern crate alloc;

// Not referenced below, and rustc does not load a dependency nothing names: this
// line is what brings in the allocator and the panic handler the shipped module
// uses, from platform/bridges/wasm/src/lib.rs.
use bridge_wasm as _;

#[cfg(target_arch = "wasm32")]
mod wasm {
    extern crate alloc;

    use alloc::vec::Vec;
    use core::{ptr, slice, str};

    /// A config the core accepts, so each call can open one and step it without a
    /// handle table of its own. The seed is the only random input a core gets.
    const CONFIG: &str = r#"{"app":"a","class":"web","contract":"2026-11-05","form":"desktop","mode":"local","os":"o","seed":"00112233445566778899aabbccddeeff","type":"core_config","version":"1.0.0"}"#;

    unsafe fn input<'a>(at: *const u8, len: usize) -> Option<&'a [u8]> {
        if len == 0 {
            return Some(&[]);
        }
        if at.is_null() {
            return None;
        }
        // SAFETY: the host promises `len` readable bytes at this address.
        Some(unsafe { slice::from_raw_parts(at, len) })
    }

    unsafe fn give(bytes: Vec<u8>, out_ptr: *mut *mut u8, out_len: *mut usize) -> i32 {
        if out_ptr.is_null() || out_len.is_null() {
            return 3;
        }
        let mut boxed = bytes.into_boxed_slice();
        let len = boxed.len();
        let at = boxed.as_mut_ptr();
        core::mem::forget(boxed);
        // SAFETY: both pointers were checked above.
        unsafe {
            ptr::write(out_ptr, at);
            ptr::write(out_len, len);
        }
        0
    }

    /// One step over JSON, the path that ships today.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn size_step_json(
        at: *const u8,
        len: usize,
        out_ptr: *mut *mut u8,
        out_len: *mut usize,
    ) -> i32 {
        // SAFETY: the host's promise about its own pointers.
        let Some(bytes) = (unsafe { input(at, len) }) else {
            return 3;
        };
        let Ok(text) = str::from_utf8(bytes) else {
            return 1;
        };
        let Ok(mut machine) = core_rules::Core::open_json(CONFIG) else {
            return 1;
        };
        let Ok(answer) = core_rules::step_json(&mut machine, text) else {
            return 1;
        };
        // SAFETY: as above.
        unsafe { give(answer.into_bytes(), out_ptr, out_len) }
    }

    /// The same step over contract message bytes.
    #[cfg(feature = "proto")]
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn size_step_proto(
        at: *const u8,
        len: usize,
        out_ptr: *mut *mut u8,
        out_len: *mut usize,
    ) -> i32 {
        // SAFETY: the host's promise about its own pointers.
        let Some(bytes) = (unsafe { input(at, len) }) else {
            return 3;
        };
        #[cfg(feature = "validate")]
        if core_rules::proto::check_in(bytes).is_err() {
            return 1;
        }
        let Ok(mut machine) = core_rules::Core::open_json(CONFIG) else {
            return 1;
        };
        let Ok(answer) = core_rules::proto::step_proto(&mut machine, bytes) else {
            return 1;
        };
        // SAFETY: as above.
        unsafe { give(answer, out_ptr, out_len) }
    }
}
