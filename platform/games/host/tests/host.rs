//! Integration tests for `game-host` covering ABI version 1 and fuel/memory limits.

use game_host::{BUTTON_DOWN, BUTTON_UP, Game, GameError, SCREEN_BYTES};
use sha2::{Digest, Sha256};

const PADDLE_WASM: &[u8] = include_bytes!("paddle.wasm");
const PLASMA_WASM: &[u8] = include_bytes!("plasma.wasm");
const LOOP_WAT: &[u8] = include_bytes!("loop.wat");

fn run_600_frames(seed: u64) -> [u8; 32] {
    let mut game = Game::load(PADDLE_WASM, seed).expect("load paddle.wasm");
    for i in 0..600u64 {
        let buttons = if (i / 30) % 2 == 0 {
            BUTTON_UP
        } else {
            BUTTON_DOWN
        };
        let screen = game.frame((i + 1) * 16, buttons).expect("frame");
        assert_eq!(screen.len(), SCREEN_BYTES);
    }
    let final_screen = game.frame(601 * 16, 0).expect("final frame");
    let mut hasher = Sha256::new();
    hasher.update(final_screen);
    hasher.finalize().into()
}

#[test]
fn loads_and_runs_600_frames() {
    let mut game = Game::load(PADDLE_WASM, 42).expect("load paddle.wasm");
    for i in 0..600u64 {
        let screen = game.frame((i + 1) * 16, BUTTON_UP).expect("frame");
        assert_eq!(screen.len(), SCREEN_BYTES);
    }
}

#[test]
fn same_seed_and_buttons_give_identical_screens_at_frame_600() {
    let first = run_600_frames(0xCAFE_BABE);
    let second = run_600_frames(0xCAFE_BABE);
    assert_eq!(first, second);
}

#[test]
fn file_with_import_is_refused() {
    let wat = br#"
(module
  (import "env" "clock" (func $clock (result i64)))
  (memory (export "memory") 4)
  (func (export "game_abi") (result i32)
    i32.const 1)
  (func (export "game_init") (param i64))
  (func (export "game_frame") (param i64 i32))
  (func (export "game_screen") (result i32)
    i32.const 0)
)
"#;
    let err = Game::load(wat, 1).err().expect("must refuse import");
    assert!(matches!(err, GameError::ImportsForbidden));
}

#[test]
fn file_with_wrong_game_abi_is_refused() {
    let wat = br#"
(module
  (memory (export "memory") 4)
  (func (export "game_abi") (result i32)
    i32.const 2)
  (func (export "game_init") (param i64))
  (func (export "game_frame") (param i64 i32))
  (func (export "game_screen") (result i32)
    i32.const 0)
)
"#;
    let err = Game::load(wat, 1).err().expect("must refuse wrong ABI");
    assert!(matches!(err, GameError::UnsupportedAbi(2)));
}

#[test]
fn game_that_loops_forever_returns_error_rather_than_hanging() {
    let mut game = Game::load(LOOP_WAT, 1).expect("load loop.wat");
    let err = game
        .frame(16, 0)
        .expect_err("infinite loop must run out of fuel");
    assert!(matches!(err, GameError::ExecutionFailed));
}

#[test]
fn memory_over_the_limit_is_refused() {
    let wat = br#"
(module
  (memory (export "memory") 257)
  (func (export "game_abi") (result i32)
    i32.const 1)
  (func (export "game_init") (param i64))
  (func (export "game_frame") (param i64 i32))
  (func (export "game_screen") (result i32)
    i32.const 0)
)
"#;
    let err = Game::load(wat, 1)
        .err()
        .expect("must refuse oversized memory");
    assert!(matches!(err, GameError::MemoryLimitExceeded));
}

#[test]
fn plasma_computes_full_screen_within_fuel_budget() {
    let mut game = Game::load(PLASMA_WASM, 7).expect("load plasma.wasm");
    let screen = game.frame(16, BUTTON_UP).expect("plasma frame");
    assert_eq!(screen.len(), SCREEN_BYTES);
    assert!(game.last_frame_fuel() > 4_000_000);
}
