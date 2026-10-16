---
title: promote requires sandbox proof (O2)
description: >
  Good workbook promotes; broken workbook is rejected (sandbox_proof_failed).
  Link-index reads; no json glue.
import plugin:lib/plugin.mq.md
import fs:lib/fs.mq.md
import sys:lib/sys.mq.md
---

# main

**p = plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-plugin
  > sys.exit code=1

**kb = "_sbx_promote_kb"**
**had = fs.exists path=`kb`**
1. `had`
  > fs.remove path=`kb`
2. *
  **_ = 0**

**prom = agent_kb_promote kb_dir=`kb` goal="sandbox promote good" workbook="sandbox-fixtures/promote-good.mq.md"**
**ok = [promoted](prom)**
> print text=`ok`

**rej = agent_kb_promote kb_dir=`kb` goal="sandbox promote bad" workbook="sandbox-fixtures/promote-bad.mq.md"**
**rej_ok = [promoted](rej)**
**reason = [reason](rej)**
> print text=`rej_ok`
> print text=`reason`
