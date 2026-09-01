# 路线图：浏览器 Marqdo（WASM）

| | |
|---|---|
| 状态 | **进行中**（C0–C1 已落地；C2 示例页已有） |
| 日期 | 2026-09-01 |
| ADR | [0002-browser-marqdo-wasm.md](../adr/0002-browser-marqdo-wasm.md) |
| 设计 | [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) |
| 相关 | [interpreter.md](interpreter.md) · [ext-web.md](../design/ext-web.md) · [next-phase.md](next-phase.md) |

实现阶段编号 **C0–C5**（Client / WASM）。每阶段结束须：文档状态更新 + CHANGELOG + 可演示验收。

---

## 总览

| 阶段 | 内容 | 状态 |
|------|------|------|
| **C0** | 设计锁定；Cargo feature 骨架；`wasm32` **check** 通过 | **done** |
| **C1** | `run_source` + `crates/marqdo-wasm`（`mq_run` ABI）；hello 打印 | **done** |
| **C2** | 官方 loader + `examples/browser-hello`；可选 CLI 糖；head 挂载约定 | **partial**（示例+loader 已有） |
| **C3** | 交互装配（事件表 → 回调）；最小 DOM 读写宿主 | pending |
| **C4** | `fetch`/定时器异步模型 ADR + 首版桥 | pending |
| **C5** | bytecode 后端可选；体积优化；与 view 调试对齐 | pending |

---

## C0 — 可移植骨架

**目标：** native 默认行为不变；存在 `--features wasm-core --no-default-features` 使核心在 `wasm32-unknown-unknown` 上 `cargo check`。

**已落地：**

- Features：`native`（默认）、`wasm-core`、`view`/`net-host`/`plugin-host`/`exec-host`/`fs-host`/`tty`/`cli`
- 可选依赖：`ureq`、`libloading`、`tiny_http`、`libc`、`clap`
- `run_source` / `load_module_from_source`（embedded `lib/`）

**验收：**

```bash
cargo check -p marqdo --target wasm32-unknown-unknown --no-default-features --features wasm-core
cargo test --test gold structure_hello
```

---

## C1 — 内存执行闭环

**目标：** 浏览器/测试调用 `run_source`，跑 `# main` + `> print`，拿回 stdout。

**已落地：**

- `pub fn run_source`（tree + bytecode）
- crate `crates/marqdo-wasm`：`mq_alloc` / `mq_dealloc` / `mq_run` / `mq_version`（长度前缀 JSON，无需 wasm-bindgen CLI）
- 浏览器默认关 fs 写 / exec / net

**验收：**

```bash
cargo test -p marqdo-wasm
cargo test --lib run_source_hello
cargo build -p marqdo-wasm --target wasm32-unknown-unknown --release
```

---

## C2 — 示例页与挂载

**目标：** 静态 HTML 打开即可跑 Marqdo。

**已有：** `examples/browser-hello/`（`index.html` + `loader.js` + README）。  
**仍缺：** `marqdo wasm build` CLI 糖；`page.head` 文档示例挂载。

---

## C3 — 交互装配

**目标：** 无手写业务 JS 的点击 → Marqdo 函数 → 改页面。

（见设计 §6）

---

## C4 — 异步 I/O

**目标：** 受控 `fetch`（及 sleep）不阻塞语言谎言。

---

## C5 — 硬化

**目标：** bytecode、体积、`view` 调试对齐。

---

## 依赖关系

```
C0 ──► C1 ──► C2 ──► C3 ──► C4
                │              │
                └──── C5 ◄─────┘
```

---

## 进度记录

| 日期 | 记录 |
|------|------|
| 2026-09-01 | ADR 0002 + 设计锁定；C0 feature-gate；C1 `run_source` + `marqdo-wasm` + browser-hello |
