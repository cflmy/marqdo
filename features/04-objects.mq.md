---
title: Objects and methods
description: Sharp heading is a type; deeper headings are functions or methods
---

# main

In Marqdo, a single sharp (#) defines an object / type constructor. Double sharp and deeper (## …) define free functions or methods. Constructing an object yields a map handle tagged with _type. Call methods with backtick object, then dot, then method name.

See doc/design/objects.md. Example gold: tests/structure/object-handle.mq.md. Official ext/llm and ext/agent use this pattern.

> print text=objects-ok
