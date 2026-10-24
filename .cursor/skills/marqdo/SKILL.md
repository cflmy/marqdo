---
name: marqdo
description: >-
  Author and edit Marqdo programs (.mq.md): Markdown markers are executable
  syntax (functions, calls, statements, returns, control flow). Use when writing
  or modifying .mq.md files, teaching Marqdo, generating Marqdo from prose,
  importing lib/* or ext/web or ext/quantum, running marqdo CLI, building dynamic
  sites with ext/web, quantum circuits with ext/quantum, or when the user
  mentions Marqdo, mq.md, markup-as-syntax, code-as-documentation, web/网页, or
  quantum/量子 extension.
---

# Marqdo development

Marqdo = **Markdown markers as syntax**. A `.mq.md` file is both documentation and a runnable program. Read this skill **before** inventing syntax from Markdown habits or Python.

<!-- CORE_SYNTAX_V2.1: must stay at the top so skill_brief budgets never truncate it -->

## CORE SYNTAX (v2.1 Occlusion — mandatory)

These rules are the **current default**. Do **not** teach Classic `[key](map)` index or “bold is return of print”. Classic migration only: [migration-classic.md](migration-classic.md).

1. **File suffix** `.mq.md`. Print with `> print text=…` / `> 打印 内容=…` — never use `**…**` to print.
2. **`**…**` = code / return expression.** `*…*` = statement (bind / expr). Closing markers touch the last token (no trailing space).
3. **`[描述](payload)` = occlusion:** `[]` is prose (never executed); `()` is the only executed part. Empty `()` → `None` stub.
4. **Read collections with `access(coll, key)`** (aliases `取` / `取值`). **Never** use `[key](map)` as index under default Occlusion.
5. **Foreach:** `- [描述](遍历 coll 为 item)` (`遍历`/`for`, `为`/`as`/`in`). Prefer explicit traversal when generating new code.
6. **Import / call:** frontmatter `import:` / `导入:`; call `lib.member`. One language (EN or ZH) per file.
7. **Control flow:** `#` object/type, `##`+ function, `+` params, `1.`/`2.` branches, `-` loops, `N. *` else. No invented `if`/`for`/`def`/`return` keywords.
8. **Prose vs code:** blank line separates comment paragraphs from executable lines.
9. **GFM tables** for data; prefer `table.put` over `json.set` chains. `ext/**` never calls `host_*`.
10. Body narrative `[描述](target)` is lazy (NarrativeRef) — reading does not execute side effects.

| Marker | Meaning (v2.1) |
|--------|----------------|
| `#` / `##`+ | Object/type · function/method |
| `*…*` | Statement |
| `**…**` | Return / code expression |
| `[描述](payload)` | Occlusion step (payload executes) |
| `access(c, k)` | Collection read |
| `> fn …` | Call |
| GFM table | Collection literal |

Canonical: `doc/design/core-surface.md`, `doc/design/document-native-development-guide.md`. Deeper tables: [reference.md](reference.md). Patterns: [examples.md](examples.md). Authoring craft: [marqdo-dev](../marqdo-dev/SKILL.md). Release: [marqdo-release](../marqdo-release/SKILL.md).

Executable-knowledge gate: [`tests/executable_knowledge/`](../../tests/executable_knowledge/README.md).

## Hard rules (do not violate)

1. Obey **CORE SYNTAX** above — Occlusion is default; do not emit Classic index links.
2. **Identifiers:** params `` + `name` ``; inside `*…*` / `**…**` bare ids are variables. Quote text literals. Standalone `>` calls: backticks where bare would be text.
3. **Structure lines** (`#` `>` `+` `-` `|`) are not wrapped in italics.
4. **Paths:** bare `/` in expressions is division; in call args / table cells unspaced `a/b` is path text. Ratios as text need quotes (`"1/5"`).
5. End side-effect-only helpers with `*None*` / `*空*` / `**` / `****` when the outer function continues. Lone `---` is a thematic break, not function end.
6. **`ext/**` never calls `host_*`.** Prefer tables over `json.set` / `json.append`.
7. **One language per file.** Sandbox trial before promoting Agent-generated trees.
8. Browser: Marqdo on WASM + `lib/browser` tables; do not hand-write business JS.


## Minimal program

```markdown
# main

> print text=Hello World!
```

Chinese builtins (same functions, no import):

```markdown
# main

> 打印 内容=你好
```

## Functions, calls, end-body

```markdown
# main

> greet who=Marqdo

## greet
    + `who`

> print text=Hello, `who`!

*None*
```

- Call: `> name key=value` or `> name value`.
- Nested helpers use deeper `#` (`##`, `###`, …).
- After a helper with only side effects, end with `*None*` / whole-line `**` / `****` when later lines must stay in the outer function. Lone `---` is only a thematic break (skipped).
- In bold assigns, prefer bare calls: `**摘要 = 结账摘要 …**` (omit `>`).

## Statements, returns, branches

```markdown
*x = 1*
*y = x + 2*

## add_one
    + `n`

**n + 1**
```

```markdown
*n = 0*

1. `n` > 0
  > print text=positive
2. `n` < 0
  > print text=negative
3. *
  > print text=zero
```

## Frontmatter imports + stdlib

Imports use nested YAML `import:` (`导入:` equivalent): file (`.mq.md`) or short name (`lib.member`). Members are **not** flattened into the caller. Call with a bare dotted path:

```markdown
---
title: example
import:
  text: lib/text.mq.md
  clock: lib/time.mq.md
---

# main

*xs = > text.split value="a,b,c" sep=","*
> print text=`xs`
*t = > clock.now_unix*
```

- Import **English** or **Chinese** library file; call that file’s API names via `lib.member` (do not mix languages).
- Instance methods stay `` > `obj`.method `` (backticks on the receiver only).
- `lib/…` resolves via `MARQDO_LIB`, cwd `lib/`, or `lib/` next to the `marqdo` binary.
- Design: [module-namespace.md](../../doc/design/module-namespace.md).
- Official optional extensions (`ext/`, not stdlib): install with `marqdo ext add llm|agent|web|quantum` (see `doc/design/ext-cli.md`). `ext/llm` — chat; **`ext/agent`** — document-driven agents (see **ext/agent** below); **`ext/web`** — dynamic sites (see below); **`ext/quantum`** — circuits + Q7/Q8 (see below).
- Native plugins: `lib/plugin` — `## load` / `unload` / `list`. C ABI: `include/marqdo_abi.h`.
- Prefer `table.put` / `表.改` for list/map element updates; keep `json` for parse/stringify/quote (see `doc/design/stdlib-table.md`).
- Builtins (no import): `print`/`打印`, `input`/`输入`, `len`/`长度`, `str`/`文本`, `int`/`整数`; literals `True`/`真`, `False`/`假`, `None`/`空`; logic `and`/`且`, `or`/`或`, `not`/`非`.

Stdlib map: [reference.md](reference.md).

## Official extension: `ext/agent` (document-driven)

Install: `marqdo ext add agent` (ZH: `智能体`). Native: `cargo build --release -p marqdo_plugin_agent`.

**Constitution (do not dilute):**

1. **Code is documentation** — the runbook `.mq.md` is the model’s ground truth, not a hidden system prompt bag.
2. **Call site is first-class** — `build_step_context` injects path/function/line so the model knows where it is in the program.
3. **Document is the knowledge base (OKF)** — writeback → promote/solidify → reuse / `llm_free`; MCP/corpus are evidence only.

APIs: `# agent` / `# 智能体` → `## step` / `## plan`. Context: standing + task + **call site** + tools + budgeted source/skill + act protocol (`CALL:` / `READ:`).

Examples: [agent-pong](../../examples/agent-pong/) · [agent-okf-flywheel](../../examples/agent-okf-flywheel/) · [agent-sandbox-okf](../../examples/agent-sandbox-okf/). Harness: `scripts/agent-harness.sh`. Design: [ext-agent.md](../../doc/design/ext-agent.md) · optimize O1–O5: [ext-ai-optimize.md](../../doc/roadmap/ext-ai-optimize.md). Dev LLM: copy [.env.example](../../.env.example) → `.env` (gitignored).

**Sandbox (O1):** `import sandbox:ext/ai/sandbox.mq.md` · `agent.trial` / `智能体.试跑` · `run`/`step` take `sandbox=` / `沙盒=` (default `False`). Profile caps are tables via `sandbox.profile`. Gold: `tests/ext/agent-sandbox-optin.mq.md`.

## Official extension: `ext/web` (Document / Endpoint / Resource)

Install: `marqdo ext add web` (ZH id: `网页`). See ADR 0007 · [ext-web-artifact.md](../../doc/design/ext-web-artifact.md).

**Do not degrade:** Document/Endpoint metadata; facade `web.page|route|serve|render|inspect`; View `|type|slot|value|attrs|style|` only; no `compose_*` / `configure` / author `ensure_plugin`. DB → `ext/data`; auth → `ext/security`; net → `ext/net`.

- EN: `import web:ext/web/web.mq.md`
- ZH: `导入 网页:ext/web/网页.mq.md`

Design: [ext-web-artifact.md](../../doc/design/ext-web-artifact.md) · example: [examples/marqdo-blog/](../../examples/marqdo-blog/).

## Official extension: `ext/quantum` (circuits + Q7/Q8)

Install: `marqdo ext add quantum` (ZH id: `量子`). Build: `cargo build --release -p marqdo_plugin_quantum`, then **`marqdo ext add quantum` again** so view loads the new `libquantum.so` (stale plugin = old black/white circuits).

- EN: `import quantum:ext/quantum/quantum.mq.md`
- ZH: `导入 量子:ext/quantum/量子.mq.md`

**Hard rules:** `ext/**` never calls `host_*`; hot path is ABI plugin; ≤12 qubits for statevector; density-matrix ops ≤6 qubits; one language file per program; quote draw string args (`kind="…"`, `theme="…"`).

**Core:** `# circuit` / `# gate` / `# density` — table `steps=` or method chain; `simulate` / `probabilities` / `run` / `draw`.

**Q7 linear algebra:** `density`, `partial_trace`, `eig`, `expect` (Pauli string, left=high bit), `purity`, `kron`, `schmidt`, `fidelity`.

**Draw kinds:** `circuit|probs|bloch|hinton|city|density|paulivec|qsphere|multibloch`.

**Q8 themes (circuit / probs / bloch):** `theme="dark"|"light"|"bw"` (ZH `主题=`; **default `dark`** — slate lab look, gate-family colors, label chips so wires never cross `q0` text). View plot chrome follows `data-theme` on the SVG. Advanced kinds (hinton/…) still Q8c pending full re-skin.

```markdown
*_ = > `qc`.draw kind="circuit" theme="dark"*
*_ = > `qc`.draw kind="probs" theme="light"*
```

Design: [ext-quantum.md](../../doc/design/ext-quantum.md) · Q7: [ext-quantum-q7.md](../../doc/design/ext-quantum-q7.md) · Q8 viz: [ext-quantum-viz-style.md](../../doc/design/ext-quantum-viz-style.md) · examples: [quantum-bell](../../examples/quantum-bell/) · [quantum-entanglement](../../examples/quantum-entanglement/).

## Foreign kernels (v1.5.0)

`lib/foreign` is now a full foreign-kernel surface, not only `run`:

- `invoke` returns a record (`ok`, `code`, `stdout`, `stderr`, `ms`, `timed_out`, `truncated`, `result`); pass `input` for JSON stdin and `output="json"` for a parsed result.
- `call` is JSON-in / JSON-out; `call_many` fans out one program over a list with ordered results.
- `session_open` starts a persistent Python/Node kernel; `session_eval` evaluates source; `session_set` / `session_get` bridge values explicitly as JSON. Close it with `session_close`.
- A view document opts into kernel state with `会话 = > foreign.session kernels=... cells=...`; the table names are free. `view` consumes only the `会话` handle and never scans source lines or reserved table names. Without `会话`, cells are stateless one-shot runs; no kernel is created. Clearing outputs calls `/api/session/clear` and keeps persistent kernel state. There is no separate notebook CLI mode or command, and no new Markdown marker exists.
- The same source remains the authority; state crossing kernels is explicit, never implicitly mirrored.

## AI authoring workflow

1. Decide language surface (English builtins + `lib/text.mq.md`, or Chinese + `lib/文本.mq.md`).
2. Write `.mq.md` with `# main`, blank lines around prose, correct markers.
3. Run: `marqdo run path/to/file.mq.md` (cwd = project root so `lib/` resolves).
4. On `path:line:col: message`, fix that span; re-run until exit 0.
5. Optional: `marqdo view .` / `marqdo debug .` for structure and breakpoints.

## Anti-patterns (common model mistakes)

| Wrong | Right |
|-------|-------|
| `**Hello**` to print | `> print text=Hello` |
| `# main` then prose then code with no blank line | Blank line before code |
| `if x > 0:` | `1. `x` > 0` |
| `* > print text=hi *` | `> print text=hi` |
| `` *`a` = 1 * `` (trailing space + backticks) | `*a = 1*` — bare bind, no trailing space before `*` / `**` |
| `*xs = > text.split value=a,b sep=,*` (bare text args) | `*xs = > text.split value="a,b" sep=","*` — quote text literals; bare words are variables |
| Forgetting empty return after nested `##` helper when outer continues | Add `*None*` / `**` / `****` (not `---`) |
| Import `lib/text` then call bare `split` | `> text.split …` (qualified) |
| Import `lib/text` then call `拆分` | Match file language (`text.split`, not 文本) |
| `json.set` / `json.append` to build maps or lists | GFM tables; sparse `table.put`; named helpers (`browser.*`, `web.*`) |
| `json.set` / `compose_*` page parts | `type: web` Document + View table / `web.render` / `web.serve` |
| Import `json` for every browser handler | `import browser:lib/browser.mq.md` + tables ([marqdo-dev](../marqdo-dev/SKILL.md)) |
| Mix `web.page` and `网页.页面` in one file | One import language per `.mq.md` |
| Call `host_web_*` from `ext/web` | Use `# app` / `# db` methods; plugin ABI only |
| Call `host_*` from `ext/quantum` | Use `# circuit` / `# density` methods; plugin ABI only |
| Bare `kind=hinton` inside `*…*` | `kind="hinton"` — quote text literals |
| Bare `theme=dark` inside `*…*` | `theme="dark"` — quote; ZH `主题="dark"` |
| Rebuild plugin but skip `ext add` | `cargo build -p marqdo_plugin_quantum` then `marqdo ext add quantum` (or set `MARQDO_QUANTUM_PLUGIN`) |

## Checklist before finishing

- [ ] File ends with `.mq.md`
- [ ] Print via `print` / `打印`, not bold
- [ ] Blank lines separate comment paragraphs from code
- [ ] `# main` exists when the file is an entry program
- [ ] Nested side-effect helpers ended with `*None*` / `**` / `****` when the outer function continues
- [ ] Imports and call names share the same language file
- [ ] Data shaped as **GFM tables** (or short helpers) — no `json.set` / `json.append` glue
- [ ] Web sites: tables + web classes only (no JSON glue); `data/` gitignored
- [ ] Browser client: `lib/browser` + tables when returning effects
- [ ] Quantum draw: quote `kind=` / `theme=`; rebuild plugin → `ext add quantum` before view
- [ ] `marqdo run` succeeds (or errors addressed)
