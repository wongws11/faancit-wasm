# Cantonese Faancit Converter in WebAssembly

https://wongws11.github.io/faancit-wasm/

A browser-based Cantonese faancit converter written in Rust and compiled to WebAssembly. The application embeds its Jyutping dictionary in the WASM binary and runs entirely in the browser.

## Prerequisites

- [Rust 1.97.1](https://www.rust-lang.org/tools/install) through `rustup`
- `wasm32-unknown-unknown`, installed automatically from `rust-toolchain.toml`
- [`wasm-bindgen-cli` 0.2.126](https://rustwasm.github.io/wasm-bindgen/)

Install the matching WASM binding generator:

```sh
cargo install --locked wasm-bindgen-cli --version 0.2.126
```

The CLI version must match the pinned `wasm-bindgen` crate version.

## Development

Run formatting, lints, and a WASM compile check:

```sh
make check
```

Run the native conversion tests:

```sh
make test
```

Build the optimized browser assets:

```sh
make build
```

The generated ES module and WASM binary are written to `web/pkg/` and are intentionally ignored by Git.

Remove Rust and generated web build artifacts:

```sh
make clean
```

## Architecture

- `build.rs` converts the JSON dictionary into compact, sorted static records at compile time.
- `src/jyutping.rs` contains binary-search lookup, homophone matching, and the faancit rules.
- `src/web.rs` contains the WASM browser entry point and DOM event handling.
- `jyutping/jyutping.json` is the source dictionary and is not shipped or parsed at runtime.
- `web/index.html` is the static GitHub Pages application shell.

Homophone results are sorted by Unicode code point to keep browser output deterministic. The converter otherwise preserves the previous rule order, diagnostics, two-character input behavior, and output format.

Each generated dictionary entry occupies 8 bytes and references shared initial and final tables. This avoids runtime JSON parsing, duplicated pronunciation strings, and persistent forward and reverse hash maps. Homophones are found by scanning the 6,059 static entries when input changes, trading negligible work for lower persistent browser memory.

## Deployment

`scripts/build-pages.sh` performs the same checks and release build used for deployment. The [Rust WASM workflow](.github/workflows/rust-wasm.yml) runs that script on pushes to `main`, uploads `web/`, and deploys it to GitHub Pages.

The deployment script also enforces a 1 MB release WASM size budget.
