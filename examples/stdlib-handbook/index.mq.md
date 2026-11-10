---
markup: occlusion
title: Standard library engineering practice
import:
  fs: lib/fs.mq.md
  path: lib/path.mq.md
  table: lib/table.mq.md
  csv: lib/csv.mq.md
  document: lib/document.mq.md
  hash: lib/hash.mq.md
  bytes: lib/bytes.mq.md
---

This project demonstrates file inventory, typed CSV aggregation and controlled
Markdown maintenance using the public standard library. No foreign program,
plugin or alternate data syntax is involved. Each operation below is callable.
The demo creates its own directory under the filesystem root and removes it
when successful; it returns the real inventory, report and update outcome.

Create a disposable directory and set MQ_WORK to its absolute path before the
following command. All relative IO is confined to that directory. The Rust
acceptance test prepares this directory and runs the same source on both runtimes.

```sh
marqdo call examples/stdlib-handbook/index.mq.md demo --fs-root "$MQ_WORK" --effects fs.write --json
```

For production, call inventory on your source directory, summarize_csv on text
from fs.read_text, or update_section with an explicit title and replacement body.
Inspect each operation Result before using its value. File inventory keeps one
unreadable file from aborting other records. Directory cleanup and the update are
separate operations, not a transaction. For high-volume inputs use the batched
collection transforms; this report uses batch transforms, including grouping and aggregation.

## inventory

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | required | Directory relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One path/ok/characters/error record per Markdown file |

Use `root` under the explicitly granted filesystem root with fs.read.
Discover Markdown files recursively, then preserve each recoverable read outcome.
A bad UTF-8 file has an error record while other documents still contribute
their actual character counts. Discovery does not execute imported source,
Markdown links, code fences or the main function of a discovered document.

The paths in the returned records are relative to the selected directory;
joining them for a read does not grant access outside --fs-root. Character counts
measure decoded text, not file byte sizes. This distinction matters for Chinese
and other multibyte content. Missing directories and traversal errors retain
native discovery diagnostics; per-file read failures remain ordinary Result data.

We **names = [Discover Markdown paths below the selected directory](fs.find(path=`root`, pattern="**/*.md"))** and **out = [Start the observed file records](table.empty_list())**.
- [Retain the actual outcome of each independent read](for names as name)
  We **file = [Resolve this discovered path under the selected directory](path.join(a=`root`, b=`name`))** and **read = [Read text without aborting other file records](fs.try_read_text(path=`file`))** and **characters = None**.
  1. [This file decoded successfully](access(`read`, "ok"))
    We **body = [Read the decoded UTF-8 source](access(read, "value"))** and **characters = [Count text characters rather than bytes](len(`body`))**.
  We **success = [Retain the recoverable read state](access(read, "ok"))** and **error = [Keep the original read diagnostic](access(read, "error"))**.

  `record` =

  | path | ok | characters | error |
  |---|---|---|---|
  | `name` | `success` | `characters` | `error` |

  We **out = [Retain the successful count or original failure](table.append(list=`out`, item=`record`))**.
After inspecting the selected directory we return *out*.

## summarize_csv

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Header-based CSV with team, amount and active fields |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | CSV with team/total/count for active rows |

Use `text` as CSV content, not a filename. Parsing begins with text cells.
The visible schema deliberately converts amount to an integer and active to a
boolean before filtering. Only active rows enter grouping; amounts are summed
per team with the table library's integer checks. Invalid cells retain their
conversion diagnostics instead of silently becoming zero.

The two-column rename record maps the summarizer's key and value fields to team
and total. The final field list fixes exported column order, including count.
No SQL, foreign interpreter, plugin, file or network operation participates.
If input comes from a file, use the native file library before this pure task.
The output is an actual CSV report that another task can write or transmit;
printing it is not required for later Marqdo functions to consume the value.

We **rows = [Parse the caller-provided CSV text](csv.parse(text=`text`))**.

`schema` =

| amount | active |
|---|---|
| "int" | "bool" |

We **typed = [Convert the business fields before filtering or arithmetic](table.cast(rows=`rows`, schema=`schema`))** and
**selected = [Select only explicitly active records](table.where(list=`typed`, key="active", value=True))** and
**summary = [Sum checked amounts and count records per team](table.summarize(list=`selected`, key="team", field="amount"))**.

`names` =

| key | value |
|---|---|
| "team" | "total" |

We **renamed = [Name the group key and total for report consumers](table.rename(list=`summary`, fields=`names`))**.

`fields` =

| field |
|---|
| "team" |
| "total" |
| "count" |

We **report = [Keep an explicit exported column order](table.project(list=`renamed`, keys=`fields`))**.
After the native transforms we return *[Serialize the actual row records as CSV](csv.stringify(rows=`report`))*.

## update_section

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Existing Markdown file under --fs-root |
| title | text | required | Exact unique heading title |
| body | text | required | Replacement section body without its heading |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Recoverable section replacement or checked-write outcome |

Use `path`, `title` and `body` with fs.write. Read the original UTF-8
source once and hash that exact text. The document library replaces only the
selected heading body; a missing or ambiguous title remains its failed Result.
The other sections and heading are retained by the structural transform.

Write only after a successful transform, passing the observed digest to the
native checked writer. A stale source is refused instead of overwriting an
edit that occurred after review. This is a preflight digest check, not a kernel
CAS or a transaction spanning several files. Concurrent writers still need
coordination, and a source-read policy error is a native diagnostic rather
than a fabricated recoverable success. No external text-processing program
participates in reading, section replacement, hashing or writing.

We **source = [Read the exact reviewed Markdown source](fs.read_text(path=`path`))** and **expected = [Hash that returned source rather than another read](hash.sha256(text=`source`))** and
**changed = [Replace only the explicitly selected section body](document.replace_section(text=`source`, title=`title`, body=`body`))**.
1. [The structural replacement succeeded](access(`changed`, "ok"))
  We **next = [Read the successful document transform](access(changed, "value"))**.
  Return *[Refuse drift before writing the reviewed document](fs.write_checked(path=`path`, text=`next`, expected_sha256=`expected`))*.
Return *changed* with its original structural failure.

## demo

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Observed file inventory, CSV report and actual document update |

Run this complete workflow with fs.write under a disposable --fs-root.
A fresh directory holds one valid Markdown document and one invalid UTF-8
document. Inventory should preserve both read outcomes. The malformed file is
intentional data for error recovery; it is never imported or executed.

The CSV sample contains two active team-a amounts and one inactive team-b row.
Its report therefore retains team a with a total of thirty and a count of two.
The section update changes Status while preserving Appendix. After reading
the saved document, normal completion removes this task's own temporary tree;
a failure before cleanup may retain it for diagnosis. Do not remove arbitrary
caller directories to make a test pass.

The returned record contains these actual values, including the update Result,
so callers can check content and failure details. It is not a fixed compliance
flag and does not claim an atomic transaction across inventory, reporting and
editing. Every operation composes the public native library, with no Python,
shell text processing, JSON construction chain or hidden source-file bridge.

We **work = [Create a unique directory inside the granted root](fs.temp_dir(prefix="stdlib-demo"))** and
**note = [Name this task own Markdown document](path.join(a=`work`, b="notes.md"))** and **bad = [Name the intentionally invalid text fixture](path.join(a=`work`, b="invalid.md"))**.
We **corpus = "# Notes\n\n## Status\n\nOld status.\n\n# Appendix\n\nKeep this paragraph.\n"** as the complete source that will be edited.
We **[Write the actual starting document](fs.atomic_write(path=`note`, text=`corpus`))** and **binary = [Construct the single invalid UTF-8 byte](bytes.from_base64(text="/w=="))**.
We **[Write the deliberate per-file failure fixture](fs.write_bytes(path=`bad`, data=`binary`))**.
We **files = [Keep both successful and failed native reads](inventory(root=`work`))** and **data = "team,amount,active\na,10,true\nb,4,false\na,20,true\n"**.
We **report = [Run the pure typed aggregation on the visible sample](summarize_csv(text=`data`))** and
**update = [Replace the chosen status section with drift protection](update_section(path=`note`, title="Status", body="Updated through public standard libraries."))** and
**after = [Observe the document that was actually saved](fs.read_text(path=`note`))**.
We **[Remove only this task unique temporary directory](fs.remove_tree(path=`work`))** after the observed results are available.

`out` =

| inventory | csv | update | document |
|---|---|---|---|
| `files` | `report` | `update` | `after` |

After completing these native tasks we return *out*.
