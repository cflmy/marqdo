# Native SPA（Marqdo WASM 全页客户端）

| | |
|---|---|
| 状态 | **Accepted · MVP** |
| 日期 | 2026-09-17 |
| 相关 | [browser-marqdo-wasm.md](browser-marqdo-wasm.md) · route D/E |

## 约定

| 表 | 列 | 用途 |
|----|-----|------|
| 客户端路由 | `路径` / `面板` / `标题` | 文档化视图；运行时用 `path` + `set_attr(hidden)` 切换 |
| 客户端状态 | `browser.store_set` / `store_get` `scope=memory` | 页内状态（列表 JSON、当前 id） |

作者零业务 JS：`web.client_embed` + `static/client.mq.md` + `lib/browser`。

## Bridge

`storage` effect 支持 `scope=memory`（进程内 `Map`），与 `local` / `session` / `cookie` 并列。

## Demo

`examples/web-spa-app/` — 列表 → 详情 → 编辑，全客户端路由。
