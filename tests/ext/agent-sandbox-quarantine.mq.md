---
markup: occlusion
title: O5 quarantine skill status
description: Promote good skill then quarantine — status becomes quarantine.
import:
  plugin: lib/plugin.mq.md
  fs: lib/fs.mq.md
  sys: lib/sys.mq.md
---

# main

**p = plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-plugin
  > sys.exit code=1

**kb = "_sbx_quar_kb"**
**had = fs.exists path=`kb`**
1. `had`
  > fs.remove path=`kb`
2. *
  **_ = 0**

**prom = agent_kb_promote kb_dir=`kb` goal="quarantine demo" workbook="sandbox-fixtures/promote-good.mq.md"**
**ok = [promoted](access(prom, "promoted"))**
**slug = [slug](access(prom, "slug"))**
> print text=`ok`

**q = agent_kb_quarantine kb_dir=`kb` slug=`slug` reason="fixture_regression"**
**qok = [ok](access(q, "ok"))**
**st = [status](access(q, "status"))**
> print text=`qok`
> print text=`st`
