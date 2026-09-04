# 路线图：浏览器 Marqdo 路线 E（前端能力补齐）

| | |
|---|---|
| 状态 | **Active** |
| 日期 | 2026-09-04 |
| 前置 | [browser-wasm-d.md](browser-wasm-d.md)（D0–D4 **Completed**） |
| 设计 | [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) §16–§17 |

## 原则（已修正）

**禁止的只有：作者手写业务 JS。**  
官方 `marqdo-bridge.js` / 构建生成物可以包含任意宿主机制（含轻量列表装配、路由、存储、WebSocket、内部模板等）。不要求「禁止 VDOM / 禁止 eval」——那些若放在官方桥内、对作者不可见，则允许。

## 结论

C+D 覆盖中小型交互；E 波补齐后，作者可仅用 `.mq.md` 完成大部分前端任务（列表 UI、SPA 导航、草稿存储、WS、并行请求）。

## 阶段

| 阶段 | 内容 | 状态 |
|------|------|------|
| **E0** | 原则修正 + 能力矩阵 | **done** |
| **E1** | `set_style` / `focus` / `blur` / `scroll_into`；键盘；wire `委托` | **done** |
| **E2** | `set_html`（任意选择器）/ `replace_children` / `render_list`；L1 `list_html` | **done** |
| **E3** | `navigate` / `replace`；`window`+`popstate` wire | **done** |
| **E4** | `storage` get/set/remove（local/session） | **done** |
| **E5** | `ws` open/send/close + message 回调 | **done** |
| **E6** | `fetch_all` / `interval`+`clear_interval` / fetch `fields`→FormData | **done** |

## 验收

- 示例 `examples/browser-app/`：无作者 `.js`，覆盖列表、storage、navigate、interval（及可选 ws 说明）。
- 冒烟覆盖 normalize + 关键效应解析（Node）；DOM/WS 以桥内实现为准。

## 非目标

- 作者仓库中的业务 `.js` / `.ts`
- 浏览器内 SQLite / 完整 ORM
- 强制作者学习官方桥源码（桥对作者不透明即可）
