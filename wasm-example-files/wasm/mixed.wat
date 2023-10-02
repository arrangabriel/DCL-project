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
        i32.const 5
        call $add
    )

    (func $add (param $op1 i32) (param $op2 i32) (result i32)
        local.get $op1
        local.get $op2
        i32.add
    )
)
