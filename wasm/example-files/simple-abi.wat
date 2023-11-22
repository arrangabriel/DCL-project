(module
    (func $f (type $tx_f) (param $tx i32) (param $state i32)
        local.get $tx
        i64.load
        i32.wrap_i64
        i32.const 4
        i32.mul
        local.get $tx
        i32.load offset=8
        i32.store
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

    (memory 1)
    (export "__enter" (func $__enter))
    (export "__step" (func $__step))
    (export "__get_utx_addrs" (func $__get_utx_addrs))
    (export "__get_utx_log2lens" (func $__get_utx_log2lens))
    (export "__get_utx_naddr" (func $__get_utx_naddr))
    (export "__get_balance" (func $__get_balance))
    (type $tx_f (func (param i32 i32) (result i32)))
    (table 2 funcref)
    (elem (i32.const 1) func $f)
)