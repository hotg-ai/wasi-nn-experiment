# My Thoughts

The goal for this experiment was to play around with some unfamiliar
technologies to see how we might develop Rune in the future.

## How Rune Works Today

## The Experiment

## Advantages over the Status Quo

Overall, this was an incredibly positive experience!

I think this is one of those times where you only appreciate how nice things
are after spending a lot of energy doing things the hard way. It also
doesn't help that a lot of our problems were self-inflicted and came about
because we didn't properly understand the requirements 😅

### Glue Code

By far the biggest benefit of auto-generated glue code is the ease of
development and iteration. It lets you code against a high-level API and avoid
writing monotonous, error-prone `unsafe` boilerplate code.

The overhead should be negligible as long as host functions aren't being called
in tight loops.

On the guest side:

- The bindings created by [`witx-bindgen`][witx-bindgen] are easy enough to use
  inside WebAssembly
  - It'd be nice if there were safe wrappers and we could use high-level
    objects (probably not possible without destructors and "true" interface
    types)
- Would still need to create our own safe wrappers
- Opens the possibility of defining a proc block using WITX and pre-compiling it
  to WebAssembly, meaning a build is just a case of linking things together
  - Improves type safety
  - Improves compile times because the WebAssembly can be cached
  - Proc blocks can be written in languages other than Rust
- We can move a proc block's dependencies from crates.io into WITX, which lets
  us move from nominal typing to structural typing
  ([explainer][nominal-vs-structural])
- We have a mechanism for "images" and exposing runtime-specific functionality
  (an image is just a collection of WITX files a Rune is designed for)
- Enables rapid iteration and evolution

[nominal-vs-structural]: https://medium.com/@thejameskyle/type-systems-structural-vs-nominal-typing-explained-56511dd969f4

On the host side:

- The trait and abstractions like `GuestPtr<T>` make implementing host functions
  very easy
- Enables rapid iteration and evolution
- Duplication of runtime-specific code is hidden behind procedural macros
- Linking code can be generated for the different WebAssembly runtimes
- Avoids a *lot* of `unsafe` code
- The host and guest can generate code from the same WITX file

In theory, migrating to WITX and generated code should be possible without too
much code churn. The idea is we would define our existing interface in a WITX
file and can then swap the manual boilerplate out for tool-generated versions.

### Different Inference Engine

If I could, I'd drop TensorFlow Lite and switch to `tract` in a heartbeat.

I forgot just how nice it is to use a pure Rust library - there's no stuffing
around with build dependencies, the Rust interface is detailed and idiomatic,
and best of all cross-compiling *Just Works* 🥰

<!-- Obligatory rant about TensorFlow Lite...

- There are no Rust bindings that are officially supported by the project
- We've wasted hundreds of engineer-hours working on our own wrapper because the
  `tflite` crate doesn't support cross-compiling and there are no official
  TensorFlow Lite bindings
- Their CMake build doesn't compile to iOS, so you need to use Bazel
- Bazel isn't something many developers have installed by default, and it isn't
  available in the main Ubuntu repos
- Bazel doesn't generate statically linked libraries, and
  [their comments][bazel-1920-comment] on [the 5 year old ticket][bazel-1920]
  for fully static libraries indicates we probably won't see static builds any
  time soon
- Linking to their shared libraries isn't really acceptable
  - You need to make sure the right version of the shared library is available
  - That means installing `rune` is no longer just a case of copying the binary
    onto your `$PATH`
  - We would need to create installers for each OS and distro
  - Anyone wanting to install `rune` from crates.io or use it as a library will
    also need to have a copy of the `*.so` on their machine

[bazel-1920]: https://github.com/bazelbuild/bazel/issues/1920
[bazel-1920-comment]: https://github.com/bazelbuild/bazel/issues/1920#issuecomment-841225964
-->

It also has support for TensorFlow and ONNX models. I don't know enough to say
how using TensorFlow/ONNX instead of TensorFlow Lite models will impact Rune
sizes and deployment, but they are both popular model formats, with ONNX having
the benefit that it can convert to/from just about anything.

The way Rune handles inference makes it super easy to drop in inference engines
for other model formats. We could use `tract-tensorflow` and `tract-onnx` to
give Rune support for TensorFlow and ONNX today if we wanted to.

## Challenges and Future Work

### Cross-Compilation

This experiment has 3 main dependencies:

- `wasmtime` - run WebAssembly
- `wiggle` - read a WITX file and generate glue that the host can use for
  defining and injecting host functions into wasmtime
- `tract-tensorflow` - inference for TensorFlow models

The `tract-tensorflow` and `wiggle` crates cross-compile to
`aarch64-linux-android` (64-bit ARM Android) with `cross` perfectly fine, but
the `wasmtime` build fails when it tries to compile some assembly.

In theory, if `wasmtime` builds on Android and iOS we'd be able to switch to
that stack.

Alternatively, if `wiggle` gained support for generating WASM3 glue code we
can use WASM3 for executing WebAssembly on iOS and Android. Having glue for the
browser's WebAssembly engine would be nice too!

### Support for Generating Glue Code

- While the guest only needs to generate one version of the glue code, you need
  a different code generator for each WebAssembly runtime.
- At the moment, `wiggle` is specific to `wasmtime`
- `witx-bindgen` supports `wasmtime` and has [experimental support for
  `wasmer`][fork], but no support for WASM3
  - Neither `wasmer` or `wasmtime` work on iOS or Android so you need to use
    WASM3
  - There is no support for web

### Inference Performance

- Haven't investigated too deeply, but it doesn't look like `tract-tensorflow`
  has any knobs enabling inference on the GPU or TPU
- We *really* want hardware acceleration
- The runnable graph's `run()` method passes tensor inputs and outputs by value,
  meaning we can't reuse allocations

### Tweaks to WASI-NN

- Originally wanted to use [the `witx-bindgen` fork][fork] created by the
  `wasmer` folks, but that was a non-starter because `witx-bindgen`
  [didn't support][witx-bindgen#85] the WASI-NN flavor of WITX
- The graph encoding should be an open type (e.g. a string) instead of an
  enum with fixed variants
- The `set_input()` and `get_output()` functions require us to copy tensors back
  and forth between guest and host
  - Might want to investigate directly using WebAssembly memory as inputs and
    outputs (see the Rune API) to avoid one set of copies
  - Should there be an optimisation that lets you use an output from one model
    as the input to another while keeping everything in the host and not
    copying into/out of WebAssembly linear memory?
- Otherwise, I think the API is pretty much bang on 👌

### Asynchronous Code

- You can specify host functions as being `async`
- Could allow the runtime to offload computation to hardware (or distributed
  nodes) and yield until the computation has finished
- WebAssembly is single-threaded, so we probably can't parallelize steps
  *within* the Rune

[tract]: https://github.com/sonos/tract
[witx-bindgen]: https://github.com/bytecodealliance/witx-bindgen
[fork]: https://github.com/wasmerio/witx-bindgen/tree/wasmer-preview-20211001
[witx-bindgen#85]: https://github.com/bytecodealliance/witx-bindgen/issues/85
