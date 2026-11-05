//! The allocator of the Wasm bridge, behind the real boundary. A long running device repeats
//! sessions for days, so the same work must stop asking for memory.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use bridge_wasm::heap::{Grow, Heap, PAGE};
use kit_core::boundary::Boundary;
use kit_core::host::{Mode, run};
use kit_core::sessions::{common, scripts};

const PAGES: usize = 16_384;

struct Region;

static START: AtomicUsize = AtomicUsize::new(0);
static USED: AtomicUsize = AtomicUsize::new(0);

impl Grow for Region {
    fn grow(&self, pages: usize) -> Option<usize> {
        if START.load(Ordering::SeqCst) == 0 {
            let layout = Layout::from_size_align(PAGES * PAGE, PAGE).unwrap();
            // SAFETY: the layout is not empty.
            START.store(unsafe { System.alloc(layout) } as usize, Ordering::SeqCst);
        }
        let used = USED.fetch_add(pages, Ordering::SeqCst);
        (used + pages <= PAGES).then(|| START.load(Ordering::SeqCst) + used * PAGE)
    }
}

static HEAP: Heap<Region> = Heap::new(Region);
static BUSY: AtomicBool = AtomicBool::new(false);

/// The test harness has threads and the allocator has none, so calls take turns here.
struct Turns;

fn turn<T>(work: impl FnOnce() -> T) -> T {
    while BUSY.swap(true, Ordering::Acquire) {
        std::hint::spin_loop();
    }
    let done = work();
    BUSY.store(false, Ordering::Release);
    done
}

// SAFETY: one call at a time reaches the heap, with the caller's own arguments. The harness has
// a few blocks aligned past what the heap serves, and those stay with the system.
unsafe impl GlobalAlloc for Turns {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > 16 {
            return unsafe { System.alloc(layout) };
        }
        turn(|| unsafe { HEAP.alloc(layout) })
    }

    unsafe fn dealloc(&self, block: *mut u8, layout: Layout) {
        if layout.align() > 16 {
            return unsafe { System.dealloc(block, layout) };
        }
        turn(|| unsafe { HEAP.dealloc(block, layout) });
    }

    unsafe fn realloc(&self, block: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.align() > 16 {
            return unsafe { System.realloc(block, layout, new_size) };
        }
        turn(|| unsafe { HEAP.realloc(block, layout, new_size) })
    }
}

#[global_allocator]
static TURNS: Turns = Turns;

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
fn h112_repeating_a_session_stops_growing_memory() {
    let common = common();
    let longest = scripts()
        .iter()
        .map(|script| run(script, &common, Mode::LayerOn).unwrap().transcript)
        .max_by_key(String::len)
        .unwrap();
    for _ in 0..20 {
        replay(&longest);
    }
    let settled = USED.load(Ordering::SeqCst);
    for _ in 0..100 {
        replay(&longest);
    }
    let used = USED.load(Ordering::SeqCst);
    println!("settled at {settled} pages, then {used}");
    assert_eq!(used, settled);
}
