# ADR 0003：浏览器异步 I/O（fetch / 定时）— 效应表 + 续体回调

| | |
|---|---|
| 状态 | **Accepted** |
| 日期 | 2026-09-01 |
| 决策者 | chaungming + 协作 |
| 相关 | [0002-browser-marqdo-wasm.md](0002-browser-marqdo-wasm.md) · [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) · [roadmap/browser-wasm.md](../roadmap/browser-wasm.md) |

---

## 背景

Marqdo 调用面今日以**同步**为主。浏览器 `fetch` / `setTimeout` 是异步的；若在 WASM 里阻塞等待，会卡死 UI，且 `wasm32-unknown-unknown` 无真实阻塞 HTTP。

C3 已有：`mq_call` 返回 Map（如 `set_text`）由官方 bridge 执行。C4 在同一通道上扩展异步效应。

---

## 决策

1. **不**在语言内核引入 async/await 关键字或假同步 `fetch`。  
2. 处理器返回值可含**效应字段**（与 `set_text` 并列），由 **JS bridge** 执行：  
   - `fetch`: `{ url, method?, then, headers?, body? }`  
   - `after`: `{ ms, then }`（定时续体）  
3. 异步完成后 bridge 调用 `mq_call(then, payload)`：  
   - fetch 成功/失败：`{ ok, status, body, error? }`  
   - after：`{ ok: true }`  
4. 能力默认关网络语义仍由**返回效应**显式开启（站点作者主动返回 `fetch`）；CORS / 鉴权仍是浏览器与服务端责任。  
5. 服务端 `lib/net` / `plugins/web` **不变**；本 ADR 仅浏览器 WASM 宿主。

---

## 后果

- 作者用表格/Map 声明「请求谁、完成后调谁」，无手写业务 JS。  
- Bridge 必须串行处理效应（先 `set_text`，再调度 `fetch`/`after`）。  
- 深层 Promise 链靠多次 `then` 回调；不在 C4 做通用 async 运行时。

---

## 否决

| 方案 | 原因 |
|------|------|
| WASM 内同步阻塞 `fetch` | 卡死主线程；平台不支持 |
| 编译期把 Marqdo 译成 async JS | 回到路线 B |
| 在核心加 `await` 关键字 | 违反 Markup-as-Syntax / 关键字纪律 |
