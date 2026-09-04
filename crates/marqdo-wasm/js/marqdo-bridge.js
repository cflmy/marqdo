/**
 * Canonical Marqdo browser bridge (routes C/D/E). Host glue only — not app logic.
 * Authors write .mq.md only; this file may grow host mechanisms freely.
 *
 * ABI: mq_alloc, mq_dealloc, mq_run, mq_boot, mq_call, mq_version
 * Effects: set_text, set_value, set_attr, set_class, toggle_class, set_style,
 *   focus, blur, scroll_into, set_html, replace_children, render_list,
 *   navigate, storage, ws, fetch, fetch_all, after, interval, clear_interval
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

function asSelList(v) {
  if (v == null) return [];
  if (Array.isArray(v)) return v.map(String);
  return [String(v)];
}

/** Normalize mount options from plain object or DOM dataset. */
export function normalizeMountOptions(raw = {}) {
  const o = raw && typeof raw === "object" ? raw : {};
  const dataset = o.dataset || {};
  const wasmUrl =
    o.wasmUrl || o.wasm || dataset.mqWasm || dataset.wasm || "./marqdo_wasm.wasm";
  const sourceUrl =
    o.sourceUrl || o.source_url || dataset.mqSourceUrl || dataset.sourceUrl || "";
  let source = o.source != null ? o.source : dataset.mqSource || "";
  if (!source && typeof o.sourceInline === "string") source = o.sourceInline;
  const playground = o.playground || dataset.mqPlayground || dataset.playground || "";
  const ready = o.ready || o.readySel || dataset.mqReady || "";
  const enable = o.enable || dataset.mqEnable || "";
  const noBoot =
    o.noBoot === true || dataset.mqNoBoot === "1" || dataset.mqNoBoot === "true";
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

export function collectMountConfigs(doc = typeof document !== "undefined" ? document : null) {
  if (!doc) return [];
  const out = [];
  const bootEl = doc.getElementById("marqdo-boot");
  if (bootEl) {
    try {
      out.push(normalizeMountOptions(JSON.parse(bootEl.textContent || "{}")));
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
      el.dataset.mqSourceUrl || el.dataset.mqSource || el.dataset.mqPlayground;
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

export function eventArgsFromDom(ev, valueFromSel) {
  const t = ev && ev.target;
  const args = { event: ev && ev.type ? ev.type : "click" };
  if (t && typeof t === "object") {
    if ("value" in t && t.value != null) args.value = String(t.value);
    if ("checked" in t) args.checked = !!t.checked;
    if (t.id) args.id = String(t.id);
    if (t.getAttribute) {
      const did = t.getAttribute("data-id");
      if (did != null) args.data_id = did;
    }
    if (typeof t.textContent === "string" && !("value" in t)) {
      args.text = t.textContent;
    }
    if (ev.key != null) args.key = String(ev.key);
    if (ev.code != null) args.code = String(ev.code);
  }
  if (ev && ev.type === "popstate") {
    args.url = typeof location !== "undefined" ? location.href : "";
    args.path = typeof location !== "undefined" ? location.pathname + location.search : "";
    args.state = ev.state;
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
      for (const [k, v] of new FormData(form)) fields[k] = String(v);
      args.fields = fields;
    }
  }
  return args;
}

function applyStyle(el, styleVal) {
  if (typeof styleVal === "string") {
    el.setAttribute("style", styleVal);
    return;
  }
  if (isPlainObject(styleVal)) {
    for (const [k, v] of Object.entries(styleVal)) {
      if (v === null || v === false) el.style.removeProperty(k);
      else el.style.setProperty(k.replace(/[A-Z]/g, (m) => "-" + m.toLowerCase()), String(v));
    }
  }
}

export function renderListItems(spec) {
  const tag = String(spec.tag || "li");
  const items = Array.isArray(spec.items) ? spec.items : [];
  return items
    .map((it) => {
      if (it == null) return "";
      if (typeof it === "string" || typeof it === "number") {
        return `<${tag}>${escapeHtml(String(it))}</${tag}>`;
      }
      if (!isPlainObject(it)) return "";
      const text = it.text != null ? String(it.text) : it.html != null ? null : "";
      const cls = it.class != null ? ` class="${escapeAttr(String(it.class))}"` : "";
      let attrs = "";
      if (isPlainObject(it.attrs)) {
        for (const [k, v] of Object.entries(it.attrs)) {
          if (v === null || v === false) continue;
          attrs += ` ${escapeAttr(k)}="${escapeAttr(String(v))}"`;
        }
      }
      const inner = it.html != null ? String(it.html) : escapeHtml(text ?? "");
      return `<${tag}${cls}${attrs}>${inner}</${tag}>`;
    })
    .join("");
}

function escapeHtml(s) {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function escapeAttr(s) {
  return escapeHtml(s).replace(/'/g, "&#39;");
}

/** Sync DOM patches (E1/E2/E3 navigate is sync too). */
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

  const setStyle = value.set_style || value.setStyle;
  if (isPlainObject(setStyle)) {
    for (const [sel, st] of Object.entries(setStyle)) {
      const el = document.querySelector(sel);
      if (el) applyStyle(el, st);
    }
  }

  for (const sel of asSelList(value.focus)) {
    const el = document.querySelector(sel);
    if (el && el.focus) el.focus();
  }
  for (const sel of asSelList(value.blur)) {
    const el = document.querySelector(sel);
    if (el && el.blur) el.blur();
  }
  for (const sel of asSelList(value.scroll_into || value.scrollInto)) {
    const el = document.querySelector(sel);
    if (el && el.scrollIntoView) el.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }

  const setHtml = value.set_html || value.setHtml;
  if (isPlainObject(setHtml)) {
    for (const [sel, html] of Object.entries(setHtml)) {
      const el = document.querySelector(sel);
      if (el) el.innerHTML = String(html);
    }
  }

  const replaceChildren = value.replace_children || value.replaceChildren;
  if (isPlainObject(replaceChildren)) {
    for (const [sel, html] of Object.entries(replaceChildren)) {
      const el = document.querySelector(sel);
      if (el) el.innerHTML = String(html);
    }
  }

  const renderList = value.render_list || value.renderList;
  if (isPlainObject(renderList)) {
    for (const [sel, spec] of Object.entries(renderList)) {
      const el = document.querySelector(sel);
      if (!el || !isPlainObject(spec)) continue;
      el.innerHTML = renderListItems(spec);
    }
  }

  const nav = value.navigate;
  if (isPlainObject(nav) && nav.url != null && typeof history !== "undefined") {
    const url = String(nav.url);
    if (nav.replace) history.replaceState(nav.state ?? null, "", url);
    else history.pushState(nav.state ?? null, "", url);
  }
}

const intervalHandles = new Map();
const wsHandles = new Map();
let wsSeq = 0;

/**
 * Run sync DOM patches then schedule async effects.
 */
export function applyEffects(exports, value, { onError } = {}) {
  applyDomPatch(value);
  if (!value || typeof value !== "object") return Promise.resolve();

  const tasks = [];

  const fetchSpec = value.fetch;
  if (fetchSpec && typeof fetchSpec === "object" && fetchSpec.url && fetchSpec.then) {
    tasks.push(runFetchEffect(exports, fetchSpec, onError));
  }

  const fetchAll = value.fetch_all || value.fetchAll;
  if (fetchAll && typeof fetchAll === "object" && fetchAll.then) {
    tasks.push(runFetchAllEffect(exports, fetchAll, onError));
  }

  const afterSpec = value.after;
  if (afterSpec && typeof afterSpec === "object" && afterSpec.then) {
    tasks.push(runAfterEffect(exports, afterSpec, onError));
  }

  const intervalSpec = value.interval;
  if (intervalSpec && typeof intervalSpec === "object" && intervalSpec.then) {
    runIntervalEffect(exports, intervalSpec, onError);
  }

  const clearIv = value.clear_interval || value.clearInterval;
  if (clearIv && typeof clearIv === "object") {
    const id = String(clearIv.id || "default");
    const h = intervalHandles.get(id);
    if (h != null) {
      clearInterval(h);
      intervalHandles.delete(id);
    }
  }

  const storageSpec = value.storage;
  if (storageSpec && typeof storageSpec === "object" && storageSpec.op) {
    tasks.push(runStorageEffect(exports, storageSpec, onError));
  }

  const wsSpec = value.ws;
  if (wsSpec && typeof wsSpec === "object" && wsSpec.op) {
    tasks.push(runWsEffect(exports, wsSpec, onError));
  }

  return Promise.all(tasks);
}

function buildFetchInit(spec) {
  const method = (spec.method || "GET").toUpperCase();
  const init = { method };
  if (spec.headers && typeof spec.headers === "object") init.headers = { ...spec.headers };
  if (spec.fields && typeof spec.fields === "object" && typeof FormData !== "undefined") {
    const fd = new FormData();
    for (const [k, v] of Object.entries(spec.fields)) fd.append(k, String(v));
    init.body = fd;
  } else if (spec.body != null && method !== "GET" && method !== "HEAD") {
    init.body = typeof spec.body === "string" ? spec.body : JSON.stringify(spec.body);
  }
  return init;
}

async function runFetchEffect(exports, spec, onError) {
  const thenFn = spec.then;
  try {
    const res = await fetch(String(spec.url), buildFetchInit(spec));
    const body = await res.text();
    const result = call(exports, thenFn, { ok: res.ok, status: res.status, body });
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

async function runFetchAllEffect(exports, spec, onError) {
  const thenFn = spec.then;
  const reqs = Array.isArray(spec.requests) ? spec.requests : [];
  try {
    const results = await Promise.all(
      reqs.map(async (r) => {
        try {
          const res = await fetch(String(r.url), buildFetchInit(r));
          const body = await res.text();
          return { ok: res.ok, status: res.status, body, url: String(r.url) };
        } catch (e) {
          return { ok: false, status: 0, body: "", error: String(e), url: String(r.url || "") };
        }
      }),
    );
    const result = call(exports, thenFn, { ok: true, results });
    if (!result.ok) {
      if (onError) onError(result.error);
      else console.error(result.error);
      return;
    }
    await applyEffects(exports, result.value, { onError });
  } catch (e) {
    if (onError) onError(String(e));
    else console.error(e);
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

function runIntervalEffect(exports, spec, onError) {
  const ms = Math.max(1, Number(spec.ms) || 1000);
  const thenFn = spec.then;
  const id = String(spec.id || "default");
  if (intervalHandles.has(id)) {
    clearInterval(intervalHandles.get(id));
    intervalHandles.delete(id);
  }
  const handle = setInterval(() => {
    try {
      const result = call(exports, thenFn, { ok: true, id });
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
  }, ms);
  intervalHandles.set(id, handle);
}

function storageStore(scope) {
  if (typeof localStorage === "undefined") return null;
  return scope === "session" ? sessionStorage : localStorage;
}

async function runStorageEffect(exports, spec, onError) {
  const op = String(spec.op || "").toLowerCase();
  const key = String(spec.key || "");
  const store = storageStore(spec.scope || "local");
  if (!store) {
    if (onError) onError("storage unavailable");
    return;
  }
  try {
    if (op === "set") {
      store.setItem(key, spec.value != null ? String(spec.value) : "");
      if (spec.then) {
        const result = call(exports, spec.then, { ok: true, op, key });
        if (result.ok) await applyEffects(exports, result.value, { onError });
      }
      return;
    }
    if (op === "remove" || op === "delete") {
      store.removeItem(key);
      if (spec.then) {
        const result = call(exports, spec.then, { ok: true, op, key });
        if (result.ok) await applyEffects(exports, result.value, { onError });
      }
      return;
    }
    if (op === "get") {
      const value = store.getItem(key);
      if (spec.then) {
        const result = call(exports, spec.then, {
          ok: true,
          op,
          key,
          value: value == null ? "" : value,
          found: value != null,
        });
        if (!result.ok) {
          if (onError) onError(result.error);
          return;
        }
        await applyEffects(exports, result.value, { onError });
      }
    }
  } catch (e) {
    if (onError) onError(String(e));
    else console.error(e);
  }
}

function runWsEffect(exports, spec, onError) {
  const op = String(spec.op || "").toLowerCase();
  return new Promise((resolve) => {
    try {
      if (op === "open") {
        if (typeof WebSocket === "undefined") {
          if (onError) onError("WebSocket unavailable");
          resolve();
          return;
        }
        const id = String(spec.id || `ws${++wsSeq}`);
        const sock = new WebSocket(String(spec.url));
        wsHandles.set(id, sock);
        sock.addEventListener("open", () => {
          if (spec.then_open || spec.thenOpen) {
            try {
              const result = call(exports, spec.then_open || spec.thenOpen, { ok: true, id });
              if (result.ok) applyEffects(exports, result.value, { onError });
            } catch (e) {
              if (onError) onError(String(e));
            }
          }
          resolve();
        });
        sock.addEventListener("message", (ev) => {
          const thenMsg = spec.then_message || spec.thenMessage;
          if (!thenMsg) return;
          try {
            const result = call(exports, thenMsg, {
              ok: true,
              id,
              data: typeof ev.data === "string" ? ev.data : String(ev.data),
            });
            if (result.ok) applyEffects(exports, result.value, { onError });
          } catch (e) {
            if (onError) onError(String(e));
          }
        });
        sock.addEventListener("close", () => {
          wsHandles.delete(id);
          const thenClose = spec.then_close || spec.thenClose;
          if (thenClose) {
            try {
              const result = call(exports, thenClose, { ok: true, id });
              if (result.ok) applyEffects(exports, result.value, { onError });
            } catch (e) {
              if (onError) onError(String(e));
            }
          }
        });
        sock.addEventListener("error", () => {
          const thenErr = spec.then_error || spec.thenError;
          if (thenErr) {
            try {
              const result = call(exports, thenErr, { ok: false, id, error: "ws error" });
              if (result.ok) applyEffects(exports, result.value, { onError });
            } catch (e) {
              if (onError) onError(String(e));
            }
          }
        });
        return;
      }
      if (op === "send") {
        const id = String(spec.id || [...wsHandles.keys()][0] || "");
        const sock = wsHandles.get(id);
        if (sock && sock.readyState === WebSocket.OPEN) {
          sock.send(spec.data != null ? String(spec.data) : "");
        } else if (onError) onError(`ws send: no open socket ${id}`);
        resolve();
        return;
      }
      if (op === "close") {
        const id = String(spec.id || [...wsHandles.keys()][0] || "");
        const sock = wsHandles.get(id);
        if (sock) sock.close();
        wsHandles.delete(id);
        resolve();
        return;
      }
      resolve();
    } catch (e) {
      if (onError) onError(String(e));
      resolve();
    }
  });
}

function bindTarget(sel) {
  if (sel === "window" || sel === "@window") return typeof window !== "undefined" ? window : null;
  if (sel === "document" || sel === "@document")
    return typeof document !== "undefined" ? document : null;
  return null;
}

/**
 * Wire events from `# main` return value (list or `{ wire: [...] }`).
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
    const delegate =
      row["委托"] || row.delegate || row.delegation || "";
    if (!sel || !fn) continue;

    const special = bindTarget(sel);
    const roots = special
      ? [special]
      : typeof document !== "undefined"
        ? [...document.querySelectorAll(sel)]
        : [];

    roots.forEach((node) => {
      node.addEventListener(ev, (domEv) => {
        try {
          if (delegate) {
            const hit =
              typeof domEv.target?.closest === "function"
                ? domEv.target.closest(delegate)
                : null;
            if (!hit || (node.contains && !node.contains(hit))) return;
          }
          if (
            ev === "click" &&
            typeof domEv.target?.closest === "function" &&
            domEv.target.closest("a[href]")
          ) {
            try {
              domEv.preventDefault();
            } catch (_) {
              /* ignore */
            }
          }
          const args = eventArgsFromDom(domEv, valueFrom);
          if (delegate && domEv.target) {
            const hit = domEv.target.closest?.(delegate);
            if (hit?.getAttribute) {
              const did = hit.getAttribute("data-id");
              if (did != null) args.data_id = did;
              if (hit.id) args.id = hit.id;
            }
          }
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

  if (result.value && !Array.isArray(result.value)) {
    await applyEffects(exports, result.value, { onError });
  }

  const n = wireEvents(exports, result.value, { onError });
  const srcLabel = opts.sourceUrl || "(inline)";
  markReady(opts, `marqdo-wasm ${ver} · wired ${n} handler(s)\nsource: ${srcLabel}`);
  return { exports, boot: result, wired: n };
}

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
