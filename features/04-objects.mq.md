---
markup: occlusion
title: Objects and methods
description: Sharp heading is a type; deeper headings are functions or methods
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

In Marqdo, a single sharp (#) defines an object / type constructor. Double sharp and deeper (## …) define free functions or methods. Constructing an object yields a map handle tagged with _type. Call methods with backtick object, then dot, then method name.

See doc/design/objects.md. Example gold: tests/structure/object-handle.mq.md. Official ext/llm and ext/agent use this pattern.

We now **[Display this step's result](> print text="objects-ok")**.
