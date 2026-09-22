<!--
P4 实验 (b) 材料 · D 臂（全文档背诵组）上下文包。
来源：commit 199b5c5^ 的 `.cursor/skills/marqdo/SKILL.md`（背诵式「压缩操作手册」）
语言部分原样冻结（Markup→meaning / 最小程序 / 函数调用 / 语句分支 / 导入与内建面）。
只读实验材料——**请勿修改**（改了就不是同一个实验）。
-->

## Markup → meaning (v0.3)

| Marker | Meaning |
|--------|---------|
| `#` | **Object / type** (constructor body). `# main` = entry object. `# Child = > Parent` = inherit (**no** implicit parent constructor; call `` `self` = > Parent … `` explicitly when needed). |
| `##` … `######` | **Function / method** (nesting by heading depth). |
| `` + `name` `` under heading | Parameter (optional `` `name`=default ``) |
| `1.` `2.` … in body | Branch arm (condition or `N. *` else). **Same-indent restart at `1.` = new branch statement** (not more arms of the previous list). |
| `- …` inside body | Loop (`while` or `` [item](coll) `` / `` [`item`](`coll`) ``) |
| Line `N. *` | Else arm |
| `> fn args` | Call (named `k=v` or positional) |
| `修饰 [fn] args` | Bracket-marked call; leading words → `名=True` |
| `` > `obj`.method args `` | Method call (`obj` must be a map with `_type`) |
| Frontmatter `import bind:path.mq.md` / `导入` | Import file (bind library name) |
| Frontmatter `import bind:lib.member` / `导入` | Short name for a library member (same keyword; no separate `use`) |
| `> lib.member …` / `> lib.Type.member …` | Call via bare dotted path (instance methods need `` `var`.m ``) |
| `**…**` | **Code** (assign / call) — closing `**` touches last token; bare ids are variables; quote text literals |
| `*…*` | **Return value** — closing `*` touches last token; bare ids are variables |
| `****` / whole-line `**` / `*None*` | Return `None` and end function body |
| Lone `---` / `***` | Markdown thematic break (skip; not function end) |
| GFM table after empty RHS bind | Collection; get with `[key](coll)` (preferred) or legacy `` `m`[^key] `` |
| `- [item](coll)` | Foreach (unchanged) |
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

*None*
```

- Call: `> name key=value` or `> name value`.
- Nested helpers use deeper `#` (`##`, `###`, …).
- After a helper with only side effects, end with `*None*` / whole-line `**` / `****` when later lines must stay in the outer function. Lone `---` is only a thematic break (skipped).
- In bold assigns, prefer bare calls: `**摘要 = 结账摘要 …**` (omit `>`).

## Statements, returns, branches

```markdown
**x = 1**
**y = x + 2**

## add_one
    + `n`

*n + 1*
```

```markdown
**n = 0**

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

**xs = > text.str_split s="a,b,c" sep=","**
> print text=`xs`
**t = > clock.now_unix**
```

- Import **English** or **Chinese** library file; call that file's API names via `lib.member` (do not mix languages).
- Instance methods stay `` > `obj`.method `` (backticks on the receiver only).
- `lib/…` resolves via `MARQDO_LIB`, cwd `lib/`, or `lib/` next to the `marqdo` binary.
- Prefer `table.put` / `表.改` for list/map element updates; keep `json` for parse/stringify/quote.
- Builtins (no import): `print`/`打印`, `input`/`输入`, `len`/`长度`, `str`/`文本`, `int`/`整数`; literals `True`/`真`, `False`/`假`, `None`/`空`; logic `and`/`且`, `or`/`或`, `not`/`非`.
