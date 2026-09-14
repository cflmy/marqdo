---
title: lib/toml
description: Read-only TOML subset parse (Mid2 M8). Tables, scalars, arrays; no array-of-tables.
---

Read-only TOML subset. Prefer for config files; not a full TOML writer.

## parse
    + `text`

Parse TOML text into maps/lists/scalars.

Caller: [toml.parse] text=`cfg`

*> host_toml_parse text=`text`*
