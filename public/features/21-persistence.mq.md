---
title: Persisting foreign execution results
description: Static snapshots and writeback — how foreign results survive after the process exits
import:
  foreign: lib/foreign.mq.md
  writeback: lib/writeback.mq.md
---

# main

Foreign code has two useful lives: as a published snapshot, and as a living document whose latest result is kept in source. They answer different questions and should not be confused.

---

Static snapshot: publish the result.

`marqdo view output` executes the document's cells and embeds their real output in the generated HTML. The `.mq.md` source is not modified. This is the default choice for user documentation, reviews and stable links: a reader sees the result without running anything, and the source stays clean.

The static snapshot is a point-in-time artifact. If an external service changes, rebuild it; do not treat it as a live dashboard.

---

Writeback: keep the result in the source.

For an exploratory or audit document, pair `foreign` with `lib/writeback`. `writeback.record` writes a keyed output slot next to the call. The slot is visible in the Markdown source, can be reviewed in a diff and can be committed with the document.

| Step | What it does |
|------|--------------|
| Foreign run / Foreign session | Execute the foreign code and return a Marqdo value |
| Writeback record | Persist that value in a named slot |
| git diff | Review the changed result like any other document change |

A real writeback in this page.

This page is itself the runnable example. The code fence below is ordinary Marqdo data; `foreign.run` executes it, then `writeback.record` writes the `total` slot into this source file.

`写回代码` =
```python
print(950)
```

**结果 = > foreign.run code=`写回代码`**

**> writeback.record value=`结果` key="total"**
<!-- marqdo-out total
950
-->

The generated slot is directly below that call. Re-running the page replaces only the `total` slot; it does not duplicate it.

Writeback is explicit. It is not hidden kernel serialization, does not persist arbitrary kernel memory, and does not touch a file unless the document calls it.

---

Choosing.

For published user docs, use the static snapshot. For a living report that must keep the latest run result in the `.mq.md` itself, use writeback. Both can coexist in one repository: static pages for readers, writeback examples for authors and audits.
