//! Small no_std paddle and ball game for the version 1 Wasm game ABI.

#![no_std]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicI32, AtomicI64, AtomicU32, AtomicU64, Ordering};

const WIDTH: i32 = 320;
const HEIGHT: i32 = 180;
const PIXELS: usize = (WIDTH as usize) * (HEIGHT as usize);

const PADDLE_W: i32 = 6;
const PADDLE_H: i32 = 32;
const BALL_SIZE: i32 = 6;
const LEFT_X: i32 = 16;
const RIGHT_X: i32 = WIDTH - 16 - PADDLE_W;

const BUTTON_UP: i32 = 1;
const BUTTON_DOWN: i32 = 2;
const BUTTON_LEFT: i32 = 4;
const BUTTON_RIGHT: i32 = 8;

const BG_COLOR: u32 = u32::from_le_bytes([0x0D, 0x11, 0x17, 0xFF]);
const NET_COLOR: u32 = u32::from_le_bytes([0x30, 0x36, 0x3D, 0xFF]);
const PADDLE_COLOR: u32 = u32::from_le_bytes([0xF0, 0xF6, 0xFC, 0xFF]);
const BALL_COLOR: u32 = u32::from_le_bytes([0x58, 0xA6, 0xFF, 0xFF]);
const SCORE_COLOR: u32 = u32::from_le_bytes([0x8B, 0x94, 0x9E, 0xFF]);

static mut SCREEN: [u32; PIXELS] = [0; PIXELS];

static RNG: AtomicU64 = AtomicU64::new(1);
static LAST_MS: AtomicI64 = AtomicI64::new(0);
static PLAYER_Y: AtomicI32 = AtomicI32::new((HEIGHT - PADDLE_H) / 2);
static RIVAL_Y: AtomicI32 = AtomicI32::new((HEIGHT - PADDLE_H) / 2);
static BALL_X: AtomicI32 = AtomicI32::new((WIDTH - BALL_SIZE) / 2);
static BALL_Y: AtomicI32 = AtomicI32::new((HEIGHT - BALL_SIZE) / 2);
static BALL_VX: AtomicI32 = AtomicI32::new(3);
static BALL_VY: AtomicI32 = AtomicI32::new(2);
static SCORE_LEFT: AtomicU32 = AtomicU32::new(0);
static SCORE_RIGHT: AtomicU32 = AtomicU32::new(0);
static PREV_PY: AtomicI32 = AtomicI32::new((HEIGHT - PADDLE_H) / 2);
static PREV_RY: AtomicI32 = AtomicI32::new((HEIGHT - PADDLE_H) / 2);
static PREV_BX: AtomicI32 = AtomicI32::new((WIDTH - BALL_SIZE) / 2);
static PREV_BY: AtomicI32 = AtomicI32::new((HEIGHT - BALL_SIZE) / 2);

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn next_rand() -> u64 {
    let mut x = RNG.load(Ordering::Relaxed);
    if x == 0 {
        x = 0x9E37_79B9_7F4A_7C15;
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    RNG.store(x, Ordering::Relaxed);
    x
}

fn reset_ball(towards_right: bool) {
    let r = next_rand();
    let offset = ((r % 61) as i32) - 30;
    let vy = if (r & 1) == 0 { 2 } else { -2 };
    let vx = if towards_right { 3 } else { -3 };
    BALL_X.store((WIDTH - BALL_SIZE) / 2, Ordering::Relaxed);
    BALL_Y.store(
        ((HEIGHT - BALL_SIZE) / 2 + offset).clamp(12, HEIGHT - 12 - BALL_SIZE),
        Ordering::Relaxed,
    );
    BALL_VX.store(vx, Ordering::Relaxed);
    BALL_VY.store(vy, Ordering::Relaxed);
}

fn fill_rect(buf: &mut [u32; PIXELS], x0: i32, y0: i32, w: i32, h: i32, color: u32) {
    let start_x = x0.max(0).min(WIDTH) as usize;
    let end_x = (x0 + w).max(0).min(WIDTH) as usize;
    let start_y = y0.max(0).min(HEIGHT) as usize;
    let end_y = (y0 + h).max(0).min(HEIGHT) as usize;
    let stride = WIDTH as usize;
    for y in start_y..end_y {
        let row = y * stride;
        let mut x = row + start_x;
        let end = row + end_x;
        while x < end {
            buf[x] = color;
            x += 1;
        }
    }
}

fn draw_net(buf: &mut [u32; PIXELS]) {
    let mut net_y = 6;
    while net_y < HEIGHT - 6 {
        fill_rect(buf, WIDTH / 2 - 1, net_y, 2, 6, NET_COLOR);
        net_y += 12;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn game_abi() -> i32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn game_init(seed: i64) {
    let mixed = (seed as u64) ^ 0xA5A5_5A5A_F0F0_0F0F;
    RNG.store(if mixed == 0 { 1 } else { mixed }, Ordering::Relaxed);
    LAST_MS.store(0, Ordering::Relaxed);
    let mid_y = (HEIGHT - PADDLE_H) / 2;
    PLAYER_Y.store(mid_y, Ordering::Relaxed);
    RIVAL_Y.store(mid_y, Ordering::Relaxed);
    PREV_PY.store(mid_y, Ordering::Relaxed);
    PREV_RY.store(mid_y, Ordering::Relaxed);
    SCORE_LEFT.store(0, Ordering::Relaxed);
    SCORE_RIGHT.store(0, Ordering::Relaxed);
    reset_ball((next_rand() & 1) == 0);
    PREV_BX.store(BALL_X.load(Ordering::Relaxed), Ordering::Relaxed);
    PREV_BY.store(BALL_Y.load(Ordering::Relaxed), Ordering::Relaxed);

    let buf_ptr = &raw mut SCREEN;
    let buf = unsafe { &mut *buf_ptr };
    fill_rect(buf, 0, 0, WIDTH, HEIGHT, BG_COLOR);
    draw_net(buf);
}

#[unsafe(no_mangle)]
pub extern "C" fn game_frame(now_ms: i64, buttons: i32) {
    let prev = LAST_MS.swap(now_ms, Ordering::Relaxed);
    let step = if prev > 0 && now_ms > prev && (now_ms - prev) > 24 {
        2
    } else {
        1
    };

    let mut py = PLAYER_Y.load(Ordering::Relaxed);
    if (buttons & (BUTTON_UP | BUTTON_LEFT)) != 0 {
        py -= 3 * step;
    }
    if (buttons & (BUTTON_DOWN | BUTTON_RIGHT)) != 0 {
        py += 3 * step;
    }
    py = py.clamp(6, HEIGHT - 6 - PADDLE_H);
    PLAYER_Y.store(py, Ordering::Relaxed);

    let mut bx = BALL_X.load(Ordering::Relaxed);
    let mut by = BALL_Y.load(Ordering::Relaxed);
    let mut vx = BALL_VX.load(Ordering::Relaxed);
    let mut vy = BALL_VY.load(Ordering::Relaxed);

    bx += vx * step;
    by += vy * step;

    if by <= 4 {
        by = 4;
        vy = -vy;
    } else if by + BALL_SIZE >= HEIGHT - 4 {
        by = HEIGHT - 4 - BALL_SIZE;
        vy = -vy;
    }

    if vx < 0
        && bx <= LEFT_X + PADDLE_W
        && bx + BALL_SIZE >= LEFT_X
        && by + BALL_SIZE >= py
        && by <= py + PADDLE_H
    {
        bx = LEFT_X + PADDLE_W;
        vx = -vx;
        let hit = (by + BALL_SIZE / 2) - (py + PADDLE_H / 2);
        vy = (hit / 5).clamp(-3, 3);
        if vy == 0 {
            vy = 1;
        }
    }

    let mut ry = RIVAL_Y.load(Ordering::Relaxed);
    let target = by + BALL_SIZE / 2 - PADDLE_H / 2;
    if ry + 2 < target {
        ry += 2 * step;
    } else if ry - 2 > target {
        ry -= 2 * step;
    }
    ry = ry.clamp(6, HEIGHT - 6 - PADDLE_H);
    RIVAL_Y.store(ry, Ordering::Relaxed);

    if vx > 0
        && bx + BALL_SIZE >= RIGHT_X
        && bx <= RIGHT_X + PADDLE_W
        && by + BALL_SIZE >= ry
        && by <= ry + PADDLE_H
    {
        bx = RIGHT_X - BALL_SIZE;
        vx = -vx;
        let hit = (by + BALL_SIZE / 2) - (ry + PADDLE_H / 2);
        vy = (hit / 5).clamp(-3, 3);
        if vy == 0 {
            vy = -1;
        }
    }

    if bx < 0 {
        SCORE_RIGHT.fetch_add(1, Ordering::Relaxed);
        reset_ball(true);
    } else if bx > WIDTH {
        SCORE_LEFT.fetch_add(1, Ordering::Relaxed);
        reset_ball(false);
    } else {
        BALL_X.store(bx, Ordering::Relaxed);
        BALL_Y.store(by, Ordering::Relaxed);
        BALL_VX.store(vx, Ordering::Relaxed);
        BALL_VY.store(vy, Ordering::Relaxed);
    }

    let draw_bx = BALL_X.load(Ordering::Relaxed);
    let draw_by = BALL_Y.load(Ordering::Relaxed);
    let left_pts = SCORE_LEFT.load(Ordering::Relaxed).min(10) as i32;
    let right_pts = SCORE_RIGHT.load(Ordering::Relaxed).min(10) as i32;

    let old_py = PREV_PY.swap(py, Ordering::Relaxed);
    let old_ry = PREV_RY.swap(ry, Ordering::Relaxed);
    let old_bx = PREV_BX.swap(draw_bx, Ordering::Relaxed);
    let old_by = PREV_BY.swap(draw_by, Ordering::Relaxed);

    let buf_ptr = &raw mut SCREEN;
    let buf = unsafe { &mut *buf_ptr };

    fill_rect(buf, LEFT_X, old_py, PADDLE_W, PADDLE_H, BG_COLOR);
    fill_rect(buf, RIGHT_X, old_ry, PADDLE_W, PADDLE_H, BG_COLOR);
    fill_rect(buf, old_bx, old_by, BALL_SIZE, BALL_SIZE, BG_COLOR);

    draw_net(buf);

    for i in 0..left_pts {
        fill_rect(buf, WIDTH / 2 - 18 - i * 8, 8, 5, 5, SCORE_COLOR);
    }
    for i in 0..right_pts {
        fill_rect(buf, WIDTH / 2 + 13 + i * 8, 8, 5, 5, SCORE_COLOR);
    }

    fill_rect(buf, LEFT_X, py, PADDLE_W, PADDLE_H, PADDLE_COLOR);
    fill_rect(buf, RIGHT_X, ry, PADDLE_W, PADDLE_H, PADDLE_COLOR);
    fill_rect(buf, draw_bx, draw_by, BALL_SIZE, BALL_SIZE, BALL_COLOR);
}

#[unsafe(no_mangle)]
pub extern "C" fn game_screen() -> i32 {
    (&raw const SCREEN) as *const u8 as i32
}
