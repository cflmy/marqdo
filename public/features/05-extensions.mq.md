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

- ext/llm — OpenAI-compatible chat. Design: doc/design/ext-llm.md
- ext/agent — agent framework + native plugin. Design: doc/design/ext-agent.md
- ext/web — HTTP / SQLite site helpers + native plugin. Design: doc/design/ext-web.md
- ext/quantum — circuits, draw, noise, formula `matrix=` custom gates. Design: doc/design/ext-quantum.md
- Installer: marqdo ext list / add / remove — doc/design/ext-cli.md
- Native plugins: lib/plugin + include/marqdo_abi.h — doc/design/ext-abi.md

> print text=ext-ok
