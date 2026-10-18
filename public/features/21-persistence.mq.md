---
title: Persisting foreign execution results
description: Static snapshots and writeback — how foreign results survive after the process exits
---

# main

Foreign code has two useful lives: as a published snapshot, and as a living document whose latest result is kept in source. They answer different questions and should not be confused.

---

## Static snapshot: publish the result

`marqdo view output` executes the document's cells and embeds their real output in the generated HTML. The `.mq.md` source is not modified. This is the default choice for user documentation, reviews and stable links: a reader sees the result without running anything, and the source stays clean.

The static snapshot is a point-in-time artifact. If an external service changes, rebuild it; do not treat it as a live dashboard.

---

## Writeback: keep the result in the source

For an exploratory or audit document, pair `foreign` with `lib/writeback`. `writeback.record` writes a keyed output slot next to the call. The slot is visible in the Markdown source, can be reviewed in a diff and can be committed with the document.

| Step | What it does |
|------|--------------|
| foreign.run / foreign.session | Execute the foreign code and return a Marqdo value |
| writeback.record | Persist that value in a named slot |
| git diff | Review the changed result like any other document change |

The checked-in runnable example is [examples/foreign-writeback/index.mq.md](https://github.com/cflmy/marqdo/blob/main/examples/foreign-writeback/index.mq.md). Run it with `marqdo run examples/foreign-writeback/index.mq.md`; it writes a `total` slot containing `950` directly below the `writeback.record` call. Re-running replaces only that keyed slot.

Writeback is explicit. It is not hidden kernel serialization, does not persist arbitrary kernel memory, and does not touch a file unless the document calls it.

---

## Choosing

For published user docs, use the static snapshot. For a living report that must keep the latest run result in the `.mq.md` itself, use writeback. Both can coexist in one repository: static pages for readers, writeback examples for authors and audits.
