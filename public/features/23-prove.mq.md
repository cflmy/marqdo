---
markup: occlusion
import:
  result: ../../lib/result.mq.md
title: Prove what a page claims: theorem tables judged by Lean 4
description: v2.8.0 marqdo prove compiles theorem tables to Lean 4 and judges them. Offline symbolic execution works without a toolchain and never outruns Lean.
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

v2.8.0 adds one command: `marqdo prove`. A theorem table in a knowledge file is compiled to Lean 4 and judged automatically. The claim stops being prose that a reader agrees with and becomes a statement a proof assistant checks.

```text
marqdo prove examples/approval.mq.md --json
```

The pipeline has six stages: load, lower to an intermediate representation, type the claim, generate a Lean module, judge it, and issue a certificate. Judging has two layers. Offline symbolic execution runs first — path enumeration over the pure fragment, milliseconds, no toolchain required. When a Lean toolchain is available it re-checks the emitted module and upgrades the verdict.

The status ladder is honest by construction: `proved > checked > unverified > rejected > unproved > error`. Offline `checked` is a decided trace for the explored paths, never a proof; `proved` exists only when Lean says so. Without a toolchain the verdict stays at `checked` or `unverified` and the process exits zero — advisory, not silent. `--require-lean` flips unjudged states into failures for CI.

A false theorem does not get a shrug. The judge produces a counterexample witness — concrete input values that walk the function down a path where the conclusion fails — and points back at the theorem-table row.

The boundary that makes this trustworthy is monotonicity: whatever the offline layer claims, Lean must be able to confirm. If the offline verdict says checked and Lean cannot prove it, or a counterexample is proven away, the command fails with `proof.divergence` and exits non-zero. A judger defect fails loudly; a green light is never defended.

The fragment is deliberately small: pure functions, branch comparisons, call-trace shapes, constructor structure, interval restatements. Loops report `proof.foreach_pending` instead of pretending. Cross-module calls stay opaque until assumption tables arrive. Every simplification is recorded in the design docs, because a proof system that hides its boundaries is marketing, not verification.

This page runs — the theorems below live in the same file that explains them. `轨迹` is the implicit call trace of the function: the big-amount path must pass through 复核, opaque calls append to it, and inlined `result.ok` constructors do not.

# 台账

## 记一笔
+ `金额`

| 参数 | 类型 | 说明 |
|---|---|---|
| 金额 | int | 记账金额（元） |

| 定理 | 结论 | 前提 | 策略 |
|---|---|---|---|
| 大额必经复核 | 包含(轨迹, "复核") | 金额 > 10000 | auto |
| 大额单次外呼 | 长度(轨迹) = 1 | 金额 > 10000 | auto |
| 必有回执 | 结果.ok = True |  | auto |

1. 当`金额`小于等于10000时
  **回执 = [直接记账](result.ok(value="记"))**
2. *
  **复核结论 = [转复核](复核(金额=`金额`))**
  **回执 = [采纳](result.ok(value=`复核结论`))**

*回执*

## 复核
+ `金额`

**复核回执 = [复核通过](result.ok(value="复核"))**

*复核回执*
