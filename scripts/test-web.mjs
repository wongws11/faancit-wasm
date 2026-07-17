import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import init, { formatPronunciation, mount } from "../web/app.mjs";

const ABSENT = 0xffffffff;
const wasmBytes = await readFile(new URL("../web/faancit_wasm.wasm", import.meta.url));
const module = await WebAssembly.compile(wasmBytes);
assert.deepEqual(WebAssembly.Module.imports(module), []);

const { exports: wasm } = await WebAssembly.instantiate(module);
const expectedExports = [
    "c",
    "h",
    "l",
    "memory",
];
assert.deepEqual(WebAssembly.Module.exports(module).map(({ name }) => name).sort(), expectedExports);
assert.equal(wasm.memory.buffer.byteLength, 65536);

function asU32(value) {
    return value >>> 0;
}

const dictionary = JSON.parse(
    await readFile(new URL("../jyutping/jyutping.json", import.meta.url), "utf8"),
);
for (const [character, expected] of Object.entries(dictionary)) {
    const packed = asU32(wasm.l(character.codePointAt(0)));
    assert.notEqual(packed, ABSENT, character);
    assert.equal(
        formatPronunciation(packed),
        `${expected.sing} ${expected.wan} ${expected.tone}`,
        character,
    );
}
assert.equal(asU32(wasm.l(0x10ffff)), ABSENT);
assert.equal(asU32(wasm.c(ABSENT, 0)), ABSENT);

class Element {
    value = "";
    innerText = "";
    listeners = new Map();

    addEventListener(name, listener) {
        this.listeners.set(name, listener);
    }

    dispatch(name) {
        this.listeners.get(name)();
    }
}

const elements = {
    input: new Element(),
    output: new Element(),
    pronunciations: new Element(),
};
const logs = [];
let now = 0;
mount(wasm, {
    document: { getElementById: (id) => elements[id] },
    performance: { now: () => (now += 0.021) },
    console: { log: (message) => logs.push(message) },
});

elements.input.value = "德紅";
elements.input.dispatch("input");
assert.equal(elements.output.innerText, "d ung 1");
assert.match(elements.pronunciations.innerText, /冬/);
assert.match(elements.pronunciations.innerText, /東/);
assert.match(logs[0], /^Faancit conversion: \d+\.\d{3} μs$/);
assert.equal(logs[1], "上字陰下字陽\n");

for (const invalid of ["", "德", "德紅字", "𠮷德", "德x"]) {
    elements.input.value = invalid;
    elements.input.dispatch("input");
    assert.equal(elements.output.innerText, "", invalid);
    assert.equal(elements.pronunciations.innerText, "", invalid);
}

for (let iteration = 0; iteration < 10_000; iteration += 1) {
    wasm.l(0x5fb7);
}
assert.equal(wasm.memory.buffer.byteLength, 65536);

const originalFetch = globalThis.fetch;
const originalDocument = globalThis.document;
const originalPerformance = globalThis.performance;
const originalConsole = globalThis.console;
globalThis.fetch = async () => new Response(wasmBytes, {
    headers: { "Content-Type": "application/wasm" },
});
globalThis.document = { getElementById: (id) => elements[id] };
globalThis.performance = { now: () => (now += 0.021) };
globalThis.console = { log: (message) => logs.push(message) };
await init(new URL("https://example.test/faancit_wasm.wasm"));
globalThis.fetch = async () => new Response(wasmBytes, {
    headers: { "Content-Type": "application/octet-stream" },
});
await init(new URL("https://example.test/faancit_wasm.wasm"));
globalThis.fetch = originalFetch;
globalThis.document = originalDocument;
globalThis.performance = originalPerformance;
globalThis.console = originalConsole;

console.log(`Validated ${Object.keys(dictionary).length} dictionary entries and browser behavior`);
