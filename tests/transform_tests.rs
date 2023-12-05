mod utils;
#[test]
fn load() {
    utils::test_transform(
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
    utils::test_transform(
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
    utils::test_transform(
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
    utils::test_transform(
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

#[test]
fn transaction() {
    utils::test_transform(
        "\
(module
  (type (;0;) (func (param i32 i32 i32) (result i32)))
  (type (;1;) (func))
  (type (;2;) (func (param i32) (result i32)))
  (type (;3;) (func (result i32)))
  (type (;4;) (func (param i32) (result i64)))
  (func $__wasm_call_ctors (type 1))
  (func $enter (type 0) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32)
    (block
      local.get 2
      i32.load align=1
      local.tee 3
      i32.const 9999
      i32.gt_u
      br_if 0
      local.get 2
      i32.load16_u offset=4 align=1
      i32.const 8
      i32.ne
      br_if 0
      local.get 0
      i32.load align=1
      local.tee 5
      local.get 3
      i32.eq
      br_if 0
      local.get 5
      i32.const 9999
      i32.gt_u
      br_if 0
      local.get 0
      i32.const 4
      i32.add
      i32.load align=1
      local.tee 6
      i32.eqz
      br_if 0
      (block
        local.get 3
        br_if 0
        local.get 5
        i32.const 3
        i32.shl
        i32.const 1216
        i32.add
        local.tee 3
        i32.load
        local.tee 5
        local.get 6
        i32.add
        local.tee 6
        local.get 5
        i32.lt_u
        br_if 1
        local.get 3
        local.get 6
        i32.store
        i32.const 0
        return
      )
      local.get 3
      i32.const 3
      i32.shl
      i32.const 1216
      i32.add
      local.tee 3
      i32.load
      local.tee 4
      local.get 6
      i32.lt_u
      br_if 0
      local.get 6
      local.get 5
      i32.const 3
      i32.shl
      i32.const 1216
      i32.add
      local.tee 5
      i32.load
      i32.const -1
      i32.xor
      i32.gt_u
      br_if 0
      local.get 3
      local.get 4
      local.get 6
      i32.sub
      i32.store
      local.get 5
      local.get 5
      i32.load
      local.get 6
      i32.add
      i32.store
    )
    i32.const 0
  )
  (func $__enter (type 0) (param i32 i32 i32) (result i32)
    i32.const 0
    i32.const 8
    i32.store16 offset=1028
    i32.const 0
    local.get 0
    i32.store offset=1024
    i32.const 0
    local.get 2
    i32.store offset=1164
    i32.const 0
    local.get 1
    i32.store offset=1160
    i32.const 0
    i64.const 0
    i64.store offset=1168 align=4
    i32.const 0
    i64.const 0
    i64.store offset=1176 align=4
    i32.const 0
    i64.const 0
    i64.store offset=1184 align=4
    i32.const 0
    i64.const 0
    i64.store offset=1192 align=4
    i32.const 0
    i32.const 0
    i32.store offset=1200
    i32.const 1
  )
  (func $__step (type 2) (param i32) (result i32)
    i32.const 1160
    i32.const 1168
    i32.const 1024
    local.get 0
    call_indirect (type 0)
  )
  (func $__get_utx_addrs (type 2) (param i32) (result i32)
    local.get 0
    i32.const 2
    i32.shl
    i32.const 1168
    i32.add
    i32.load
  )
  (func $__get_utx_log2lens (type 2) (param i32) (result i32)
    local.get 0
    i32.const 1196
    i32.add
    i32.load8_u
  )
  (func $__get_utx_naddr (type 3) (result i32)
    i32.const 0
    i32.load8_u offset=1203
  )
  (func $__get_balance (type 4) (param i32) (result i64)
    local.get 0
    i32.const 3
    i32.shl
    i32.const 1216
    i32.add
    i64.load
  )
  (func $__get_user (type 3) (result i32)
    i32.const 0
    i32.load offset=1024
  )
  (func $__get_transform_storage (type 2) (param i32) (result i32)
    local.get 0
    i32.const 1030
    i32.add
    i32.load8_s
  )
  (table (;0;) 2 2 funcref)
  (memory (;0;) 3)
  (global $__stack_pointer (mut i32) (i32.const 146752))
  (global (;1;) i32 (i32.const 1024))
  (global (;2;) i32 (i32.const 81216))
  (global (;3;) i32 (i32.const 81216))
  (global (;4;) i32 (i32.const 146752))
  (global (;5;) i32 (i32.const 1024))
  (global (;6;) i32 (i32.const 146752))
  (global (;7;) i32 (i32.const 196608))
  (global (;8;) i32 (i32.const 0))
  (global (;9;) i32 (i32.const 1))
  (export \"memory\" (memory 0))
  (export \"__wasm_call_ctors\" (func $__wasm_call_ctors))
  (export \"enter\" (func $enter))
  (export \"__enter\" (func $__enter))
  (export \"__step\" (func $__step))
  (export \"__get_utx_addrs\" (func $__get_utx_addrs))
  (export \"__get_utx_log2lens\" (func $__get_utx_log2lens))
  (export \"__get_utx_naddr\" (func $__get_utx_naddr))
  (export \"__get_balance\" (func $__get_balance))
  (export \"__get_user\" (func $__get_user))
  (export \"__get_transform_storage\" (func $__get_transform_storage))
  (export \"__indirect_function_table\" (table 0))
  (export \"__dso_handle\" (global 1))
  (export \"__data_end\" (global 2))
  (export \"__stack_low\" (global 3))
  (export \"__stack_high\" (global 4))
  (export \"__global_base\" (global 5))
  (export \"__heap_base\" (global 6))
  (export \"__heap_end\" (global 7))
  (export \"__memory_base\" (global 8))
  (export \"__table_base\" (global 9))
  (elem (;0;) (i32.const 1) func $enter)
)",

        "\
(module
    (func $__wasm_call_ctors (type 1))
    (func $enter (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        local.get 3
        i32.store offset=14
        local.get $state
        local.get 4
        i32.store offset=18
        local.get $state
        local.get 5
        i32.store offset=22
        local.get $state
        local.get 6
        i32.store offset=26
        (block
            local.get 2
            local.set $memory_address
            local.get $utx
            local.get $memory_address
            i32.const 0
            i32.add
            i32.store
            local.get $utx
            i32.const 1
            i32.store8 offset=35
            local.get $state
            local.get 3
            i32.store offset=14
            local.get $state
            local.get 4
            i32.store offset=18
            local.get $state
            local.get 5
            i32.store offset=22
            local.get $state
            local.get 6
            i32.store offset=26
            i32.const 2
            return
        )
        i32.const 0
    )
    (func $__enter (type 0) (param i32 i32 i32) (result i32)
        i32.const 0
        i32.const 8
        i32.store16 offset=1028
        i32.const 0
        local.get 0
        i32.store offset=1024
        i32.const 0
        local.get 2
        i32.store offset=1164
        i32.const 0
        local.get 1
        i32.store offset=1160
        i32.const 0
        i64.const 0
        i64.store offset=1168 align=4
        i32.const 0
        i64.const 0
        i64.store offset=1176 align=4
        i32.const 0
        i64.const 0
        i64.store offset=1184 align=4
        i32.const 0
        i64.const 0
        i64.store offset=1192 align=4
        i32.const 0
        i32.const 0
        i32.store offset=1200
        i32.const 1
    )
    (func $__step (type 2) (param i32) (result i32)
        i32.const 1160
        i32.const 1168
        i32.const 1024
        local.get 0
        call_indirect (type 0)
    )
    (func $__get_utx_addrs (type 2) (param i32) (result i32)
        local.get 0
        i32.const 2
        i32.shl
        i32.const 1168
        i32.add
        i32.load
    )
    (func $__get_utx_log2lens (type 2) (param i32) (result i32)
        local.get 0
        i32.const 1196
        i32.add
        i32.load8_u
    )
    (func $__get_utx_naddr (type 3) (result i32)
        i32.const 0
        i32.load8_u offset=1203
    )
    (func $__get_balance (type 4) (param i32) (result i64)
        local.get 0
        i32.const 3
        i32.shl
        i32.const 1216
        i32.add
        i64.load
    )
    (func $__get_user (type 3) (result i32)
        i32.const 0
        i32.load offset=1024
    )
    (func $__get_transform_storage (type 2) (param i32) (result i32)
        local.get 0
        i32.const 1030
        i32.add
        i32.load8_s
    )
    (func $enter_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=14
        local.set 3
        local.get $state
        i32.load offset=18
        local.set 4
        local.get $state
        i32.load offset=22
        local.set 5
        local.get $state
        i32.load offset=26
        local.set 6
        (block
            local.get $utx
            i32.load
            i32.load
            local.tee 3
            i32.const 9999
            i32.gt_u
            br_if 0
            local.get 2
            local.set $memory_address
            local.get $utx
            local.get $memory_address
            i32.const 4
            i32.add
            i32.store
            local.get $utx
            i32.const 1
            i32.store8 offset=35
            local.get $state
            local.get 3
            i32.store offset=14
            local.get $state
            local.get 4
            i32.store offset=18
            local.get $state
            local.get 5
            i32.store offset=22
            local.get $state
            local.get 6
            i32.store offset=26
            i32.const 3
            return
        )
        i32.const 0
    )
    (func $enter_1_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=14
        local.set 3
        local.get $state
        i32.load offset=18
        local.set 4
        local.get $state
        i32.load offset=22
        local.set 5
        local.get $state
        i32.load offset=26
        local.set 6
        (block
            local.get $utx
            i32.load
            i32.load16_u
            i32.const 8
            i32.ne
            br_if 0
            local.get 0
            local.set $memory_address
            local.get $utx
            local.get $memory_address
            i32.const 0
            i32.add
            i32.store
            local.get $utx
            i32.const 1
            i32.store8 offset=35
            local.get $state
            local.get 3
            i32.store offset=14
            local.get $state
            local.get 4
            i32.store offset=18
            local.get $state
            local.get 5
            i32.store offset=22
            local.get $state
            local.get 6
            i32.store offset=26
            i32.const 4
            return
        )
        i32.const 0
    )
    (func $enter_1_1_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=14
        local.set 3
        local.get $state
        i32.load offset=18
        local.set 4
        local.get $state
        i32.load offset=22
        local.set 5
        local.get $state
        i32.load offset=26
        local.set 6
        (block
            local.get $utx
            i32.load
            i32.load
            local.tee 5
            local.get 3
            i32.eq
            br_if 0
            local.get 5
            i32.const 9999
            i32.gt_u
            br_if 0
            local.get 0
            i32.const 4
            i32.add
            local.set $memory_address
            local.get $utx
            local.get $memory_address
            i32.const 0
            i32.add
            i32.store
            local.get $utx
            i32.const 1
            i32.store8 offset=35
            local.get $state
            local.get 3
            i32.store offset=14
            local.get $state
            local.get 4
            i32.store offset=18
            local.get $state
            local.get 5
            i32.store offset=22
            local.get $state
            local.get 6
            i32.store offset=26
            i32.const 5
            return
        )
        i32.const 0
    )
    (func $enter_1_1_1_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=14
        local.set 3
        local.get $state
        i32.load offset=18
        local.set 4
        local.get $state
        i32.load offset=22
        local.set 5
        local.get $state
        i32.load offset=26
        local.set 6
        (block
            local.get $utx
            i32.load
            i32.load
            local.tee 6
            i32.eqz
            br_if 0
            local.get $state
            local.get 3
            i32.store offset=14
            local.get $state
            local.get 4
            i32.store offset=18
            local.get $state
            local.get 5
            i32.store offset=22
            local.get $state
            local.get 6
            i32.store offset=26
            (block
                local.get 3
                br_if 0
                local.get 5
                i32.const 3
                i32.shl
                i32.const 1216
                i32.add
                local.tee 3
                local.set $memory_address
                local.get $utx
                local.get $memory_address
                i32.const 0
                i32.add
                i32.store
                local.get $utx
                i32.const 1
                i32.store8 offset=35
                local.get $state
                local.get 3
                i32.store offset=14
                local.get $state
                local.get 4
                i32.store offset=18
                local.get $state
                local.get 5
                i32.store offset=22
                local.get $state
                local.get 6
                i32.store offset=26
                i32.const 6
                return
            )
            local.get 3
            i32.const 3
            i32.shl
            i32.const 1216
            i32.add
            local.tee 3
            local.set $memory_address
            local.get $utx
            local.get $memory_address
            i32.const 0
            i32.add
            i32.store
            local.get $utx
            i32.const 1
            i32.store8 offset=35
            local.get $state
            local.get 3
            i32.store offset=14
            local.get $state
            local.get 4
            i32.store offset=18
            local.get $state
            local.get 5
            i32.store offset=22
            local.get $state
            local.get 6
            i32.store offset=26
            i32.const 7
            return
        )
        i32.const 0
    )
    (func $enter_1_1_1_1_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=14
        local.set 3
        local.get $state
        i32.load offset=18
        local.set 4
        local.get $state
        i32.load offset=22
        local.set 5
        local.get $state
        i32.load offset=26
        local.set 6
        (block
            (block
                local.get $utx
                i32.load
                i32.load
                local.tee 5
                local.get 6
                i32.add
                local.tee 6
                local.get 5
                i32.lt_u
                br_if 1
                local.get 3
                local.get 6
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
                local.get $state
                local.get 3
                i32.store offset=14
                local.get $state
                local.get 4
                i32.store offset=18
                local.get $state
                local.get 5
                i32.store offset=22
                local.get $state
                local.get 6
                i32.store offset=26
                i32.const 8
                return
            )
            local.get 3
            i32.const 3
            i32.shl
            i32.const 1216
            i32.add
            local.tee 3
            local.set $memory_address
            local.get $utx
            local.get $memory_address
            i32.const 0
            i32.add
            i32.store
            local.get $utx
            i32.const 1
            i32.store8 offset=35
            local.get $state
            local.get 3
            i32.store offset=14
            local.get $state
            local.get 4
            i32.store offset=18
            local.get $state
            local.get 5
            i32.store offset=22
            local.get $state
            local.get 6
            i32.store offset=26
            i32.const 7
            return
        )
        i32.const 0
    )
    (func $enter_1_1_1_1_2 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=14
        local.set 3
        local.get $state
        i32.load offset=18
        local.set 4
        local.get $state
        i32.load offset=22
        local.set 5
        local.get $state
        i32.load offset=26
        local.set 6
        (block
            local.get $utx
            i32.load
            i32.load
            local.tee 4
            local.get 6
            i32.lt_u
            br_if 0
            local.get 6
            local.get 5
            i32.const 3
            i32.shl
            i32.const 1216
            i32.add
            local.tee 5
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
            local.get $state
            local.get 3
            i32.store offset=18
            local.get $state
            local.get 4
            i32.store offset=22
            local.get $state
            local.get 5
            i32.store offset=26
            local.get $state
            local.get 6
            i32.store offset=30
            i32.const 9
            return
        )
        i32.const 0
    )
    (func $enter_1_1_1_1_1_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=14
        local.set 3
        local.get $state
        i32.load offset=18
        local.set 4
        local.get $state
        i32.load offset=22
        local.set 5
        local.get $state
        i32.load offset=26
        local.set 6
        (block
            (block
                local.get $utx
                i32.load
                local.get $state
                i32.load offset=6
                i32.store
                i32.const 0
                return
            )
            local.get 3
            i32.const 3
            i32.shl
            i32.const 1216
            i32.add
            local.tee 3
            local.set $memory_address
            local.get $utx
            local.get $memory_address
            i32.const 0
            i32.add
            i32.store
            local.get $utx
            i32.const 1
            i32.store8 offset=35
            local.get $state
            local.get 3
            i32.store offset=14
            local.get $state
            local.get 4
            i32.store offset=18
            local.get $state
            local.get 5
            i32.store offset=22
            local.get $state
            local.get 6
            i32.store offset=26
            i32.const 7
            return
        )
        i32.const 0
    )
    (func $enter_1_1_1_1_2_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=18
        local.set 3
        local.get $state
        i32.load offset=22
        local.set 4
        local.get $state
        i32.load offset=26
        local.set 5
        local.get $state
        i32.load offset=30
        local.set 6
        (block
            local.get $state
            i32.load offset=14
            local.get $utx
            i32.load
            i32.load
            i32.const -1
            i32.xor
            i32.gt_u
            br_if 0
            local.get 3
            local.get 4
            local.get 6
            i32.sub
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
            local.get $state
            local.get 3
            i32.store offset=14
            local.get $state
            local.get 4
            i32.store offset=18
            local.get $state
            local.get 5
            i32.store offset=22
            local.get $state
            local.get 6
            i32.store offset=26
            i32.const 10
            return
        )
        i32.const 0
    )
    (func $enter_1_1_1_1_2_1_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=14
        local.set 3
        local.get $state
        i32.load offset=18
        local.set 4
        local.get $state
        i32.load offset=22
        local.set 5
        local.get $state
        i32.load offset=26
        local.set 6
        (block
            local.get $utx
            i32.load
            local.get $state
            i32.load offset=6
            i32.store
            local.get 5
            local.get 5
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
            local.get $state
            local.get 3
            i32.store offset=18
            local.get $state
            local.get 4
            i32.store offset=22
            local.get $state
            local.get 5
            i32.store offset=26
            local.get $state
            local.get 6
            i32.store offset=30
            i32.const 11
            return
        )
        i32.const 0
    )
    (func $enter_1_1_1_1_2_1_1_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=18
        local.set 3
        local.get $state
        i32.load offset=22
        local.set 4
        local.get $state
        i32.load offset=26
        local.set 5
        local.get $state
        i32.load offset=30
        local.set 6
        (block
            local.get $state
            i32.load offset=14
            local.get $utx
            i32.load
            i32.load
            local.get 6
            i32.add
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
            local.get $state
            local.get 3
            i32.store offset=14
            local.get $state
            local.get 4
            i32.store offset=18
            local.get $state
            local.get 5
            i32.store offset=22
            local.get $state
            local.get 6
            i32.store offset=26
            i32.const 12
            return
        )
        i32.const 0
    )
    (func $enter_1_1_1_1_2_1_1_1_1 (type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)
        (local i32 i32 i32 i32)
        (local $memory_address i32)
        (local $i32_local i32)
        (local $i64_local i64)
        (local $f32_local f32)
        (local $f64_local f64)
        local.get $state
        i32.load offset=14
        local.set 3
        local.get $state
        i32.load offset=18
        local.set 4
        local.get $state
        i32.load offset=22
        local.set 5
        local.get $state
        i32.load offset=26
        local.set 6
        (block
            local.get $utx
            i32.load
            local.get $state
            i32.load offset=6
            i32.store
        )
        i32.const 0
    )
    (type (;0;) (func (param i32 i32 i32) (result i32)))
    (type (;1;) (func))
    (type (;2;) (func (param i32) (result i32)))
    (type (;3;) (func (result i32)))
    (type (;4;) (func (param i32) (result i64)))
    (global $__stack_pointer (mut i32) (i32.const 146752))
    (global (;1;) i32 (i32.const 1024))
    (global (;2;) i32 (i32.const 81216))
    (global (;3;) i32 (i32.const 81216))
    (global (;4;) i32 (i32.const 146752))
    (global (;5;) i32 (i32.const 1024))
    (global (;6;) i32 (i32.const 146752))
    (global (;7;) i32 (i32.const 196608))
    (global (;8;) i32 (i32.const 0))
    (global (;9;) i32 (i32.const 1))
    (export \"memory\" (memory 0))
    (export \"__wasm_call_ctors\" (func $__wasm_call_ctors))
    (export \"enter\" (func $enter))
    (export \"__enter\" (func $__enter))
    (export \"__step\" (func $__step))
    (export \"__get_utx_addrs\" (func $__get_utx_addrs))
    (export \"__get_utx_log2lens\" (func $__get_utx_log2lens))
    (export \"__get_utx_naddr\" (func $__get_utx_naddr))
    (export \"__get_balance\" (func $__get_balance))
    (export \"__get_user\" (func $__get_user))
    (export \"__get_transform_storage\" (func $__get_transform_storage))
    (export \"__indirect_function_table\" (table 0))
    (export \"__dso_handle\" (global 1))
    (export \"__data_end\" (global 2))
    (export \"__stack_low\" (global 3))
    (export \"__stack_high\" (global 4))
    (export \"__global_base\" (global 5))
    (export \"__heap_base\" (global 6))
    (export \"__heap_end\" (global 7))
    (export \"__memory_base\" (global 8))
    (export \"__table_base\" (global 9))
    (table 13 funcref)
    (elem (i32.const 1) func $enter $enter_1 $enter_1_1 $enter_1_1_1 $enter_1_1_1_1 $enter_1_1_1_1_1 $enter_1_1_1_1_2 $enter_1_1_1_1_1_1 $enter_1_1_1_1_2_1 $enter_1_1_1_1_2_1_1 $enter_1_1_1_1_2_1_1_1 $enter_1_1_1_1_2_1_1_1_1)
    (memory 10)
    (type $utx_f (func (param i32 i32 i32) (result i32)))
)"
    );
}
