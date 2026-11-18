---
markup: occlusion
title: Native release boundary
import:
  engineering: ext/engineering/index.mq.md
  result: lib/result.mq.md
---

The release handbook composes native Marqdo libraries and this narrow extension
boundary. Ordinary file reads/writes, version decisions, asset lists and state
checks stay in Marqdo. Authenticated service requests, bounded OS commands and
ZIP-format inspection use the engineering native plugin. No Python loader,
generated command program or duplicate release orchestrator is involved.

A native plugin has the calling OS user's authority. Root is a working/artifact
location and cannot be described as an OS sandbox. These calls need plugin
effects; their public task contracts additionally declare any native filesystem,
network or environment operations they perform. Reading the handbook authorizes
none of its external publication steps.

## request

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| operation | text | required | Native boundary operation selected by the task |
| payload | map | required | Structured arguments; never credentials |
| root | text | "." | Existing checkout/artifact root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Actual boundary Result; failed child exit stays failed |

Use `operation`, `payload` and `root`="." for a documented external boundary.
The extension reports transport success separately from a child's exit code.
Release commands require code zero and no timeout, so convert a completed but
failed child into a release failure before another phase can continue. Stdout
and stderr retain bounded diagnostic evidence with known credentials redacted.
Other operations preserve their own Result without manufacturing success.

We **answer = [Invoke the maintained native engineering boundary](engineering.request(operation=`operation`, payload=`payload`, root=`root`))**.
1. not access(`answer`, "ok")
  Return *answer* with its original error.
1. `operation` == "command"
  We **execution = [Read the bounded child result](access(answer, "value"))**.
  1. access(`execution`, "code") != 0 or access(`execution`, "timed_out")
    Return *[Keep the unsuccessful child visible](result.error(code="release.command_failed", message=str(`execution`), operation="command"))*.
Return *answer* when this requested boundary completed successfully.
