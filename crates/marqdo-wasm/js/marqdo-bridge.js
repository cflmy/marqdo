/**
 * Canonical Marqdo browser bridge (route C/D). Host glue only — not app logic.
 * Copied to site dirs by `marqdo wasm build`.
 *
 * ABI: mq_alloc, mq_dealloc, mq_run, mq_boot, mq_call, mq_version
 * Packed results: u32le length + UTF-8 JSON { ok, stdout, error, value }
 *
 * Effects (ADR 0003 + route D): set_text, set_value, set_attr, set_class,
 * toggle_class, set_html (#trusted* only), fetch, after
 * Mount: mount() / data-mq-* / #marqdo-boot auto-start (author zero JS)
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

function isPlainObject(v) {
  return v != null && typeof v === "object" && !Array.isArray(v);
}

/** Normalize mount options from plain object or DOM dataset. */
export function normalizeMountOptions(raw = {}) {
  const o = raw && typeof raw === "object" ? raw : {};
  const dataset = o.dataset || {};
  const wasmUrl =
    o.wasmUrl ||
    o.wasm ||
    dataset.mqWasm ||
    dataset.wasm ||
    "./marqdo_wasm.wasm";
  const sourceUrl =
    o.sourceUrl ||
    o.source_url ||
    dataset.mqSourceUrl ||
    dataset.sourceUrl ||
    "";
  let source = o.source != null ? o.source : dataset.mqSource || "";
  if (!source && typeof o.sourceInline === "string") source = o.sourceInline;
  const playground =
    o.playground ||
    dataset.mqPlayground ||
    dataset.playground ||
    "";
  const ready =
    o.ready ||
    o.readySel ||
    dataset.mqReady ||
    "";
  const enable =
    o.enable ||
    dataset.mqEnable ||
    "";
  const noBoot =
    o.noBoot === true ||
    dataset.mqNoBoot === "1" ||
    dataset.mqNoBoot === "true";
  return {
    wasmUrl: String(wasmUrl),
    sourceUrl: sourceUrl ? String(sourceUrl) : "",
    source: source ? String(source) : "",
    playground: playground ? String(playground) : "",
    ready: ready ? String(ready) : "",
    enable: enable ? String(enable) : "",
    noBoot,
    onError: typeof o.onError === "function" ? o.onError : null,
  };
}

/**
 * Collect mount configs from `#marqdo-boot` JSON and `script[data-mq-*]`.
 * Pure DOM scan — safe to call once after DOM ready.
 */
export function collectMountConfigs(doc = typeof document !== "undefined" ? document : null) {
  if (!doc) return [];
  const out = [];
  const bootEl = doc.getElementById("marqdo-boot");
  if (bootEl) {
    try {
      const json = JSON.parse(bootEl.textContent || "{}");
      out.push(normalizeMountOptions(json));
    } catch (e) {
      console.error("marqdo-boot JSON:", e);
    }
  }
  const scripts = doc.querySelectorAll(
    "script[data-mq-source-url], script[data-mq-source], script[data-mq-playground], script[data-mq-wasm]",
  );
  for (const el of scripts) {
    if (el.dataset.mqNoBoot === "1" || el.dataset.mqNoBoot === "true") continue;
    const hasWork =
      el.dataset.mqSourceUrl ||
      el.dataset.mqSource ||
      el.dataset.mqPlayground;
    if (!hasWork && !el.dataset.mqWasm) continue;
    if (!hasWork) continue;
    out.push(
      normalizeMountOptions({
        dataset: el.dataset,
        wasmUrl: el.dataset.mqWasm,
        sourceUrl: el.dataset.mqSourceUrl,
        source: el.dataset.mqSource,
        playground: el.dataset.mqPlayground,
        ready: el.dataset.mqReady,
        enable: el.dataset.mqEnable,
      }),
    );
  }
  return out;
}

/** Build mq_call args from a DOM event + optional value_from selector. */
export function eventArgsFromDom(ev, valueFromSel) {
  const t = ev && ev.target;
  const args = {
    event: ev && ev.type ? ev.type : "click",
  };
  if (t && typeof t === "object") {
    if ("value" in t && t.value != null) args.value = String(t.value);
    if ("checked" in t) args.checked = !!t.checked;
    if (t.id) args.id = String(t.id);
    if (typeof t.textContent === "string" && !("value" in t)) {
      args.text = t.textContent;
    }
  }
  if (valueFromSel && typeof document !== "undefined") {
    const src = document.querySelector(valueFromSel);
    if (src) {
      if ("value" in src) args.value = String(src.value);
      else args.value = src.textContent != null ? String(src.textContent) : "";
    }
  }
  if (ev && ev.type === "submit" && t) {
    const form = typeof t.closest === "function" ? t.closest("form") : t;
    if (form && typeof FormData !== "undefined") {
      try {
        ev.preventDefault();
      } catch (_) {
        /* ignore */
      }
      const fields = {};
      for (const [k, v] of new FormData(form)) {
        fields[k] = String(v);
      }
      args.fields = fields;
    }
  }
  return args;
}

/** Apply C3/C4/D3 return effects: DOM patches then fetch/after. */
export function applyDomPatch(value) {
  if (!value || typeof value !== "object") return;
  if (typeof document === "undefined") return;

  const setText = value.set_text || value.setText;
  if (isPlainObject(setText)) {
    for (const [sel, text] of Object.entries(setText)) {
      const el = document.querySelector(sel);
      if (el) el.textContent = String(text);
    }
  }

  const setValue = value.set_value || value.setValue;
  if (isPlainObject(setValue)) {
    for (const [sel, v] of Object.entries(setValue)) {
      const el = document.querySelector(sel);
      if (el && "value" in el) el.value = String(v);
    }
  }

  const setAttr = value.set_attr || value.setAttr;
  if (isPlainObject(setAttr)) {
    for (const [sel, attrs] of Object.entries(setAttr)) {
      const el = document.querySelector(sel);
      if (!el || !isPlainObject(attrs)) continue;
      for (const [name, raw] of Object.entries(attrs)) {
        if (raw === null || raw === false) el.removeAttribute(name);
        else el.setAttribute(name, String(raw));
      }
    }
  }

  const setClass = value.set_class || value.setClass;
  if (isPlainObject(setClass)) {
    for (const [sel, cls] of Object.entries(setClass)) {
      const el = document.querySelector(sel);
      if (el) el.className = String(cls);
    }
  }

  const toggleClass = value.toggle_class || value.toggleClass;
  if (isPlainObject(toggleClass)) {
    for (const [sel, cls] of Object.entries(toggleClass)) {
      const el = document.querySelector(sel);
      if (el && cls) el.classList.toggle(String(cls));
    }
  }

  const setHtml = value.set_html || value.setHtml;
  if (isPlainObject(setHtml)) {
    for (const [sel, html] of Object.entries(setHtml)) {
      if (!String(sel).startsWith("#trusted")) continue;
      const el = document.querySelector(sel);
      if (el) el.innerHTML = String(html);
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
    const valueFrom =
      row["值选择器"] || row.value_from || row.valueFrom || row.from || "";
    if (!sel || !fn) continue;
    const nodes = document.querySelectorAll(sel);
    nodes.forEach((node) => {
      node.addEventListener(ev, (domEv) => {
        try {
          const args = eventArgsFromDom(domEv, valueFrom);
          const result = call(exports, fn, args);
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

function markReady(opts, message) {
  if (opts.ready && typeof document !== "undefined") {
    const el = document.querySelector(opts.ready);
    if (el) {
      el.classList?.remove?.("err");
      el.textContent = message;
    }
  }
  if (opts.enable && typeof document !== "undefined") {
    for (const sel of String(opts.enable).split(",")) {
      const s = sel.trim();
      if (!s) continue;
      document.querySelectorAll(s).forEach((n) => {
        n.disabled = false;
        n.removeAttribute("disabled");
      });
    }
  }
}

function markError(opts, err) {
  const msg = String(err);
  if (opts.onError) opts.onError(msg);
  if (opts.ready && typeof document !== "undefined") {
    const el = document.querySelector(opts.ready);
    if (el) {
      el.classList?.add?.("err");
      el.textContent = msg;
    }
  } else {
    console.error(msg);
  }
}

function wirePlayground(exports, playgroundSpec, { onError } = {}) {
  const parts = String(playgroundSpec)
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  const sourceSel = parts[0] || "#src";
  const runSel = parts[1] || "#run";
  const outSel = parts[2] || "#out";
  const runBtn = document.querySelector(runSel);
  const src = document.querySelector(sourceSel);
  const out = document.querySelector(outSel);
  if (!runBtn || !src) return 0;
  runBtn.disabled = false;
  runBtn.removeAttribute("disabled");
  runBtn.addEventListener("click", () => {
    try {
      const result = runSource(exports, src.value || "");
      if (out) {
        out.classList?.toggle?.("err", !result.ok);
        out.textContent = result.ok
          ? (result.stdout || "(empty stdout)") +
            (result.value != null && result.value !== "None"
              ? `\n[return] ${typeof result.value === "string" ? result.value : JSON.stringify(result.value)}`
              : "")
          : result.error || "error";
      }
    } catch (e) {
      if (onError) onError(String(e));
      else if (out) {
        out.classList?.add?.("err");
        out.textContent = String(e);
      }
    }
  });
  return 1;
}

/**
 * One-shot: load wasm, boot session (or playground), wire events.
 * @returns {{ exports, boot, wired }}
 */
export async function mount(rawOpts = {}) {
  const opts = normalizeMountOptions(rawOpts);
  const onError = (err) => markError(opts, err);

  let exports;
  try {
    exports = await loadWasm(opts.wasmUrl);
  } catch (e) {
    onError(e);
    throw e;
  }

  const ver = readCString(exports.memory, exports.mq_version());

  if (opts.playground) {
    const n = wirePlayground(exports, opts.playground, { onError });
    markReady(opts, `marqdo-wasm ${ver} · playground ready`);
    return { exports, boot: null, wired: n };
  }

  let source = opts.source;
  if (!source && opts.sourceUrl) {
    const res = await fetch(opts.sourceUrl);
    if (!res.ok) {
      const err = `failed to load ${opts.sourceUrl} (${res.status})`;
      onError(err);
      throw new Error(err);
    }
    source = await res.text();
  }
  if (!source) {
    const err = "mount: need source, sourceUrl, or playground";
    onError(err);
    throw new Error(err);
  }

  const result = boot(exports, source);
  if (!result.ok) {
    onError(result.error || "boot failed");
    throw new Error(result.error || "boot failed");
  }

  // If boot value is a map with effects alongside wire, apply them.
  if (result.value && !Array.isArray(result.value)) {
    await applyEffects(exports, result.value, { onError });
  }

  const n = wireEvents(exports, result.value, { onError });
  const srcLabel = opts.sourceUrl || "(inline)";
  markReady(opts, `marqdo-wasm ${ver} · wired ${n} handler(s)\nsource: ${srcLabel}`);
  return { exports, boot: result, wired: n };
}

/** Auto-mount from DOM configs (route D). */
export async function autoMount(doc) {
  const configs = collectMountConfigs(doc);
  const results = [];
  for (const cfg of configs) {
    if (cfg.noBoot) continue;
    results.push(await mount(cfg));
  }
  return results;
}

const g = typeof globalThis !== "undefined" ? globalThis : undefined;
if (g && g.document) {
  const start = () => {
    autoMount(g.document).catch((e) => console.error("marqdo autoMount:", e));
  };
  if (g.document.readyState === "loading") {
    g.document.addEventListener("DOMContentLoaded", start);
  } else {
    queueMicrotask(start);
  }
}
