(module
  (memory (export "memory") 4)
  (func (export "game_abi") (result i32)
    i32.const 1)
  (func (export "game_init") (param i64))
  (func (export "game_frame") (param i64 i32)
    (loop $forever
      br $forever))
  (func (export "game_screen") (result i32)
    i32.const 0)
)
