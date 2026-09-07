---
title: lib/cli
description: Parse argv flags (Mid M5). --key value / --key=value / --flag; positionals under _.
---

## parse
    + `args`=None

Parse `--key value` / `--key=value` / `--flag`. A bare `--flag` must not be followed by a non-option token if you need it as boolean (put flags last, or use `--flag=true`). Positionals land under `_`.

**> host_cli_parse args=`args`**
