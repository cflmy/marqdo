---
title: ext/web/_capability
description: Internal — load native web plugin once. Authors must not call this; import web = capability (ADR 0007).
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

## load

Load ABI v2 `web` plugin if present. Fail loud when missing.

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/web: native web plugin not found (build plugins/web or marqdo ext add web)
  > sys.exit code=1
****
