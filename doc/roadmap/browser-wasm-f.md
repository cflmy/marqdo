# 路线图：浏览器 Marqdo 路线 F（进阶宿主能力）

| | |
|---|---|
| 状态 | **Planned** |
| 日期 | 2026-09-04 |
| 前置 | [browser-wasm-e.md](browser-wasm-e.md)（E **Completed**） |

## 原则

同 E：禁止作者手写业务 JS；官方 bridge 可扩展。

## 阶段（未做，相对「全部前端」仍缺）

| 阶段 | 内容 |
|------|------|
| **F1** | `input[type=file]` → 读文件 / DataURL 回 `mq_call` |
| **F2** | Canvas 2D 指令表（fill/stroke/path）由桥执行 |
| **F3** | Audio / 简单 MediaStream 开关 |
| **F4** | IntersectionObserver / ResizeObserver → wire |
| **F5** | 拖放 `drag`/`drop` wire + dataTransfer 文本 |
| **F6** | Service Worker / 离线缓存（可选大工程） |

E 已覆盖大部分表单 / SPA / 存储 / 网络前端任务；F 面向媒体与更细浏览器 API。
