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
| `crates/marqdo-wasm` | `cdylib`：`mq_*` ABI + `js/marqdo-bridge.js` |
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

### 6.1 作者面（示例）

```markdown
---
import json:lib/json.mq.md
---

# main

*`count` = 0*

`wire` =

| @ | 选择器 | 事件 | 调用 |
|---|--------|------|------|
| 1 | "#bump" | click | bump |

**`wire`**

## bump
*`count` = count + 1*
*`label` = > str count*
*`patch` = > json.set map=None key="#count" value=label*
**> json.set map=None key="set_text" value=patch**
```

`# main` 的返回值是 **wire 表**（`@` 行向记录列表）。Loader 调用 `mq_boot`，再 `wireEvents`。

### 6.2 桥约定

| 步骤 | API |
|------|-----|
| 启动 | `mq_boot(source)` → `{ ok, value }`；`value` = wire 列表或 `{ wire: [...] }` |
| 事件 | Loader `addEventListener` → `mq_call(fn, {"event":"click"})` |
| 回写 | 返回 Map 可含 `set_text`: `{ "#sel": "text" }`（仅 `textContent`） |

行键名认：`选择器`/`selector`、`事件`/`event`、`调用`/`call`。

含 `/` 的 CSS 等单元格仍 **引号优先**（见 [table-cell-expressions.md](table-cell-expressions.md)）。

L1 `浏览器.交互装配`（ext）可后置；C3 以 bridge + 返回 wire 表即可验收。

---

## 7. 异步模型（C4）

锁定：[ADR 0003](../adr/0003-browser-async-effects.md)。

处理器返回 Map 可含（与 `set_text` 并列）：

| 字段 | 含义 | 完成后 `mq_call` 实参 |
|------|------|----------------------|
| `fetch` | `{ url, method?, then, headers?, body? }` | `{ ok, status, body, error? }` |
| `after` | `{ ms, then }` | `{ ok: true }` |

Bridge：`applyEffects(exports, value)` — 先 DOM，再调度异步；**不**在 WASM 内阻塞。

示例见 `examples/browser-hello/fetch.mq.md`。

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
| 页面壳 | `头装配` 挂 `module` → `marqdo-bridge.js`；`static` 提供 `.wasm` |
| 浏览器 L1 | `客户端挂载` / `client_embed`；效应约定见 ADR 0003 |
| 核心 / crate | `run_source`、`BrowserSession`、`marqdo-wasm` ABI |

挂载示例：

```markdown
`头` =

| 关系 | 地址 |
|------|------|
| module | /static/marqdo-bridge.js |

*p = > page.头装配 表=头*
```

```bash
marqdo wasm build -o static   # marqdo_wasm.wasm + marqdo-bridge.js
# app.static dir=static
```

业务页再 `boot` 自己的 `.mq.md`（见 `examples/browser-hello/`）。

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
| wasm 体积过大 | `release-wasm` profile（`opt-level=z` + LTO + strip）；`marqdo wasm build` 打印 KiB，有则跑 `wasm-opt -Oz` |
| 异步拖垮语言 | ADR 0003 效应表；不阻塞 WASM |
| Feature-gate 伤 native | 默认 features = 今日行为；CI 全绿再合 |
| 作者误以浏览器替代服务器安全 | 文档强调鉴权在服务端 |
| 会话要 bytecode | **未做**：`BrowserSession` 固定 tree（缺 entry-env invoke）；`run_source` 已支持 bytecode |

---

## 14. C5 硬化（已落地）

| 项 | 状态 |
|----|------|
| `profile.release-wasm` + `marqdo wasm build` 体积报告 | done（~0.9 MiB） |
| 可选 `wasm-opt` | done |
| `run_source` bytecode 金样 | done |
| 规范 bridge + 构建拷贝 | done（`crates/marqdo-wasm/js/`） |
| Node ABI 冒烟 | done（`tests/wasm`） |
| 站点 `头装配` / L1 `client_embed` | done |
| 会话 bytecode / view 断点复用 | **非本版目标** |

---

## 16. 路线 D — 作者零 JS（Active）

路线图：[browser-wasm-d.md](../roadmap/browser-wasm-d.md)。

**原则：** 业务与交互 100% `.mq.md`；`marqdo-bridge.js` 仅宿主胶。作者仓库可不含自写 `.js`。

### 16.1 一键挂载

| API | 含义 |
|-----|------|
| `mount({ wasmUrl, source \| sourceUrl, … })` | `loadWasm` → `boot` → `wireEvents` → 初始效应 |
| `script[data-mq-source-url]` / `data-mq-wasm` | 模块脚本属性自启 |
| `#marqdo-boot` | `application/json` 配置块 |
| `data-mq-playground` | 官方 playground（`#src,#run,#out`），仍非业务 JS |

`ext/web`：`client_embed` / `客户端挂载` 参数 `bridge`、`wasm`、`source`、`boot`（默认 true）。

```bash
marqdo wasm build -o static   # marqdo_wasm.wasm + marqdo-bridge.js
# app.static dir=static
# intro 或主区嵌入 > web.client_embed source="/static/client.mq.md"
```

### 16.2 DOM / 表单效应（D3）

与 `set_text` / `fetch` / `after` 并列（白名单；**默认无**任意脚本注入）：

| 字段 | 作用 |
|------|------|
| `set_text` | `textContent` |
| `set_value` | input/textarea/select 的 `value` |
| `set_attr` | 属性；值为 `null`/`false` 则 `removeAttribute` |
| `set_class` | `className` 字符串 |
| `toggle_class` | `{ "#sel": "cls" }` 切换 class |
| `set_html` | **仅**键前缀 `#trusted` 的选择器才写 `innerHTML`；其它键忽略 |

Wire 行可选 `值选择器` / `value_from`：事件触发时把该节点内容写入实参 `value`。  
事件实参默认含：`event`、`value`、`checked`、`id`（来自 event.target）；`submit` 另含 `fields` 表单字典。

L1：`web.dom_patch` / `网页.DOM补丁` 合并效应键；`web.text_patch` / `网页.文本补丁` 快捷 `set_text`。

### 16.3 非目标（D）

| # | 非目标 |
|---|--------|
| D-N1 | 用 Marqdo 重写 bridge 自身 |
| D-N2 | 作者手写业务 JS / TS（官方桥内机制允许） |
| D-N3 | ~~开放无前缀的 set_html~~（E 已允许任意选择器；内容由 MQ 产出） |

---

## 17. 路线 E — 前端能力补齐（Active）

路线图：[browser-wasm-e.md](../roadmap/browser-wasm-e.md)。

**原则：** 唯一硬禁是**作者手写业务 JS**。官方 bridge 可实现列表装配、路由、storage、WebSocket、内部模板/轻量 VDOM 等；作者只写 `.mq.md` + 效应表。

| 波 | 要点 |
|----|------|
| E1 | `set_style` / `focus` / `blur` / `scroll_into`；wire `委托`；键盘事件 |
| E2 | `set_html` / `replace_children` / `render_list` |
| E3 | `navigate` + `popstate`（选择器 `window`） |
| E4 | `storage` `{ op, key, value?, scope?, then? }` |
| E5 | `ws` `{ op: open\|send\|close, … }` |
| E6 | `fetch_all` / `interval` / `clear_interval` / fetch `fields` |

### 17.1 效应速查（E）

| 字段 | 含义 |
|------|------|
| `set_style` | `{ "#sel": { color: "red" } }` 或 `{ "#sel": "color:red" }` |
| `focus` / `blur` | `"#sel"` 或选择器数组 |
| `scroll_into` | `"#sel"` |
| `set_html` / `replace_children` | `{ "#sel": "<b>…</b>" }`（内容由 MQ 产出；XSS 由作者保证） |
| `render_list` | `{ "#sel": { tag: "li", items: ["a", { text, class, attrs }] } }` |
| `navigate` | `{ url, replace?: bool }` |
| `storage` | `{ op: get\|set\|remove, key, value?, scope?: local\|session, then? }` |
| `ws` | `{ op: open\|send\|close, url?, data?, id?, then_message?, then_open?, … }` |
| `fetch_all` | `{ requests: [fetchSpec…], then }` → `mq_call` 得 `{ results: [...] }` |
| `interval` | `{ ms, then, id? }`；`clear_interval: { id }` |
| `fetch.fields` | 对象 → `FormData` body |

Wire 列：`委托` / `delegate` = 在容器上监听，目标须 `closest(delegate)`；选择器 `window` 绑 `window`。

## 15. 决策摘要

| 问题 | 答案 |
|------|------|
| 路线？ | **C**：解释器/VM → WASM |
| 作者语言？ | 仅 `.mq.md` |
| JS 角色？ | Loader + 宿主桥 |
| 服务端？ | 保留；不塞进浏览器 |
| 第一步实现？ | C0 门控可编译 + C1 `run_source` 打印闭环 |
