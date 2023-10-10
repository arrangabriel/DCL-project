(module
  (type (;0;) (func))
  (type (;1;) (func (param i32 i32 i32) (result i32)))
  (func $__wasm_call_ctors (;0;) (type 0))
  (func $f_a (;1;) (type 1) (param i32 i32 i32) (result i32)
    local.get 1
    i64.const 3
    i64.store
    i32.const 1
  )
  (func $f_b (;2;) (type 1) (param i32 i32 i32) (result i32)
    local.get 2
    local.get 1
    i64.load offset=8
    i64.store
    i32.const 0
  )
  (func $f_c (;3;) (type 1) (param i32 i32 i32) (result i32)
    local.get 1
    local.get 1
    local.get 1
    call $f_a
    drop
    local.get 1
    local.get 1
    local.get 2
    call $f_b
    drop
    i32.const 0
  )
  (table (;0;) 2 2 funcref)
  (memory (;0;) 2)
  (global $__stack_pointer (;0;) (mut i32) i32.const 66560)
  (global (;1;) i32 i32.const 1024)
  (global (;2;) i32 i32.const 1024)
  (global (;3;) i32 i32.const 1024)
  (global (;4;) i32 i32.const 66560)
  (global (;5;) i32 i32.const 1024)
  (global (;6;) i32 i32.const 66560)
  (global (;7;) i32 i32.const 131072)
  (global (;8;) i32 i32.const 0)
  (global (;9;) i32 i32.const 1)
  (export "memory" (memory 0))
  (export "__wasm_call_ctors" (func $__wasm_call_ctors))
  (export "f_a" (func $f_a))
  (export "f_b" (func $f_b))
  (export "f_c" (func $f_c))
  (export "__indirect_function_table" (table 0))
  (export "__dso_handle" (global 1))
  (export "__data_end" (global 2))
  (export "__stack_low" (global 3))
  (export "__stack_high" (global 4))
  (export "__global_base" (global 5))
  (export "__heap_base" (global 6))
  (export "__heap_end" (global 7))
  (export "__memory_base" (global 8))
  (export "__table_base" (global 9))
  (elem (;0;) (i32.const 1) func $f_b)
  (@producers
    (processed-by "Apple clang" "14.0.3 (clang-1403.0.22.14.1)")
  )
)