# Cantonese Faancit Converter in WebAssembly

https://wongws11.github.io/faancit-wasm/

A deliberately tiny Cantonese faancit converter with a Rust core compiled to WebAssembly and a handwritten browser adapter. It runs entirely in the browser.

## Prerequisites

- [Rust 1.97.1](https://www.rust-lang.org/tools/install) through `rustup`
- `wasm32-unknown-unknown`, installed automatically from `rust-toolchain.toml`
- [Binaryen 131](https://github.com/WebAssembly/binaryen/releases/tag/version_131)
- Node.js 22 or newer for exhaustive WASM and browser-adapter tests

`wasm-opt --version` must report `wasm-opt version 131 (version_131)`.

## Development

Run formatting, lints, JavaScript syntax checks, and a WASM compile check:

```sh
make check
```

Run native Rust tests:

```sh
make test
```

Build the optimized `web/faancit_wasm.wasm` deployment asset:

```sh
make build
```

Run exhaustive generated-WASM and browser-adapter tests:

```sh
make test-web
```

Remove generated artifacts:

```sh
make clean
```

## Architecture

- `build.rs` validates and compiles the JSON dictionary into static bitmap, rank, and pronunciation tables.
- `src/jyutping.rs` implements the packed pronunciation type and faancit rules.
- `src/abi.rs` exposes three allocation-free numeric functions to JavaScript.
- `web/app.mjs` owns DOM access, timing, diagnostics, and display formatting.
- `jyutping/jyutping.json` is build-time input and is not deployed or parsed at runtime.

Each pronunciation is a `u16`: 5 initial bits, 6 final bits, and 3 tone bits. Initial and final indexes are frequency-ordered to improve transfer compression. A Unicode membership bitmap and sparse rank checkpoints map code points to the 6,059 packed pronunciations without storing character keys. The public Rust API exposes a valid-by-construction `JyutpingChar` and type-safe `Tone` enum.

The WASM boundary only accepts and returns numbers. This removes `wasm-bindgen`, `web-sys`, runtime JSON parsing, persistent hash maps, Rust-side DOM objects, and browser-path heap allocation. The module has no imports and reserves a 32 KiB stack, fitting its initial linear memory in one 64 KiB page.

Homophones are scanned in Unicode order, preserving deterministic output. Diagnostic flags cross the numeric ABI and are rendered by JavaScript in the original rule order.

Current optimized assets are approximately 30 KiB raw / 20 KiB gzip for WASM and 4 KiB raw / 1.5 KiB gzip for the browser adapter.

## Deployment

`scripts/build-pages.sh` runs Rust checks, native tests, the Binaryen-optimized release build, all 6,059 dictionary checks through the final WASM ABI, browser-adapter behavior tests, memory assertions, and raw/gzip size budgets.

The [Rust WASM workflow](.github/workflows/rust-wasm.yml) installs checksum-verified Binaryen 131, uploads `web/`, and deploys it to GitHub Pages on pushes to `main`.
