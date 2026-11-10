---
markup: occlusion
title: Path algebra and isolated native file copying
import:
  path: lib/path.mq.md
  fs: lib/fs.mq.md
---

This page composes native path and filesystem libraries. Call main with fs.write and a disposable --fs-root; the return contains actual observations for a caller or View.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Joined/normalized path text and actual copied content |

Joining and normalizing transform path text without granting filesystem access. The illustrative out/demo.txt is not an actual write destination. All real IO uses a unique native temporary directory, so this page neither requires a pre-existing out directory nor overwrites a caller's same-named files.

Grant fs.write and choose a disposable root. Native encoding, path and root checks still apply. Read the copied content before cleaning this run's directory; the returned value should be path-m2. The result contains observations rather than live file paths. An intermediate failure retains its native diagnostic and can leave this run's directory for inspection and caller cleanup.

The complete flow demonstrates text-only path algebra, actual writing, copying, reading and scoped cleanup. A normalized string is not a filesystem permission, and a printed marker alone would not prove what bytes were copied.

We **requested = [Join the illustrative output path](path.join(a="out", b="demo.txt"))** and **normalized = [Remove redundant path segments without filesystem IO](path.normalize(path="out/./x/../demo.txt"))** before the file demonstration.
For actual IO we **work = [Create a unique directory inside the granted root](fs.temp_dir(prefix="path-demo"))** and **original = [Name this run own source file](path.join(a=`work`, b="demo.txt"))** and **copied = [Name its isolated copy](path.join(a=`work`, b="demo-copy.txt"))**.
We **[Write the actual text to copy](fs.write_text(path=`original`, text="path-m2"))** and **[Copy only this run own file](fs.copy_file(src=`original`, dest=`copied`))**.
We **copied_text = [Observe the actual copied content](fs.read_text(path=`copied`))** and then **[Clean only this run unique directory](fs.remove_tree(path=`work`))**.

`observed` =

| requested | normalized | copied_text |
|---|---|---|
| `requested` | `normalized` | `copied_text` |

After the isolated file operation we return *observed*. The temporary files have been removed.
