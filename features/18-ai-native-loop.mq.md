---
title: AI-native loop (contracts + MLSP)
description: Progressive contracts, structured diagnostics, and the Marqdo Language Server for AI
---

# main

Milestone (v1.1.0): the AI-native loop — so an AI can verify Marqdo, not just write it.

Three pieces. A core surface guard (`doc/design/core-surface.md`) pins the 19 core constructs, so a single-sided change (list / doc / golden sample) turns CI red. Progressive contracts are optional `参数` / `返回` / `字段` tables embedded in the narrative, checked at four boundaries: call argument, return, table bind, key access. MLSP for AI (`marqdo mlsp`) is a line-delimited JSON language service; the AI queries syntax on demand instead of memorizing the whole grammar.

Static contract check:

```bash
marqdo check tests/contracts/ok_contract.mq.md
```

Structured diagnostics — errors as data:

```bash
marqdo run tests/structure/hello.mq.md --json
```

Query syntax on demand (MLSP):

```bash
printf '%s\n' '{"id":1,"method":"syntax","params":{"query":"返回"}}' | marqdo mlsp
```

MLSP methods: `locate` (find a symbol / construct card), `syntax` (rule text + sample + doc anchor), `validate` (structured diagnostics), `repair_targets` (bounded span), `repair_apply` (bounded line edits, out-of-scope refused), `schema` (contract of one unit).

> print text=Contracts are optional: with no contract table, a file stays fully dynamic.

> print text=Diagnostics carry code, severity, suggestion, doc_anchor and doc_quote.

> print text=MLSP syntax cards embed the rule text and a sample, so AI need not pre-learn the grammar.

See validation results: `doc/roadmap/perf-validation-report-2026-09-23.md`.
