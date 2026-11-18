---
markup: occlusion
title: AI recovery — safe defaults for short SKILL functions
description: Reuse the native error policy, durable checkpoints and effect ledger.
import:
  agent: ext/ai/agent.mq.md
---

Error handling belongs to Marqdo, not the model's prompt. The native policy
never blindly repeats uncertain writes. Every resume must revalidate the
current permission and source revision; a saved checkpoint is not authorization.

## decide

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| error | map | required | Structured AgentErrorV1 |
| attempts | number | 0 | Number of attempts already made |
| effect_class | text | "read" | Declared tool effect |
| cancelled | bool | False | Parent cancellation status |
| fingerprint_repeats | number | 0 | Identical failures without new evidence |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Safe recovery action with budget |

Use `error`, `attempts`=0, `effect_class`="read", `cancelled`=False, `fingerprint_repeats`=0 as the inputs for this operation.

This returns an action, not an automatic retry or permission to execute. For
write or unknown effect, consult the effect ledger before taking another step.

Return *[Select a bounded recovery action](agent.error_decide_retry(error=`error`, attempts=`attempts`, effect_class=`effect_class`, cancelled=`cancelled`, fingerprint_repeats=`fingerprint_repeats`))*.

## save

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable logical run ID |
| task | any | None | Current task or user request |
| status | text | "waiting_input" | Current run status |
| effect_ledger_watermark | any | None | Durable ledger boundary if available |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | CheckpointV2 write result |

Use `id`, `task`=None, `status`="waiting_input", `effect_ledger_watermark`=None as the inputs for this operation.

Use the existing atomic checkpoint writer. When ledger evidence is absent,
a resumed writer requires manual review rather than automatic repetition.
This function saves metadata; it does not suspend a Rust execution frame.

Return *[Save a versioned checkpoint](agent.resume_save(id=`id`, task=`task`, status=`status`, checkpoint_v2=True, effect_ledger_watermark=`effect_ledger_watermark`))*.

## load

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Previously saved logical run ID |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Checkpoint or missing/unsafe recovery status |

Use `id` as the inputs for this operation.

Loading a checkpoint does not execute any saved step. The caller must inspect
recovery_mode and write_auto_resume before dispatching any effectful operation.

Return *[Inspect the saved checkpoint](agent.resume_load(id=`id`))*.
