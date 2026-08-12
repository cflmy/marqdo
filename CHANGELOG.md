# Changelog

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
