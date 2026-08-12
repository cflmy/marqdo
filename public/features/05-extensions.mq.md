---
title: Official extensions (ext/)
description: Optional packages — not stdlib
---

# main

ext/ holds official optional extensions. They are not part of lib/. Resolve via MARQDO_EXT, ./ext, or ext/ next to the binary.

- ext/llm — OpenAI-compatible chat object. Design: doc/design/ext-llm.md
- ext/agent — agent development framework (layout helpers today; compose with ext/llm for model-backed orchestration). Design: doc/design/ext-agent.md
- ext/quantum — state-vector circuits, `steps=` tables, `draw kind=circuit|probs|bloch`, named `gate` + `matches_matrix`, teaching `noise`, `barrier`/`measure`/`append`/`state`. Design: doc/design/ext-quantum.md · Q5: doc/design/ext-quantum-q5.md
- Installer: marqdo ext list / add / remove — doc/design/ext-cli.md
- Native plugins: lib/plugin + include/marqdo_abi.h — doc/design/ext-abi.md

> print text=ext-ok
