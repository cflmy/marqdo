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
    if ("value" in t && t.value != null && t.type !== "file") args.value = String(t.value);
    if ("checked" in t) args.checked = !!t.checked;
    if (t.id) args.id = String(t.id);
    if (t.getAttribute) {
      const did = t.getAttribute("data-id");
      if (did != null) args.data_id = did;
      const drag = t.getAttribute("data-drag");
      if (drag != null) args.drag = drag;
    }
    if (typeof t.textContent === "string" && !("value" in t)) {
      args.text = t.textContent;
    }
    if (ev.key != null) args.key = String(ev.key);
    if (ev.code != null) args.code = String(ev.code);
    if (t.files && t.files.length) {
      const f = t.files[0];
      args.file_name = f.name;
      args.file_size = f.size;
      args.file_type = f.type || "";
      args.file_count = t.files.length;
    }
  }
  if (ev && ev.type === "popstate") {
    args.url = typeof location !== "undefined" ? location.href : "";
    args.path = typeof location !== "undefined" ? location.pathname + location.search : "";
    args.state = ev.state;
  }
  if (ev && ev.dataTransfer) {
    try {
      const txt = ev.dataTransfer.getData("text") || ev.dataTransfer.getData("text/plain");
      if (txt) args.drop_text = txt;
    } catch (_) {
      /* ignore */
    }
  }
  if (valueFromSel && typeof document !== "undefined") {
    const src = document.querySelector(valueFromSel);
    if (src) {
      if ("value" in src && src.type !== "file") args.value = String(src.value);
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

function qsAll(sel) {
  if (typeof document === "undefined") return [];
  try {
    return [...document.querySelectorAll(sel)];
  } catch (_) {
    return [];
  }
}

function qsOne(sel) {
  const all = qsAll(sel);
  return all[0] || null;
}

/** Sync DOM patches (E1/E2/E3 navigate is sync too). */
export function applyDomPatch(value) {
  if (!value || typeof value !== "object") return;
  if (typeof document === "undefined") return;

  const setText = value.set_text || value.setText;
  if (isPlainObject(setText)) {
    for (const [sel, text] of Object.entries(setText)) {
      for (const el of qsAll(sel)) el.textContent = String(text);
    }
  }

  const setValue = value.set_value || value.setValue;
  if (isPlainObject(setValue)) {
    for (const [sel, v] of Object.entries(setValue)) {
      for (const el of qsAll(sel)) {
        if ("value" in el) el.value = String(v);
      }
    }
  }

  const setAttr = value.set_attr || value.setAttr;
  if (isPlainObject(setAttr)) {
    for (const [sel, attrs] of Object.entries(setAttr)) {
      if (!isPlainObject(attrs)) continue;
      for (const el of qsAll(sel)) {
        for (const [name, raw] of Object.entries(attrs)) {
          if (raw === null || raw === false) el.removeAttribute(name);
          else el.setAttribute(name, String(raw));
        }
      }
    }
  }

  const setClass = value.set_class || value.setClass;
  if (isPlainObject(setClass)) {
    for (const [sel, cls] of Object.entries(setClass)) {
      for (const el of qsAll(sel)) el.className = String(cls);
    }
  }

  const addClass = value.add_class || value.addClass;
  if (isPlainObject(addClass)) {
    for (const [sel, cls] of Object.entries(addClass)) {
      const names = String(cls).split(/\s+/).filter(Boolean);
      for (const el of qsAll(sel)) for (const c of names) el.classList.add(c);
    }
  }

  const removeClass = value.remove_class || value.removeClass;
  if (isPlainObject(removeClass)) {
    for (const [sel, cls] of Object.entries(removeClass)) {
      const names = String(cls).split(/\s+/).filter(Boolean);
      for (const el of qsAll(sel)) for (const c of names) el.classList.remove(c);
    }
  }

  const toggleClass = value.toggle_class || value.toggleClass;
  if (isPlainObject(toggleClass)) {
    for (const [sel, cls] of Object.entries(toggleClass)) {
      for (const el of qsAll(sel)) {
        if (cls) el.classList.toggle(String(cls));
      }
    }
  }

  const setStyle = value.set_style || value.setStyle;
  if (isPlainObject(setStyle)) {
    for (const [sel, st] of Object.entries(setStyle)) {
      for (const el of qsAll(sel)) applyStyle(el, st);
    }
  }

  for (const sel of asSelList(value.focus)) {
    const el = qsOne(sel);
    if (el && el.focus) el.focus();
  }
  for (const sel of asSelList(value.blur)) {
    const el = qsOne(sel);
    if (el && el.blur) el.blur();
  }
  for (const sel of asSelList(value.scroll_into || value.scrollInto)) {
    const el = qsOne(sel);
    if (el && el.scrollIntoView) el.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }

  const setHtml = value.set_html || value.setHtml;
  if (isPlainObject(setHtml)) {
    for (const [sel, html] of Object.entries(setHtml)) {
      for (const el of qsAll(sel)) el.innerHTML = String(html);
    }
  }

  const appendHtml = value.append_html || value.appendHtml;
  if (isPlainObject(appendHtml)) {
    for (const [sel, html] of Object.entries(appendHtml)) {
      for (const el of qsAll(sel)) el.insertAdjacentHTML("beforeend", String(html));
    }
  }

  const replaceChildren = value.replace_children || value.replaceChildren;
  if (isPlainObject(replaceChildren)) {
    for (const [sel, html] of Object.entries(replaceChildren)) {
      for (const el of qsAll(sel)) el.innerHTML = String(html);
    }
  }

  const renderList = value.render_list || value.renderList;
  if (isPlainObject(renderList)) {
    for (const [sel, spec] of Object.entries(renderList)) {
      if (!isPlainObject(spec)) continue;
      const html = renderListItems(spec);
      for (const el of qsAll(sel)) el.innerHTML = html;
    }
  }

  const removeSel = value.remove || value.remove_nodes || value.removeNodes;
  if (removeSel) {
    for (const sel of asSelList(removeSel)) {
      for (const el of qsAll(sel)) el.remove();
    }
  }

  const nav = value.navigate;
  if (isPlainObject(nav) && nav.url != null && typeof history !== "undefined") {
    const url = String(nav.url);
    if (nav.replace) history.replaceState(nav.state ?? null, "", url);
    else history.pushState(nav.state ?? null, "", url);
  }

  const clip = value.clipboard || value.copy;
  if (clip != null && typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
    const text = isPlainObject(clip) ? String(clip.text ?? "") : String(clip);
    navigator.clipboard.writeText(text).catch(() => {});
  }

  const dl = value.download;
  if (isPlainObject(dl) && dl.body != null) {
    const mime = String(dl.mime || "text/plain;charset=utf-8");
    const name = String(dl.filename || "download.txt");
    const blob = new Blob([String(dl.body)], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = name;
    a.click();
    URL.revokeObjectURL(url);
  }

  const canvasSpec = value.canvas;
  if (isPlainObject(canvasSpec) && canvasSpec.sel) {
    runCanvasCommands(canvasSpec);
  }

  const audioSpec = value.audio;
  if (isPlainObject(audioSpec) && audioSpec.op) {
    runAudioEffect(audioSpec);
  }
}

const intervalHandles = new Map();
const wsHandles = new Map();
const audioHandles = new Map();
const observeHandles = new Map();
let wsSeq = 0;
let observeSeq = 0;

/** @param {{ sel: string, commands?: object[], clear?: boolean }} spec */
export function runCanvasCommands(spec) {
  if (typeof document === "undefined") return;
  const el = qsOne(String(spec.sel));
  if (!el || typeof el.getContext !== "function") return;
  const ctx = el.getContext("2d");
  if (!ctx) return;
  if (spec.clear) ctx.clearRect(0, 0, el.width || 0, el.height || 0);
  const cmds = Array.isArray(spec.commands)
    ? spec.commands
    : isPlainObject(spec.commands)
      ? Object.keys(spec.commands)
          .sort()
          .map((k) => spec.commands[k])
      : [];
  for (const raw of cmds) {
    if (!isPlainObject(raw)) continue;
    const op = String(raw.op || raw.cmd || "").toLowerCase();
    try {
      if (op === "clear" || op === "clearrect") {
        ctx.clearRect(
          Number(raw.x) || 0,
          Number(raw.y) || 0,
          Number(raw.w ?? raw.width ?? el.width) || 0,
          Number(raw.h ?? raw.height ?? el.height) || 0,
        );
      } else if (op === "fillstyle") {
        ctx.fillStyle = String(raw.value ?? raw.color ?? "#000");
      } else if (op === "strokestyle") {
        ctx.strokeStyle = String(raw.value ?? raw.color ?? "#000");
      } else if (op === "linewidth") {
        ctx.lineWidth = Number(raw.value ?? raw.width) || 1;
      } else if (op === "font") {
        ctx.font = String(raw.value || "16px sans-serif");
      } else if (op === "fillrect") {
        if (raw.fill != null) ctx.fillStyle = String(raw.fill);
        ctx.fillRect(Number(raw.x) || 0, Number(raw.y) || 0, Number(raw.w ?? raw.width) || 0, Number(raw.h ?? raw.height) || 0);
      } else if (op === "strokerect") {
        if (raw.stroke != null) ctx.strokeStyle = String(raw.stroke);
        ctx.strokeRect(Number(raw.x) || 0, Number(raw.y) || 0, Number(raw.w ?? raw.width) || 0, Number(raw.h ?? raw.height) || 0);
      } else if (op === "beginpath") {
        ctx.beginPath();
      } else if (op === "closepath") {
        ctx.closePath();
      } else if (op === "moveto") {
        ctx.moveTo(Number(raw.x) || 0, Number(raw.y) || 0);
      } else if (op === "lineto") {
        ctx.lineTo(Number(raw.x) || 0, Number(raw.y) || 0);
      } else if (op === "arc") {
        ctx.arc(
          Number(raw.x) || 0,
          Number(raw.y) || 0,
          Number(raw.r ?? raw.radius) || 0,
          Number(raw.start ?? 0),
          Number(raw.end ?? Math.PI * 2),
          !!raw.ccw,
        );
      } else if (op === "stroke") {
        if (raw.stroke != null) ctx.strokeStyle = String(raw.stroke);
        ctx.stroke();
      } else if (op === "fill") {
        if (raw.fill != null) ctx.fillStyle = String(raw.fill);
        ctx.fill();
      } else if (op === "filltext") {
        if (raw.fill != null) ctx.fillStyle = String(raw.fill);
        ctx.fillText(String(raw.text ?? ""), Number(raw.x) || 0, Number(raw.y) || 0);
      } else if (op === "drawimage" && raw.src) {
        const img = new Image();
        img.src = String(raw.src);
        // sync only if already loaded; else fire-and-forget draw on load
        const draw = () => {
          ctx.drawImage(
            img,
            Number(raw.x) || 0,
            Number(raw.y) || 0,
            Number(raw.w ?? raw.width ?? img.width) || img.width,
            Number(raw.h ?? raw.height ?? img.height) || img.height,
          );
        };
        if (img.complete) draw();
        else img.onload = draw;
      }
    } catch (_) {
      /* skip bad cmd */
    }
  }
}

function runAudioEffect(spec) {
  const op = String(spec.op || "").toLowerCase();
  const id = String(spec.id || spec.sel || "default");
  try {
    if (op === "play") {
      // Optional beep via Web Audio (no external file).
      if (spec.freq != null || spec.frequency != null || String(spec.src || "") === "beep") {
        const AC = window.AudioContext || window.webkitAudioContext;
        if (!AC) return;
        let ctx = audioHandles.get(`${id}:ctx`);
        if (!ctx) {
          ctx = new AC();
          audioHandles.set(`${id}:ctx`, ctx);
        }
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();
        osc.type = String(spec.wave || "sine");
        osc.frequency.value = Number(spec.freq ?? spec.frequency ?? 440);
        const vol = spec.volume != null ? Math.min(1, Math.max(0, Number(spec.volume))) : 0.15;
        gain.gain.value = vol;
        osc.connect(gain);
        gain.connect(ctx.destination);
        const dur = Number(spec.ms ?? spec.duration ?? 280) / 1000;
        osc.start();
        gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + Math.max(0.05, dur));
        osc.stop(ctx.currentTime + Math.max(0.05, dur) + 0.02);
        return;
      }
      let audio = audioHandles.get(id);
      if (spec.sel) {
        const el = qsOne(String(spec.sel));
        if (el && el.tagName === "AUDIO") audio = el;
      }
      if (!audio) {
        audio = new Audio(spec.src != null ? String(spec.src) : undefined);
        audioHandles.set(id, audio);
      } else if (spec.src != null) {
        audio.src = String(spec.src);
      }
      if (spec.volume != null) audio.volume = Math.min(1, Math.max(0, Number(spec.volume)));
      if (spec.loop != null) audio.loop = !!spec.loop;
      const p = audio.play();
      if (p && p.catch) p.catch(() => {});
      return;
    }
    const audio = spec.sel ? qsOne(String(spec.sel)) : audioHandles.get(id);
    if (!audio) return;
    if (op === "pause") audio.pause();
    else if (op === "stop") {
      audio.pause();
      try {
        audio.currentTime = 0;
      } catch (_) {
        /* ignore */
      }
    }
  } catch (_) {
    /* ignore */
  }
}

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

  const readFile = value.read_file || value.readFile;
  if (readFile && typeof readFile === "object" && readFile.then) {
    tasks.push(runReadFileEffect(exports, readFile, onError));
  }

  const observe = value.observe;
  if (observe) {
    const specs = Array.isArray(observe)
      ? observe
      : observe && typeof observe === "object" && observe.then
        ? [observe]
        : [];
    for (const spec of specs) runObserveEffect(exports, spec, onError);
  }

  const unobserve = value.unobserve;
  if (unobserve) {
    const specs = Array.isArray(unobserve)
      ? unobserve
      : unobserve && typeof unobserve === "object"
        ? [unobserve]
        : [];
    for (const spec of specs) {
      const id = String(spec.id || "");
      const h = observeHandles.get(id);
      if (h) {
        try {
          h.disconnect();
        } catch (_) {
          /* ignore */
        }
        observeHandles.delete(id);
      }
    }
  }

  return Promise.all(tasks);
}

async function runReadFileEffect(exports, spec, onError) {
  const thenFn = spec.then;
  const sel = String(spec.sel || "");
  const as = String(spec.as || "text").toLowerCase();
  const input = qsOne(sel);
  if (!input || !input.files || !input.files.length) {
    const result = call(exports, thenFn, { ok: false, error: "no file", sel });
    if (result.ok) await applyEffects(exports, result.value, { onError });
    return;
  }
  const file = input.files[Number(spec.index) || 0];
  try {
    let body = "";
    let data_url = "";
    if (as === "data_url" || as === "dataurl") {
      data_url = await new Promise((resolve, reject) => {
        const r = new FileReader();
        r.onload = () => resolve(String(r.result || ""));
        r.onerror = () => reject(r.error || new Error("read failed"));
        r.readAsDataURL(file);
      });
    } else {
      body = await file.text();
    }
    const result = call(exports, thenFn, {
      ok: true,
      name: file.name,
      size: file.size,
      type: file.type || "",
      body,
      data_url,
      as,
    });
    if (!result.ok) {
      if (onError) onError(result.error);
      return;
    }
    await applyEffects(exports, result.value, { onError });
  } catch (e) {
    try {
      const result = call(exports, thenFn, { ok: false, error: String(e), name: file.name });
      if (result.ok) await applyEffects(exports, result.value, { onError });
      else if (onError) onError(result.error);
    } catch (e2) {
      if (onError) onError(String(e2));
    }
  }
}

function runObserveEffect(exports, spec, onError) {
  const kind = String(spec.kind || spec.type || "intersect").toLowerCase();
  const sel = String(spec.sel || "");
  const thenFn = spec.then;
  const id = String(spec.id || `obs${++observeSeq}`);
  const el = qsOne(sel);
  if (!el || !thenFn) return;

  const prev = observeHandles.get(id);
  if (prev) {
    try {
      prev.disconnect();
    } catch (_) {
      /* ignore */
    }
  }

  const fire = (payload) => {
    try {
      const result = call(exports, thenFn, payload);
      if (!result.ok) {
        if (onError) onError(result.error);
        return;
      }
      applyEffects(exports, result.value, { onError });
    } catch (e) {
      if (onError) onError(String(e));
    }
  };

  if (kind === "resize" && typeof ResizeObserver !== "undefined") {
    const ro = new ResizeObserver((entries) => {
      const en = entries[0];
      const cr = en?.contentRect;
      fire({
        ok: true,
        kind: "resize",
        id,
        width: cr ? cr.width : 0,
        height: cr ? cr.height : 0,
      });
    });
    ro.observe(el);
    observeHandles.set(id, ro);
    return;
  }

  if (typeof IntersectionObserver !== "undefined") {
    const io = new IntersectionObserver(
      (entries) => {
        const en = entries[0];
        if (!en) return;
        fire({
          ok: true,
          kind: "intersect",
          id,
          intersecting: !!en.isIntersecting,
          ratio: en.intersectionRatio,
        });
        if (spec.once && en.isIntersecting) {
          try {
            io.disconnect();
          } catch (_) {
            /* ignore */
          }
          observeHandles.delete(id);
        }
      },
      { threshold: spec.threshold != null ? Number(spec.threshold) : 0.1 },
    );
    io.observe(el);
    observeHandles.set(id, io);
  }
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
  const reqs = Array.isArray(spec.requests)
    ? spec.requests
    : isPlainObject(spec.requests)
      ? Object.keys(spec.requests)
          .sort()
          .map((k) => spec.requests[k])
          .filter((r) => r && typeof r === "object")
      : [];
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
  if (scope === "cookie") return null;
  if (typeof localStorage === "undefined") return null;
  return scope === "session" ? sessionStorage : localStorage;
}

function cookieGet(key) {
  if (typeof document === "undefined") return null;
  const prefix = encodeURIComponent(key) + "=";
  for (const part of document.cookie.split(";")) {
    const s = part.trim();
    if (s.startsWith(prefix)) return decodeURIComponent(s.slice(prefix.length));
  }
  return null;
}

function cookieSet(key, value, days) {
  if (typeof document === "undefined") return;
  let extra = "; path=/";
  if (days != null && Number(days) > 0) {
    const d = new Date();
    d.setTime(d.getTime() + Number(days) * 864e5);
    extra += "; expires=" + d.toUTCString();
  }
  document.cookie = encodeURIComponent(key) + "=" + encodeURIComponent(value) + extra;
}

function cookieRemove(key) {
  if (typeof document === "undefined") return;
  document.cookie = encodeURIComponent(key) + "=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT";
}

async function runStorageEffect(exports, spec, onError) {
  const op = String(spec.op || "").toLowerCase();
  const key = String(spec.key || "");
  const scope = String(spec.scope || "local");
  try {
    if (scope === "cookie") {
      if (op === "set") {
        cookieSet(key, spec.value != null ? String(spec.value) : "", spec.days);
      } else if (op === "remove" || op === "delete") {
        cookieRemove(key);
      } else if (op === "get") {
        const value = cookieGet(key);
        if (spec.then) {
          const result = call(exports, spec.then, {
            ok: true,
            op,
            key,
            value: value == null ? "" : value,
            found: value != null,
            scope,
          });
          if (!result.ok) {
            if (onError) onError(result.error);
            return;
          }
          await applyEffects(exports, result.value, { onError });
        }
        return;
      }
      if (spec.then && op !== "get") {
        const result = call(exports, spec.then, { ok: true, op, key, scope });
        if (result.ok) await applyEffects(exports, result.value, { onError });
      }
      return;
    }

    const store = storageStore(scope);
    if (!store) {
      if (onError) onError("storage unavailable");
      return;
    }
    if (op === "set") {
      store.setItem(key, spec.value != null ? String(spec.value) : "");
      if (spec.then) {
        const result = call(exports, spec.then, { ok: true, op, key, scope });
        if (result.ok) await applyEffects(exports, result.value, { onError });
      }
      return;
    }
    if (op === "remove" || op === "delete") {
      store.removeItem(key);
      if (spec.then) {
        const result = call(exports, spec.then, { ok: true, op, key, scope });
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
          scope,
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
          // Drag/drop hosts need preventDefault on dragover/drop; dragstart can set payload.
          if (
            ev === "dragover" ||
            ev === "dragenter" ||
            ev === "drop" ||
            ev === "dragstart"
          ) {
            try {
              domEv.preventDefault();
            } catch (_) {
              /* ignore */
            }
          }
          if (ev === "dragstart" && domEv.dataTransfer) {
            try {
              const payload =
                (domEv.target &&
                  typeof domEv.target.getAttribute === "function" &&
                  domEv.target.getAttribute("data-drag")) ||
                (typeof domEv.target?.textContent === "string"
                  ? domEv.target.textContent.trim()
                  : "") ||
                "";
              if (payload) {
                domEv.dataTransfer.setData("text/plain", payload);
                domEv.dataTransfer.setData("text", payload);
                domEv.dataTransfer.effectAllowed = "copyMove";
              }
            } catch (_) {
              /* ignore */
            }
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
              const drag = hit.getAttribute("data-drag");
              if (drag != null) args.drag = drag;
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
