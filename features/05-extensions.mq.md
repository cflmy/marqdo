---
markup: occlusion
title: Official extensions (ext/)
description: Optional packages — not stdlib
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

ext/ holds official optional extensions. They are not part of lib/. Resolve via `~/.marqdo/ext`, MARQDO_EXT, ./ext, or ext/ next to the binary.

Install (v0.2+):

```text
marqdo ext list
marqdo ext add llm
marqdo ext add agent
marqdo ext add web
marqdo ext add quantum
marqdo ext add linalg
```

`ext add` downloads L1 sources and prebuilt natives when available. Order: CDN https://ext.marqdo.com → GitHub Releases → proxy mirror. Pack SemVer is `ext/VERSION` and may differ from the CLI.

Rebuild natives locally only when developing plugins:

```text
cargo build --release -p marqdo_plugin_agent
cargo build --release -p marqdo_plugin_quantum
cargo build --release -p marqdo_plugin_linalg
bash ./scripts/build-web-plugin.sh
marqdo ext add agent
marqdo ext add web
marqdo ext add quantum
```

Packages (not Markdown list markers — those are loops in Marqdo):

ext/llm — OpenAI-compatible chat. Design: doc/design/ext-llm.md

ext/agent — agent framework + native plugin. Design: doc/design/ext-agent.md

ext/web provides Documents/Endpoints, recursive View, document-head and request resources. Decompose layout to Markdown or native-control leaves; data, authentication and transport belong to ext/data, ext/security and ext/net. Follow examples/web-site/index.mq.md and examples/web-net-site/index.mq.md for complete workflows. Browser interaction uses Marqdo clients and the official WASM bridge. Development, builds and real acceptance run through marqdo call in skills/marqdo-dev/index.mq.md; the current author contract starts at ext/web/index.mq.md.

ext/quantum — circuits, draw, noise, formula matrix custom gates, Q7 density/viz, Q8 themed SVG (`theme=dark|light|bw`). Design: doc/design/ext-quantum.md · doc/design/ext-quantum-viz-style.md

Installer: marqdo ext list / add / remove — doc/design/ext-cli.md

Native plugins: lib/plugin + include/marqdo_abi.h — doc/design/ext-abi.md

We now **[Display this step's result](> print text="ext-ok")**.
