//! Full-screen integer plasma sample computing all 57,600 pixels every frame from a sine table.

#![no_std]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicI32, Ordering};

const WIDTH: usize = 320;
const HEIGHT: usize = 180;
const PIXELS: usize = WIDTH * HEIGHT;

static mut SCREEN: [u32; PIXELS] = [0; PIXELS];
static PHASE: AtomicI32 = AtomicI32::new(0);
static BIAS_X: AtomicI32 = AtomicI32::new(0);
static BIAS_Y: AtomicI32 = AtomicI32::new(0);

#[rustfmt::skip]
const SIN256: [u8; 256] = [
    128, 131, 134, 137, 140, 143, 146, 149, 152, 155, 158, 162, 165, 167, 170, 173,
    176, 179, 182, 185, 188, 190, 193, 196, 198, 201, 203, 206, 208, 211, 213, 215,
    218, 220, 222, 224, 226, 228, 230, 232, 234, 235, 237, 238, 240, 241, 243, 244,
    245, 246, 248, 249, 250, 250, 251, 252, 253, 253, 254, 254, 254, 255, 255, 255,
    255, 255, 255, 255, 254, 254, 254, 253, 253, 252, 251, 250, 250, 249, 248, 246,
    245, 244, 243, 241, 240, 238, 237, 235, 234, 232, 230, 228, 226, 224, 222, 220,
    218, 215, 213, 211, 208, 206, 203, 201, 198, 196, 193, 190, 188, 185, 182, 179,
    176, 173, 170, 167, 165, 162, 158, 155, 152, 149, 146, 143, 140, 137, 134, 131,
    128, 124, 121, 118, 115, 112, 109, 106, 103, 100,  97,  93,  90,  88,  85,  82,
     79,  76,  73,  70,  67,  65,  62,  59,  57,  54,  52,  49,  47,  44,  42,  40,
     37,  35,  33,  31,  29,  27,  25,  23,  21,  20,  18,  17,  15,  14,  12,  11,
     10,   9,   7,   6,   5,   5,   4,   3,   2,   2,   1,   1,   1,   0,   0,   0,
      0,   0,   0,   0,   1,   1,   1,   2,   2,   3,   4,   5,   5,   6,   7,   9,
     10,  11,  12,  14,  15,  17,  18,  20,  21,  23,  25,  27,  29,  31,  33,  35,
     37,  40,  42,  44,  47,  49,  52,  54,  57,  59,  62,  65,  67,  70,  73,  76,
     79,  82,  85,  88,  90,  93,  97, 100, 103, 106, 109, 112, 115, 118, 121, 124,
];

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn game_abi() -> i32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn game_init(seed: i64) {
    let s = (seed as i32) ^ ((seed >> 32) as i32);
    PHASE.store(s & 0xFF, Ordering::Relaxed);
    BIAS_X.store((s >> 8) & 0xFF, Ordering::Relaxed);
    BIAS_Y.store((s >> 16) & 0xFF, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "C" fn game_frame(now_ms: i64, buttons: i32) {
    let mut bx = BIAS_X.load(Ordering::Relaxed);
    let mut by = BIAS_Y.load(Ordering::Relaxed);
    if (buttons & 1) != 0 {
        by = by.wrapping_sub(2);
    }
    if (buttons & 2) != 0 {
        by = by.wrapping_add(2);
    }
    if (buttons & 4) != 0 {
        bx = bx.wrapping_sub(2);
    }
    if (buttons & 8) != 0 {
        bx = bx.wrapping_add(2);
    }
    BIAS_X.store(bx, Ordering::Relaxed);
    BIAS_Y.store(by, Ordering::Relaxed);

    let step = PHASE.fetch_add(1, Ordering::Relaxed);
    let t = ((now_ms as i32) >> 4).wrapping_add(step) as usize;
    let ox = (bx as usize).wrapping_add(t);
    let oy = (by as usize).wrapping_add(t << 1);

    let buf_ptr = &raw mut SCREEN;
    let buf = unsafe { &mut *buf_ptr };

    let mut y = 0usize;
    while y < HEIGHT {
        let sy1 = SIN256[(y.wrapping_mul(3).wrapping_add(oy)) & 255] as u32;
        let sy2 = SIN256[(y.wrapping_mul(5).wrapping_sub(ox)) & 255] as u32;
        let sy = sy1 + sy2;
        let row = y * WIDTH;
        let mut x = 0usize;
        while x < WIDTH {
            let sx1 = SIN256[(x.wrapping_mul(2).wrapping_add(ox)) & 255] as u32;
            let sx2 = SIN256[(x.wrapping_add(y).wrapping_add(t)) & 255] as u32;
            let idx = ((sx1 + sx2 + sy) >> 2) as usize;
            let r = SIN256[idx & 255] as u32;
            let g = SIN256[(idx + 85) & 255] as u32;
            let b = SIN256[(idx + 170) & 255] as u32;
            buf[row + x] = r | (g << 8) | (b << 16) | 0xFF00_0000;
            x += 1;
        }
        y += 1;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn game_screen() -> i32 {
    (&raw const SCREEN) as *const u8 as i32
}
