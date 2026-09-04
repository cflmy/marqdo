# browser-app（路线 E）

作者**零业务 JS**：官方 bridge 自启；逻辑在 `app.mq.md`。

| 能力 | 演示 |
|------|------|
| 列表 CRUD | 添加 / 委托删除（`bag` + `set_html`） |
| storage | 草稿 localStorage |
| navigate | `/` `/about` `/net` + `popstate` |
| interval | 计时起停 |
| fetch_all | 并行两个 uuid |
| ws | `wss://echo.websocket.events`（依赖外网） |
| clipboard / download | 复制/下载列表文本 |

```bash
cargo build -p marqdo --release
./target/release/marqdo wasm build -o examples/browser-app
cd examples/browser-app && python3 -m http.server 8766
```

见 [browser-wasm-e.md](../../doc/roadmap/browser-wasm-e.md)。
