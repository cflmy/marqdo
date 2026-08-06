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
4. **Do not invent keywords** `if` / `else` / `while` / `for` / `def` / `return`. Control flow is Markdown: `+` branch, `-` loop, arm-head lone `*` = else, `#` = function, `**…**` = return.
5. **Structure lines are not wrapped in italics.** `#` `>` `+` `-` `|` lines stand alone; use `*…*` only for general statements (bindings / expressions).
6. **Paths in bare expressions:** `/` is division. Prefer `` `path` `` vars, same-dir names, or call args — not raw `a/b` in expressions.
7. Prefer ending side-effect-only function bodies with a lone `---` or `***` line (or `****` empty return) so later siblings are not swallowed.

## Markup → meaning (v0.1)

| Marker | Meaning |
|--------|---------|
| `#` | **Object / type** (constructor body). `# main` = entry object. |
| `##` … `######` | **Function / method** (nesting by heading depth). |
| `- name` under a heading | Parameter |
| `- …` inside body | Loop |
| `+ …` or `1.` | Branch arm (condition) |
| Line that is only `*` | Else arm |
| `> fn args` | Call (named `k=v` or positional) |
| `` > `obj`.method args `` | Method call (`obj` must be a map with `_type`) |
| Frontmatter `> path.mq.md` | Import |
| `*…*` | Statement (bind / expr) |
| `**…**` | Return value |
| `****` or `**` + spaces + `**` | Return `None` and end function body |
| Lone `---` / `***` in function body | End function body (no value) |
| GFM table after empty RHS bind | Collection |
| `` ```lang `` | Foreign code block (via `lib/foreign`) |
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
    - who

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

# add_one
    - n

**`n` + 1**
```

```markdown
*`n` = 0*

+ `n` > 0
  > print text=positive
+ `n` < 0
  > print text=negative
+ *
  > print text=zero
```

## Frontmatter imports + stdlib

```markdown
---
title: example
> lib/text.mq.md
---

# main

*`xs` = > split value=a,b,c sep=,*
> print text=`xs`
```

- Import **English** or **Chinese** library file; call that file’s API names (do not mix).
- `lib/…` resolves via `MARQDO_LIB`, cwd `lib/`, or `lib/` next to the `marqdo` binary.
- Official optional extensions: `ext/llm.mq.md` / `ext/大模型.mq.md` — `# llm` / `# 大模型` object handles + `## complete` / `## 运行`. Free `## load_env`. See `doc/design/ext-llm.md` and `doc/design/objects.md`.
- Native plugins (optional shared libs): `lib/plugin.mq.md` / `lib/插件.mq.md` — `## load` / `## unload` / `## list`; after load, registered names are callable directly. C ABI: `include/marqdo_abi.h`, design `doc/design/ext-abi.md`, demo `plugins/demo`.
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
| `if x > 0:` | `+ `x` > 0` |
| `* > print text=hi *` | `> print text=hi` |
| Forgetting `---` after nested `##` helper | Add `---` / `***` / `****` |
| Import `lib/text` then call `拆分` | Match file language |

## Checklist before finishing

- [ ] File ends with `.mq.md`
- [ ] Print via `print` / `打印`, not bold
- [ ] Blank lines separate comment paragraphs from code
- [ ] `# main` exists when the file is an entry program
- [ ] Nested functions ended with `---` / `***` / `****` when needed
- [ ] Imports and call names share the same language file
- [ ] `marqdo run` succeeds (or errors addressed)
