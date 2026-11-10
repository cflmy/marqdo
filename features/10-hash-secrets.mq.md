---
markup: occlusion
title: Mid stdlib — hash and secrets
description: lib/hash + lib/secrets without plugins (Mid M3)
import:
  hash: lib/hash.mq.md
  sec: lib/secrets.mq.md
---

# Digests and secure tokens


Integrity digests and cryptographic tokens stay in `lib/` — do not use `math.random` for secrets.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

We first **d = [Compute d](hash.sha256(text="marqdo"))** for the next step.
We first **t = [Compute t](sec.token_hex(n=8))** for the next step.
We first **L = [Compute L](len(`t`))** for the next step.
We now **[Display this step's result](> print text=`d`)**.
We now **[Display this step's result](> print text=`L`)**.
1. `L` == 16
  We now **[Display this step's result](> print text="mid-m3-ok")**.
2. *
  We now **[Display this step's result](> print text="mid-m3-fail")**.
