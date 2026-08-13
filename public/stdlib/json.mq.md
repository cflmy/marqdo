---
title: lib/json — JSON
description: Shared EN/ZH path; parse and stringify
import json:lib/json.mq.md
---

# main

Import lib/json.mq.md from both English and Chinese docs (no translated twin). Functions: parse(text), stringify(value), get(value, key), keys(value), quote(text) — JSON string literal with quotes (for building request bodies). Objects become the map runtime type.

*`obj` = > json.parse text={} *

*`ty` = > type `obj` *

> print text=`ty`

*`out` = > json.stringify value=`obj` *

> print text=`out`
