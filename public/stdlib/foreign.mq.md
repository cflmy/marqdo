---
title: lib/foreign — external languages
description: Run local interpreters from Marqdo (English)
> lib/foreign.mq.md
---

# main

Import lib/foreign.mq.md. Bind a code fence like a formula: empty-RHS assign then a ```lang block. The value type is code. Call run code=`name` to execute via your local interpreter. run_lang runs an inline source string.

`hi` =
```python
print("hello-from-python")
```

Default command is python (Windows) or python3 (Unix). Override with set_cmd, env MARQDO_FOREIGN_PYTHON, or the command box in live view. Failures ask you to check that configuration.

*`out` = > foreign.run code=`hi` *

> print text=`out`
