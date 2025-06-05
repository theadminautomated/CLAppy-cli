(module
  (import "host" "selected_text" (func $selected_text (param i32) (result i32)))
  (import "host" "stdout" (func $stdout (param i32 i32)))
  (memory (export "memory") 1)
  (func (export "run") (local $len i32)
    i32.const 0
    call $selected_text
    local.set $len
    i32.const 0
    local.get $len
    call $stdout
  )
)
