---
name: marqdo-dev
description: Write, refactor and verify Marqdo programs, standard libraries and extensions through the executable development handbook. Use native Marqdo file tools and marqdo call; extend capability gaps through plugins or explicit external tools.
---

Read [the executable development entry](../marqdo-dev/index.mq.md), then [the programming guide](../../doc/library-development.mq.md). The same guide governs programs, libraries and executable SKILLs. For installed skills, resolve these paths against the Marqdo source checkout.

Use `**statement**` to execute and `*return*` to return; write `[intent](actual code)` inside executable positions, and `access(coll, key)` to read collections. The guide provides real examples. Native file tasks live in [files.mq.md](../marqdo-dev/files.mq.md): read, bounded excerpt, write, patch, search, inventory, audit and apply. Use excerpt for long generated lines; its source locations remain one-based and clipped text is inspection evidence, not replacement source. Do not generate Python or shell programs for ordinary file operations.

Prepare the native engineering plugin with the development entry before using its analysis or process tasks. The repository development binary is `target/debug/marqdo`; installed older binaries may need a bootstrap Cargo build through `lib/sys.mq.md exec`. All subsequent task execution uses `marqdo call`. Keep decisions and sequencing in readable mq.md; use a new extension or a documented foreign tool only for a real capability gap.

The guide is authoritative for narrative over 50%, immediate parameter/return tables, inline statements and returns, GFM data, index.mq.md organization, recursive Markdown web components, failure evidence and verification. Read each task's contract and grant its actual effects. External processes retain their OS authority. Do not rewrite intentional test fixtures or restore removed compose_* APIs.

Use the development entry's audit task to discover and review maintained repository sources. Its failed Result retains every finding; write_report=True saves the same evidence under .marqdo/audit. It does not execute inspected source or treat known placeholder warnings as completed logic.

Build current WASM with the entry's build_browser task. If the system target is incompatible, call [toolchain.mq.md](../marqdo-dev/toolchain.mq.md) rust_wasm_linux and pass its returned compiler path. Prepare the workspace browser driver with prepare_browser, then run test_browser for actual SPA/history and two-tab collaboration behavior. These tasks retain native file IO and use existing compiler/browser tools only at their documented boundaries.
