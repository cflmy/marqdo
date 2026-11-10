---
markup: occlusion
title: Mid stdlib — regex and encoding
description: lib/re + lib/encoding without plugins (Mid M1)
import:
  re: lib/re.mq.md
  enc: lib/encoding.mq.md
---

# Regex and encoding in lib/


Mid-tier stdlib: everyday primitives stay in `lib/`, not ABI plugins.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

We first **ok = [Compute ok](re.is_match(text="user@example.com", pattern="^[^@]+@[^@]+$"))** for the next step.
We first **tok = [Compute tok](enc.base64_encode(text="marqdo"))** for the next step.
We first **hex = [Compute hex](enc.hex_encode(text="ok"))** for the next step.
We now **[Display this step's result](> print text=`ok`)**.
We now **[Display this step's result](> print text=`tok`)**.
We now **[Display this step's result](> print text=`hex`)**.
1. `ok`
  We now **[Display this step's result](> print text="mid-m1-ok")**.
2. *
  We now **[Display this step's result](> print text="mid-m1-fail")**.
