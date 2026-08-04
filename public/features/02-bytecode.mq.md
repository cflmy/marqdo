---
title: 字节码后端
description: --backend bytecode 与树遍历共用语义
---

# main

里程碑：字节码原型。

同一套 .mq.md 可用树遍历或字节码 VM 执行。命令示例：

  marqdo run FILE --backend tree

  marqdo run FILE --backend bytecode

下面这段在两种后端下输出一致：

> print text=Bytecode backend：same program, alternate engine.
