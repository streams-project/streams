//! The six exported functions, called the way a host calls them. Sessions go through here, so the
//! bytes a session sees are the bytes that crossed the real boundary. Input is placed with
//! `core_alloc`, as a host with its own memory must do on Wasm.

use std::ptr;

use bridge_c::{CoreHandle, OK, core_alloc, core_close, core_free, core_open, core_step};

/// One open core behind the boundary.
pub struct Boundary {
    handle: *mut CoreHandle,
}

impl Boundary {
    /// Open from the text a host gives `core_open`.
    ///
    /// # Errors
    /// The boundary gave no handle.
    pub fn open(config: &str) -> Result<Self, String> {
        // SAFETY: the pointer and length come from one string.
        let handle = unsafe { core_open(config.as_ptr(), config.len()) };
        if handle.is_null() {
            return Err(format!("the boundary refused the config: {config}"));
        }
        Ok(Self { handle })
    }

    /// One step of JSON text.
    ///
    /// # Errors
    /// Any status but ok.
    pub fn step(&mut self, input: &str) -> Result<String, String> {
        let mut out: *mut u8 = ptr::null_mut();
        let mut len = 0;
        let placed = core_alloc(input.len());
        // SAFETY: the buffer holds this many bytes, the handle is open, both out pointers are
        // live, and the buffer goes back once.
        let status = unsafe {
            ptr::copy_nonoverlapping(input.as_ptr(), placed, input.len());
            let status = core_step(self.handle, placed, input.len(), &raw mut out, &raw mut len);
            core_free(placed, input.len());
            status
        };
        if status != OK {
            return Err(format!("the boundary gave status {status} for {input}"));
        }
        // SAFETY: on ok the bridge wrote a live buffer of this length, and it goes back once.
        let bytes = unsafe {
            let bytes = std::slice::from_raw_parts(out, len).to_vec();
            core_free(out, len);
            bytes
        };
        String::from_utf8(bytes).map_err(|error| error.to_string())
    }
}

impl Drop for Boundary {
    fn drop(&mut self) {
        // SAFETY: the handle came from core_open and this is its one close.
        unsafe { core_close(self.handle) };
    }
}
