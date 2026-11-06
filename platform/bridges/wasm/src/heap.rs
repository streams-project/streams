//! The allocator. Sizes round up to a power of two with one free list per size, and memory only
//! grows, so nothing here walks or merges blocks. One caller at a time, as the boundary promises.

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;

/// Bytes in one page of linear memory.
pub const PAGE: usize = 65_536;

const SMALLEST: u32 = 4;
const SIZES: usize = 28;
const ALIGN: usize = 16;
/// Pages asked for at a time, so a run of small blocks makes few grow calls.
const LEAST_GROWTH: usize = 8;

/// Where fresh memory comes from.
pub trait Grow {
    /// Add `pages` and give the address of the first, or nothing when there is no more.
    fn grow(&self, pages: usize) -> Option<usize>;
}

struct State {
    /// Address of the first free block of each size. 0 means none.
    free: [usize; SIZES],
    top: usize,
    end: usize,
}

pub struct Heap<G> {
    memory: G,
    state: UnsafeCell<State>,
}

// SAFETY: one caller at a time is the promise of the boundary, and this target has no threads.
unsafe impl<G> Sync for Heap<G> {}

/// Index of the free list for this layout, or nothing when no list fits it.
fn size_of(layout: Layout) -> Option<usize> {
    if layout.align() > ALIGN {
        return None;
    }
    let bits = layout.size().checked_next_power_of_two()?.trailing_zeros();
    let index = bits.max(SMALLEST).checked_sub(SMALLEST)? as usize;
    (index < SIZES).then_some(index)
}

impl<G> Heap<G> {
    pub const fn new(memory: G) -> Self {
        Self {
            memory,
            state: UnsafeCell::new(State {
                free: [0; SIZES],
                top: 0,
                end: 0,
            }),
        }
    }
}

impl<G: Grow> Heap<G> {
    fn fresh(&self, state: &mut State, size: usize) -> Option<usize> {
        if state.end - state.top < size {
            let pages = size.div_ceil(PAGE).max(LEAST_GROWTH);
            let start = self.memory.grow(pages)?;
            if start != state.end {
                state.top = start;
            }
            state.end = start + pages * PAGE;
        }
        let block = state.top;
        state.top += size;
        Some(block)
    }
}

// SAFETY: a block is on a free list or held by one caller, never both, every block is 16 aligned
// because sizes are multiples of 16 from a page start, and a list only holds blocks of its size.
unsafe impl<G: Grow> GlobalAlloc for Heap<G> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let Some(index) = size_of(layout) else {
            return ptr::null_mut();
        };
        // SAFETY: one caller at a time, so this is the only reference.
        let state = unsafe { &mut *self.state.get() };
        let block = state.free[index];
        if block != 0 {
            // SAFETY: a free block holds the address of the next one in its first word.
            state.free[index] = unsafe { (block as *const usize).read() };
            return block as *mut u8;
        }
        self.fresh(state, 1 << (index as u32 + SMALLEST))
            .map_or(ptr::null_mut(), |block| block as *mut u8)
    }

    unsafe fn dealloc(&self, block: *mut u8, layout: Layout) {
        if let Some(index) = size_of(layout) {
            // SAFETY: one caller at a time, and the block is at least 16 bytes and now unused.
            unsafe {
                let state = &mut *self.state.get();
                block.cast::<usize>().write(state.free[index]);
                state.free[index] = block as usize;
            }
        }
    }

    unsafe fn realloc(&self, block: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let Ok(wanted) = Layout::from_size_align(new_size, layout.align()) else {
            return ptr::null_mut();
        };
        if size_of(wanted).is_some() && size_of(wanted) == size_of(layout) {
            return block;
        }
        // SAFETY: the caller's promises carry over, and the two blocks differ.
        unsafe {
            let moved = self.alloc(wanted);
            if !moved.is_null() {
                ptr::copy_nonoverlapping(block, moved, layout.size().min(new_size));
                self.dealloc(block, layout);
            }
            moved
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::{alloc, dealloc};
    use std::cell::Cell;

    /// A block of host memory that stands in for linear memory.
    struct Fake {
        start: *mut u8,
        pages: usize,
        used: Cell<usize>,
    }

    impl Fake {
        fn new(pages: usize) -> Self {
            let layout = Layout::from_size_align(pages * PAGE, PAGE).unwrap();
            // SAFETY: the layout is not empty.
            let start = unsafe { alloc(layout) };
            assert!(!start.is_null());
            Self {
                start,
                pages,
                used: Cell::new(0),
            }
        }
    }

    impl Drop for Fake {
        fn drop(&mut self) {
            let layout = Layout::from_size_align(self.pages * PAGE, PAGE).unwrap();
            // SAFETY: this is the block `new` took.
            unsafe { dealloc(self.start, layout) };
        }
    }

    impl Grow for &Fake {
        fn grow(&self, pages: usize) -> Option<usize> {
            let used = self.used.get();
            if used + pages > self.pages {
                return None;
            }
            self.used.set(used + pages);
            Some(self.start as usize + used * PAGE)
        }
    }

    fn layout(size: usize) -> Layout {
        Layout::from_size_align(size, 1).unwrap()
    }

    #[test]
    fn a_freed_block_comes_back_for_the_same_size() {
        let fake = Fake::new(16);
        let heap = Heap::new(&fake);
        // SAFETY: each block is used within its size and freed with its layout.
        unsafe {
            let one = heap.alloc(layout(100));
            let two = heap.alloc(layout(120));
            assert_ne!(one, two);
            heap.dealloc(one, layout(100));
            assert_eq!(heap.alloc(layout(70)), one);
        }
    }

    #[test]
    fn a_grow_within_a_size_keeps_the_block_and_a_move_keeps_the_bytes() {
        let fake = Fake::new(16);
        let heap = Heap::new(&fake);
        // SAFETY: each block is used within its size and freed with its layout.
        unsafe {
            let block = heap.alloc(layout(20));
            block.write_bytes(9, 20);
            assert_eq!(heap.realloc(block, layout(20), 30), block);
            let moved = heap.realloc(block, layout(30), 1_000);
            assert_ne!(moved, block);
            assert_eq!(std::slice::from_raw_parts(moved, 20), [9; 20]);
            assert_eq!(heap.realloc(moved, layout(1_000), 10).read(), 9);
        }
    }

    #[test]
    fn an_alignment_past_16_is_refused_and_16_is_met() {
        let fake = Fake::new(16);
        let heap = Heap::new(&fake);
        for align in [1, 2, 4, 8, 16] {
            let wanted = Layout::from_size_align(24, align).unwrap();
            // SAFETY: the layout is not empty.
            let block = unsafe { heap.alloc(wanted) };
            assert_eq!(block as usize % align, 0);
            assert_eq!(block as usize % 16, 0);
        }
        let wanted = Layout::from_size_align(24, 32).unwrap();
        // SAFETY: the layout is not empty.
        assert!(unsafe { heap.alloc(wanted) }.is_null());
    }

    #[test]
    fn no_more_memory_gives_null_and_a_small_block_still_comes() {
        let fake = Fake::new(8);
        let heap = Heap::new(&fake);
        // SAFETY: the layouts are not empty.
        unsafe {
            assert!(heap.alloc(layout(9 * PAGE)).is_null());
            assert!(!heap.alloc(layout(100)).is_null());
        }
        assert_eq!(fake.used.get(), 8);
    }

    #[test]
    fn a_size_past_the_largest_list_is_refused() {
        assert_eq!(size_of(layout(1 << 31)), Some(27));
        assert_eq!(size_of(layout((1 << 31) + 1)), None);
        assert_eq!(size_of(layout(1)), Some(0));
    }

    #[test]
    fn repeating_one_size_does_not_grow_memory() {
        let fake = Fake::new(16);
        let heap = Heap::new(&fake);
        for _ in 0..10_000 {
            // SAFETY: each block is freed with its layout.
            unsafe {
                let block = heap.alloc(layout(5_000));
                assert!(!block.is_null());
                heap.dealloc(block, layout(5_000));
            }
        }
        assert_eq!(fake.used.get(), LEAST_GROWTH);
    }

    #[test]
    fn live_blocks_never_share_a_byte_under_a_long_mixed_run() {
        let fake = Fake::new(4_096);
        let heap = Heap::new(&fake);
        let mut live: Vec<(*mut u8, usize, u8)> = Vec::new();
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        for round in 0..200_000_u32 {
            let roll = next();
            if live.len() > 400 || (!live.is_empty() && roll % 3 == 0) {
                let (block, size, mark) = live.swap_remove((roll >> 8) as usize % live.len());
                // SAFETY: this block is live with this size, and goes back once.
                unsafe {
                    assert!(
                        std::slice::from_raw_parts(block, size)
                            .iter()
                            .all(|b| *b == mark)
                    );
                    heap.dealloc(block, layout(size));
                }
            } else {
                let size = 1 + (roll >> 16) as usize % (1 << (4 + roll % 13));
                let mark = round.to_le_bytes()[0];
                // SAFETY: the size is above 0, and the block is filled within its size.
                unsafe {
                    let block = heap.alloc(layout(size));
                    assert!(!block.is_null());
                    block.write_bytes(mark, size);
                    live.push((block, size, mark));
                }
            }
        }
    }
}
