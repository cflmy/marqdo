---
markup: occlusion
title: ext/web/_capability
description: >-
  Web ABI load skill. Prefer domain modules; call load when diagnosing plugins.
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
---

This boundary loads the native Go web plugin. Domain render/server tools load
it automatically; pure layout and component constructors do not need it.

```sh
marqdo call ext/web/_capability/index.mq.md load --effects plugin --json
```

Build with bash scripts/build-web-plugin.sh, or install with marqdo ext add web.
For local development set MARQDO_WEB_PLUGIN to the built library's absolute path.
Success returns null; missing libraries also require checking process status.
Only this boundary imports lib/plugin, including through the Chinese facade.

## load

Locate the ABI v2 web shared library through plugin.native_path, then
plugin.load it. If the library is missing, print a short install hint and exit
non-zero so CI and agents fail loudly. After a successful load, later web star
names are available in this process. This is the only sanctioned place in the
package that talks to lib/plugin for web. Domain skills call load at the start
of each tool that needs the native server or renderer.

This tool takes no parameters.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | null after a successful load |

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/web: native web plugin not found (build plugins/web or marqdo ext add web)
  > sys.exit code=1
****
