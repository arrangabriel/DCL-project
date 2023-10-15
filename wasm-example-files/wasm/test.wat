(module
  (func $load (param $tx i32) (param $utx i32) (param $state i32) (result i32)
    (local $address i32)
    i32.const 100
    local.set $address
    local.get $utx
    local.get $address
    i32.store
    i32.const 0
  )
  (func $load_1 (param $tx i32) (param $utx i32) (param $state i32) (result i32)
    local.get $utx
    i32.load
    i32.load
    drop
    i32.const 0
  )
  (func $store (param $tx i32) (param $utx i32) (param $state i32) (result i32)
    (local $address i32)
    (local $value f64)
    i32.const 200
    f64.const 20
    local.set $value
    local.set $address
    local.get $state
    local.get $value
    f64.store
    local.get $utx
    local.get $address
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
  (memory 10)
)
