# ADR 0002：浏览器内 Marqdo = WASM 解释器（路线 C）

| | |
|---|---|
| 状态 | **Accepted** |
| 日期 | 2026-09-01 |
| 决策者 | chaungming + 协作 |
| 相关 | [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) · [roadmap/browser-wasm.md](../roadmap/browser-wasm.md) · [interpreter.md](../roadmap/interpreter.md) · [0001](0001-implementation-language.md) |

---

## 背景

`ext/web` 已把站点装配成「源码以 `.mq.md` 为主、服务端 Rust 解释执行」。作者面仍缺**浏览器内**的同一语言：事件、局部状态、客户端 fetch/WS 回调今天要么手写 JS，要么完全不做。

候选路线：

| 代号 | 做法 |
|------|------|
| **A** | 服务端驱动 + 极薄固定增强（类 HTMX） |
| **B** | 把 Marqdo **编译/翻译成 JavaScript** |
| **C** | 把 **Rust 解释器/字节码 VM 编成 WASM**，浏览器跑同一套语义 |

产品叙事目标：**网络项目源码树可以只有 `.mq.md`**（JS/WASM 为生成或发行物，不是第二作者语言）。

---

## 决策

1. **选定路线 C**：浏览器内运行 **与主机同构的 Marqdo 语义**（parse → tree 和/或 bytecode VM），制品为 `marqdo.wasm` + 极薄 JS 宿主桥。  
2. **不选定 B 作为主路径**：翻译到 JS 易语义漂移，长期要维护两套后端；B 至多作实验，不替代 C。  
3. **A 不对立**：内容站 / SSR / 表单 POST 仍以服务端 `ext/web` 为主；C 负责「必须在客户端执行的 Marqdo」。  
4. **同一内核、两套宿主**：`server`（今日 CLI + axum 插件）与 `browser`（DOM / `fetch` / 定时器 / storage）；**不是**把 axum、`plugins/web`、SQLite 服务端塞进浏览器。  
5. **另开设计与路线图**（见相关链接）；实现按 **C0→C5** 切片，金样对齐主机行为。  
6. **与「编译 mq.md → WASM 机器码」区分**：本 ADR 是 **解释器/VM 上 WASM**；把 `.mq.md` 直接编译成独立 wasm 模块仍属 `interpreter.md` Phase III 可选项，**另案**，不替代本决策。

---

## 后果

- 核心库必须可 **feature-gate** 掉 `libloading` / `ureq` / `tiny_http` / TTY `libc` 等，以便 `wasm32-unknown-unknown` 链接。  
- 新增公共 API：`run_source`（内存源码 + 可选导入图），供 WASM 与测试共用。  
- 浏览器侧宿主 API 以表驱动「交互装配」暴露（对称于 `样式装配`）；`ext/**` 仍不直接调 `host_*`。  
- 发行物可包含 `.wasm` / 生成的 loader；**作者仓库可不提交手写业务 JS**。  
- view 断点 / 静态文档站调试可复用同一 WASM 解释器（与 `next-phase.md` 所述一致）。

---

## 否决 / 非目标（本 ADR）

| 方案 / 目标 | 原因 |
|-------------|------|
| 以 B（编译到 JS）为主交付 | 语义双轨；与「一种语言」冲突 |
| 浏览器内完整复刻 `plugins/web` 服务器 | 协议与权限模型不同；服务端仍 axum |
| 浏览器 `dlopen` 原生插件 | WASM 无 POSIX 动态库模型 |
| v1 消灭一切 JS 字节 | 胶水与发行 loader 允许存在；禁止的是**业务手写 JS** |
| 用 Daphne/ASGI 托管 Marqdo | 已否决，见 [web-asgi-servers-and-marqdo.md](../design/web-asgi-servers-and-marqdo.md) |
