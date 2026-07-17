#!/bin/sh
set -eu

make check
make test
make build
node scripts/test-web.mjs
node scripts/test-browser.mjs

test -f web/index.html
test -f web/app.mjs
test -f web/faancit_wasm.wasm

wasm_size=$(wc -c < web/faancit_wasm.wasm)
js_size=$(wc -c < web/app.mjs)
wasm_gzip_size=$(gzip -9 -c web/faancit_wasm.wasm | wc -c)
js_gzip_size=$(gzip -9 -c web/app.mjs | wc -c)

test "$wasm_size" -lt 32000
test "$js_size" -lt 4000
test "$wasm_gzip_size" -lt 22000
test "$js_gzip_size" -lt 1500

printf 'WASM deployment asset: %s bytes\n' "$wasm_size"
printf 'WASM gzip transfer:    %s bytes\n' "$wasm_gzip_size"
printf 'JavaScript asset:      %s bytes\n' "$js_size"
printf 'JavaScript gzip:       %s bytes\n' "$js_gzip_size"
