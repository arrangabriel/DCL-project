mod utils;

#[test]
fn joined_lines() {
    utils::test_transform(
        "\
(module
    (func $f_1 (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 1
        drop)

    (func $f_2 (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 1
        drop )

    (func $f_3 (param $tx i32) (param $utx i32) (param $state i32) (result i32) )
)",
        "\
(module
    (func $f_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        i32.const 1
        drop
    )
    (func $f_2 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        i32.const 1
        drop
    )
    (func $f_3 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
    )
    (table 4 funcref)
    (elem (i32.const 1) func $f_1 $f_2 $f_3)
    (memory 10)
    (type $utx_f (func (param i32 i32 i32) (result i32)))
)",
    );
}

#[test]
fn single_assignment_conversion() {
    utils::test_transform(
        "\
(module
    (func $without_locals (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 1
        local.get 1
        i32.add
        local.set 1
        i32.const 1
        local.get 1
        i32.add
        local.tee 1
        local.set 2
        local.get 1
        local.get 2
        i32.add
        drop
    )
    (func $with_locals (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i64)
        local.get 3
        local.tee 3
        local.set 2
        local.get 2
        drop
    )
)",
        "\
(module
    (func $without_locals (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        i32.const 1
        local.get 1
        i32.add
        local.set 3
        i32.const 1
        local.get 3
        i32.add
        local.tee 3
        local.set 4
        local.get 3
        local.get 4
        i32.add
        drop
    )
    (func $with_locals (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i64 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get 3
        local.tee 3
        local.set 6
        local.get 6
        drop
    )
    (table 3 funcref)
    (elem (i32.const 1) func $without_locals $with_locals)
    (memory 10)
    (type $utx_f (func (param i32 i32 i32) (result i32)))
)"
    );
}
