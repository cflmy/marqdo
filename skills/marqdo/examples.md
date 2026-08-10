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

*`a` = 2*
*`b` = > add_one n=`a`*
> print text=`b`

## add_one
    + `n`

**`n` + 1**
```

## 5. Branch with else

```markdown
# main

*`n` = 0*

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

- [`果`](`篮子`)
  > print text=`果`
```

## 6b. Map table + footnote get

```markdown
# main

`分类` =

| 苹果 | 黄瓜 |
|------|------|
| 水果 | 蔬菜 |

*`种` = `分类`[^苹果] *
> print text=`种`

*`第一` = `篮子`[^1] *
```

One column → list; ≥2 columns + one data row → map; ≥2 columns + many rows → map of lists. Footnotes are 1-based on lists; map keys are header text. Foreach on a map walks **keys**.

Row-oriented records (SQL-like): first header `@` / `行` / `row` → list of maps (marker column excluded):

```markdown
`订单` =

| 行 | 品名 | 数量 |
|----|------|------|
| 1 | 苹果 | 2 |

*`名` = `订单`[^1][^品名] *
```

## 7. Import text stdlib

```markdown
---
title: split demo
> lib/text.mq.md
---

# main

*`parts` = > split value=a,b,c sep=,*
> print text=`parts`
```

## 8. Comment paragraphs (blank-line rule)

```markdown
# main

This whole paragraph is a comment, including lines that look like code:
*`not_executed` = 1*
> print text=also still comment

After a blank line, code runs:

> print text=this runs
```

## 9. Frontmatter import only

```markdown
---
description: loads math
> lib/math.mq.md
---

# main

*`t` = > num value=3.5*
> print text=`t`
```

## 10. Object handle + method

See `tests/structure/object-handle.mq.md` and `ext/llm.mq.md`: `# Type` constructs a map with `_type`; methods use `` > `obj`.method `` and read `self` / `自`.

## 11. Agent layout (ABI)

```markdown
---
> ext/agent.mq.md
---

# main

> load_native

*`ws` = > agent *
*`n` = > `ws`.ensure_layout *
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
