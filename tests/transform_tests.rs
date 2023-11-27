use pretty_assertions::assert_eq;

use chop_up::transform_wat_string;

#[test]
fn load() {
    test_transform(
        "\
(module
    (func $load (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 1
        i32.load
        drop
        i32.const 0
    )
)",
        "\
(module
    (func $load (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        i32.const 1
        local.set $memory_address
        local.get $utx
        local.get $memory_address
        i32.const 0
        i32.add
        i32.store
        local.get $utx
        i32.const 1
        i32.store8 offset=35
        i32.const 2
    )
    (func $load_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $utx
        i32.load
        i32.load
        drop
        i32.const 0
    )
    (table 3 funcref)
    (elem (i32.const 1) func $load $load_1)
    (memory 10)
    (type $utx_f (func (param i32 i32 i32) (result i32)))
)",
    );
}

#[test]
fn store() {
    test_transform(
        "\
(module
    (func $store (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        i32.const 0
        i32.const 1
        i32.store
        i32.const 0
    )
)
    ",
        "\
(module
    (func $store (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        i32.const 0
        i32.const 1
        local.set $i32_local
        local.set $memory_address
        local.get $state
        local.get $i32_local
        i32.store offset=6
        local.get $utx
        local.get $memory_address
        i32.const 0
        i32.add
        i32.store
        local.get $utx
        i32.const 1
        i32.store8 offset=35
        i32.const 2
    )
    (func $store_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $utx
        i32.load
        local.get $state
        i32.load offset=6
        i32.store
        i32.const 0
    )
    (table 3 funcref)
    (elem (i32.const 1) func $store $store_1)
    (memory 10)
    (type $utx_f (func (param i32 i32 i32) (result i32)))
)",
    );
}

#[test]
fn block() {
    test_transform(
        "\
(module
    (func $block (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        f32.const 1
        (block $block_a
            i32.const 1
            (block $block_b
                i32.const 1
                i32.const 2
                i32.load
                i32.add
                drop
            )
            i32.load
            drop
        )
        drop
        i32.const 0
    )
    (memory 1)
)",
        "\
(module
    (func $block (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        f32.const 1
        local.tee $f32_local
        local.get $state
        local.get $f32_local
        f32.store offset=14
        (block $block_a
            i32.const 1
            local.tee $i32_local
            local.get $state
            local.get $i32_local
            i32.store offset=18
            (block $block_b
                i32.const 1
                i32.const 2
                local.set $memory_address
                local.get $utx
                local.get $memory_address
                i32.const 0
                i32.add
                i32.store
                local.get $utx
                i32.const 1
                i32.store8 offset=35
                local.set $i32_local
                local.get $state
                local.get $i32_local
                i32.store offset=22
                i32.const 2
                return
            )
            local.set $memory_address
            local.get $utx
            local.get $memory_address
            i32.const 0
            i32.add
            i32.store
            local.get $utx
            i32.const 1
            i32.store8 offset=35
            i32.const 3
            return
        )
        drop
        i32.const 0
    )
    (func $block_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        f32.load offset=14
        (block $block_a
            local.get $state
            i32.load offset=18
            (block $block_b
                local.get $state
                i32.load offset=22
                local.get $utx
                i32.load
                i32.load
                i32.add
                drop
            )
            local.set $memory_address
            local.get $utx
            local.get $memory_address
            i32.const 0
            i32.add
            i32.store
            local.get $utx
            i32.const 1
            i32.store8 offset=35
            i32.const 3
            return
        )
        drop
        i32.const 0
    )
    (func $block_2 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        f32.load offset=14
        (block $block_a
            local.get $utx
            i32.load
            i32.load
            drop
        )
        drop
        i32.const 0
    )
    (table 4 funcref)
    (elem (i32.const 1) func $block $block_1 $block_2)
    (memory 10)
    (type $utx_f (func (param i32 i32 i32) (result i32)))
)",
    );
}

#[test]
fn stack_and_locals() {
    test_transform(
        "\
(module
    (func $stack_and_locals (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i64)
        i64.const 1
        i32.const 1
        i32.const 2
        local.set 3
        i32.const 1000
        i32.load
        i32.add
        local.get 3
        i32.add
        drop
        drop
        i32.const 0
    )
    (memory 1)
)",
        "\
(module
    (func $stack_and_locals (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i64)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        i64.const 1
        i32.const 1
        i32.const 2
        local.set 3
        i32.const 1000
        local.set $memory_address
        local.get $utx
        local.get $memory_address
        i32.const 0
        i32.add
        i32.store
        local.get $utx
        i32.const 1
        i32.store8 offset=35
        local.set $i32_local
        local.get $state
        local.get $i32_local
        i32.store offset=14
        local.set $i64_local
        local.get $state
        local.get $i64_local
        i64.store offset=18
        local.get $state
        local.get 3
        i32.store offset=26
        local.get $state
        local.get 4
        i64.store offset=30
        i32.const 2
    )
    (func $stack_and_locals_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i64)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=26
        local.set 3
        local.get $state
        i64.load offset=30
        local.set 4
        local.get $state
        i64.load offset=18
        local.get $state
        i32.load offset=14
        local.get $utx
        i32.load
        i32.load
        i32.add
        local.get 3
        i32.add
        drop
        drop
        i32.const 0
    )
    (table 3 funcref)
    (elem (i32.const 1) func $stack_and_locals $stack_and_locals_1)
    (memory 10)
    (type $utx_f (func (param i32 i32 i32) (result i32)))
)"
    );
}

fn test_transform(input: &str, expected_output: &str) {
    let mut output_vec: Vec<u8> = Vec::new();
    transform_wat_string(input, &mut output_vec, 6, false, false).unwrap();
    let output_wat = String::from_utf8(output_vec).unwrap();
    assert_eq!(output_wat.trim(), expected_output.trim());
}
