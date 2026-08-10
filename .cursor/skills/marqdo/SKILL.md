---
name: marqdo
description: >-
  Author and edit Marqdo programs (.mq.md): Markdown markers are executable
  syntax (functions, calls, statements, returns, control flow). Use when writing
  or modifying .mq.md files, teaching Marqdo, generating Marqdo from prose,
  importing lib/*, running marqdo CLI, or when the user mentions Marqdo,
  mq.md, markup-as-syntax, or code-as-documentation.
---

# Marqdo development

Marqdo = **Markdown markers as syntax**. A `.mq.md` file is both documentation and a runnable program. Read this skill **before** inventing syntax from Markdown habits or Python.

Canonical design (repo): `doc/design/markdown-mapping.md`, `doc/design/keywords.md`. Deeper tables: [reference.md](reference.md). Copy-paste patterns: [examples.md](examples.md).

## Hard rules (do not violate)

1. **File suffix** must be `.mq.md` (never plain `.md` for executable sources).
2. **Output is not `**bold**`.** Print with `> print text=…` (or `> 打印 内容=…`). Bold `**…**` is **return value only**.
3. **Prose vs code:** after a blank line, an unmarked first line starts a **comment paragraph**; every following non-blank line stays comment until the next blank line. Put a **blank line** between narration and executable lines.
4. **Do not invent keywords** `if` / `else` / `while` / `for` / `def` / `return`. Control flow is Markdown: **`+` params** (under heading), **`1.` `2.` … branches**, **`-` loops**, arm `N. *` = else, `#` = **object/type**, `##`+ = **function/method**, `**…**` = return.
5. **Identifiers use backticks** — params `` + `name` ``, bindings `` `x` ``, foreach `` [`item`](`coll`) ``. **Complex text** uses `"..."` with escapes; bare tokens stay literal (no `\n` magic).
6. **Structure lines are not wrapped in italics.** `#` `>` `+` `-` `|` lines stand alone; use `*…*` only for general statements (bindings / expressions).
7. **Paths:** In bare *expressions* `/` is division. In **call args / param defaults**, unspaced `a/b` and quoted `".marqdo/agent-kb"` are path text — no `json.parse` needed.
8. Prefer ending side-effect-only function bodies with a lone `---` or `***` line (or `****` empty return) so later siblings are not swallowed.
9. **`ext/**` never calls `host_*`.** Agent/OKF helpers are plugin names (`agent_kb_*`, …) after `plugin.load`. Do **not** add agent/OKF domain code to `src/host/` (core bloat). See `doc/design/ext-agent.md` §4.

## Markup → meaning (v0.2)

| Marker | Meaning |
|--------|---------|
| `#` | **Object / type** (constructor body). `# main` = entry object. |
| `##` … `######` | **Function / method** (nesting by heading depth). |
| `` + `name` `` under heading | Parameter (optional `` `name`=default ``) |
| `1.` `2.` … in body | Branch arm (condition or `N. *` else). **Same-indent restart at `1.` = new branch statement** (not more arms of the previous list). |
| `- …` inside body | Loop (`while` or `` [`item`](`coll`) ``) |
| Line `N. *` | Else arm |
| `> fn args` | Call (named `k=v` or positional) |
| `` > `obj`.method args `` | Method call (`obj` must be a map with `_type`) |
| Frontmatter `> path.mq.md` [`as` / `作为` name] | Import (binds library name) |
| Frontmatter `> use lib.member` [`as` name] / `使用` | Bind short name to a library path |
| `> lib.member …` / `> lib.Type.member …` | Call via bare dotted path (instance methods need `` `var`.m ``) |
| `*…*` | Statement (bind / expr) |
| `**…**` | Return value |
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
*`x` = 1*
*`y` = `x` + 2*

## add_one
    + `n`

**`n` + 1**
```

```markdown
*`n` = 0*

1. `n` > 0
  > print text=positive
2. `n` < 0
  > print text=negative
3. *
  > print text=zero
```

## Frontmatter imports + stdlib

Imports bind a **library name** (default = file stem; optional `as` / `作为`). Members are **not** flattened into the caller. Call with a bare dotted path:

```markdown
---
title: example
> lib/text.mq.md
> lib/time.mq.md as clock
---

# main

*`xs` = > text.split value=a,b,c sep=,*
> print text=`xs`
*`t` = > clock.now_unix *
```

- Import **English** or **Chinese** library file; call that file’s API names via `lib.member` (do not mix languages).
- Instance methods stay `` > `obj`.method `` (backticks on the receiver only).
- `lib/…` resolves via `MARQDO_LIB`, cwd `lib/`, or `lib/` next to the `marqdo` binary.
- Design: [module-namespace.md](../../doc/design/module-namespace.md).
- Official optional extensions (`ext/`, not stdlib): install with `marqdo ext add llm` / `add agent` (see `doc/design/ext-cli.md`). `ext/llm` — chat; `ext/agent` — document-driven agents: **step** (default writeback) / **plan** workbook ([ext-agent.md](../../doc/design/ext-agent.md), [ext-agent-plan.md](../../doc/design/ext-agent-plan.md)).
- Native plugins: `lib/plugin` — `## load` / `unload` / `list`. C ABI: `include/marqdo_abi.h`.
- Builtins (no import): `print`/`打印`, `input`/`输入`, `len`/`长度`, `str`/`文本`, `int`/`整数`; literals `True`/`真`, `False`/`假`, `None`/`空`; logic `and`/`且`, `or`/`或`, `not`/`非`.

Stdlib map: [reference.md](reference.md).

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
| Forgetting `---` after nested `##` helper | Add `---` / `***` / `****` |
| Import `lib/text` then call bare `split` | `> text.split …` (qualified) |
| Import `lib/text` then call `拆分` | Match file language (`text.split`, not 文本) |

## Checklist before finishing

- [ ] File ends with `.mq.md`
- [ ] Print via `print` / `打印`, not bold
- [ ] Blank lines separate comment paragraphs from code
- [ ] `# main` exists when the file is an entry program
- [ ] Nested functions ended with `---` / `***` / `****` when needed
- [ ] Imports and call names share the same language file
- [ ] `marqdo run` succeeds (or errors addressed)
