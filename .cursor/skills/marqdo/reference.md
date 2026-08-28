# Marqdo reference (AI)

Load this when you need stdlib names, CLI, or edge cases. Core rules stay in [SKILL.md](SKILL.md).

## CLI

```text
marqdo --version
marqdo run FILE.mq.md
marqdo run FILE.mq.md --backend bytecode
marqdo view PATH          # docs browser, default port 7429
marqdo debug PATH         # debugger UI, default port 7430
marqdo catalog PATH -o .marqdo
marqdo ext list
marqdo ext add llm|agent|web|quantum
marqdo ext remove llm|agent|web|quantum
```

Diagnostics look like `path:line:col: message` (1-based line/col).

**v0.2 surface:** parameters `` + `name` ``; branches `` 1. `cond` `` / `` 2. * `` (same-indent restart at `1.` = **new** branch statement); loops `` - [item](coll) `` or `` - [`item`](`coll`) ``; identifiers in backticks (foreach header may omit ticks). See `doc/design/markdown-mapping.md`.

Stdlib search order for `lib/…` imports: `MARQDO_LIB`, `./lib`, directory of the `marqdo` executable (and a few parents). Prefer the **bundle zip** (`marqdo.exe` + `lib/`) over a bare exe.

官方扩展（`ext/…`，非 stdlib）：`marqdo ext add` 装到 `MARQDO_EXT` 或 `~/.marqdo/ext`；导入仍写 `import llm:ext/llm.mq.md`。解析还含 cwd `ext/`、二进制旁 `ext/`。对象：`doc/design/objects.md`。Agent：`doc/design/ext-agent.md` · 安装器：`doc/design/ext-cli.md`。

## Builtins (no import)

| EN | ZH | Role |
|----|----|------|
| `print` | `打印` | stdout; arg `text` / `内容` |
| `input` | `输入` | one line; arg `prompt` / `提示` |
| `len` | `长度` | length; arg `value` / `值` |
| `str` | `文本` | to text; arg `value` / `值` |
| `int` | `整数` | to int; arg `value` / `值` |

Keywords (not variable names): `True`/`真`, `False`/`假`, `None`/`空`, `and`/`且`, `or`/`或`, `not`/`非`.

## Standard library files

Import one file; use **that** file’s function names.

| EN import | ZH import | Typical APIs (EN / ZH) |
|-----------|-----------|-------------------------|
| `lib/text.mq.md` | `lib/文本.mq.md` | text helpers / 去空白·拆分·拼接… |
| `lib/table.mq.md` | `lib/表.mq.md` | `put`/`改` + list/map（见 `doc/design/stdlib-table.md`） |
| `lib/fs.mq.md` | `lib/文件.mq.md` | filesystem |
| `lib/sys.mq.md` | `lib/系统.mq.md` | process / cwd / load_dotenv |
| `lib/plugin.mq.md` | `lib/插件.mq.md` | load native ABI plugins (`doc/design/ext-abi.md`) |
| `lib/time.mq.md` | `lib/时间.mq.md` | time/format |
| `lib/net.mq.md` | `lib/网络.mq.md` | HTTP(S) helpers (+ optional headers) |
| `lib/json.mq.md` | (see design) | JSON (+ `quote`) |
| `lib/math.mq.md` | `lib/数学.mq.md` | num, trig, random, formula, plot |
| `lib/foreign.mq.md` | `lib/外联.mq.md` | run foreign fenced blocks |
| `ext/llm.mq.md` | `ext/大模型.mq.md` | `# llm` / `# 大模型` object + chat methods |
| `ext/agent.mq.md` | `ext/智能体.mq.md` | agent **framework** (layout now; LLM orchestration roadmap in `ext-agent.md`) |
| `ext/web/web.mq.md` | `ext/web/网页.mq.md` | dynamic sites — see **ext/web** section below |

Open the imported `.mq.md` under `lib/` or `ext/` to see exact `##` / `#` names and parameters. Gold tests: `tests/lib/`, `tests/structure/`, `tests/ext/`.

## ext/web (dynamic sites)

Install: `marqdo ext add web` (`网页`). Native: `cargo build --release -p marqdo_plugin_web`.

| EN import | ZH import | Core types |
|-----------|-----------|------------|
| `ext/web/web.mq.md` | `ext/web/网页.mq.md` | `# page` `# style` `# db` `# form` `# app` `# auth` `# cache` `# storage` |

| Area | EN methods (representative) | Notes |
|------|----------------------------|-------|
| Page | `compose_components`, `compose_main`, `render`, `meta`, `paginate` | GFM tables; cells are literal path strings |
| DB | `init`, `insert`, `select`, `paginate`, `get`, `update`, `delete`, `migrate`, `fts`, `search`, `count`, `事务`/`txn` | init supports `unique`/`index`/`fk`; auto `created_at`/`updated_at` |
| App | `route`, `static`, `configure`, `listen`, `auth`, `gate`, `upload`, `download`, `gallery`, `route_rss`, `redirect`, `error_page`, `sitemap`, `robots` | W0–W7 + P3 complete |
| Auth | `login`, `check`, `logout`, `hash_password` | argon2 + CSRF + SQLite sessions; `role` for RBAC |

Drivers (Postgres / Redis / S3): `db url=postgres://…`, `cache url=memory:`, `storage url=file:…` — see `doc/design/ext-web-drivers.md`.

Example project: `examples/marqdo-blog/`. Capability matrix: `doc/design/web-net-capabilities.md`.

## Formula + plot (math)

```markdown
---
import math:lib/math.mq.md
---

# main

`f` =
$$
x^2 - 2
$$

*_svg = > plot formula=`f` var=x min=-3 max=3*
```

Empty RHS assignment + following `$$…$$` binds a formula value. Narrative-only `$$` without binding is display, not assignment.

## Foreign (Python example)

```markdown
---
import foreign:lib/foreign.mq.md
---

# main

`hi` =
```python
print("hello-from-python")
```
```

Lines starting with `` ` `` can be code (binding). Keep a blank line before/after fences per comment rules.

## Function-end details

| Form | Effect |
|------|--------|
| Lone `---` or `***` (≥3 same chars) at function body top level | End function; no return value |
| `****` or empty bold | Return `None` and end body |
| `**expr**` | Return `expr`; does **not** by itself cut off later sibling statements — still use HR or shallower heading |

File-leading paired `---` is frontmatter only; body `---` is not frontmatter.

## Call arguments

- Named: `> greet who=World`
- Positional: `> greet World`
- Spaces in values: allowed in named forms as in docs (`who=marqdo user`)
- Inline call in statement: `` *`n` = > len value=`s`* ``

See `doc/design/call-arguments.md`.

## Identifiers

- Variables: `` `name` ``
- Comparisons: `==` `>` `<` `>=` `<=`
- Do not use keyword tokens as names

## Design docs (repo)

| Doc | Topic |
|-----|--------|
| `doc/design/markdown-mapping.md` | Syntax constitution |
| `doc/design/objects.md` | `#` objects / `##` methods |
| `doc/design/keywords.md` | Builtins / keywords |
| `doc/design/return-hr-and-code-surface.md` | Function end / surfaces |
| `doc/design/stdlib-i18n.md` | EN/ZH lib files |
| `doc/design/stdlib-modules.md` | L1 modules including plugin |
| `doc/design/ext-abi.md` | Native C ABI v1 |
| `doc/design/ext-llm.md` | `ext/llm` |
| `doc/design/ext-agent.md` | `ext/agent` agent **framework** (layout shipped; LLM orchestration roadmap) |
| `doc/design/ext-cli.md` | `marqdo ext list/add/remove` |
| `doc/design/ext-web.md` | `ext/web` dynamic sites (author API) |
| `doc/design/web-net-capabilities.md` | ext/web capability matrix (W0–W7 + P3) |
| `doc/design/view-debug.md` | view / debug hosts |
