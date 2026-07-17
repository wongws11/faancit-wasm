const ABSENT = 0xffffffff;
const INITIALS = ["z", "j", "s", "c", "g", "l", "h", "m", "d", "b", "t", "w", "f", "k", "p", "n", "ng", "", "gw", "kw"];
const FINALS = ["au", "ou", "ing", "ai", "an", "ei", "i", "ung", "ong", "in", "iu", "eoi", "aa", "aan", "yun", "oeng", "o", "uk", "ik", "yu", "aai", "u", "am", "ok", "at", "im", "aau", "eon", "oi", "it", "ang", "aam", "ui", "un", "aap", "e", "aak", "aat", "ip", "yut", "oek", "ap", "on", "ak", "aang", "ut", "eot", "", "ek", "eng", "ot", "oe", "ng"];

const DIAGNOSTICS = [
    [1 << 0, "上字陽聲塞音\n"],
    [1 << 1, "下字平聲則聲母送氣\n"],
    [1 << 2, "下字仄聲則聲母不送氣\n"],
    [1 << 3, "古無輕唇音\n"],
    [1 << 4, "上字陰下字陽\n"],
    [1 << 5, "上字陽下字陰\n"],
    [1 << 6, "上字陽聲而上字塞音或下字擦音\n"],
];

function asU32(value) {
    return value >>> 0;
}

function clear(output, pronunciations) {
    output.innerText = "";
    pronunciations.innerText = "";
}

export function formatPronunciation(packed) {
    return `${INITIALS[packed & 0x1f]} ${FINALS[(packed >>> 5) & 0x3f]} ${(packed >>> 11) + 1}`;
}

export function mount(wasm, environment = globalThis) {
    const { document, performance, console } = environment;
    const input = document.getElementById("input");
    const output = document.getElementById("output");
    const pronunciations = document.getElementById("pronunciations");
    if (!input || !output || !pronunciations) {
        throw new Error("missing required converter elements");
    }

    const render = () => {
        const characters = input.value[Symbol.iterator]();
        const upperCharacter = characters.next();
        const lowerCharacter = characters.next();
        if (upperCharacter.done || lowerCharacter.done || !characters.next().done) {
            clear(output, pronunciations);
            return;
        }

        const startedAt = performance.now();
        const upper = asU32(wasm.l(upperCharacter.value.codePointAt(0)));
        const lower = asU32(wasm.l(lowerCharacter.value.codePointAt(0)));
        if (upper === ABSENT || lower === ABSENT) {
            clear(output, pronunciations);
            return;
        }

        const combined = asU32(wasm.c(upper, lower));
        if (combined === ABSENT) {
            clear(output, pronunciations);
            return;
        }
        const result = combined & 0xffff;
        const flags = combined >>> 16;
        output.innerText = formatPronunciation(result);

        let homophones = "";
        let codepoint = asU32(wasm.h(result, 0));
        while (codepoint !== ABSENT) {
            homophones += `${String.fromCodePoint(codepoint)} `;
            codepoint = asU32(wasm.h(result, codepoint));
        }
        pronunciations.innerText = homophones;

        const elapsed = performance.now() - startedAt;
        console.log(
            elapsed < 1
                ? `Faancit conversion: ${(elapsed * 1000).toFixed(3)} μs`
                : `Faancit conversion: ${elapsed.toFixed(3)} ms`,
        );

        let diagnostics = "";
        for (const [flag, message] of DIAGNOSTICS) {
            if ((flags & flag) !== 0) diagnostics += message;
        }
        if (diagnostics !== "") console.log(diagnostics);
    };

    input.addEventListener("input", render);
    return render;
}

export default async function init(url = new URL("./faancit_wasm.wasm", import.meta.url)) {
    const response = await fetch(url);
    let instance;
    if (WebAssembly.instantiateStreaming) {
        try {
            ({ instance } = await WebAssembly.instantiateStreaming(response.clone()));
        } catch (error) {
            if (response.headers.get("Content-Type") === "application/wasm") throw error;
        }
    }
    if (!instance) {
        ({ instance } = await WebAssembly.instantiate(await response.arrayBuffer()));
    }
    mount(instance.exports);
    return instance.exports;
}
