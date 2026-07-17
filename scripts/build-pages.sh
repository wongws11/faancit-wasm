#!/bin/sh
set -eu

make clean
make check
make test
make build

test -f web/index.html
test -f web/pkg/faancit_wasm.js
test -f web/pkg/faancit_wasm_bg.wasm

wasm_size=$(wc -c < web/pkg/faancit_wasm_bg.wasm)
test "$wasm_size" -lt 1000000
printf 'WASM deployment asset: %s bytes\n' "$wasm_size"
