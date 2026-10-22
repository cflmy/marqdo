---
title: 贝尔态：两个量子比特如何一起回答
description: 从一条假设开始，搭电路、看概率、画图，并用断言验收。
import quantum:ext/quantum/quantum.mq.md
---

# main

贝尔态：一次可读的量子实验。

我们先做一个假设：如果对第一个量子比特施加 Hadamard 门，再用它控制第二个量子比特翻转，
那么测量结果不会出现「一个 0、一个 1」，只会出现「00」或「11」。

下面的表不是配置，而是实验步骤本身。读者先读实验设计，再读执行结果。

`步骤` =

| 步骤 | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

**`电路` = > quantum.circuit qubits=2 steps=`步骤`**
**`概率` = > `电路`.probabilities**

> print text=`概率`

---

我们期待什么？如果假设成立，`概率`应该只在「00」与「11」上有值，而且各占一半。
先把两个关键概率取出来，读者不必在长映射里找：

**`p00` = [00](概率)**
**`p11` = [11](概率)**

> print text=`p00`
> print text=`p11`

下面的断言不是装饰：如果电路写错，或者门接错，运行会直接失败。

1. `p00` > 0.49
  > print text=bell-00-ok
2. *
  > print text=bell-00-fail

1. `p11` > 0.49
  > print text=bell-11-ok
2. *
  > print text=bell-11-fail

---

图不是截图，而是这份文档运行后生成的派生物。电路图和概率图都来自同一个`电路`。

**`_` = > `电路`.draw path="bell-circuit.svg" theme="light"**
**`_` = > `电路`.draw kind="probs" path="bell-probs.svg" theme="light"**

> print text=bell-draw-ok
