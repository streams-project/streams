//! Sandboxed WebAssembly game host for version 1 of the game ABI.

mod wat;

use thiserror::Error;
use wasmi::{
    CompilationMode, Config, Engine, Linker, Memory, Module, Store, StoreLimits,
    StoreLimitsBuilder, TypedFunc,
};

pub const ABI_VERSION: i32 = 1;
pub const SCREEN_WIDTH: usize = 320;
pub const SCREEN_HEIGHT: usize = 180;
pub const SCREEN_BYTES: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 4;
pub const MAX_MEMORY_BYTES: usize = 16 * 1024 * 1024;
const WASM_PAGE_BYTES: u64 = 65_536;
const MAX_MEMORY_PAGES: u64 = (MAX_MEMORY_BYTES as u64) / WASM_PAGE_BYTES;

// `paddle.wasm` uses 37,391 fuel units per frame and `plasma.wasm` (computing all 57,600
// pixels every frame from a sine table) uses 4,049,915 fuel units per frame. A limit of
// 10,000,000 fuel units gives over 2x headroom for a full-screen per-pixel effect while
// stopping an infinite loop in a few milliseconds.
const FUEL_PER_FRAME: u64 = 10_000_000;

pub const BUTTON_UP: u32 = 1;
pub const BUTTON_DOWN: u32 = 2;
pub const BUTTON_LEFT: u32 = 4;
pub const BUTTON_RIGHT: u32 = 8;
pub const BUTTON_OK: u32 = 16;
pub const BUTTON_BACK: u32 = 32;

const REQUIRED_EXPORTS: [&str; 5] = [
    "memory",
    "game_abi",
    "game_init",
    "game_frame",
    "game_screen",
];

#[derive(Debug, Error)]
pub enum GameError {
    #[error("invalid Wasm game module")]
    InvalidModule,
    #[error("game module must not import anything")]
    ImportsForbidden,
    #[error("game module exports do not match ABI version 1")]
    InvalidExports,
    #[error("unsupported game ABI version {0}")]
    UnsupportedAbi(i32),
    #[error("game linear memory exceeds the 16 MB limit")]
    MemoryLimitExceeded,
    #[error("game screen lies outside linear memory")]
    ScreenOutOfBounds,
    #[error("game execution trapped or ran out of fuel")]
    ExecutionFailed,
}

pub struct Game {
    store: Store<StoreLimits>,
    memory: Memory,
    game_frame: TypedFunc<(i64, i32), ()>,
    game_screen: TypedFunc<(), i32>,
    last_fuel: u64,
}

impl Game {
    /// Refuses a file that is not a game of this ABI, or that imports anything.
    pub fn load(bytes: &[u8], seed: u64) -> Result<Self, GameError> {
        let wasm = wat::maybe_assemble(bytes)?;

        let mut config = Config::default();
        config.consume_fuel(true);
        config.compilation_mode(CompilationMode::Eager);
        let engine = Engine::new(&config);

        let module = Module::new(&engine, &wasm).map_err(|_| GameError::InvalidModule)?;

        if module.imports().next().is_some() {
            return Err(GameError::ImportsForbidden);
        }

        let mut export_count = 0usize;
        for export in module.exports() {
            export_count += 1;
            if !REQUIRED_EXPORTS.contains(&export.name()) {
                return Err(GameError::InvalidExports);
            }
            if let Some(mem_ty) = export.ty().memory() {
                if mem_ty.minimum() > MAX_MEMORY_PAGES {
                    return Err(GameError::MemoryLimitExceeded);
                }
                if mem_ty.maximum().is_some_and(|max| max > MAX_MEMORY_PAGES) {
                    return Err(GameError::MemoryLimitExceeded);
                }
            }
        }
        if export_count != REQUIRED_EXPORTS.len() {
            return Err(GameError::InvalidExports);
        }

        let limits = StoreLimitsBuilder::new()
            .memory_size(MAX_MEMORY_BYTES)
            .memories(1)
            .instances(1)
            .trap_on_grow_failure(true)
            .build();
        let mut store = Store::new(&engine, limits);
        store.limiter(|lim| lim);
        store
            .set_fuel(FUEL_PER_FRAME)
            .map_err(|_| GameError::ExecutionFailed)?;

        let linker = Linker::<StoreLimits>::new(&engine);
        let instance = linker
            .instantiate_and_start(&mut store, &module)
            .map_err(|_| GameError::InvalidModule)?;

        let memory = instance
            .get_memory(&store, "memory")
            .ok_or(GameError::InvalidExports)?;
        if memory.data_size(&store) > MAX_MEMORY_BYTES {
            return Err(GameError::MemoryLimitExceeded);
        }

        let game_abi = instance
            .get_typed_func::<(), i32>(&store, "game_abi")
            .map_err(|_| GameError::InvalidExports)?;
        let game_init = instance
            .get_typed_func::<i64, ()>(&store, "game_init")
            .map_err(|_| GameError::InvalidExports)?;
        let game_frame = instance
            .get_typed_func::<(i64, i32), ()>(&store, "game_frame")
            .map_err(|_| GameError::InvalidExports)?;
        let game_screen = instance
            .get_typed_func::<(), i32>(&store, "game_screen")
            .map_err(|_| GameError::InvalidExports)?;

        store
            .set_fuel(FUEL_PER_FRAME)
            .map_err(|_| GameError::ExecutionFailed)?;
        let abi = game_abi
            .call(&mut store, ())
            .map_err(|_| GameError::ExecutionFailed)?;
        if abi != ABI_VERSION {
            return Err(GameError::UnsupportedAbi(abi));
        }

        store
            .set_fuel(FUEL_PER_FRAME)
            .map_err(|_| GameError::ExecutionFailed)?;
        game_init
            .call(&mut store, seed as i64)
            .map_err(|_| GameError::ExecutionFailed)?;

        Ok(Self {
            store,
            memory,
            game_frame,
            game_screen,
            last_fuel: 0,
        })
    }

    /// Fuel consumed during the most recent `frame` call.
    pub fn last_frame_fuel(&self) -> u64 {
        self.last_fuel
    }

    /// One step. `buttons` is the bits above. Runs out of fuel rather than running forever.
    pub fn frame(&mut self, now_ms: u64, buttons: u32) -> Result<&[u8], GameError> {
        self.store
            .set_fuel(FUEL_PER_FRAME)
            .map_err(|_| GameError::ExecutionFailed)?;

        self.game_frame
            .call(&mut self.store, (now_ms as i64, buttons as i32))
            .map_err(|_| GameError::ExecutionFailed)?;

        let ptr = self
            .game_screen
            .call(&mut self.store, ())
            .map_err(|_| GameError::ExecutionFailed)?;
        let remaining = self.store.get_fuel().unwrap_or(0);
        self.last_fuel = FUEL_PER_FRAME.saturating_sub(remaining);
        if ptr < 0 {
            return Err(GameError::ScreenOutOfBounds);
        }

        let offset = ptr as usize;
        let end = offset
            .checked_add(SCREEN_BYTES)
            .ok_or(GameError::ScreenOutOfBounds)?;
        let data = self.memory.data(&self.store);
        if end > data.len() {
            return Err(GameError::ScreenOutOfBounds);
        }

        Ok(&data[offset..end])
    }
}
