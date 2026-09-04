# browser-hello（WASM C1–C4 / 路线 D）

浏览器内跑 Marqdo。**无作者业务 JS**——只保留官方 `marqdo-bridge.js`（`marqdo wasm build` 拷贝）。

```bash
# 仓库根目录
cargo build -p marqdo --release
./target/release/marqdo wasm build -o examples/browser-hello

cd examples/browser-hello
python3 -m http.server 8765
```

| 页 | 说明 |
|----|------|
| [/](http://127.0.0.1:8765/) | playground：`data-mq-playground` + textarea → `mq_run` |
| [/interact.html](http://127.0.0.1:8765/interact.html) | `counter.mq.md` wire → 点击改 DOM |
| [/fetch.html](http://127.0.0.1:8765/fetch.html) | `fetch` / `after` 效应 |
| [/form.html](http://127.0.0.1:8765/form.html) | `input` + `value` / `set_class`（D3） |

自启方式：脚本上的 `data-mq-source-url` / `data-mq-playground`，或 `#marqdo-boot` JSON。详见 [browser-wasm-d.md](../../doc/roadmap/browser-wasm-d.md)。
