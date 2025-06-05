(module
  (import "host" "selected_text" (func $selected_text (param i32) (result i32)))
  (import "host" "llm_complete" (func $llm_complete (param i32 i32 i32) (result i32)))
  (import "host" "stdout" (func $stdout (param i32 i32)))
  (memory (export "memory") 1)
  (func (export "run") (local $len i32) (local $out_len i32)
    ;; fetch selected text into memory offset 0
    i32.const 0
    call $selected_text
    local.set $len
    ;; call llm_complete(selected) -> result at offset 100
    i32.const 0
    local.get $len
    i32.const 100
    call $llm_complete
    local.set $out_len
    ;; print result
    i32.const 100
    local.get $out_len
    call $stdout
  )
)
