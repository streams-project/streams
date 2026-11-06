//! The six C functions. They carry bytes between the core and a host and hold no rules.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::{ptr, slice, str};

#[cfg(not(feature = "layer"))]
use core_rules::Core as Machine;
use core_rules::{Step, step_json};
#[cfg(feature = "layer")]
use offline::Machine;

/// The number `core_abi` gives. It stays frozen; what changes lives inside the messages.
pub const ABI: u32 = 1;
/// The step ran and the out pointers hold its message.
pub const OK: i32 = 0;
/// The in message did not read. Nothing changed.
pub const BAD_MESSAGE: i32 = 1;
/// The core panicked, now or earlier. Close this handle and open a new one.
pub const PANICKED: i32 = 2;
/// A pointer was null where one is needed. Nothing changed.
pub const BAD_CALL: i32 = 3;

/// One running core. Opaque to a host.
pub struct CoreHandle {
    machine: Machine,
    broken: bool,
}

/// Bytes at a host's pointer. A null pointer reads as empty only with a length of 0.
unsafe fn bytes_at<'a>(at: *const u8, len: usize) -> Option<&'a [u8]> {
    if len == 0 {
        return Some(&[]);
    }
    // SAFETY: the host promises `len` readable bytes at a pointer that is not null.
    (!at.is_null()).then(|| unsafe { slice::from_raw_parts(at, len) })
}

fn step_bytes<S: Step>(stepper: &mut S, broken: &mut bool, input: &[u8]) -> Result<Box<[u8]>, i32> {
    if *broken {
        return Err(PANICKED);
    }
    let text = str::from_utf8(input).map_err(|_| BAD_MESSAGE)?;
    match catch_unwind(AssertUnwindSafe(|| step_json(stepper, text))) {
        Ok(Ok(output)) => Ok(output.into_bytes().into_boxed_slice()),
        Ok(Err(_)) => Err(BAD_MESSAGE),
        Err(_) => {
            *broken = true;
            Err(PANICKED)
        }
    }
}

/// The ABI number.
#[unsafe(no_mangle)]
pub extern "C" fn core_abi() -> u32 {
    ABI
}

/// Room for `len` bytes, or null when there is none.
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
/// `ptr` and `len` are one such buffer, given back once, or `ptr` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn core_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        // SAFETY: this buffer began as a boxed slice of this length.
        drop(unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(ptr, len)) });
    }
}

/// Open a core from a config, or null when it does not read.
///
/// # Safety
/// `cfg_ptr` holds `cfg_len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn core_open(cfg_ptr: *const u8, cfg_len: usize) -> *mut CoreHandle {
    // SAFETY: passed on from this function's own promise.
    let Some(bytes) = (unsafe { bytes_at(cfg_ptr, cfg_len) }) else {
        return ptr::null_mut();
    };
    let Ok(text) = str::from_utf8(bytes) else {
        return ptr::null_mut();
    };
    match catch_unwind(|| Machine::open_json(text)) {
        Ok(Ok(machine)) => Box::into_raw(Box::new(CoreHandle {
            machine,
            broken: false,
        })),
        _ => ptr::null_mut(),
    }
}

/// One step. On `OK` the out pointers hold a message for `core_free`, otherwise null and 0.
///
/// # Safety
/// `handle` came from `core_open` and is open. `in_ptr` holds `in_len` readable bytes. The out
/// pointers can be written.
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
    // SAFETY: passed on from this function's own promise, and the pointers are not null.
    let (open, input) = unsafe {
        out_ptr_out.write(ptr::null_mut());
        out_len_out.write(0);
        (&mut *handle, bytes_at(in_ptr, in_len))
    };
    let Some(input) = input else {
        return BAD_CALL;
    };
    match step_bytes(&mut open.machine, &mut open.broken, input) {
        Ok(output) => {
            let len = output.len();
            // SAFETY: both out pointers were checked above.
            unsafe {
                out_ptr_out.write(Box::into_raw(output).cast());
                out_len_out.write(len);
            }
            OK
        }
        Err(status) => status,
    }
}

/// Close a core.
///
/// # Safety
/// `handle` came from `core_open`, closed once, or is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn core_close(handle: *mut CoreHandle) {
    if !handle.is_null() {
        // SAFETY: the handle began as this box. A panic in a drop must not cross the boundary.
        let _ = catch_unwind(AssertUnwindSafe(|| drop(unsafe { Box::from_raw(handle) })));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contract::core_msg::{CoreIn, CoreOut};

    const IN: &[u8] =
        br#"{"contract":"2026-11-05","type":"core_in","now_ms":0,"mono_ms":0,"events":[]}"#;

    /// Stands in for a fault in the core.
    struct Bomb;

    impl Step for Bomb {
        fn step(&mut self, _: CoreIn) -> CoreOut {
            panic!("planted");
        }
    }

    #[test]
    fn h112_a_panic_comes_back_as_a_status_and_the_handle_stays_refused() {
        let mut broken = false;
        assert_eq!(step_bytes(&mut Bomb, &mut broken, IN).err(), Some(PANICKED));
        assert!(broken);
        assert_eq!(step_bytes(&mut Bomb, &mut broken, IN).err(), Some(PANICKED));
    }

    #[test]
    fn a_message_that_does_not_read_never_reaches_the_core() {
        let mut broken = false;
        let bad = step_bytes(&mut Bomb, &mut broken, b"{").err();
        assert_eq!(bad, Some(BAD_MESSAGE));
        assert!(!broken);
    }
}
