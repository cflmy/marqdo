---
title: Positional arguments
description: Bind by parameter order; named form still works
---

# main

Positional args bind in declaration order (no `who=` needed):

> greet Marqdo

Named form still works:

> greet who=World

## greet
    - who

> print text=Hello `who`!
