---
title: Official extensions (ext/)
description: Optional packages — not stdlib
---

# main

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

ext/web — HTTP / SQLite·Postgres site helpers + native plugin (default *Go libweb*, v0.4.0). W0–W7, P3, W8, and customization C0–C4; acceptance example `examples/anlian-mq/`. Browser embed (routes D/E/F, v0.3.4): `web.client_embed` auto-mounts WASM — zero author JS; `lib/browser` + GFM tables; build with `marqdo wasm build`; examples: browser-hello · browser-app · browser-media · web-client-site. Design: doc/design/ext-web.md · WASM: doc/adr/0002-browser-marqdo-wasm.md · roadmaps D/E/F.

ext/quantum — circuits, draw, noise, formula matrix custom gates, Q7 density/viz, Q8 themed SVG (`theme=dark|light|bw`). Design: doc/design/ext-quantum.md · doc/design/ext-quantum-viz-style.md

Installer: marqdo ext list / add / remove — doc/design/ext-cli.md

Native plugins: lib/plugin + include/marqdo_abi.h — doc/design/ext-abi.md

> print text=ext-ok
