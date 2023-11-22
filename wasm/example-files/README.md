### Notes on writing `.wat` (WebAssembly text-format)

#### To validate some semantics without a runtime I have found two tools to be of use.

1. From the [wasm-tools crate](https://github.com/bytecodealliance/wasm-tools) one can use `parse` + `validate` like so:
    ```sh
    $ wasm-tools parse <wat-file> | wasm-tools validate
   ```
2. From [*WABT* (WebAssembly binary toolkit)](https://github.com/WebAssembly/wabt) one can use `wat2wasm` directly. In
   my opinion the error messages are clearer than for the first option. The downside is it produces a `.wasm` file, but
   this can be stopped with `--output=-`
    ```sh
   $ wat2wasm <wat-file> --output=- 1>/dev/null
   ```
