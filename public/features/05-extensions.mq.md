---
title: Official extensions (ext/)
description: Optional packages — not stdlib
---

# main

ext/ holds official optional extensions. They are not part of lib/. Resolve via MARQDO_EXT, ./ext, or ext/ next to the binary.

Install (v0.2+):

```text
marqdo ext list
marqdo ext add llm
marqdo ext add agent
marqdo ext add web
marqdo ext add quantum
```

For packages with a native plugin, build first then add:

```text
cargo build --release -p marqdo_plugin_agent
cargo build --release -p marqdo_plugin_web
cargo build --release -p marqdo_plugin_quantum
marqdo ext add agent
marqdo ext add web
marqdo ext add quantum
```

Packages (not Markdown list markers — those are loops in Marqdo):

ext/llm — OpenAI-compatible chat. Design: doc/design/ext-llm.md

ext/agent — agent framework + native plugin. Design: doc/design/ext-agent.md

ext/web — HTTP / SQLite·Postgres site helpers + native plugin. **W0–W7 + P3 + W8** (middleware, CRUD+FTS, security, SEO/RSS, upload/gallery, RBAC, sitemap, favicon/head/images). **Browser embed (route D)**: `web.client_embed` auto-mounts WASM (`wasm` + `source` + `boot`) — zero author JS; build with `marqdo wasm build`; examples: examples/browser-hello/ · examples/web-client-site/. Design: doc/design/ext-web.md · capabilities: doc/design/web-net-capabilities.md · WASM: doc/adr/0002-browser-marqdo-wasm.md · roadmap D: doc/roadmap/browser-wasm-d.md · example: examples/marqdo-blog/

ext/quantum — circuits, draw, noise, formula matrix custom gates, Q7 density/viz, Q8 themed SVG (`theme=dark|light|bw`). Design: doc/design/ext-quantum.md · doc/design/ext-quantum-viz-style.md

Installer: marqdo ext list / add / remove — doc/design/ext-cli.md

Native plugins: lib/plugin + include/marqdo_abi.h — doc/design/ext-abi.md

> print text=ext-ok
