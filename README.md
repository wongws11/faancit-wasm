# Cantonese Faancit Converter in WebAssembly

https://wongws11.github.io/faancit-wasm/

A port of the previous Cantonese Faancit Converter to WebAssembly, built with [TinyGo](https://tinygo.org/).

## Prerequisites

- [Go 1.26+](https://golang.org/dl/)
- [TinyGo 0.41+](https://tinygo.org/getting-started/install/)

## Build

```sh
make build
```

This compiles the WASM binary to `web/main.wasm`. The TinyGo `wasm_exec.js` is already committed in `web/`.

## Test

```sh
make test
```

## Clean

```sh
make clean
```

## CI/CD

GitHub Actions builds with TinyGo and deploys to GitHub Pages. See [go-wasm.yml](.github/workflows/go-wasm.yml).

## Binary Size

Using TinyGo instead of standard Go reduces the WASM binary from ~3.7MB to ~868KB (4.3x smaller).
