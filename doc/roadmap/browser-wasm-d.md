# 路线图：浏览器 Marqdo 路线 D（作者零 JS）

| | |
|---|---|
| 状态 | **Completed**（D0–D4 已落地；续作见 [browser-wasm-e.md](browser-wasm-e.md)） |
| 日期 | 2026-09-03 |
| 前置 | [browser-wasm.md](browser-wasm.md)（C0–C5 **Completed**） |
| ADR | [0002](../adr/0002-browser-marqdo-wasm.md) · [0003](../adr/0003-browser-async-effects.md) |
| 设计 | [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) §16 |

## 一句话

作者侧**零业务 JS**：页面逻辑与交互只写 `.mq.md`；官方 `marqdo-bridge.js` 仍作不透明宿主胶（加载 WASM / DOM / fetch），由 `mount` + `client_embed` 自动启动。

## 阶段

| 阶段 | 内容 | 状态 |
|------|------|------|
| **D0** | 路线图 + 设计增补 | **done** |
| **D1** | bridge `mount` + data-mq / `#marqdo-boot` 自启；hello 去掉 loader/interact/fetch.js | **done** |
| **D2** | `client_embed` / `客户端挂载`：`wasm` / `source` / `boot` | **done** |
| **D3** | DOM/表单效应 + wire 传 `value`；`dom_patch` L1 | **done** |
| **D4** | `examples/web-client-site` + skill / README | **done** |

## 非目标

- 消灭官方 bridge 文件本身（宿主胶必须存在）
- 任意 `eval(JS)` / React 式 VDOM
- 把 axum / SQLite 搬进浏览器（仍属 C 非目标）

## 验收

- 示例无作者业务 `.js`（允许构建拷贝的 `marqdo-bridge.js`）
- `ext/web` 一行挂载即可 boot + wire
- 点击、表单 input、`fetch` 效应可演示
