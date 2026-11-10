---
name: marqdo-dev
description: Write, refactor and verify Marqdo programs, standard libraries and extensions through the executable development handbook. Use native Marqdo file tools and marqdo call; extend capability gaps through plugins or explicit external tools.
---

Read [the executable development entry](../../../skills/marqdo-dev/index.mq.md), then [the programming guide](../../../doc/library-development.mq.md). The same guide governs programs, libraries and executable SKILLs. For installed skills, resolve these paths against the Marqdo source checkout.

Use `**statement**` to execute and `*return*` to return; write `[intent](actual code)` inside executable positions, and `access(coll, key)` to read collections. The guide provides real examples. Native file tasks live in [files.mq.md](../../../skills/marqdo-dev/files.mq.md): read, write, patch, search, inventory, audit and apply. Do not generate Python or shell programs for ordinary file operations.

Prepare the native engineering plugin with the development entry before using its analysis or process tasks. The repository development binary is `target/debug/marqdo`; installed older binaries may need a bootstrap Cargo build through `lib/sys.mq.md exec`. All subsequent task execution uses `marqdo call`. Keep decisions and sequencing in readable mq.md; use a new extension or a documented foreign tool only for a real capability gap.

The guide is authoritative for narrative over 50%, immediate parameter/return tables, inline statements and returns, GFM data, index.mq.md organization, recursive Markdown web components, failure evidence and verification. Read each task's contract and grant its actual effects. External processes retain their OS authority. Do not rewrite intentional test fixtures or restore removed compose_* APIs.

Use the development entry's audit task to discover and review maintained repository sources. Its failed Result retains every finding; write_report=True saves the same evidence under .marqdo/audit. It does not execute inspected source or treat known placeholder warnings as completed logic.
