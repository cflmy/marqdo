# browser-hello（WASM C1–C3）

浏览器内跑 Marqdo（路线 C）。

```bash
# 仓库根目录
cargo build -p marqdo --release
./target/release/marqdo wasm build -o examples/browser-hello

cd examples/browser-hello
python3 -m http.server 8765
```

| 页 | 说明 |
|----|------|
| [/](http://127.0.0.1:8765/) | C1：编辑源码 → `mq_run` 打印 |
| [/interact.html](http://127.0.0.1:8765/interact.html) | C3：`counter.mq.md` 交互表 → 点击改 DOM |
| [/fetch.html](http://127.0.0.1:8765/fetch.html) | C4：`fetch` / `after` 效应续体 |

- `marqdo-bridge.js`：官方宿主桥（`boot` / `call` / `wireEvents` / `applyEffects`）
- 业务源码只有 `.mq.md`；异步见 [ADR 0003](../../doc/adr/0003-browser-async-effects.md)
