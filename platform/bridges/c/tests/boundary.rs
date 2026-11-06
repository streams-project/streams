//! The six functions, called as a C host calls them. Covers H110 H112.

use std::ptr;

use bridge_c::{
    ABI, BAD_CALL, BAD_MESSAGE, CoreHandle, OK, core_abi, core_alloc, core_close, core_free,
    core_open, core_step,
};

const CONFIG: &str = r#"{"app":"sample-web","class":"web","contract":"2026-11-05","form":"desktop","mode":"local","os":"sample-os","seed":"00112233445566778899aabbccddeeff","type":"core_config","version":"1.0.0"}"#;
const LAYERED: &str = r#"{"app":"sample-web","class":"web","contract":"2026-11-05","form":"desktop","layer":{"boot_id":"boot-0","storage_bytes":1000},"mode":"local","os":"sample-os","seed":"00112233445566778899aabbccddeeff","type":"core_config","version":"1.0.0"}"#;
const FIRST: &str =
    r#"{"contract":"2026-11-05","events":[],"mono_ms":0,"now_ms":1000,"type":"core_in"}"#;
const SECOND: &str =
    r#"{"contract":"2026-11-05","events":[],"mono_ms":5,"now_ms":1005,"type":"core_in"}"#;

fn open(text: &[u8]) -> *mut CoreHandle {
    // SAFETY: the pointer and length come from one slice.
    unsafe { core_open(text.as_ptr(), text.len()) }
}

fn step(handle: *mut CoreHandle, input: &[u8]) -> (i32, String) {
    let mut out: *mut u8 = ptr::null_mut();
    let mut len = 0;
    // SAFETY: the pointer and length come from one slice, and both out pointers are live.
    let status = unsafe {
        core_step(
            handle,
            input.as_ptr(),
            input.len(),
            &raw mut out,
            &raw mut len,
        )
    };
    if out.is_null() {
        return (status, String::new());
    }
    // SAFETY: the bridge wrote a live buffer of this length, and it goes back once.
    unsafe {
        let text = String::from_utf8(std::slice::from_raw_parts(out, len).to_vec()).unwrap();
        core_free(out, len);
        (status, text)
    }
}

fn close(handle: *mut CoreHandle) {
    // SAFETY: the handle came from core_open and is closed once.
    unsafe { core_close(handle) };
}

#[test]
fn h110_the_abi_number_is_one_and_a_step_gives_a_core_out_message() {
    assert_eq!(core_abi(), ABI);
    assert_eq!(ABI, 1);
    let handle = open(CONFIG.as_bytes());
    assert!(!handle.is_null());
    let (status, text) = step(handle, FIRST.as_bytes());
    assert_eq!(status, OK);
    assert!(
        text.starts_with(r#"{"contract":"2026-11-05","type":"core_out""#),
        "{text}"
    );
    close(handle);
}

#[test]
fn a_config_that_does_not_read_gives_no_handle() {
    assert!(open(b"not json").is_null());
    assert!(open(&[0xff, 0xfe]).is_null());
    assert!(open(br#"{"seed":"short"}"#).is_null());
    assert!(open(b"").is_null());
    // SAFETY: a null pointer with a length is refused before it is read.
    assert!(unsafe { core_open(ptr::null(), 4) }.is_null());
}

#[test]
fn a_message_that_does_not_read_changes_nothing() {
    let run = |garbage: bool| {
        let handle = open(CONFIG.as_bytes());
        let mut seen = vec![step(handle, FIRST.as_bytes())];
        if garbage {
            assert_eq!(step(handle, b"{").0, BAD_MESSAGE);
            assert_eq!(step(handle, &[0xff]).0, BAD_MESSAGE);
            assert_eq!(step(handle, b"").0, BAD_MESSAGE);
            let deep = "[".repeat(4_000);
            assert_eq!(step(handle, deep.as_bytes()).0, BAD_MESSAGE);
        }
        seen.push(step(handle, SECOND.as_bytes()));
        close(handle);
        seen
    };
    assert_eq!(run(false), run(true));
}

#[test]
fn h112_a_null_handle_or_out_pointer_is_a_bad_call_and_never_reads_them() {
    let mut out: *mut u8 = ptr::null_mut();
    let mut len = 0;
    let input = FIRST.as_bytes();
    let handle = open(CONFIG.as_bytes());
    // SAFETY: each call hands a null that the bridge checks before any read.
    unsafe {
        assert_eq!(
            core_step(
                ptr::null_mut(),
                input.as_ptr(),
                input.len(),
                &raw mut out,
                &raw mut len
            ),
            BAD_CALL
        );
        assert_eq!(
            core_step(
                handle,
                input.as_ptr(),
                input.len(),
                ptr::null_mut(),
                &raw mut len
            ),
            BAD_CALL
        );
        assert_eq!(
            core_step(
                handle,
                input.as_ptr(),
                input.len(),
                &raw mut out,
                ptr::null_mut()
            ),
            BAD_CALL
        );
        assert_eq!(
            core_step(handle, ptr::null(), 3, &raw mut out, &raw mut len),
            BAD_CALL
        );
    }
    close(handle);
}

#[test]
fn nothing_to_free_or_close_is_a_no_op() {
    // SAFETY: nulls are checked first, and the zero length buffer came from core_alloc.
    unsafe {
        core_free(ptr::null_mut(), 0);
        core_close(ptr::null_mut());
        let empty = core_alloc(0);
        assert!(!empty.is_null());
        core_free(empty, 0);
    }
}

#[test]
fn the_allocator_gives_room_a_host_can_fill_and_refuses_a_size_it_cannot_give() {
    let size = 70_000;
    let bytes = core_alloc(size);
    assert!(!bytes.is_null());
    // SAFETY: the buffer has this length and goes back once.
    unsafe {
        ptr::write_bytes(bytes, 7, size);
        core_free(bytes, size);
    }
    assert!(core_alloc(usize::MAX).is_null());
}

#[test]
fn two_handles_run_apart() {
    let one = open(CONFIG.as_bytes());
    let two = open(CONFIG.as_bytes());
    assert_eq!(step(one, FIRST.as_bytes()), step(two, FIRST.as_bytes()));
    assert_eq!(step(one, SECOND.as_bytes()), step(two, SECOND.as_bytes()));
    close(one);
    close(two);
}

#[cfg(feature = "layer")]
#[test]
fn a_layer_key_turns_the_layer_on_and_the_first_effect_reads_the_store() {
    let handle = open(LAYERED.as_bytes());
    let (status, text) = step(handle, FIRST.as_bytes());
    assert_eq!(status, OK);
    assert!(text.contains(r#""kind":"store""#), "{text}");
    close(handle);
}

#[cfg(not(feature = "layer"))]
#[test]
fn a_build_with_the_layer_left_out_runs_the_core_alone_with_a_layer_key() {
    let handle = open(LAYERED.as_bytes());
    let (status, text) = step(handle, FIRST.as_bytes());
    assert_eq!(status, OK);
    assert!(!text.contains(r#""kind":"store""#), "{text}");
    close(handle);
}
