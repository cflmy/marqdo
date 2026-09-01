# browser-hello（WASM C1）

在浏览器里跑 Marqdo `print`（路线 C）。

```bash
# 仓库根目录
cargo build -p marqdo-wasm --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/marqdo_wasm.wasm examples/browser-hello/

cd examples/browser-hello
python3 -m http.server 8765
# 打开 http://127.0.0.1:8765/
```

- `loader.js` 仅为官方宿主桥（分配内存 / 调 `mq_run`），不是业务语言。
- 设计：[browser-marqdo-wasm.md](../../doc/design/browser-marqdo-wasm.md)
