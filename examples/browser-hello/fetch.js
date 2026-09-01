import {
  applyDomPatch,
  boot,
  loadWasm,
  readCString,
  wireEvents,
} from "./marqdo-bridge.js";

async function main() {
  const log = document.getElementById("log");
  const loadBtn = document.getElementById("load");
  const pingBtn = document.getElementById("ping");

  let exports;
  try {
    exports = await loadWasm("./marqdo_wasm.wasm");
  } catch (e) {
    log.classList.add("err");
    log.textContent = String(e);
    return;
  }

  const srcRes = await fetch("./fetch.mq.md");
  if (!srcRes.ok) {
    log.classList.add("err");
    log.textContent = `failed to load fetch.mq.md (${srcRes.status})`;
    return;
  }
  const source = await srcRes.text();
  const result = boot(exports, source);
  if (!result.ok) {
    log.classList.add("err");
    log.textContent = result.error || "boot failed";
    return;
  }

  const n = wireEvents(exports, result.value, {
    onError: (err) => {
      log.classList.add("err");
      log.textContent = err;
    },
  });
  applyDomPatch({ set_text: { "#status": "idle" } });

  const ver = readCString(exports.memory, exports.mq_version());
  log.classList.remove("err");
  log.textContent = `marqdo-wasm ${ver} · wired ${n} · ADR 0003 effects\nsource: fetch.mq.md`;
  loadBtn.disabled = false;
  pingBtn.disabled = false;
}

main();
