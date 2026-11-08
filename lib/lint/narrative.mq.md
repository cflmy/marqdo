---
markup: occlusion
title: lib/lint/narrative
description: >-
  Narration-ratio skill (≥50%). Canonical checker: marqdo check --narrative-ratio.
---

Narration ratio skill. Every authored function unit should mix narration with
code so narration is more than half the text. This is the first skill-as-tools
example called out in the author vision notes: the page is documentation, and
its headings are tools. The canonical path checker lives in the host as
marqdo check --narrative-ratio so directory dogfood does not need filesystem
effects inside a pure call.

For AI agents: run marqdo check --narrative-ratio on edited library files.
This is a readability signal; parsing, contracts, executable examples and real
behavior tests provide correctness evidence. Headings name actual tools only.

## remind

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | rule reminder map |

Return a small map stating the narration threshold and the CLI to run. Takes no
parameters. Use this when an agent needs a tool-shaped reminder without opening
design docs. The threshold is one half. Contract tables count as narration;
call lines and bold statements count as code.

`out` =

| threshold | cli | tables |
|-----------|-----|--------|
| 0.5 | marqdo check --narrative-ratio FILE | narration |

Return *out*.
