# Changelog

## Unreleased

### Added
- **浏览器 Marqdo（路线 C）**： [ADR 0002](doc/adr/0002-browser-marqdo-wasm.md) · [design](doc/design/browser-marqdo-wasm.md) · [roadmap C0–C5](doc/roadmap/browser-wasm.md)。
- **C0/C1**：`wasm-core` feature 门控；`run_source`；crate `crates/marqdo-wasm`（`mq_run` ABI）；示例 [examples/browser-hello](examples/browser-hello/)。
- **C2/C3**：`marqdo wasm build`；会话 `mq_boot`/`mq_call`；GFM wire 表 + `set_text` DOM 回写；[interact.html](examples/browser-hello/interact.html) 计数器示例。
- **C4**：[ADR 0003](doc/adr/0003-browser-async-effects.md) 效应表（`fetch` / `after`）+ bridge 续体；[fetch.html](examples/browser-hello/fetch.html)。
- **C5**：`release-wasm` 体积配置；`marqdo wasm build` 报告 KiB 并可选 `wasm-opt`；`run_source` bytecode 单测；会话仍为 tree。
- **WASM 收口**：规范 `crates/marqdo-wasm/js/marqdo-bridge.js`；`wasm build` 同时拷贝 bridge；Node 冒烟 `tests/wasm`；`web.client_embed` / `网页.客户端挂载`；路线图标 Completed。

### Fixed
- **Admin UI**：主区取消 `max-width:56rem` 限制，表格外包可横向滚动的 `.table-wrap`，单元格完整展示并自动换行；表单加宽且 textarea 可随内容增高。
- **`marqdo ext add`**：安装含原生插件的扩展时**先**定位/自动 `cargo build` 再拷贝 `.mq.md`，避免只装上文稿、运行时报 `native plugin not found`；扩大 `.so` 搜索路径。
- **`web_style` / 样式装配**：正确输出 `@keyframes` 块（停点为嵌套规则，不再写成 `0%: opacity: 0`）。
- **样式表含 `/` 的值**：文档明确**引号优先**（`"16/9"`、`"1/5"`）；不引入靠空格消歧的表单元格比率折叠。

## v0.3.1 — 2026-08-29

### Highlights

**W8 站点资源**：favicon / Head 资源表 / 图片装配一次补齐；并修复相对入口路径下 `entry_dir` 双重拼接导致的嵌套目录问题。

```bash
git checkout v0.3.1
cargo build --release
marqdo ext add web   # 改插件后请重新 add
```

### Added
- **`ext/web` W8（站点图标 / Head / 图片装配）** ([web-assets-and-images.md](doc/design/web-assets-and-images.md))：`app.icons` / `应用.图标` 挂 `GET /favicon.ico`；`page.head` / `头装配`；`make_images` / `图片装配`；`meta` 认 `icon`/`favicon`/`apple-touch-icon`；`static` 约定 `favicon.*`；金样 `web-assets-smoke` / `web-assets-live`；[marqdo-blog](examples/marqdo-blog/) 接入。

### Fixed
- **`host_query("entry_dir")`**：相对入口脚本（如 `tests/ext/….mq.md`）时对进程 cwd 绝对化，避免与 `for_run` 已设的脚本目录 cwd 再次拼接成 `tests/ext/tests/ext/…`。

## v0.3.0 — 2026-08-29

### Highlights

本版把 **`ext/web` 网络能力（W0–W7 + P3）**、**`ext/quantum` Q7/Q8**、**`ext/agent` A1–A4** 一并收口到正式发布；并修了网页扩展叙述行误解析、补充 ASGI/生产部署说明与 agent 下一波（B0–B5）缺口调研。

**如何拿到本版**

```bash
# 从源码（推荐跟 main / 本 tag）
git clone https://github.com/cflmy/marqdo.git && cd marqdo
git checkout v0.3.0
cargo build --release
./target/release/marqdo version

# 或从 GitHub Releases 下载 Windows 包 / 源码 zip（见 Release 资产）
marqdo version --check
```

**扩展（非 stdlib）**

```bash
cargo build --release -p marqdo_plugin_web -p marqdo_plugin_agent -p marqdo_plugin_quantum
marqdo ext add web && marqdo ext add agent && marqdo ext add quantum
# 中文 id：网页 / 智能体 / 量子 / 大模型
```

### Added
- **调研：`ext/agent` A0–A4 之后缺口** ([agent-framework-gaps-after-a4.md](doc/research/agent-framework-gaps-after-a4.md))：产品化示例、评测 harness、真 MCP、plan resume/HITL、飞轮指标与建议分期 B0–B5；挂到 [ext-agent-optimize.md](doc/roadmap/ext-agent-optimize.md) 与 [doc/README.md](doc/README.md)。
- **生产部署调研（ASGI）** ([web-asgi-servers-and-marqdo.md](doc/design/web-asgi-servers-and-marqdo.md))：不能挂 Daphne/Uvicorn；路径为反向代理 → 嵌入式 `listen`（axum）。
- **`ext/agent` A4（RAG/MCP 证据工具）**: `corpus_search` 本地语料关键词检索；`mcp_list_tools` / `mcp_call` 读 JSON fixture；返回 `authority=workbook`。金样 `agent-tools-rag-a4`。
- **`ext/agent` A3（上下文预算）**: `source_brief` / 加深版 `skill_brief`；`build_step_context` 默认截断并提示 `READ:source|skill`；`step` 支持最多 `max_reads` 次加深；父 `READ:skill`。金样 `agent-context-budget-a3`。
- **`ext/agent` A2（过程可见）**: `plan` 过程事件默认写入返回 map 的 `events`（SSE 仍仅 `stream=True`）；OKF 命中记 `REUSE` decision；view plan 卡渲染过程时间线。对齐 [agent-streaming.md](doc/roadmap/agent-streaming.md)。
- **`ext/agent` A1（OKF 复用飞轮）**: `agent_kb_list_tasks` 返回 description/aliases/status/llm_free/hits；`plan` 命中路径暴露 `match`/`score`（summary 含 match kind）；soft_match 策展提示带 status/llm_free/description；view plan 卡显示 match。金样 `agent-kb-plan-hit`。路线 [ext-agent-optimize.md](doc/roadmap/ext-agent-optimize.md)。
- **`ext/web` 样式装配（`web_style` / `网页.样式装配`）**: 样式即数据表格 —— GFM 样式表（`|选择器|属性|值|`，可用 `|媒体|` 列分组进 `@media` 块）经 `样式装配` 函数转成 CSS 文本，再拼装成完整主题。替代"整段手写 CSS 字符串"的写法，贯彻 文档即代码。见 [marqdo-blog](examples/marqdo-blog/styles/theme.mq.md)。
- **网络能力调研（`doc/design/web-net-capabilities.md`）**: 盘点 `ext/web` + `plugins/web` + `lib/net` 现状，对照主流语言网络栈（FastAPI / Express / Flask / axum），给出「开发一个完整 Web 项目」所需能力的差距清单与分波次补强路线（W1–W7 + P3，**2026-08-28 已全部落地**）。
- **AI Skill + 文档（ext/web 完结复核）**: `skills/marqdo/` 与 `.cursor/skills/marqdo/` 增加 **ext/web 动态站** 专节（硬规则、API 摘要、§13 最小站点样例）；`doc/design/web-net-capabilities.md` 结论/对照表与 W7+P3 实现对齐；`doc/design/ai-skill.md` 更新用途说明。
- **`ext/quantum` Q7（高阶线性代数 + 高级可视化）**: 密度矩阵 / 部分迹 / Hermitian 谱 / Schmidt / Pauli 期望 / 纯度；SVG：hinton、city、density、paulivec、qsphere、multibloch。全部经 `plugins/quantum` ABI。设计 [ext-quantum-q7.md](doc/design/ext-quantum-q7.md)；金样 `quantum-linalg-smoke`、`quantum-viz-advanced-smoke`；示例 [quantum-entanglement](examples/quantum-entanglement/)。
- **`ext/quantum` Q8a/Q8b（可视化美学）**: `draw theme=dark|light|bw`（默认 dark）；标签芯片 + gutter 消除线穿字；门族分色；probs/bloch 共用令牌；view 对 `data-theme` 换图框。设计 [ext-quantum-viz-style.md](doc/design/ext-quantum-viz-style.md)。改插件后须 `marqdo ext add quantum` 再开 view。

### Fixed
- **`ext/web` / `网页.mq.md`**：叙述行勿以反引号开头，避免被解析为语句（`examples/marqdo-blog` 可正常 `listen`）。

### Changed
- **Frontmatter import syntax**: `import bind:target` / `导入 bind:target` (file `.mq.md` or short name `lib.member`). Removed legacy `> path.mq.md` / `> use` imports. See [module-namespace.md](doc/design/module-namespace.md).

## v0.2.0 — 2026-08-12

### Highlights — official extension libraries

This release centers on **`ext/`**: optional packages that are **not** part of the embedded stdlib. Install them with the CLI; native plugins (`.so` / `.dll`) ship beside L1 Markdown APIs.

**How to get extensions**

```bash
# List what the installer knows about
marqdo ext list

# Install from the repo's ./ext (or MARQDO_EXT_SOURCE), into ~/.marqdo/ext (or MARQDO_EXT)
marqdo ext add llm
marqdo ext add agent
marqdo ext add web
marqdo ext add quantum

# Chinese ids work the same (大模型 / 智能体 / 网页 / 量子)
marqdo ext add 量子

marqdo ext remove quantum
```

Native plugins (agent / web / quantum) must be built once, then `ext add` copies them into `MARQDO_EXT/native/`:

```bash
cargo build --release -p marqdo_plugin_agent
cargo build --release -p marqdo_plugin_web
cargo build --release -p marqdo_plugin_quantum
marqdo ext add agent
marqdo ext add web
marqdo ext add quantum
```

Runtime also resolves plugins from `CARGO_TARGET_DIR`, `target/`, next to the `marqdo` binary, or `MARQDO_*_PLUGIN` env vars. Design: [ext-cli.md](doc/design/ext-cli.md) · [ext-abi.md](doc/design/ext-abi.md).

| Package | What you get |
|---------|----------------|
| **llm** | OpenAI-compatible chat — [ext-llm.md](doc/design/ext-llm.md) |
| **agent** | Agent layout / orchestration helpers + native plugin — [ext-agent.md](doc/design/ext-agent.md) |
| **web** | HTTP + SQLite site helpers + native plugin — [ext-web.md](doc/design/ext-web.md) |
| **quantum** | State-vector circuits, draw, noise, formula `matrix=` custom gates — [ext-quantum.md](doc/design/ext-quantum.md) |

User-facing intro: [`public/features/05-extensions.mq.md`](public/features/05-extensions.mq.md) / [`05-扩展.mq.md`](public/features/05-扩展.mq.md).

### Added
- Full **ext CLI** catalog: `llm`, `agent`, `web`, `quantum` (EN/ZH ids).
- **Quantum (Q0–Q6)**: gates, `run`/`steps`, draw (circuit/probs/bloch), heatmaps, teaching noise incl. amplitude damping, **custom gates from `$$` / `matrix=`**.
- **Web** extension (listen / render / SQLite) with L1 EN/ZH.
- **Agent** native plugin path + installer copy into `native/`.
- View **Variables** panel: previews + click-to-open rich modal (matrices / KaTeX).
- Formula matrix parse (`pmatrix` / `[[…]]`) for executable gate matrices.

### Changed
- Release notes and docs emphasize extensions vs embedded stdlib.
- Highlight.js CDN path for view pages; Variables panel script escaping fixed.

## v0.1.2 — 2026-08-07

### Added
- **Embedded standard library**: official `lib/*.mq.md` ships inside the `marqdo` binary; disk `lib/` and `MARQDO_LIB` still override.
- **`lib/writeback`** / **`lib/subtask`**: Jupyter-style writeback; concurrent subtasks (file / function / foreign).
- **Surface syntax v0.2**: `` + `param` `` parameters, `1.` ordered branches, backtick identifiers, quoted strings.
- **`marqdo version --check`**: compare installed CLI with latest GitHub release.
- **VS Code extension v0.0.6** (branch **`vscode-extension` only**): v0.2 grammar, update check — see [doc/design/vscode-extension-commit.md](doc/design/vscode-extension-commit.md)

### Changed
- Subtask `spawn` accepts `path=`, `fn=`, `code=`, or `lang=`+`source=` (not file-only).
- Release notes: standalone `.exe` includes stdlib; bundle/stdlib zips remain optional for overrides.

## v0.1.1

- v0.2 syntax migration, writeback/subtask v1 (file subprocess only), view input deferral, optional parameters.

## v0.1.0

- Initial public releases: tree + bytecode backends, `view` / `debug` / `catalog`, core stdlib, `ext/` installer.
