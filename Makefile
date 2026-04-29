WASM_OUT := web/main.wasm

.PHONY: build clean test update-wasm-exec

build:
	tinygo build -o $(WASM_OUT) -target wasm -no-debug ./

update-wasm-exec:
	cp "$$(tinygo env TINYGOROOT)/targets/wasm_exec.js" web/wasm_exec.js

test:
	go test ./...

clean:
	rm -f $(WASM_OUT)
