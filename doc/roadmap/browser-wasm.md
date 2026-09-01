# 路线图：浏览器 Marqdo（WASM）

| | |
|---|---|
| 状态 | **进行中**（C0–C4 已落地） |
| 日期 | 2026-09-01 |
| ADR | [0002](../adr/0002-browser-marqdo-wasm.md) · [0003 异步效应](../adr/0003-browser-async-effects.md) |
| 设计 | [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) |

| 阶段 | 内容 | 状态 |
|------|------|------|
| **C0** | feature 门控；`wasm32` check | **done** |
| **C1** | `run_source` + `mq_run` | **done** |
| **C2** | loader + `marqdo wasm build` | **done** |
| **C3** | `mq_boot`/`mq_call`；wire；`set_text` | **done** |
| **C4** | `fetch` / `after` 效应 + 续体（ADR 0003） | **done** |
| **C5** | bytecode / 体积 / view 调试 | pending |

## C4 — 异步效应

- Bridge：`applyEffects` 识别返回值中的 `fetch` / `after`
- 完成后 `mq_call(then, payload)`，可再链式效应
- 示例：`fetch.html` + `fetch.mq.md`

**验收：** 点「请求 UUID」后 `#status` 出现 httpbin JSON；点「延迟 pong」约 600ms 后显示 `pong`。

## C5

见设计硬化项。

## 进度

| 日期 | 记录 |
|------|------|
| 2026-09-01 | C0–C3 |
| 2026-09-01 | C4 ADR 0003 + fetch/after demo |
