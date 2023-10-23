(module
    (func $load (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 100
        i32.load
    )

    (func $store (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 200
        f64.const 20
        f64.store
        i32.const 0
    )

    (func $load_and_store (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 100
        i32.load
        i64.const 5
        i64.const 10
        i64.add
        i64.store
        i32.const 0
    )

    (func $__step (param i32) (result i32)
      i32.const 1040
      i32.const 1056
      i32.const 1024
      local.get 0
      call_indirect (type 0)
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
      i32.const 3
    )

    (memory 3) ;; What should the size of this memory actually be?
    (export "__enter" (func $__enter))
    (export "__step" (func $__step))
    (;
      This type is needed for call_indirect.
      It is the signature for each microtransaction.
      (param $tx i32) (param $utx i32) (param $state i32) (result i32)
    ;)
    (type $utx_f (func (param i32 i32 i32) (result i32)))
)
