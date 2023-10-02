(module
    (func $add (param $op1 i32) (param $op2 i32) (result i32)
        local.get $op1
        local.get $op2
        i32.add
    )
    (func $divide (param $op1 i32) (param $op2 i32) (param $op3 f32) (result f32)
        local.get $op2
        local.get $op1
        call $add
        f32.convert_i32_s
        local.get $op3
        f32.div
    )
)
