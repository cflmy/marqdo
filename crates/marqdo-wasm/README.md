# Marqdo browser WASM (route C)

Raw `cdylib` exports — no `wasm-bindgen` CLI required.

```bash
# from repo root
cargo build -p marqdo-wasm --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/marqdo_wasm.wasm examples/browser-hello/
```

Open `examples/browser-hello/index.html` via a local static server (ES modules / fetch need http).

See [doc/design/browser-marqdo-wasm.md](../../doc/design/browser-marqdo-wasm.md).
