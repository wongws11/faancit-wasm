WASM_TARGET := wasm32-unknown-unknown
WASM_BINDGEN_VERSION := 0.2.126
WASM_INPUT := target/$(WASM_TARGET)/release/faancit_wasm.wasm
WASM_OUT_DIR := web/pkg

.PHONY: build check check-wasm-bindgen clean test

build: check-wasm-bindgen
	cargo build --release --target $(WASM_TARGET)
	wasm-bindgen --target web --no-typescript --out-dir $(WASM_OUT_DIR) \
		--out-name faancit_wasm $(WASM_INPUT)

check:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo check --target $(WASM_TARGET)

check-wasm-bindgen:
	@command -v wasm-bindgen >/dev/null || { \
		echo "wasm-bindgen-cli $(WASM_BINDGEN_VERSION) is required" >&2; \
		echo "Install it with: cargo install --locked wasm-bindgen-cli --version $(WASM_BINDGEN_VERSION)" >&2; \
		exit 1; \
	}
	@test "$$(wasm-bindgen --version)" = "wasm-bindgen $(WASM_BINDGEN_VERSION)" || { \
		echo "wasm-bindgen-cli $(WASM_BINDGEN_VERSION) is required; found $$(wasm-bindgen --version)" >&2; \
		exit 1; \
	}

test:
	cargo test --all-targets

clean:
	cargo clean
	rm -rf $(WASM_OUT_DIR) web/main.wasm
