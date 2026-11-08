---
markup: occlusion
title: lib/cli
description: Parse argv flags (Mid M5). --key value / --key=value / --flag; positionals under _.
---

Parse process argv into a map. Flags and positionals (under _).

Convert argv tokens to a small flag/positional map.

Behavior and limits: Values stay text; bare flags become booleans and positionals live under _. Omitted args use host argv. This is a small parser, not an argparse schema system.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/cli.rs](../src/host/cli.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| args | any | None | List of argv tokens; None reads the current host argv |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Parse --key value / --key=value / --flag |

Use `args`=None as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/cli.mq.md parse 'args=["report","--count=3","--quiet"]' --effects none --json
```

Parse --key value / --key=value / --flag. A bare --flag must not be followed by a non-option token if you need it as boolean (put flags last, or use --flag=true). Positionals land under _. Omit args to use process argv.

Caller: [cli.parse]

Return *[Parse --key value / --key=value / --flag](> host_cli_parse args=`args`)*.
