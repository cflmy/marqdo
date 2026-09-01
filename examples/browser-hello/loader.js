import {
  loadWasm,
  readCString,
  runSource,
} from "./marqdo-bridge.js";

const DEFAULT_SRC = `# main

> print text=Hello from browser WASM!
`;

async function main() {
  const out = document.getElementById("out");
  const btn = document.getElementById("run");
  const src = document.getElementById("src");
  if (!src.value.trim()) src.value = DEFAULT_SRC;

  let exports;
  try {
    exports = await loadWasm("./marqdo_wasm.wasm");
  } catch (e) {
    out.classList.add("err");
    out.textContent = String(e);
    return;
  }
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
          (result.value != null && result.value !== "None"
            ? `\n[return] ${typeof result.value === "string" ? result.value : JSON.stringify(result.value)}`
            : "")
        : result.error || "error";
    } catch (e) {
      out.classList.add("err");
      out.textContent = String(e);
    }
  });
}

main();
