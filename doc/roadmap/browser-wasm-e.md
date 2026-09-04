# 路线图：浏览器 Marqdo 路线 E（前端能力补齐）

| | |
|---|---|
| 状态 | **Completed** |
| 日期 | 2026-09-04 |
| 前置 | [browser-wasm-d.md](browser-wasm-d.md)（D0–D4 **Completed**） |
| 续作 | [browser-wasm-f.md](browser-wasm-f.md)（媒体 / 文件 / Canvas 等） |
| 设计 | [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) §16–§17 |

## 原则（已修正）

**禁止的只有：作者手写业务 JS。**  
官方 `marqdo-bridge.js` / 构建生成物可以包含任意宿主机制（含轻量列表装配、路由、存储、WebSocket、内部模板等）。

## 阶段

| 阶段 | 内容 | 状态 |
|------|------|------|
| **E0** | 原则修正 + 能力矩阵 | **done** |
| **E1** | `set_style` / `focus` / `blur` / `scroll_into`；键盘；wire `委托`；多节点 `querySelectorAll` | **done** |
| **E2** | `set_html` / `append_html` / `replace_children` / `render_list` / `remove`；L1 `list_html` | **done** |
| **E3** | `navigate` / `replace`；`window`+`popstate` | **done** |
| **E4** | `storage` local/session/**cookie** | **done** |
| **E5** | `ws` open/send/close + callbacks | **done** |
| **E6** | `fetch_all`（数组或 map）/ `interval` / FormData / `clipboard` / `download` | **done** |
| **E7** | `add_class` / `remove_class`；示例列表真删除 + net 面板 | **done** |

## 验收

- [examples/browser-app/](../../examples/browser-app/)：列表 CRUD、storage、三页路由、interval、fetch_all、ws、clipboard、download
- 冒烟：`tests/wasm/smoke.mjs`
