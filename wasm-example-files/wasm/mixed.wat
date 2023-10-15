(module
    (func $load (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 100
        i32.load
        drop
    )

    (func $store (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 200
        f64.const 20
        f64.store
    )

    (memory 10)
)
