# Changelog

## Unreleased

### Added
- **`ext/web` 样式装配（`web_style` / `网页.样式装配`）**: 样式即数据表格 —— GFM 样式表（`|选择器|属性|值|`，可用 `|媒体|` 列分组进 `@media` 块）经 `样式装配` 函数转成 CSS 文本，再拼装成完整主题。替代"整段手写 CSS 字符串"的写法，贯彻 文档即代码。见 [marqdo-blog](examples/marqdo-blog/styles/theme.mq.md)。
- **网络能力调研（`doc/design/web-net-capabilities.md`）**: 盘点 `ext/web` + `plugins/web` + `lib/net` 现状，对照主流语言网络栈（FastAPI / Express / Flask / axum），给出「开发一个完整 Web 项目」所需能力的差距清单与分波次补强路线（W1–W7 + P3，**2026-08-28 已全部落地**）。
- **AI Skill + 文档（ext/web 完结复核）**: `skills/marqdo/` 与 `.cursor/skills/marqdo/` 增加 **ext/web 动态站** 专节（硬规则、API 摘要、§13 最小站点样例）；`doc/design/web-net-capabilities.md` 结论/对照表与 W7+P3 实现对齐；`doc/design/ai-skill.md` 更新用途说明。
- **`ext/quantum` Q7（高阶线性代数 + 高级可视化）**: 密度矩阵 / 部分迹 / Hermitian 谱 / Schmidt / Pauli 期望 / 纯度；SVG：hinton、city、density、paulivec、qsphere、multibloch。全部经 `plugins/quantum` ABI。设计 [ext-quantum-q7.md](doc/design/ext-quantum-q7.md)；金样 `quantum-linalg-smoke`、`quantum-viz-advanced-smoke`；示例 [quantum-entanglement](examples/quantum-entanglement/)。

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
