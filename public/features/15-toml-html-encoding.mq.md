---
title: Mid stdlib — toml, html, base32
description: Config TOML + HTML escape + base32 (Mid2 M8)
import toml:lib/toml.mq.md
import html:lib/html.mq.md
import enc:lib/encoding.mq.md
---

# Config and safe text without plugins

Parse a TOML subset for tools, escape HTML before assembly, encode with base32.

# main

*cfg = > toml.parse text="app = \"demo\"\n[server]\nport = 8080\n"*
> print text=`cfg`[^app]
*srv = `cfg`[^server]*
> print text=`srv`[^port]

*safe = > html.escape text="<tag>"*
> print text=`safe`

*b = > enc.base32_encode text="ok"*
> print text=`b`
