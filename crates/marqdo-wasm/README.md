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
marqdo wasm build -o examples/browser-hello
# or:
cargo build -p marqdo-wasm --target wasm32-unknown-unknown --profile release-wasm
```

Prints size in KiB; runs `wasm-opt -Oz` when Binaryen is on `PATH`.

See [doc/design/browser-marqdo-wasm.md](../../doc/design/browser-marqdo-wasm.md).
