/**
 * Canonical Marqdo browser bridge (route C). Host glue only — not app logic.
 * Copied to site dirs by `marqdo wasm build`.
 *
 * ABI: mq_alloc, mq_dealloc, mq_run, mq_boot, mq_call, mq_version
 * Packed results: u32le length + UTF-8 JSON { ok, stdout, error, value }
 *
 * Effects (ADR 0003): set_text, fetch, after
 */

export function readCString(memory, ptr) {
  const bytes = new Uint8Array(memory.buffer);
  let end = ptr;
  while (bytes[end] !== 0) end += 1;
  return new TextDecoder().decode(bytes.subarray(ptr, end));
}

function decodePacked(exports, outPtr) {
  if (!outPtr) throw new Error("null wasm result pointer");
  const view = new DataView(exports.memory.buffer);
  const jsonLen = view.getUint32(outPtr, true);
  const jsonBytes = new Uint8Array(exports.memory.buffer, outPtr + 4, jsonLen);
  const text = new TextDecoder().decode(jsonBytes);
  exports.mq_dealloc(outPtr, 4 + jsonLen);
  return JSON.parse(text);
}

function writeUtf8(exports, text) {
  const encoded = new TextEncoder().encode(text);
  const ptr = exports.mq_alloc(encoded.length || 1);
  if (!ptr) throw new Error("mq_alloc failed");
  new Uint8Array(exports.memory.buffer, ptr, encoded.length).set(encoded);
  return { ptr, len: encoded.length };
}

export function runSource(exports, source) {
  const { ptr, len } = writeUtf8(exports, source);
  const outPtr = exports.mq_run(ptr, len);
  exports.mq_dealloc(ptr, len || 1);
  return decodePacked(exports, outPtr);
}

export function boot(exports, source) {
  const { ptr, len } = writeUtf8(exports, source);
  const outPtr = exports.mq_boot(ptr, len);
  exports.mq_dealloc(ptr, len || 1);
  return decodePacked(exports, outPtr);
}

export function call(exports, name, argsObj = {}) {
  const n = writeUtf8(exports, name);
  const a = writeUtf8(exports, JSON.stringify(argsObj ?? {}));
  const outPtr = exports.mq_call(n.ptr, n.len, a.ptr, a.len);
  exports.mq_dealloc(n.ptr, n.len || 1);
  exports.mq_dealloc(a.ptr, a.len || 1);
  return decodePacked(exports, outPtr);
}

export async function loadWasm(url = "./marqdo_wasm.wasm") {
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(
      `failed to fetch ${url} (${res.status}). Build with: marqdo wasm build -o .`,
    );
  }
  const { instance } = await WebAssembly.instantiateStreaming(res, {});
  return instance.exports;
}

/** Apply C3/C4 return effects: set_text, fetch, after */
export function applyDomPatch(value) {
  if (!value || typeof value !== "object") return;
  const setText = value.set_text || value.setText;
  if (setText && typeof setText === "object") {
    for (const [sel, text] of Object.entries(setText)) {
      const el = document.querySelector(sel);
      if (el) el.textContent = String(text);
    }
  }
}

/**
 * Run sync DOM patches then schedule async effects (ADR 0003).
 */
export function applyEffects(exports, value, { onError } = {}) {
  applyDomPatch(value);
  if (!value || typeof value !== "object") return Promise.resolve();

  const tasks = [];

  const fetchSpec = value.fetch;
  if (fetchSpec && typeof fetchSpec === "object" && fetchSpec.url && fetchSpec.then) {
    tasks.push(runFetchEffect(exports, fetchSpec, onError));
  }

  const afterSpec = value.after;
  if (afterSpec && typeof afterSpec === "object" && afterSpec.then) {
    tasks.push(runAfterEffect(exports, afterSpec, onError));
  }

  return Promise.all(tasks);
}

async function runFetchEffect(exports, spec, onError) {
  const thenFn = spec.then;
  const method = (spec.method || "GET").toUpperCase();
  const init = { method };
  if (spec.headers && typeof spec.headers === "object") {
    init.headers = spec.headers;
  }
  if (spec.body != null && method !== "GET" && method !== "HEAD") {
    init.body = typeof spec.body === "string" ? spec.body : JSON.stringify(spec.body);
  }
  try {
    const res = await fetch(String(spec.url), init);
    const body = await res.text();
    const result = call(exports, thenFn, {
      ok: res.ok,
      status: res.status,
      body,
    });
    if (!result.ok) {
      if (onError) onError(result.error);
      else console.error(result.error);
      return;
    }
    await applyEffects(exports, result.value, { onError });
    if (result.stdout) console.log(result.stdout);
  } catch (e) {
    try {
      const result = call(exports, thenFn, {
        ok: false,
        status: 0,
        body: "",
        error: String(e),
      });
      if (result.ok) await applyEffects(exports, result.value, { onError });
      else if (onError) onError(result.error);
    } catch (e2) {
      if (onError) onError(String(e2));
      else console.error(e2);
    }
  }
}

function runAfterEffect(exports, spec, onError) {
  const ms = Math.max(0, Number(spec.ms) || 0);
  const thenFn = spec.then;
  return new Promise((resolve) => {
    setTimeout(() => {
      try {
        const result = call(exports, thenFn, { ok: true });
        if (!result.ok) {
          if (onError) onError(result.error);
          else console.error(result.error);
          resolve();
          return;
        }
        Promise.resolve(applyEffects(exports, result.value, { onError })).then(resolve);
        if (result.stdout) console.log(result.stdout);
      } catch (e) {
        if (onError) onError(String(e));
        else console.error(e);
        resolve();
      }
    }, ms);
  });
}

/**
 * Wire events from `# main` return value.
 * Accepts a list of row maps, or `{ wire: [...] }`.
 */
export function wireEvents(exports, bootValue, { onError } = {}) {
  let rows = [];
  if (Array.isArray(bootValue)) rows = bootValue;
  else if (bootValue && Array.isArray(bootValue.wire)) rows = bootValue.wire;

  for (const row of rows) {
    const sel = row["选择器"] || row.selector || row.sel;
    const ev = row["事件"] || row.event || "click";
    const fn = row["调用"] || row.call || row.fn;
    if (!sel || !fn) continue;
    const nodes = document.querySelectorAll(sel);
    nodes.forEach((node) => {
      node.addEventListener(ev, () => {
        try {
          const result = call(exports, fn, { event: ev });
          if (!result.ok) {
            if (onError) onError(result.error);
            else console.error(result.error);
            return;
          }
          applyEffects(exports, result.value, { onError });
          if (result.stdout) console.log(result.stdout);
        } catch (e) {
          if (onError) onError(String(e));
          else console.error(e);
        }
      });
    });
  }
  return rows.length;
}
