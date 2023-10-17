(module
    (func $load (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 100
        i32.load
    )

    (func $store (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 200
        f64.const 20
        f64.store
        i32.const 0
    )

    (func $load_and_store (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 100
        i32.load
        i64.const 5
        i64.const 10
        i64.add
        i64.store
        i32.const 0
    )

    (memory 10)
)
