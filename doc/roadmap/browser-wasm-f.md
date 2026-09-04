# 路线图：浏览器 Marqdo 路线 F（进阶宿主能力）

| | |
|---|---|
| 状态 | **Active**（F1–F5 桥 + 示例；F6 可选未做） |
| 日期 | 2026-09-04 |
| 前置 | [browser-wasm-e.md](browser-wasm-e.md)（E **Completed**） |
| 写法 | 客户端 `.mq.md`：**GFM 表格 + `lib/browser`**，禁止 `json.set` 链（[marqdo-dev](../../.cursor/skills/marqdo-dev/SKILL.md)） |

## 原则

同 E：禁止作者手写业务 JS；官方 bridge 可扩展。作者数据与效应用**可读表格/助手**表达。

## 阶段

| 阶段 | 内容 | 状态 |
|------|------|------|
| **F1** | `read_file` → text / DataURL → `mq_call` | done |
| **F2** | Canvas 2D 指令表（`canvas.commands`） | done |
| **F3** | Audio play / pause / stop；`src=beep` Web Audio | done |
| **F4** | `observe` / `unobserve`（Intersection / Resize） | done |
| **F5** | drag/drop wire + `data-drag` / `drop_text` | done |
| **F6** | Service Worker / 离线缓存 | optional / skipped |

示例：[examples/browser-media/](../../examples/browser-media/) · 助手：`lib/browser.mq.md`。
