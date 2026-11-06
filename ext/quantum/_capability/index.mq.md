---
markup: occlusion
title: ext/quantum/_capability
description: >-
  Quantum ABI load skill. Prefer domain modules; call load when diagnosing plugins.
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
---

Capability

This file is still a skill and a tool surface. It exists so the quantum package
can load the native ABI plugin once. Everyday author programs and agents should
call domain modules which invoke load for you. Reach for this page when checking
whether the plugin is installed, not when building a Bell state.

## For AI agents

If you were given only this file, use it only to check the plugin. Day-to-day work
should call circuit or simulate modules instead.

Call example (plain text):
  marqdo call ext/quantum/_capability/index.mq.md load --effects plugin --json

*None*

## load

Locate the ABI v2 quantum shared library through plugin.native_path, then
plugin.load it. If the library is missing, print a short install hint and exit
non-zero so CI and agents fail loudly. After a successful load, later quantum
star names are available in this process. This is the only sanctioned place in
the package that talks to lib/plugin for quantum. Domain skills call load at the
start of each tool that needs the simulator.

This tool takes no parameters.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | null after a successful load |

**p = > plugin.native_path name="quantum"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/quantum: native quantum plugin not found (build marqdo_plugin_quantum or marqdo ext add quantum)
  > sys.exit code=1
****
