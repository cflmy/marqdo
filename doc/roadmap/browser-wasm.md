# 路线图：浏览器 Marqdo（WASM）

| | |
|---|---|
| 状态 | **C0–C5 首轮完结**（会话 bytecode / view 复用仍可加深） |
| 日期 | 2026-09-01 |
| ADR | [0002](../adr/0002-browser-marqdo-wasm.md) · [0003](../adr/0003-browser-async-effects.md) |
| 设计 | [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) |

| 阶段 | 内容 | 状态 |
|------|------|------|
| **C0** | feature 门控；`wasm32` check | **done** |
| **C1** | `run_source` + `mq_run` | **done** |
| **C2** | loader + `marqdo wasm build` | **done** |
| **C3** | session wire + `set_text` | **done** |
| **C4** | `fetch` / `after` 效应 | **done** |
| **C5** | `release-wasm` 体积；`wasm-opt`；`run_source` bytecode 测 | **done**（会话仍 tree） |

## C5

```bash
marqdo wasm build -o examples/browser-hello   # prints KiB; wasm-opt if installed
```

- Profile：`release-wasm`（`opt-level=z`, LTO, strip）
- `BrowserSession` 明确拒绝 bytecode（待 VM entry-env）
- 一发 `run_source(..., backend=bytecode)` 单测绿

## 进度

| 日期 | 记录 |
|------|------|
| 2026-09-01 | C0–C4 |
| 2026-09-01 | C5 体积 profile + 文档 |
