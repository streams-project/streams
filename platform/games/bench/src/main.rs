//! Speed benchmark running 1,000 frames of the sample paddle game inside `game-host`.

use std::time::Instant;

use game_host::{BUTTON_DOWN, BUTTON_UP, Game, SCREEN_BYTES};

const PADDLE_WASM: &[u8] = include_bytes!("../../host/tests/paddle.wasm");
const FRAMES: usize = 1_000;

fn main() {
    let mut game = Game::load(PADDLE_WASM, 0x1234_5678).expect("load paddle.wasm");
    let mut times_ms = Vec::with_capacity(FRAMES);

    let total_start = Instant::now();
    for i in 0..FRAMES {
        let buttons = if (i / 45) % 2 == 0 {
            BUTTON_UP
        } else {
            BUTTON_DOWN
        };
        let t0 = Instant::now();
        let screen = game
            .frame(((i + 1) as u64) * 16, buttons)
            .expect("frame execution");
        std::hint::black_box(screen[0]);
        std::hint::black_box(screen[SCREEN_BYTES - 1]);
        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        times_ms.push(elapsed_ms);
    }
    let total_secs = total_start.elapsed().as_secs_f64();

    times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_ms = times_ms[FRAMES / 2];
    let p95_ms = times_ms[(FRAMES * 95) / 100];
    let worst_ms = times_ms[FRAMES - 1];
    let fps = (FRAMES as f64) / total_secs;

    println!(
        "frames={FRAMES} median_ms={median_ms:.3} p95_ms={p95_ms:.3} worst_ms={worst_ms:.3} fps={fps:.1}"
    );
}
