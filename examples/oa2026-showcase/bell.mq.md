---
markup: occlusion
title: 贝尔态：两个量子比特如何一起回答
description: 从一条假设开始，搭电路、看概率、画图，并用断言验收。
import:
  quantum: ext/quantum/circuit/index.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

贝尔态：一次可读的量子实验。

我们先做一个假设：如果对第一个量子比特施加 Hadamard 门，再用它控制第二个量子比特翻转，
那么测量结果不会出现「一个 0、一个 1」，只会出现「00」或「11」。

下面的表不是配置，而是实验步骤本身。读者先读实验设计，再读执行结果。

`步骤` =

| 步骤 | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

We first **电路 = [Compute 电路](quantum.circuit(qubits=2, steps=`步骤`))** for the next step.
We first **概率 = [Compute 概率](> `电路`.probabilities)** for the next step.

We now **[Display this step's result](> print text=`概率`)**.

---

我们期待什么？如果假设成立，`概率`应该只在「00」与「11」上有值，而且各占一半。
先把两个关键概率取出来，读者不必在长映射里找：

We first **p00 = [00](access(`概率`, "00"))** for the next step.
We first **p11 = [11](access(`概率`, "11"))** for the next step.

We now **[Display this step's result](> print text=`p00`)**.
We now **[Display this step's result](> print text=`p11`)**.

下面的断言不是装饰：如果电路写错，或者门接错，运行会直接失败。

1. `p00` > 0.49
  We now **[Display this step's result](> print text="bell-00-ok")**.
2. *
  We now **[Display this step's result](> print text="bell-00-fail")**.

1. `p11` > 0.49
  We now **[Display this step's result](> print text="bell-11-ok")**.
2. *
  We now **[Display this step's result](> print text="bell-11-fail")**.

---

图不是截图，而是这份文档运行后生成的派生物。电路图和概率图都来自同一个`电路`。

We first **_ = [Compute _](> `电路`.draw path="bell-circuit.svg" theme="light")** for the next step.
We first **_ = [Compute _](> `电路`.draw kind="probs" path="bell-probs.svg" theme="light")** for the next step.

We now **[Display this step's result](> print text="bell-draw-ok")**.
