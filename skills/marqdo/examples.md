# Marqdo examples (AI)

## 1. Hello (English)

```markdown
# main

> print text=Hello World!
```

## 2. Hello (Chinese builtins)

```markdown
# main

> 打印 内容=你好，世界！
```

## 3. Function + named call + body end

```markdown
# main

Greeting via a nested function:

> greet who=Marqdo

## greet
    + `who`

> print text=Hello, `who`!

---
```

## 4. Bindings and return

```markdown
# main

*a = 2*
*b = > add_one n=`a`*
> print text=`b`

## add_one
    + `n`

**n + 1**
```

## 5. Branch with else

```markdown
# main

*n = 0*

1. `n` > 0
  > print text=positive
2. `n` < 0
  > print text=negative
3. *
  > print text=zero
```

## 6. Foreach loop

```markdown
# main

`篮子` =

| 果 |
|----|
| 苹果 |

- [果](篮子)
  > print text=`果`
```

## 6b. Map table + footnote get

```markdown
# main

`分类` =

| 苹果 | 黄瓜 |
|------|------|
| 水果 | 蔬菜 |

*种 = 分类[^苹果]*
> print text=`种`

*第一 = 篮子[^1]*
```

One column → list; ≥2 columns + one data row → map; ≥2 columns + many rows → map of lists. Data cells are expressions (same as call-arg values): bare words, numbers, `` `var` ``, quoted strings; URLs and `gpt-4o-mini` stay text via path/hyphen folding. Footnotes: lists use 1-based digit indices; maps use key text (including digit keys like `[^00]`). Foreach on a map walks **keys**.

Row-oriented records (SQL-like): first header `@` / `行` / `row` → list of maps (marker column excluded):

```markdown
`订单` =

| 行 | 品名 | 数量 |
|----|------|------|
| 1 | 苹果 | 2 |

*名 = 订单[^1][^品名]*
```

## 7. Import text stdlib

```markdown
---
title: split demo
import text:lib/text.mq.md
---

# main

*parts = > split value=a,b,c sep=,*
> print text=`parts`
```

## 8. Comment paragraphs (blank-line rule)

```markdown
# main

This whole paragraph is a comment, including lines that look like code:
*not_executed = 1*
> print text=also still comment

After a blank line, code runs:

> print text=this runs
```

## 9. Frontmatter import only

```markdown
---
description: loads math
import math:lib/math.mq.md
---

# main

*t = > num value=3.5*
> print text=`t`
```

## 10. Object handle + method

See `tests/structure/object-handle.mq.md` and `ext/llm.mq.md`: `# Type` constructs a map with `_type`; methods use `` > `obj`.method `` and read `self` / `自`.

## 10b. Object inheritance

```markdown
# Greeter

## hello
    + `who`

*msg = "Hello, `who`!"*
**msg**

# Loud = > Greeter

## hello
    + `who`

*msg = "HELLO, `who`!"*
**msg**
```

`_type` stays the most specific name (`Loud`). Methods walk the base chain; same-name `##` on the child overrides. **No implicit super** — if the child needs parent fields, call the parent explicitly:

```markdown
# Child = > Parent
    + `name`

*self = > Parent name=`name`*
*self = > json.set map=`self` key=extra value=1*
**self**
```

## 11. Agent layout (ABI)

```markdown
---
import agent:ext/agent.mq.md
---

# main

> load_native

*ws = > agent*
*n = > `ws`.ensure_layout*
> print text=`n`
```

Requires `MARQDO_AGENT_PLUGIN` pointing at the built `agent` shared library (see `doc/design/ext-agent.md`).

## 12. What not to emit

```markdown
# BAD — bold is return, not print
**Hello**

# BAD — Python control keywords
if x > 0:
  print(x)

# BAD — wrapping a call in italics
*> print text=hi*

# BAD — old branch syntax (removed)
+ `x` > 0

# GOOD — ordered list arms
1. `x` > 0
  > print text=pos
2. *
  > print text=other

# GOOD — another independent branch (restart at 1.)
1. `y` > 0
  > print text=y-pos
2. *
  > print text=y-other
```

## 13. Dynamic site (ext/web, minimal)

```markdown
---
title: hello-site
import web:ext/web/web.mq.md
import db:db/index.mq.md
---

# main

`shell` =

| 组件 | 样式 |
|------|------|
| nav | |

`main` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | posts.title | |
| body | posts.summary | |

*store = > db.open*
*css = "body{font-family:sans-serif}"*
*page = > web.page title="Hello"*
*page = > page.compose_components components=`shell`*
*page = > page.compose_main main=`main`*
*page = > page.css css=`css`*
*app = > web.app page=`page` port=8080*
*app = > app.static prefix="/static" dir="static"*
> listen app=`app`
```

- Import `ext/web/web.mq.md` **or** `ext/web/网页.mq.md` — not both; match API language to the file.
- Full blog/CMS patterns: `examples/marqdo-blog/` (routes, SEO, RSS, auth, upload, WS).
- Run after `marqdo ext add web` and building `marqdo_plugin_web`.

## 14. Quantum entanglement lab (ext/quantum Q7)

```markdown
---
import quantum:ext/quantum/quantum.mq.md
---

# main

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

*qc = > quantum.circuit qubits=2 steps=`steps`*
*rho = > `qc`.density*
*red = > `rho`.partial_trace keep=0*
*sch = > `qc`.schmidt cut=1*
*_ = > `qc`.draw kind="hinton"*
> print text=`red`
```

- Full lab: `examples/quantum-entanglement/`. Bell-only: `examples/quantum-bell/`.
- Build `marqdo_plugin_quantum` then `marqdo ext add quantum`.
