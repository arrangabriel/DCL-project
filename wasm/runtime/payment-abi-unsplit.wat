(module
  (type (;0;) (func (param i32 i32 i32) (result i32)))
  (type (;1;) (func))
  (type (;2;) (func (param i32) (result i32)))
  (type (;3;) (func (result i32)))
  (type (;4;) (func (param i32) (result i64)))
  (func $__wasm_call_ctors (type 1))
  (func $enter (type 0) (param i32 i32 i32) (result i32)
    (local i32 i32)
    block  ;; label = @1
      local.get 2
      i32.load align=1
      local.tee 3
      i32.const 9999
      i32.gt_u
      br_if 0 (;@1;)
      local.get 2
      i32.load16_u offset=4 align=1
      i32.const 8
      i32.ne
      br_if 0 (;@1;)
      local.get 0
      i32.load align=1
      local.tee 2
      local.get 3
      i32.eq
      br_if 0 (;@1;)
      local.get 2
      i32.const 9999
      i32.gt_u
      br_if 0 (;@1;)
      local.get 0
      i32.const 4
      i32.add
      i32.load align=1
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        local.get 3
        br_if 0 (;@2;)
        local.get 2
        i32.const 3
        i32.shl
        i32.const 1088
        i32.add
        local.tee 3
        i32.load
        local.tee 2
        local.get 0
        i32.add
        local.tee 0
        local.get 2
        i32.lt_u
        br_if 1 (;@1;)
        local.get 3
        local.get 0
        i32.store
        i32.const 0
        return
      end
      local.get 3
      i32.const 3
      i32.shl
      i32.const 1088
      i32.add
      local.tee 3
      i32.load
      local.tee 4
      local.get 0
      i32.lt_u
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      i32.const 3
      i32.shl
      i32.const 1088
      i32.add
      local.tee 2
      i32.load
      i32.const -1
      i32.xor
      i32.gt_u
      br_if 0 (;@1;)
      local.get 3
      local.get 4
      local.get 0
      i32.sub
      i32.store
      local.get 2
      local.get 2
      i32.load
      local.get 0
      i32.add
      i32.store
    end
    i32.const 0
  )
  (func $__enter (type 0) (param i32 i32 i32) (result i32)
    i32.const 0
    i32.const 8
    i32.store16 offset=1028
    i32.const 0
    local.get 0
    i32.store offset=1024
    i32.const 0
    local.get 2
    i32.store offset=1036
    i32.const 0
    local.get 1
    i32.store offset=1032
    i32.const 1
  )
  (func $__step (type 2) (param i32) (result i32)
    i32.const 1032
    i32.const 1040
    i32.const 1024
    local.get 0
    call_indirect (type 0)
  )
  (func $__get_utx_addrs (type 2) (param i32) (result i32)
    local.get 0
    i32.const 2
    i32.shl
    i32.const 1040
    i32.add
    i32.load
  )
  (func $__get_utx_log2lens (type 2) (param i32) (result i32)
    local.get 0
    i32.const 1068
    i32.add
    i32.load8_u
  )
  (func $__get_utx_naddr (type 3) (result i32)
    i32.const 0
    i32.load8_u offset=1075
  )
  (func $__get_balance (type 4) (param i32) (result i64)
    local.get 0
    i32.const 3
    i32.shl
    i32.const 1088
    i32.add
    i64.load
  )
  (table (;0;) 2 2 funcref)
  (memory (;0;) 3)
  (global $__stack_pointer (mut i32) (i32.const 146624))
  (global (;1;) i32 (i32.const 1024))
  (global (;2;) i32 (i32.const 81088))
  (global (;3;) i32 (i32.const 81088))
  (global (;4;) i32 (i32.const 146624))
  (global (;5;) i32 (i32.const 1024))
  (global (;6;) i32 (i32.const 146624))
  (global (;7;) i32 (i32.const 196608))
  (global (;8;) i32 (i32.const 0))
  (global (;9;) i32 (i32.const 1))
  (export "memory" (memory 0))
  (export "__wasm_call_ctors" (func $__wasm_call_ctors))
  (export "enter" (func $enter))
  (export "__enter" (func $__enter))
  (export "__step" (func $__step))
  (export "__get_utx_addrs" (func $__get_utx_addrs))
  (export "__get_utx_log2lens" (func $__get_utx_log2lens))
  (export "__get_utx_naddr" (func $__get_utx_naddr))
  (export "__get_balance" (func $__get_balance))
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
  (elem (;0;) (i32.const 1) func $enter)
)
