---
markup: occlusion
title: ext/quantum/_capability
description: >-
  Quantum ABI load skill. Prefer domain modules; call load when diagnosing plugins.
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
---

Native capability — diagnose plugin discovery, then return to a domain module.

This boundary locates and loads the ABI v2 quantum library. Domain tools call
load themselves, so normal experiments do not need a separate setup call.
The Chinese capability file is an equivalent language facade of this boundary.

```sh
marqdo call ext/quantum/_capability/index.mq.md load --effects plugin --json
```

Success returns null. Missing native libraries cause a nonzero exit with an
installation hint; this helper uses sys.exit, so callers must also inspect
process status instead of assuming every failure is a JSON envelope. Install
with marqdo ext add quantum, or build marqdo_plugin_quantum in the source repo.
MARQDO_QUANTUM_PLUGIN may point to a specific shared library for development.

Only this boundary imports lib/plugin. Do not add plugin loaders or host_*
operations to domain code. If discovery changes, validate both language roots
and tests/call_cli.rs before returning to [circuit](../circuit/index.mq.md).

## load

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | null after a successful load |

Locate the ABI v2 quantum shared library through plugin.native_path, then
plugin.load it. If the library is missing, print a short install hint and exit
non-zero so CI and agents fail loudly. After a successful load, later quantum
star names are available in this process. This is the only sanctioned place in
the package that talks to lib/plugin for quantum. Domain skills call load at the
start of each tool that needs the simulator.

This tool takes no parameters.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="quantum"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/quantum: native quantum plugin not found (build marqdo_plugin_quantum or marqdo ext add quantum)"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
Return *None*.
