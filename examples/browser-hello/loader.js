/**
 * Thin loader for marqdo_wasm.wasm (C1 raw ABI).
 * Not application logic — only host bridge.
 */

const DEFAULT_SRC = `# main

> print text=Hello from browser WASM!
`;

function readCString(memory, ptr) {
  const bytes = new Uint8Array(memory.buffer);
  let end = ptr;
  while (bytes[end] !== 0) end += 1;
  return new TextDecoder().decode(bytes.subarray(ptr, end));
}

function runSource(exports, source) {
  const encoded = new TextEncoder().encode(source);
  const inPtr = exports.mq_alloc(encoded.length || 1);
  if (!inPtr) throw new Error("mq_alloc failed");
  new Uint8Array(exports.memory.buffer, inPtr, encoded.length).set(encoded);
  const outPtr = exports.mq_run(inPtr, encoded.length);
  exports.mq_dealloc(inPtr, encoded.length || 1);
  if (!outPtr) throw new Error("mq_run returned null");
  const view = new DataView(exports.memory.buffer);
  const jsonLen = view.getUint32(outPtr, true);
  const jsonBytes = new Uint8Array(exports.memory.buffer, outPtr + 4, jsonLen);
  const text = new TextDecoder().decode(jsonBytes);
  exports.mq_dealloc(outPtr, 4 + jsonLen);
  return JSON.parse(text);
}

async function main() {
  const out = document.getElementById("out");
  const btn = document.getElementById("run");
  const src = document.getElementById("src");
  if (!src.value.trim()) src.value = DEFAULT_SRC;

  const wasmPath = "./marqdo_wasm.wasm";
  const res = await fetch(wasmPath);
  if (!res.ok) {
    out.classList.add("err");
    out.textContent =
      `failed to fetch ${wasmPath} (${res.status}).\n` +
      `Build: cargo build -p marqdo-wasm --target wasm32-unknown-unknown --release\n` +
      `Copy: cp target/wasm32-unknown-unknown/release/marqdo_wasm.wasm examples/browser-hello/`;
    return;
  }
  const { instance } = await WebAssembly.instantiateStreaming(res, {});
  const exports = instance.exports;
  const ver = readCString(exports.memory, exports.mq_version());
  out.classList.remove("err");
  out.textContent = `marqdo-wasm ${ver} ready.\n`;
  btn.disabled = false;

  btn.addEventListener("click", () => {
    try {
      const result = runSource(exports, src.value);
      out.classList.toggle("err", !result.ok);
      out.textContent = result.ok
        ? (result.stdout || "(empty stdout)") +
          (result.value && result.value !== "None" ? `\n[return] ${result.value}` : "")
        : result.error || "error";
    } catch (e) {
      out.classList.add("err");
      out.textContent = String(e);
    }
  });
}

main();
