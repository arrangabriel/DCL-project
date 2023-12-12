# chop-up

The function of this tool is to split up code representing `transactions` into so called `microtransactions`.
A `transaction` on a high level is simply a procedure that operates on a global state, reading and/or writing to it.
In a system multiple transactions may be executing concurrently, and must therefore be scheduled so as appear serial in their execution.
To aid in scheduling it would be of great help to know exactly which addresses a transaction will operate on.
In some cases the addresses can be calculated a priori, but this is not possible if the address of one access is derived from the result of another. 
I.e. reading a value, interpreting it as an address and subsequently reading or writing to said address.

To solve this a `microtransactional` runtime has been proposed wherein a larger transaction in a sense is suspended at each memory access, 
yielding the address it intends to access to the runtime. 
With this information the runtime can make informed scheduling decisions when interleaving the set of microtransactions.
Performing this yield is not a trivial task, as it requires transforming the code itself to conform to the syntax and semantics of such a system, 
while preserving the semantics of the original transaction.
This tool performs the transformation on transactions in the [WebAssembly text-format](https://developer.mozilla.org/en-US/docs/WebAssembly/Understanding_the_text_format).

# Usage

Build using cargo
```shell
$ cargo build
```

Alternatively build and run with single command
```shell
$ cargo run [subcommand] [opts...]
```

Run tests
```shell
$ cargo test
```

## Transformation

Run on `.wat` file
```shell
$ chop_up split [input] [state size] [opts...] > [output]
```

Optional flags:
 - `--skip-safe` - attempt to make optimized split decisions
 - `--explain` - add explanatory comments to output

## Analysis

Run on `.wat` file
```shell
$ chop_up analyze [input] [output format]
```

> output format is one of: `standard` or `csv`

# Build and run examples

To build the examples a [wasi-enabled](https://github.com/WebAssembly/wasi-sdk) compiler is needed.

```shell
$ cd wasm/runtime
```

Run without transformation
```shell
$ make check-{payment|auction}
```

Run with transformation
```shell
$ make check-{payment|auction}-split
```
