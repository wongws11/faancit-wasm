WASM_TARGET := wasm32-unknown-unknown
BINARYEN_VERSION := 131
WASM_OPT ?= wasm-opt
WASM_INPUT := target/$(WASM_TARGET)/release/faancit_wasm.wasm
WASM_OUT := web/faancit_wasm.wasm

.PHONY: build check check-wasm-opt clean test test-web

build: check-wasm-opt
	cargo build --release --target $(WASM_TARGET)
	$(WASM_OPT) -Oz --enable-bulk-memory --strip-debug --strip-producers $(WASM_INPUT) -o $(WASM_OUT)

check:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo check --target $(WASM_TARGET)
	node --check web/app.mjs
	node --check scripts/test-browser.mjs
	node --check scripts/test-web.mjs

check-wasm-opt:
	@command -v $(WASM_OPT) >/dev/null || { \
		echo "Binaryen $(BINARYEN_VERSION) is required" >&2; \
		exit 1; \
	}
	@test "$$($(WASM_OPT) --version)" = "wasm-opt version $(BINARYEN_VERSION) (version_$(BINARYEN_VERSION))" || { \
		echo "Binaryen $(BINARYEN_VERSION) is required; found $$($(WASM_OPT) --version)" >&2; \
		exit 1; \
	}

test:
	cargo test --all-targets

test-web: build
	node scripts/test-web.mjs
	node scripts/test-browser.mjs

clean:
	cargo clean
	rm -rf web/pkg web/main.wasm $(WASM_OUT)
