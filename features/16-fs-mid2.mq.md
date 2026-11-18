---
markup: occlusion
title: Native directory trees, enumeration and file metadata
import:
  fs: lib/fs.mq.md
---

This complete demonstration uses native filesystem tools. Call main with fs.write and a disposable --fs-root. It returns actual metadata and relative paths after cleaning its own temporary directory.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Actual byte size, directory entry count and relative paths |

Create a unique temporary directory, then one child directory and one text file. Every write remains under the granted root, avoiding collisions with a pre-existing demo folder. Paths are built explicitly; no generated reader/writer or foreign process participates.

The content ok occupies two UTF-8 bytes. File size measures stored bytes rather than text character count. Enumeration returns relative paths for the child directory and file; counting those paths is another distinct observation. The example does not depend on operating-system enumeration order. Sort explicitly if a later workflow requires stable ordering.

Collect the real metadata and paths before removing this run's unique tree. Returned paths describe an observation, not still-live artifacts. A failure can retain its native diagnostic and this run's directory for caller inspection. Cleanup must stay scoped to the directory created by this invocation.

We **work = [Create a unique directory inside the granted root](fs.temp_dir(prefix="fs-tree-demo"))** and **child = work + "/sub"** and **note = work + "/readme.txt"**.
We **[Create this run own child directory](fs.make_dirs(path=`child`))** and **[Write the exact UTF-8 example content](fs.write_text(path=`note`, text="ok"))**.
We **metadata = [Read the actual file metadata](fs.stat(path=`note`))** and **paths = [Enumerate this run directory entries](fs.walk(path=`work`))** and **size = [Retain storage byte size](access(metadata, "size"))** and **entries = [Count the actual enumerated paths](len(`paths`))**.
We **[Remove only this run unique directory tree](fs.remove_tree(path=`work`))** after collecting the observations.

`observed` =

| size | entries | paths |
|---|---|---|
| `size` | `entries` | `paths` |

After observing and cleaning this run we return *observed*.
