---
title: sandbox CoW — write stays upper until commit
description: >
  Readable trial: seed on disk, mutate in session, host tree unchanged,
  then commit / discard. Prefer link index over json.get.
import:
  sandbox: ext/ai/sandbox.mq.md
  fs: lib/fs.mq.md
---

# main

Prepare a tiny workspace under this test directory (reset leftovers from prior runs).

**had = fs.exists path="_sbx_cow"**
1. `had`
  > fs.remove path="_sbx_cow"
2. *
  **_ = 0**

> fs.make_dir path="_sbx_cow"
> fs.write_text path="_sbx_cow/seed.txt" text=seed-lower

**sb = sandbox.open root="_sbx_cow"**
**id = [id](sb)**

Session edits (upper / whiteout only).

> sandbox.write id=`id` path="seed.txt" text=seed-upper
> sandbox.write id=`id` path="new.txt" text=brand-new
> sandbox.remove id=`id` path="seed.txt"

**pending = sandbox.diff id=`id`**
**n = len value=`pending`**
> print text=`n`

Host tree must still show the lower seed; `new.txt` must not exist yet.

**lower = fs.read_text path="_sbx_cow/seed.txt"**
> print text=`lower`
**ex_new = fs.exists path="_sbx_cow/new.txt"**
> print text=`ex_new`

Commit applies upper + whiteout.

> sandbox.commit id=`id`

**after = fs.read_text path="_sbx_cow/new.txt"**
> print text=`after`
**seed_gone = fs.exists path="_sbx_cow/seed.txt"**
> print text=`seed_gone`

Discard must not touch the host.

**sb2 = sandbox.open root="_sbx_cow"**
**id2 = [id](sb2)**
> sandbox.write id=`id2` path="new.txt" text=should-discard
> sandbox.discard id=`id2`

**kept = fs.read_text path="_sbx_cow/new.txt"**
> print text=`kept`
