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
    + `root`

| 参数 | 类型 | 说明 |
|---|---|---|
| root | text | Directory relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One path/ok/characters/error record per Markdown file |

Discover Markdown files, then retain each read outcome. A bad UTF-8 document
is represented as an error record; valid documents still contribute counts.

**names = fs.find path=`root` pattern="**/*.md"**
**out = table.empty_list**
- [inspect each discovered file](遍历 names 为 name)
  **file = path.join a=`root` b=`name`**
  **read = fs.try_read_text path=`file`**
  **characters = None**
  1. [read succeeded](access(`read`, "ok"))
    **body = access(read, "value")**
    **characters = len `body`**
  2. *
    **characters = None**
  **success = access(read, "ok")**
  **error = access(read, "error")**
  `record` =

  | path | ok | characters | error |
  |---|---|---|---|
  | `name` | `success` | `characters` | `error` |

  **out = table.append list=`out` item=`record`**
*out*

## summarize_csv
    + `text`

| 参数 | 类型 | 说明 |
|---|---|---|
| text | text | Header-based CSV: team, amount and active fields |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | CSV with team/total/count for active rows |

CSV cells start as text. Convert them deliberately, select active rows, group
by team, and sum integer amounts with overflow checks. Nothing passes through SQL.

**rows = csv.parse text=`text`**
`schema` =

| amount | active |
|---|---|
| int | bool |

**typed = table.cast rows=`rows` schema=`schema`**
**selected = table.where list=`typed` key="active" value=True**
**summary = table.summarize list=`selected` key="team" field="amount"**
`names` =

| key | value |
|---|---|
| team | total |

**renamed = table.rename list=`summary` fields=`names`**
`fields` =

| field |
|---|
| team |
| total |
| count |

**report = table.project list=`renamed` keys=`fields`**
*> csv.stringify rows=`report`*

## update_section
    + `path`
    + `title`
    + `body`

| 参数 | 类型 | 说明 |
|---|---|---|
| path | text | Existing Markdown file under the granted root |
| title | text | Exact, unique heading title |
| body | text | Replacement section body, without its heading |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Recoverable replacement/write outcome |

Compute the expected digest from the exact source that was read. Transform
only the selected body; fs.write_checked detects stale content before replacing.
Concurrent writers still need coordination: this is not a kernel CAS primitive.

**source = fs.read_text path=`path`**
**expected = hash.sha256 text=`source`**
**changed = document.replace_section text=`source` title=`title` body=`body`**
1. [section replacement succeeded](access(`changed`, "ok"))
  **next = access(changed, "value")**
  *> fs.write_checked path=`path` text=`next` expected_sha256=`expected`*
2. *
  *changed*

## demo

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Actual file inventory, CSV report and document update |

Prepare a small valid corpus and an invalid UTF-8 file, then run all three
operations. Validate the returned data; this function never fabricates a pass flag.

**work = fs.temp_dir prefix="stdlib-demo"**
**note = path.join a=`work` b="notes.md"**
**bad = path.join a=`work` b="invalid.md"**
**corpus = "# Notes\n\n## Status\n\nOld status.\n\n# Appendix\n\nKeep this paragraph.\n"**
> fs.atomic_write path=`note` text=`corpus`
**binary = bytes.from_base64 text="/w=="**
> fs.write_bytes path=`bad` data=`binary`
**files = inventory root=`work`**
**data = "team,amount,active\na,10,true\nb,4,false\na,20,true\n"**
**report = summarize_csv text=`data`**
**update = update_section path=`note` title="Status" body="Updated through public standard libraries."**
**after = fs.read_text path=`note`**
> fs.remove_tree path=`work`
`out` =

| inventory | csv | update | document |
|---|---|---|---|
| `files` | `report` | `update` | `after` |

*out*
