# WASI-NN Experiment

[![Continuous integration](https://github.com/hotg-ai/wasi-nn-experiment/workflows/Continuous%20integration/badge.svg?branch=master)](https://github.com/hotg-ai/wasi-nn-experiment/actions)

([API Docs])

Experiments with [`wasmtime`][wasmtime], [the `wasi-nn` proposal][wasi-nn],
and [`tract`][tract].

## Getting Started

To use this experiment, you will first need to compile the guest code to
WebAssembly.

```shell
$ cargo build --target wasm32-unknown-unknown --package guest
```

We can then use the host application to load the WebAssembly module and
run the model it contains.

```shell
$ cargo  --package host -- target/wasm32-unknown-unknown/debug/guest.wasm
```

During development, I use `cargo watch` and this one-liner to automatically
recompile and re-run the guest after every change:

```shell
$ cargo watch --clear \
    -x "check --workspace" \
    -x "test --workspace" \
    -x "doc --workspace --document-private-items" \
    -x "build --package guest --target wasm32-unknown-unknown" \
    -x "run --package host -- target/wasm32-unknown-unknown/debug/guest.wasm"
```

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE.md) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT.md) or
   http://opensource.org/licenses/MIT)

at your option.

It is recommended to always use [cargo-crev][crev] to verify the
trustworthiness of each of your dependencies, including this one.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

The intent of this crate is to be free of soundness bugs. The developers will
do their best to avoid them, and welcome help in analysing and fixing them.

[API Docs]: https://hotg-ai.github.io/wasi-nn-experiment
[crev]: https://github.com/crev-dev/cargo-crev
[wasmtime]: https://github.com/bytecodealliance/wasmtime
[wasi-nn]: https://github.com/WebAssembly/wasi-nn
[tract]: https://github.com/sonos/tract
