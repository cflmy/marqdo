# Marqdo browser WASM (route C)

Raw `cdylib` exports — no `wasm-bindgen` CLI required.

| Export | Role |
|--------|------|
| `mq_alloc` / `mq_dealloc` | linear memory |
| `mq_run` | one-shot `# main` (C1) |
| `mq_boot` / `mq_call` | session + handlers (C3) |
| `mq_version` | version C string |

```bash
# from repo root (C5 size profile)
marqdo call skills/marqdo-dev/index.mq.md build_browser --effects plugin,process.exec,fs.write --fs-root . --json
# or:
cargo build -p marqdo-wasm --target wasm32-unknown-unknown --profile release-wasm
```

Prints size in KiB; runs `wasm-opt -Oz` when Binaryen is on `PATH`.

See [doc/design/browser-marqdo-wasm.md](../../doc/design/browser-marqdo-wasm.md).

Use the maintained [browser-app](../../examples/browser-app/index.mq.md) for a runnable native Document and browser program. The ABI counter fixture lives in tests/wasm/counter.mq.md, with no independent HTML demo host. The former browser-hello and browser-media directories were retired in 2.7.0; their host capabilities remain documented in lib/browser.mq.md.
