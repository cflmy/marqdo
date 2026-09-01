# 路线图：浏览器 Marqdo（WASM）

| | |
|---|---|
| 状态 | **Completed**（路线 C 首版功能完备） |
| 日期 | 2026-09-01 |
| ADR | [0002](../adr/0002-browser-marqdo-wasm.md) · [0003](../adr/0003-browser-async-effects.md) |
| 设计 | [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) |

| 阶段 | 内容 | 状态 |
|------|------|------|
| **C0** | feature 门控；`wasm32` check | **done** |
| **C1** | `run_source` + `mq_run` | **done** |
| **C2** | loader + `marqdo wasm build`（含 bridge 拷贝） | **done** |
| **C3** | session wire + `set_text` | **done** |
| **C4** | `fetch` / `after` 效应 | **done** |
| **C5** | `release-wasm` 体积；冒烟；站点挂载文档 / L1 | **done** |

## 交付物

| 产物 | 路径 |
|------|------|
| ABI crate | `crates/marqdo-wasm` |
| 官方 bridge | `crates/marqdo-wasm/js/marqdo-bridge.js` |
| CLI | `marqdo wasm build -o DIR` |
| 示例 | `examples/browser-hello/` |
| 冒烟 | `tests/wasm/smoke.mjs` · `tests/wasm_smoke.rs` |
| L1 | `web.client_embed` / `网页.客户端挂载` |

## 明确非目标（本版不做）

- 浏览器会话 bytecode（缺 entry-env invoke；一发 `run_source` 已支持 bytecode）
- 把 axum / SQLite 服务端搬进 WASM
- 业务手写 JS 框架

## 进度

| 日期 | 记录 |
|------|------|
| 2026-09-01 | C0–C5 完结并 push；收口 bridge 规范路径 + 冒烟 + 挂载 L1 |
