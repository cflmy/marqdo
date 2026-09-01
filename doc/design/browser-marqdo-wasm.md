# 浏览器 Marqdo（WASM 解释器）设计

| | |
|---|---|
| 状态 | **Accepted（设计锁定）** · 实现见路线图 C0–C5 |
| 日期 | 2026-09-01 |
| ADR | [0002-browser-marqdo-wasm.md](../adr/0002-browser-marqdo-wasm.md) |
| 路线图 | [roadmap/browser-wasm.md](../roadmap/browser-wasm.md) |
| 相关 | [ext-web.md](ext-web.md) · [bytecode.md](bytecode.md) · [web-assets-and-images.md](web-assets-and-images.md) · [table-cell-expressions.md](table-cell-expressions.md) |

---

## 0. 一句话

**源码只有 `.mq.md`；浏览器跑与 CLI 同构的 Marqdo VM（WASM）；JS 只做宿主桥与加载器，不做业务语言。**

---

## 1. 目标与非目标

### 1.1 目标

| # | 目标 |
|---|------|
| G1 | 浏览器内执行合法 `.mq.md`（至少 `# main` + 打印/绑定/分支/循环/表/调用） |
| G2 | 与主机 **tree**（及后续 **bytecode**）金样在共享子集上 stdout/返回值一致 |
| G3 | 作者用 GFM 表声明交互（选择器 / 事件 / 调用），对称于 `样式装配` |
| G4 | 仓库源码树可不含业务 `.js`；允许提交或由工具生成 `marqdo.wasm` + 官方 loader |
| G5 | `import lib/…` 在浏览器可用（嵌入式 `lib/` 或打包进 wasm） |

### 1.2 非目标（本波 / C0–C3）

| # | 非目标 |
|---|--------|
| N1 | 在浏览器复刻 axum、`web_listen`、SQLite 服务端、admin UI |
| N2 | `plugin.load` 加载原生 `.so` / 浏览器内 dlopen |
| N3 | 把整站 `plugins/web` ABI 原样搬进 WASM |
| N4 | 把 `.mq.md` 编译成独立机器码 WASM（异于「解释器上 WASM」） |
| N5 | 强制内容站必须加载 WASM（纯 SSR 站可继续零客户端逻辑） |

---

## 2. 架构

```
┌─────────────────────────────────────────────────────────┐
│  Page (HTML shell from ext/web SSR or static)           │
│  ┌──────────────┐  ┌─────────────────────────────────┐  │
│  │ DOM / CSS    │  │ Official JS loader (thin)       │  │
│  └──────▲───────┘  │  - WebAssembly.instantiate      │  │
│         │ bridge   │  - marqdo_run_source / on_event │  │
│         │          └──────────────┬──────────────────┘  │
│         │                         │                     │
│         │              ┌──────────▼──────────┐          │
│         │              │  marqdo.wasm        │          │
│         │              │  parse → load →     │          │
│         └──────────────│  Interpreter | Vm   │          │
│           host_dom_*   │  HostContext(browser)│          │
│           host_fetch_* │  embedded lib/      │          │
│                        └─────────────────────┘          │
└─────────────────────────────────────────────────────────┘

Server (unchanged):
  marqdo run / plugins/web (axum) — SSR, DB, auth, full site
```

**原则：** 一套语言语义；**HostContext 分型**（`native` vs `browser`），不是两套语法。

---

## 3. 与现有流水线对齐

主机今日（见 `src/lib.rs`）：

```
path → fs::read → load_module → Interpreter|Vm → HostContext(native)
```

浏览器目标：

```
source: &str (+ optional import map)
  → load_module_from_memory
  → Interpreter|Vm (capture stdout)
  → HostContext(browser caps)
  → { ok, stdout, value_json, error }
```

公共库 API（C0/C1 必须落地）：

| API | 职责 |
|-----|------|
| `run_source(source, opts) -> Result<RunCapture>` | 无路径入口；cwd/fs_root 虚拟 |
| `RunOptions` 浏览器默认 | `allow_fs_write=false`, `allow_exec=false`, `allow_net=false`（net 经显式 bridge 再开） |
| `load` 导入 | `lib/**` → `embedded_lib`；用户模块 → JS 注入的 `HashMap<path, source>` |

后端优先级：

1. **C1：tree**（与默认 CLI 一致，调试简单）  
2. **C2+：bytecode** 可选（体积/性能；金样双后端）

---

## 4. Cargo / 制品布局

### 4.1 Feature 门（核心 crate `marqdo`）

| Feature | 默认（native） | `wasm-core` | 作用 |
|---------|----------------|-------------|------|
| `cli` | on | off | `main` / clap 仅 bin |
| `view` | on | off | `tiny_http` 文档浏览器 |
| `net-host` | on | off | `ureq` HTTP 客户端 |
| `plugin-host` | on | off | `libloading` 原生插件 |
| `exec-host` | on | off | `std::process` / foreign |
| `fs-host` | on | off | 真实磁盘读写（可读可后续经 VFS 桥） |
| `wasm-core` | off | on | 浏览器子集编译开关 |

`wasm32-unknown-unknown` 构建：**不得**链接 `libloading`、`ureq`、`tiny_http`、TTY `libc`。

### 4.2 包

| 路径 | 角色 |
|------|------|
| `marqdo`（根） | 核心；feature-gate |
| `crates/marqdo-wasm` | `cdylib`：`mq_alloc` / `mq_dealloc` / `mq_run` / `mq_version`（长度前缀 JSON；**无** wasm-bindgen CLI） |
| `ext/web` 或 `web/browser` | L1：`浏览器.运行` / `交互装配`（C3+） |
| `examples/browser-hello/` | 静态页 + wasm 冒烟 |
| `tests/browser/` 或金样脚本 | Node/wasmtime 跑 `run_source` 对齐 |

发行：`marqdo wasm build`（C2 CLI 糖）或 `wasm-pack build crates/marqdo-wasm`。

---

## 5. 宿主能力矩阵

| 能力 | Native | Browser C1 | Browser C3+ |
|------|--------|------------|-------------|
| `print` / 捕获 stdout | ✓ | ✓ → 可镜像 `console` | ✓ |
| `input` | stdin / preset | **仅 preset / JS 喂入** | 可选 prompt 桥 |
| builtins / 表 / 算术 | ✓ | ✓ | ✓ |
| `lib/math` `lib/json` `lib/collection` | ✓ | ✓（embedded） | ✓ |
| `lib/fs` 真磁盘 | ✓ | ✗ 明确报错 | 可选 OPFS/VFS 桥 |
| `lib/net` ureq | ✓ | ✗ | `fetch` 桥（异步模型见 §7） |
| `plugin.load` | ✓ | ✗ | ✗（改 L1 浏览器专用） |
| `exec` / foreign | ✓ | ✗ | ✗ |
| DOM / 事件 | ✗ | stub | **交互装配** |
| `sleep` | thread | 0 或 JS timer（异步） | timer |

错误策略：**显式失败**（诊断字符串含 `unavailable in browser wasm`），禁止静默空成功。

---

## 6. 交互装配（C3，对称样式装配）

作者面（示意，关键字以落地时 `ext` 双语名为准；**不在核心发明 `if`/`on` 关键字**）：

```markdown
`交互` =

| 选择器 | 事件 | 调用 |
|--------|------|------|
| "#btn" | click | handle_click |
| ".item" | click | select_item |

*`_`` = > 浏览器.交互装配 表=`交互` *
```

语义：

1. 装配函数把表登记到 WASM 侧 registry。  
2. Loader 在 DOM 上 `addEventListener`；触发时调用导出 `marqdo_dispatch(event_id, payload_json)`。  
3. 调用解析为 Marqdo 函数（同页已 `load` 的模块内 `## handle_click`）。  
4. 返回值约定（C3 最小）：`None` / 文本 / Map（如 `{dom:…}` 指令列表）由桥执行，**避免在核心写死 React**。

值列含 `/` 的 CSS 等仍遵守 [table-cell-expressions.md](table-cell-expressions.md)：**引号优先**（`"16/9"`）。

---

## 7. 异步模型（C4 才深化）

Marqdo 今日调用多为**同步**。浏览器 `fetch` / 定时器是异步的。

C1–C3 约定：

- 同步子集先完整。  
- `fetch`：C4 引入 **宿主 Promise → 续体** 或 **显式 `> 浏览器.请求` 返回 future 句柄 + 轮询/回调表**；需短 ADR 补丁，不在 C1 假装同步阻塞整个页面。

在续体方案落地前：网络仍以**服务端** `ext/web` / `lib/net`（native）为主。

---

## 8. 安全

| 项 | 规则 |
|----|------|
| 默认能力 | 无磁盘写、无 exec、无任意 net |
| 源码来源 | 同源或构建时嵌入；不默认 `eval` 远程不可信脚本除非站点显式开启 |
| DOM | 仅经装配表/白名单 API；不暴露任意 `innerHTML` 字符串执行第三方脚本 |
| CSRF / Cookie | 浏览器 Marqdo **不替代**服务端鉴权；cookie 仍走 HTTP |

---

## 9. 与 `ext/web` 分工

| 层 | 负责 |
|----|------|
| 服务端 `plugins/web` | 路由、DB、会话、SSR HTML、样式/头/图装配 |
| 页面壳 | 输出时可选嵌入 `<script type="module" src="marqdo-loader.js">` + wasm |
| 浏览器 L1 | `运行` / `交互装配` /（后）`请求` |
| 核心 | 可移植 VM + `run_source` + browser `HostFn` 子集 |

W8 资源表可增加「挂 wasm / module」行（既有 `page.head` / `头装配`），属 C2/C3 集成，不阻塞 C1。

---

## 10. 测试与验收

| 级 | 内容 |
|----|------|
| 单元 | `run_source("…# main\n> print…")` native 与 wasm 同测 |
| 金样子集 | `tests/structure/hello.mq.md` 等无 fs/net/plugin 的样例 |
| 浏览器冒烟 | `examples/browser-hello`：按钮点击 → Marqdo 函数 → 改 DOM 文本 |
| 回归 | CI：`cargo test` native；另 job `wasm-pack build` / `cargo check --target wasm32-unknown-unknown --no-default-features --features wasm-core` |

---

## 11. 文档与技能同步

- ADR 0002、本文、路线图 C0–C5。  
- `.cursor/skills/marqdo/SKILL.md`：补充「浏览器 WASM / 交互装配 / 引号 CSS」。  
- `CHANGELOG`：Unreleased 记录设计锁定与各 C 完成项。

---

## 12. 风险

| 风险 | 缓解 |
|------|------|
| wasm 体积过大 | 裁剪 feature；后期 `wasm-opt`；bytecode 路径评估 |
| 异步拖垮语言 | C4 单独立项；此前不承诺同步 `fetch` |
| Feature-gate 伤 native | 默认 features = 今日行为；CI 全绿再合 |
| 作者误以浏览器替代服务器安全 | 文档强调鉴权在服务端 |

---

## 13. 决策摘要

| 问题 | 答案 |
|------|------|
| 路线？ | **C**：解释器/VM → WASM |
| 作者语言？ | 仅 `.mq.md` |
| JS 角色？ | Loader + 宿主桥 |
| 服务端？ | 保留；不塞进浏览器 |
| 第一步实现？ | C0 门控可编译 + C1 `run_source` 打印闭环 |
