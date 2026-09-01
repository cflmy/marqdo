import {
  applyDomPatch,
  boot,
  loadWasm,
  readCString,
  wireEvents,
} from "./marqdo-bridge.js";

async function main() {
  const log = document.getElementById("log");
  const bump = document.getElementById("bump");
  const reset = document.getElementById("reset");

  let exports;
  try {
    exports = await loadWasm("./marqdo_wasm.wasm");
  } catch (e) {
    log.classList.add("err");
    log.textContent = String(e);
    return;
  }

  const srcRes = await fetch("./counter.mq.md");
  if (!srcRes.ok) {
    log.classList.add("err");
    log.textContent = `failed to load counter.mq.md (${srcRes.status})`;
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
  // Ensure initial display matches Marqdo state.
  applyDomPatch({ set_text: { "#count": "0" } });

  const ver = readCString(exports.memory, exports.mq_version());
  log.classList.remove("err");
  log.textContent = `marqdo-wasm ${ver} · wired ${n} handler(s)\nsource: counter.mq.md`;
  bump.disabled = false;
  reset.disabled = false;
}

main();
