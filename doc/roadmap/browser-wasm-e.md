# 路线图：浏览器 Marqdo 路线 E（前端能力补齐）

| | |
|---|---|
| 状态 | **Planned** |
| 日期 | 2026-09-04 |
| 前置 | [browser-wasm-d.md](browser-wasm-d.md)（D0–D4 **Completed**） |
| 设计 | [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) §16–§17 |

## 结论（能力审视）

路线 C+D 已能覆盖**中小型交互站**的常见路径：挂载会话、wire 事件、文本/表单回写、`fetch`/`after`、作者零业务 JS。

**尚不能**宣称「替代大部分 / 全部前端 JS」。缺口集中在：列表级 UI 装配、客户端路由与历史、本地持久化、实时通道、媒体/画布、以及更细的 DOM 读写与委托事件。

| 类别 | 今日 | 典型 JS 任务 | 路线 E |
|------|------|--------------|--------|
| 启动 / 会话 | `mount` / `client_embed` | 手写 loader | 已够用 |
| 点击 / 表单 | wire + `value`/`fields` | addEventListener | 扩委托 / 更多事件 |
| DOM 补丁 | `set_text`/`value`/`attr`/`class` | jQuery 式改节点 | `set_style`、`focus`、`scroll`、列表 diff |
| 网络 | 单次 `fetch` 效应 | axios / 多并行 | 并行 `fetch_all`、上传进度 |
| 定时 | `after` | setInterval | `interval` / `cancel` |
| 路由 | 无 | history / SPA | `navigate` / `popstate` wire |
| 存储 | 无 | localStorage / IDB | `storage` 效应 |
| 实时 | 服务端有 `ws`；浏览器无 | WebSocket | 浏览器 `ws` 效应 |
| 组件列表 | SSR 为主 | 客户端 render list | `set_html(#trusted*)` + 模板装配 L1 |
| 媒体 / 画布 | 无 | Audio / Canvas / WebGL | 后置（非 E 核心） |
| 原生插件 | 浏览器无 `.so` | wasm 插件 | 明确非目标 |

原则不变：官方 bridge 仍是宿主胶；业务只写 `.mq.md`；不引入通用 VDOM / 任意 `eval(JS)`。

## 阶段

| 阶段 | 内容 | 状态 |
|------|------|------|
| **E0** | 本文 + 设计 §17 能力矩阵 | **done**（文档） |
| **E1** | DOM 补齐：`set_style`、`focus`/`blur`、`scroll_into`；wire `keydown`/`keyup`；事件委托（可选 `委托` 列） | planned |
| **E2** | 列表 / 片段：`replace_children` 或受信 `#trusted*` 模板 L1（`web.client_list`）；避免手写巨段 HTML | planned |
| **E3** | 客户端路由：`navigate` / `replace` 效应 + `popstate` wire；与 `ext/web` SSR 深链共存约定 | planned |
| **E4** | 持久化：`storage.get/set/remove`（local/session）；可选简单 cookie 读写白名单 | planned |
| **E5** | 实时：浏览器 `ws` 效应（open / send / on_message → `mq_call`）；对齐服务端 `ws` 语义 | planned |
| **E6** | 异步增强：`fetch` 并行表、`interval`+`clear`、上传 `FormData` 字段表 | planned |

## 验收（E 收口时）

- 无作者业务 JS 的情况下，可做：多页客户端导航、表单草稿存 localStorage、列表增删改、同源 WebSocket 回显。
- 冒烟 / 示例覆盖 E1–E5 各至少一例。
- 体积与安全：效应仍白名单；`set_html` 维持 `#trusted*` 前缀。

## 明确非目标（E）

- React/Vue 式 VDOM 与虚拟列表框架
- 浏览器内 SQLite / 完整 ORM
- Service Worker / PWA 全套（可另开路线）
- 用 Marqdo 重写 bridge 自身
