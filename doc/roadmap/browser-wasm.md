# 路线图：浏览器 Marqdo（WASM）

| | |
|---|---|
| 状态 | **进行中**（C0–C3 已落地） |
| 日期 | 2026-09-01 |
| ADR | [0002-browser-marqdo-wasm.md](../adr/0002-browser-marqdo-wasm.md) |
| 设计 | [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) |
| 相关 | [interpreter.md](interpreter.md) · [ext-web.md](../design/ext-web.md) · [next-phase.md](next-phase.md) |

实现阶段编号 **C0–C5**（Client / WASM）。每阶段结束须：文档状态更新 + CHANGELOG + 可演示验收。

---

## 总览

| 阶段 | 内容 | 状态 |
|------|------|------|
| **C0** | Cargo feature 骨架；`wasm32` check | **done** |
| **C1** | `run_source` + `mq_run` | **done** |
| **C2** | loader + `examples/browser-hello` + `marqdo wasm build` | **done** |
| **C3** | 会话 `mq_boot`/`mq_call`；wire 表；`set_text` DOM 回写 | **done** |
| **C4** | `fetch`/定时器异步模型 | pending |
| **C5** | bytecode / 体积 / view 调试 | pending |

---

## C2 — 示例页与挂载

```bash
marqdo wasm build -o examples/browser-hello
cd examples/browser-hello && python3 -m http.server 8765
```

---

## C3 — 交互装配

- `BrowserSession` + `Interpreter::invoke_in_entry`
- ABI：`mq_boot` / `mq_call`
- 示例：`interact.html` + `counter.mq.md`
- Bridge：`wireEvents` / `applyDomPatch`

**验收：** 打开 interact 页，点「加一」计数递增；源码仅为 `.mq.md` + 官方 bridge。

---

## C4 / C5

见设计文 §7 / §12。

---

## 进度记录

| 日期 | 记录 |
|------|------|
| 2026-09-01 | ADR 0002；C0–C1 |
| 2026-09-01 | C2 `wasm build`；C3 session + interact 示例 |
