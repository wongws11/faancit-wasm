import assert from "node:assert/strict";
import { constants, access, readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { spawn } from "node:child_process";

const candidates = [
    process.env.CHROME_BIN,
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/usr/bin/google-chrome",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
].filter(Boolean);
let chrome;
for (const candidate of candidates) {
    try {
        await access(candidate, constants.X_OK);
        chrome = candidate;
        break;
    } catch {}
}
assert.ok(chrome, "Chrome or Chromium is required for the browser smoke test");

const fixture = `<!doctype html><html><body>
<input id="input"><p id="output"></p><p id="pronunciations"></p>
<script type="module">
import init from "/app.mjs";
const wasm = await init(new URL("/faancit_wasm.wasm", location.href));
input.value = "德紅";
input.dispatchEvent(new Event("input"));
document.body.dataset.result = output.innerText;
document.body.dataset.homophones = pronunciations.innerText;
document.body.dataset.memory = wasm.memory.buffer.byteLength;
</script></body></html>`;
const assets = {
    "/": ["text/html; charset=utf-8", Buffer.from(fixture)],
    "/app.mjs": ["text/javascript; charset=utf-8", await readFile("web/app.mjs")],
    "/faancit_wasm.wasm": ["application/wasm", await readFile("web/faancit_wasm.wasm")],
};
const server = createServer((request, response) => {
    const asset = assets[request.url];
    if (!asset) {
        response.writeHead(404).end();
        return;
    }
    response.writeHead(200, { "Content-Type": asset[0] });
    response.end(asset[1]);
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const { port } = server.address();

try {
    const html = await new Promise((resolve, reject) => {
        const browser = spawn(chrome, [
            "--headless=new",
            "--disable-gpu",
            "--virtual-time-budget=3000",
            "--dump-dom",
            `http://127.0.0.1:${port}/`,
        ]);
        let stdout = "";
        let stderr = "";
        const timeout = setTimeout(() => {
            browser.kill("SIGKILL");
            reject(new Error(`Chrome smoke test timed out: ${stderr}`));
        }, 15_000);
        browser.stdout.on("data", (chunk) => (stdout += chunk));
        browser.stderr.on("data", (chunk) => (stderr += chunk));
        browser.on("error", reject);
        browser.on("close", (code) => {
            clearTimeout(timeout);
            if (code === 0) resolve(stdout);
            else reject(new Error(`Chrome exited ${code}: ${stderr}`));
        });
    });
    assert.match(html, /data-result="d ung 1"/);
    assert.match(html, /data-homophones="[^"]*冬/);
    assert.match(html, /data-homophones="[^"]*東/);
    assert.match(html, /data-memory="65536"/);
    console.log("Validated real browser initialization and DOM conversion");
} finally {
    server.closeAllConnections();
    await new Promise((resolve) => server.close(resolve));
}
