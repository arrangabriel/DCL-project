(module
    (memory 10)
    (func $store (param $index i32) (param $value f64)
        local.get $index
        local.get $value
        f64.store
    )

    (func $load (param $index i32) (result i32)
        local.get $index
        i32.load
    )
)