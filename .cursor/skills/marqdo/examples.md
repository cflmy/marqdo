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
    - who

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
    - n

**`n` + 1**
```

## 5. Branch with else

```markdown
# main

*`n` = 0*

+ `n` > 0
  > print text=positive
+ `n` < 0
  > print text=negative
+ *
  > print text=zero
```

## 6. Import text stdlib

```markdown
---
title: split demo
> lib/text.mq.md
---

# main

*`parts` = > split value=a,b,c sep=,*
> print text=`parts`
```

## 7. Comment paragraphs (blank-line rule)

```markdown
# main

This whole paragraph is a comment, including lines that look like code:
*`not_executed` = 1*
> print text=also still comment

After a blank line, code runs:

> print text=this runs
```

## 8. Frontmatter import only

```markdown
---
description: loads math
> lib/math.mq.md
---

# main

*`t` = > num value=3.5*
> print text=`t`
```

## 9. Object handle + method

See `tests/structure/object-handle.mq.md` and `ext/llm.mq.md`: `# Type` constructs a map with `_type`; methods use `` > `obj`.method `` and read `self` / `自`.

## 10. Agent layout (ABI)

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

## 11. What not to emit

```markdown
# BAD — bold is return, not print
**Hello**

# BAD — Python control keywords
if x > 0:
  print(x)

# BAD — wrapping a call in italics
*> print text=hi*
```
