# 路线图：浏览器 Marqdo（WASM）

| | |
|---|---|
| 状态 | **进行中**（设计已锁定） |
| 日期 | 2026-09-01 |
| ADR | [0002-browser-marqdo-wasm.md](../adr/0002-browser-marqdo-wasm.md) |
| 设计 | [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) |
| 相关 | [interpreter.md](interpreter.md) · [ext-web.md](../design/ext-web.md) · [next-phase.md](next-phase.md) |

实现阶段编号 **C0–C5**（Client / WASM）。每阶段结束须：文档状态更新 + CHANGELOG + 可演示验收。

---

## 总览

| 阶段 | 内容 | 状态 |
|------|------|------|
| **C0** | 设计锁定；Cargo feature 骨架；`wasm32` **check** 通过（可空跑） | **docs done · impl next** |
| **C1** | `run_source` + wasm-bindgen 导出；hello 打印；无 DOM | pending |
| **C2** | 官方 loader + `examples/browser-hello`；可选 `marqdo wasm build`；head 挂载约定 | pending |
| **C3** | 交互装配（事件表 → 回调）；最小 DOM 读写宿主 | pending |
| **C4** | `fetch`/定时器异步模型 ADR + 首版桥 | pending |
| **C5** | bytecode 后端可选；体积优化；与 view 调试对齐 | pending |

---

## C0 — 可移植骨架

**目标：** native 默认行为不变；存在 `--features wasm-core --no-default-features` 使核心在 `wasm32-unknown-unknown` 上 `cargo check`。

**任务：**

1. ADR + 设计文（本文相关）已合入。  
2. `Cargo.toml` features：`cli`/`view`/`net-host`/`plugin-host`/`exec-host`/`fs-host`/`wasm-core`。  
3. `cfg` 隔离：`view`、`ext_cli`、`version_check`（ureq）、`host::{net,plugin,foreign,subtask}` 等。  
4. `input_feed` TTY/`libc` 仅非 wasm。  
5. CI 或本地脚本：`rustup target add wasm32-unknown-unknown && cargo check -p marqdo --target wasm32-unknown-unknown --no-default-features --features wasm-core`。

**验收：** 上述 check 退出 0；全量 `cargo test`（默认 features）仍绿。

---

## C1 — 内存执行闭环

**目标：** JS/wasm 调用 `run_source`，跑 `# main` + `> print`，拿回 stdout。

**任务：**

1. `pub fn run_source(source: &str, opts: &RunOptions) -> Result<RunCapture>`（及可选虚拟 imports）。  
2. crate `crates/marqdo-wasm`：`#[wasm_bindgen] pub fn run(source: &str) -> String`（JSON：`{ok,stdout,error}`）。  
3. 嵌入 `lib/` 只读导入可用。  
4. 拒绝 fs/exec/plugin/net 并返回明确错误字符串。  
5. 金样：至少 `hello` 结构子集在 native `run_source` 单测通过。

**验收：** wasm 实例化后 `run(hello_source)` 得到 `Hello World!`。

---

## C2 — 示例页与挂载

**目标：** 静态 HTML 打开即可跑 Marqdo；文档说明如何嵌入站点。

**任务：**

1. `examples/browser-hello/`：`index.html` + loader + 构建说明。  
2. （可选）`marqdo wasm build` 包装 `wasm-pack`。  
3. 与 W8 `page.head`：文档示例挂 `type=module` + wasm（实现可后置）。  
4. README / tutorial 短节。

**验收：** 按 README 本地 `python -m http.server`（或等价）打开示例见打印结果。

---

## C3 — 交互装配

**目标：** 无手写业务 JS 的点击 → Marqdo 函数 → 改页面。

**任务：**

1. 宿主：`dom_text_set` / `dom_text_get` / `add_listener`（经 bindgen）。  
2. L1 `ext/web` 或 `ext/browser`：`交互装配` / `wire_events`。  
3. 示例：按钮计数或问候。  
4. 设计补一节返回值指令格式（若 Map 驱 DOM）。

**验收：** 示例页点击按钮，DOM 文本变化；网络面板无第三方业务脚本（仅官方 loader）。

---

## C4 — 异步 I/O

**目标：** 受控 `fetch`（及 sleep）不阻塞语言谎言。

**任务：**

1. 短 ADR：续体 / 回调表 / Promise 桥选型。  
2. `浏览器.请求` 首版 + 金样/示例。  
3. 能力开关默认关，示例显式开。

**验收：** 示例请求公共 HTTP（或 mock）并把正文写回 DOM。

---

## C5 — 硬化

**目标：** 可选项生产化。

**任务：**

1. `--backend bytecode` 在 wasm 对齐。  
2. `wasm-opt`、code splitting 评估。  
3. view / 静态页调试：加载同一 wasm 跑片段（呼应 `next-phase.md`）。  
4. 体积与启动预算写入设计文。

**验收：** 文档中的预算表 + CI 双后端子集金样。

---

## 依赖关系

```
C0 ──► C1 ──► C2 ──► C3 ──► C4
                │              │
                └──── C5 ◄─────┘（可与 C4 部分并行）
```

服务端 `ext/web` W0–W8 **不阻塞** C0–C2；C3 与 head/assets 集成最顺。

---

## 进度记录

| 日期 | 记录 |
|------|------|
| 2026-09-01 | ADR 0002 + 设计锁定；路线图开篇；开始 C0/C1 实现 |
