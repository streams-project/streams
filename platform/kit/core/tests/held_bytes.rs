//! A closed core holds no bytes. A leak here would end a long session on a device with little
//! memory, so every byte the six functions allocate is counted back in.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicI64, Ordering};

use kit_core::boundary::Boundary;
use kit_core::host::{Mode, run};
use kit_core::sessions::{common, scripts};

struct Counting;

static LIVE: AtomicI64 = AtomicI64::new(0);

// SAFETY: every call goes straight to the system allocator with the same arguments.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(layout.size() as i64, Ordering::SeqCst);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, block: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as i64, Ordering::SeqCst);
        unsafe { System.dealloc(block, layout) };
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

/// Replay the in lines of a transcript through the boundary, reopening at each open line.
fn replay(transcript: &str) {
    let mut open = None;
    for line in transcript.lines() {
        if let Some(config) = line.strip_prefix("open ") {
            drop(open.take());
            open = Some(Boundary::open(config).unwrap());
        } else if let Some(input) = line.strip_prefix("in ") {
            let core: &mut Boundary = open.as_mut().unwrap();
            core.step(input).unwrap();
        }
    }
}

#[test]
fn h112_a_closed_core_holds_no_bytes_after_any_session() {
    let common = common();
    let mut failures = Vec::new();
    for script in scripts() {
        for mode in [Mode::LayerOff, Mode::LayerOn] {
            let transcript = run(&script, &common, mode).unwrap().transcript;
            replay(&transcript);
            let before = LIVE.load(Ordering::SeqCst);
            replay(&transcript);
            let held = LIVE.load(Ordering::SeqCst) - before;
            if held != 0 {
                failures.push(format!("{} {mode:?} kept {held} bytes", script.name));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
