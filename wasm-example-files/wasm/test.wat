(module
  (func $load (param $tx i32) (param $utx i32) (param $state i32) (result i32)
    (local $memory_address i32)
    i32.const 100
    local.set $memory_address
    local.get $utx
    local.get $memory_address
    i32.store
    i32.const 0
  )
  (func $load_1 (param $tx i32) (param $utx i32) (param $state i32) (result i32)
    local.get $utx
    i32.load
    i32.load
  )
  (func $store (param $tx i32) (param $utx i32) (param $state i32) (result i32)
    (local $memory_address i32)
    (local $value_to_store f64)
    i32.const 200
    f64.const 20
    local.set $value_to_store
    local.set $memory_address
    local.get $state
    local.get $value_to_store
    f64.store
    local.get $utx
    local.get $memory_address
    i32.store
    i32.const 1
  )
  (func $store_1 (param $tx i32) (param $utx i32) (param $state i32) (result i32)
    local.get $utx
    i32.load
    local.get $state
    f64.load
    f64.store
    i32.const 0
  )
  (func $load_and_store (param $tx i32) (param $utx i32) (param $state i32) (result i32)
    (local $memory_address i32)
    i32.const 100
    local.set $memory_address
    local.get $utx
    local.get $memory_address
    i32.store
    i32.const 2
  )
  (func $load_and_store_1 (param $tx i32) (param $utx i32) (param $state i32) (result i32)
    (local $memory_address i32)
    (local $value_to_store i64)
    local.get $utx
    i32.load
    i32.load
    i64.const 5
    i64.const 10
    i64.add
    local.set $value_to_store
    local.set $memory_address
    local.get $state
    local.get $value_to_store
    i64.store
    local.get $utx
    local.get $memory_address
    i32.store
    i32.const 3
  )
  (func $load_and_store_2 (param $tx i32) (param $utx i32) (param $state i32) (result i32)
    local.get $utx
    i32.load
    local.get $state
    i64.load
    i64.store
    i32.const 0
  )
  (memory 10)
  (table 4 funcref)
  (elem (i32.const 0) $load_1 $store_1 $load_and_store_1 $load_and_store_2)
)
