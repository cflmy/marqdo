---
title: Engineering Knowledge demo
description: Tiny project demonstrating Capability Catalog + reuse + decisions.
---

# Engineering Knowledge demo

This folder is a miniature “large engineering” slice: config resolution already exists.
Before generating a new loader, run:

```bash
marqdo knowledge examples/engineering-knowledge -o examples/engineering-knowledge/.marqdo
marqdo reuse "load and resolve configuration" examples/engineering-knowledge -o examples/engineering-knowledge/.marqdo
marqdo duplicate examples/engineering-knowledge -o examples/engineering-knowledge/.marqdo
```

Expected: **REUSE config.resolve**, not a new `load_configuration`.

# main

* "see README via catalog/reuse CLI"
