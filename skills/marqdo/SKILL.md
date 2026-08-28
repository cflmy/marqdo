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

Canonical design (repo): `doc/design/markdown-mapping.md`, `doc/design/keywords.md`. Deeper tables: [reference.md](reference.md). Copy-paste patterns: [examples.md](examples.md).

## Hard rules (do not violate)

1. **File suffix** must be `.mq.md` (never plain `.md` for executable sources).
2. **Output is not `**bold**`.** Print with `> print text=…` (or `> 打印 内容=…`). Bold `**…**` is **return value only**.
3. **Prose vs code:** after a blank line, an unmarked first line starts a **comment paragraph**; every following non-blank line stays comment until the next blank line. Put a **blank line** between narration and executable lines.
4. **Do not invent keywords** `if` / `else` / `while` / `for` / `def` / `return`. Control flow is Markdown: **`+` params** (under heading), **`1.` `2.` … branches**, **`-` loops**, arm `N. *` = else, `#` = **object/type**, `##`+ = **function/method**, `**…**` = return.
5. **Identifiers use backticks** — params `` + `name` ``. **Exemptions:** foreach `` - [item](coll) ``; footnote `` name[^key] ``; **inside italic `*…*` / bold `**…**`**, bare ids are **variables** (Python-style unified namespace) and method-call receivers are bare too. **Drop the backticks on value-expression variable names and method receivers inside `*…*` / `**…**`** — they are redundant once the `*…*`/`**…**` markers mark the segment as code: `*分类 = 分类[^苹果]*`, `**n * 2**`, `*p = > page.主体装配 组件=home*`. **Text literals must be quoted**: `*a = > text.split value=src sep=","*`. Backticks **still required** where a bare word is *not* a variable in **standalone `>` calls** (bare = text there): `` > str `n` ``, `` > `obj`.method ``. **No trailing space** inside `*…*` / `**…**` wrapped code: the closing marker must touch the last token directly (`*a = 1*`, `**n**`), never `` *a = 1 * ``.
6. **Structure lines are not wrapped in italics.** `#` `>` `+` `-` `|` lines stand alone; use `*…*` only for general statements (bindings / expressions).
7. **Paths:** In bare *expressions* `/` is division. In **call args / param defaults**, unspaced `a/b` and quoted `".marqdo/agent-kb"` are path text — no `json.parse` needed.
8. Prefer ending side-effect-only function bodies with a lone `---` or `***` line (or `****` empty return) so later siblings are not swallowed.
9. **`ext/**` never calls `host_*`.** Agent/OKF helpers are plugin names (`agent_kb_*`, …) after `plugin.load`. Do **not** add agent/OKF domain code to `src/host/` (core bloat). See `doc/design/ext-agent.md` §4.

## Markup → meaning (v0.2)

| Marker | Meaning |
|--------|---------|
| `#` | **Object / type** (constructor body). `# main` = entry object. `# Child = > Parent` = inherit (**no** implicit parent constructor; call `` `self` = > Parent … `` explicitly when needed). |
| `##` … `######` | **Function / method** (nesting by heading depth). |
| `` + `name` `` under heading | Parameter (optional `` `name`=default ``) |
| `1.` `2.` … in body | Branch arm (condition or `N. *` else). **Same-indent restart at `1.` = new branch statement** (not more arms of the previous list). |
| `- …` inside body | Loop (`while` or `` [item](coll) `` / `` [`item`](`coll`) ``) |
| Line `N. *` | Else arm |
| `> fn args` | Call (named `k=v` or positional) |
| `` > `obj`.method args `` | Method call (`obj` must be a map with `_type`) |
| Frontmatter `import bind:path.mq.md` / `导入` | Import file (bind library name) |
| Frontmatter `import bind:lib.member` / `导入` | Short name for a library member (same keyword; no separate `use`) |
| `> lib.member …` / `> lib.Type.member …` | Call via bare dotted path (instance methods need `` `var`.m ``) |
| `*…*` | Statement (bind / expr) — closing `*` touches last token, **no trailing space**; bare ids are variables |
| `**…**` | Return value — closing `**` touches last token, **no trailing space**; bare ids are variables |
| `****` or `**` + spaces + `**` | Return `None` and end function body |
| Lone `---` / `***` in function body | End function body (no value) |
| GFM table after empty RHS bind | Collection (1-col list / ≥2-col map / `@`·`行`·`row` → list of maps); `` `x`[^1] `` / `` `m`[^key] `` |
| `` ```lang `` | Foreign code block (via `lib/foreign`) |
| `` `"text"` `` | Quoted string (`\n` `\t` `\\` `\"`; `` `var` `` inside); bare tokens unescaped |
| Unmarked prose | Comment |

Entry: load file (often `index.mq.md`) → collect defs → run `# main`.

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

---
```

- Call: `> name key=value` or `> name value`.
- Nested helpers use deeper `#` (`##`, `###`, …).
- After a helper with only side effects, end with `---` / `***`.

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

Imports use `import bind:target` (`导入` equivalent): file (`.mq.md`) or short name (`lib.member`). Members are **not** flattened into the caller. Call with a bare dotted path:

```markdown
---
title: example
import text:lib/text.mq.md
import clock:lib/time.mq.md
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
- Official optional extensions (`ext/`, not stdlib): install with `marqdo ext add llm|agent|web|quantum` (see `doc/design/ext-cli.md`). `ext/llm` — chat; `ext/agent` — document-driven agents: **step** (default writeback) / **plan** workbook ([ext-agent.md](../../doc/design/ext-agent.md), [ext-agent-plan.md](../../doc/design/ext-agent-plan.md)); **`ext/web`** — dynamic sites (W0–W7 + P3 complete; see below); **`ext/quantum`** — circuits + Q7 density/viz + Q8 themed SVG (see below).
- Native plugins: `lib/plugin` — `## load` / `unload` / `list`. C ABI: `include/marqdo_abi.h`.
- Prefer `table.put` / `表.改` for list/map element updates; keep `json` for parse/stringify/quote (see `doc/design/stdlib-table.md`).
- Builtins (no import): `print`/`打印`, `input`/`输入`, `len`/`长度`, `str`/`文本`, `int`/`整数`; literals `True`/`真`, `False`/`假`, `None`/`空`; logic `and`/`且`, `or`/`或`, `not`/`非`.

Stdlib map: [reference.md](reference.md).

## Official extension: `ext/web` (dynamic sites)

Install: `marqdo ext add web` (ZH id: `网页`). Build native plugin first: `cargo build --release -p marqdo_plugin_web`.

Import **one language file** — never mix EN/ZH API names in the same `.mq.md`:

- EN: `import web:ext/web/web.mq.md`
- ZH: `导入 网页:ext/web/网页.mq.md`

**Hard rules (web):**

- Authors use **GFM tables + `#` classes** — no `json.parse` / `json.set` glue, no hand-built part JSON.
- Table cells stay **literal strings**; path text like `` `posts`.`title` `` is resolved by web classes.
- **`ext/**` never calls `host_*`** — hot path is native `plugins/web` ABI.
- HTTPS: terminate at reverse proxy; set `cookie_secure=True` on auth (no in-process TLS).

**Typical layout:**

```text
index.mq.md          # entry + home page table + listen
pages/               # sub-pages
components/          # reusable |属性|值|样式| tables
styles/              # CSS modules
db/                  # schema + open/init
data/                # sqlite runtime (gitignore)
```

**Core objects (EN / ZH):** `# page`/`页面`, `# style`/`样式`, `# db`/`数据库`, `# form`/`表单`, `# app`/`应用`, `# auth`/`鉴权`, `# cache`/`缓存`, `# storage`/`存储`.

**Typical flow:** `db.init` → `page.compose_components` + `page.compose_main` → `page.render` → `app` with `route` / `static` / `configure` / `listen`.

**Shipped surface (W0–W7 + P3):** middleware + JSON API (`app.configure`); transactions, pagination, FTS search (`db.migrate`, `db.fts`, `db.search`); security (argon2, CSRF, SQLite sessions, login rate limit); SEO / RSS / Markdown (`page.meta`, `route_rss`, `lib/net.markdown_parse`); upload / download / gallery; sitemap / robots / error pages / redirects; RBAC (`app.gate`, user `role`); audit timestamps + FK in `db.init`; ETag on downloads.

Design: [ext-web.md](../../doc/design/ext-web.md) · capability matrix: [web-net-capabilities.md](../../doc/design/web-net-capabilities.md) · example: [examples/marqdo-blog/](../../examples/marqdo-blog/).

Web patterns: [examples.md](examples.md) §13 · API index: [reference.md](reference.md).

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
| Forgetting `---` after nested `##` helper | Add `---` / `***` / `****` |
| Import `lib/text` then call bare `split` | `> text.split …` (qualified) |
| Import `lib/text` then call `拆分` | Match file language (`text.split`, not 文本) |
| `json.set` to build page parts | GFM tables + `page.compose_*` / `web.page` methods |
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
- [ ] Nested functions ended with `---` / `***` / `****` when needed
- [ ] Imports and call names share the same language file
- [ ] Web sites: tables + web classes only (no JSON glue); `data/` gitignored
- [ ] Quantum draw: quote `kind=` / `theme=`; rebuild plugin → `ext add quantum` before view
- [ ] `marqdo run` succeeds (or errors addressed)
