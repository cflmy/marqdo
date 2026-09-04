# browser-app（路线 E）

作者**零业务 JS**：官方 bridge 自启；交互在 `app.mq.md`。

覆盖：`set_html` 列表、`storage` 草稿、`navigate`+`popstate`、`interval`、事件 `委托`。

```bash
# 仓库根
cargo build -p marqdo --release
./target/release/marqdo wasm build -o examples/browser-app
# 若 build 覆盖 bridge，可再：cp crates/marqdo-wasm/js/marqdo-bridge.js examples/browser-app/

cd examples/browser-app
python3 -m http.server 8766
# http://127.0.0.1:8766/
```

原则：禁止作者手写 JS；官方桥内机制（列表装配、路由、存储等）允许。见 [browser-wasm-e.md](../../doc/roadmap/browser-wasm-e.md)。
