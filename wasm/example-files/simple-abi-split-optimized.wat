(module
    (func $f (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $tx
        i64.load
        i32.wrap_i64
        i32.const 4
        i32.mul
        local.get $tx
        i32.load offset=8
        local.set $i32_local          ;;Save value for store
        local.set $memory_address     ;;Save address for store
        local.get $state
        local.get $i32_local
        i32.store
        local.get $utx
        local.get $memory_address
        i32.const 0                   ;;Convert =offset to value
        i32.add
        i32.store
        local.get $utx                ;;Save naddr = 1
        i32.const 1
        i32.store8 offset=63
        i32.const 2                   ;;Return index to next microtransaction
    )
    (func $f_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $utx                ;;Restore store address
        i32.load
        local.get $state              ;;Restore store value
        i32.load
        i32.store
        i32.const 0
    )
    (func $__enter (param i64 i64 i32) (result i32)
      i32.const 0
      i32.const 12
      i32.store16 offset=1032
      i32.const 0
      local.get 0
      i64.store offset=1024
      i32.const 0
      local.get 2
      i32.store offset=1048
      i32.const 0
      local.get 1
      i64.store offset=1040
      i32.const 1 ;; table entry for first function
    )
    (func $__step (param i32) (result i32)
      i32.const 1040
      i32.const 1056
      i32.const 1024
      local.get 0
      call_indirect (type 0)
    )
    (func $__get_utx_addrs (param i32) (result i64)
      local.get 0
      i32.const 3
      i32.shl
      i32.const 1056
      i32.add
      i64.load
    )
    (func $__get_utx_log2lens (param i32) (result i32)
      local.get 0
      i32.const 1112
      i32.add
      i32.load8_u
    )
    (func $__get_utx_naddr (result i32)
      i32.const 0
      i32.load8_u offset=1119
    )
    (func $__get_balance (param i32) (result i32)
      local.get 0
      i32.const 4
      i32.mul
      i32.load
    )
    (export "__enter" (func $__enter))
    (export "__step" (func $__step))
    (export "__get_utx_addrs" (func $__get_utx_addrs))
    (export "__get_utx_log2lens" (func $__get_utx_log2lens))
    (export "__get_utx_naddr" (func $__get_utx_naddr))
    (export "__get_balance" (func $__get_balance))
    (table 3 funcref)
    (elem (i32.const 1) func $f $f_1)
    (memory 10)
    (type $utx_f (func (param i32 i32 i32) (result i32)))
)
