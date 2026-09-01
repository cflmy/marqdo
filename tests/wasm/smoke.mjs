#!/usr/bin/env node
/**
 * WASM ABI smoke (C1/C3): boot counter.mq.md, call bump twice.
 * Usage: node tests/wasm/smoke.mjs [path/to/marqdo_wasm.wasm]
 */
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "../..");

const wasmPath =
  process.argv[2] ||
  path.join(root, "examples/browser-hello/marqdo_wasm.wasm");
const srcPath = path.join(root, "examples/browser-hello/counter.mq.md");

if (!fs.existsSync(wasmPath)) {
  console.error("missing wasm:", wasmPath);
  console.error("build: marqdo wasm build -o examples/browser-hello");
  process.exit(2);
}

const buf = fs.readFileSync(wasmPath);
const { instance } = await WebAssembly.instantiate(buf, {});
const e = instance.exports;
const enc = new TextEncoder();
const dec = new TextDecoder();

function pack(ptr) {
  const len = new DataView(e.memory.buffer).getUint32(ptr, true);
  const j = JSON.parse(dec.decode(new Uint8Array(e.memory.buffer, ptr + 4, len)));
  e.mq_dealloc(ptr, 4 + len);
  return j;
}

function write(s) {
  const b = enc.encode(s);
  const p = e.mq_alloc(b.length || 1);
  new Uint8Array(e.memory.buffer, p, b.length).set(b);
  return { p, n: b.length };
}

function boot(source) {
  const { p, n } = write(source);
  const o = e.mq_boot(p, n);
  e.mq_dealloc(p, n || 1);
  return pack(o);
}

function call(name, args) {
  const a = write(name);
  const b = write(JSON.stringify(args || {}));
  const o = e.mq_call(a.p, a.n, b.p, b.n);
  e.mq_dealloc(a.p, a.n || 1);
  e.mq_dealloc(b.p, b.n || 1);
  return pack(o);
}

const source = fs.readFileSync(srcPath, "utf8");
const b = boot(source);
if (!b.ok) {
  console.error("boot failed", b.error);
  process.exit(1);
}
if (!Array.isArray(b.value) || b.value.length < 1) {
  console.error("expected wire list", b.value);
  process.exit(1);
}
const c1 = call("bump", {});
const c2 = call("bump", {});
if (!c1.ok || !c2.ok) {
  console.error("call failed", c1, c2);
  process.exit(1);
}
if (c2.value?.set_text?.["#count"] !== "2") {
  console.error("expected count 2, got", c2.value);
  process.exit(1);
}
console.log("wasm smoke ok");
