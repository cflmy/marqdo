---
markup: occlusion
title: 代码即文档，文档即知识 — OKF 与 EKC
description: Marqdo 如何以一份可执行源为唯一权威，把它投影成 OKF 风格知识与 Engineering Knowledge Compiler 制品
---

# main

Marqdo 的宪法是「代码即文档，文档即知识库」。`.mq.md` 不是程序截图：它同时是可读源码、可执行单元和知识页。

这个承诺只有在知识始终从源派生时才成立。OKF 提供可移植知识包的形态；EKC 提供工程化投影。二者都不成为第二份权威。

---

OKF 的形，Marqdo 的义。

OKF（Open Knowledge Format）用 Markdown + YAML frontmatter 表示知识：一页一概念、路径即身份、链接成图、强制面刻意保持很小。Marqdo 借用这个形态，但正文是可执行语法。

因此概念页可以直接指向证明它的文件：

```yaml
type: Capability
title: 持久外联内核
description: 在命名内核中运行 Python 或 Node 并保留状态
resource: public/features/19-外联视图单元.mq.md
status: stable
verified:
  by: marqdo-knowledge/verify
tags:
  - foreign
  - view-cells
```

同一页可以在编辑器里读、从 catalog 链接、被 Agent 查询，也可以直接运行。未知字段保留；未知类型降级为可读正文。概念页不是源文件的复制品。

---

哪些是生成的，哪些不是。

| 层 | 作者 | 职责 |
|---|---|---|
| 源 | 人或编码 Agent | .mq.md；唯一语义权威 |
| 目录包 | marqdo catalog | 模块、符号、能力与图投影 |
| 任务知识 | plan 晋升 | 任务、技能与可执行资源包 |
| 静态站 | marqdo view output | 人读 HTML，永远不是权威 |

生成物带 GENERATED 标记，可以删除重建；没有它们源树仍能运行。Catalog 是索引，不是语言特性。

---

EKC：Engineering Knowledge Compiler。

EKC 把仓库投影成工程知识系统，并在 Agent 写新代码前提供 preflight：

```text
.mq.md ──► EKC ──► catalog L0–L4
                 ├─ graph
                 ├─ context pack
                 └─ Agent preflight
```

| 层 | 知识 | 例子 |
|---|---|---|
| L0 Repository | 语言、规模、根目录 | 仓库总览 |
| L1 Module | imports、exports、职责 | 外联内核模块 |
| L2 Symbol | 签名、调用、指纹 | Python 分析函数 |
| L3 Capability | 工程能做什么 | 运行持久外联内核 |
| L4 Knowledge | 决策、约束、失败 | 为什么状态桥必须显式 |

图记录 implements、uses、depends_on、tested_by、documented_by、constrained_by、decided_by、supersedes、conflicts_with、deprecated_by 等关系。Agent 先沿图发现已有能力，而不是重新造轮子。

---

Engineering First Information：先发现，再创建。

复用协议是硬顺序：

```text
REUSE  →  ADAPT  →  CREATE
```

preflight 可以在实现开始前返回已有能力、API、约束、决策与失败经验。知识图缺失时返回明确的 missing_knowledge，而不是静默编造。

五条原则：

| # | 原则 |
|---|---|
| 1 | Discover before Create |
| 2 | Reuse before Adapt |
| 3 | Adapt before Duplicate |
| 4 | Verify before Promote |
| 5 | Code changes update knowledge |

---

命令。

```bash
# 把仓库投影成 catalog + EKC 制品
marqdo catalog .

# 写代码前先问：已经有了什么？
marqdo knowledge preflight "增加一个持久外联内核"
marqdo find "外联 会话"
marqdo reuse "实现 JSON 并行扇出"

# 看一次修改会影响什么
marqdo impact public/features/19-外联视图单元.mq.md

# 校验知识投影
marqdo knowledge verify
```

`marqdo catalog` 保留人读的 `index.md` 与模块页；EKC 在 `.marqdo/` 下增加 L0–L4。两者都从同一棵源树生成。

---

每个声称的能力都必须有可执行资源。

一个知识页说“这个工程能分析订单”，就必须指向一个真正能跑的 `.mq.md` 资源。这是 OKF 形态的笔记与可执行知识系统的区别。

在外联示例里，页面本身就是证明：Python 调用、持久内核和状态桥都从读者正在看的同一份 Markdown 源运行。没有另一个会漂移的演示项目。

---

继续阅读。

OKF 设计：[OKF 设计](../../doc/design/okf.md)

Engineering Knowledge Compiler：[Engineering Knowledge Compiler](../../doc/design/engineering-knowledge.md)

Catalog CLI：[Catalog CLI](../../doc/design/catalog-cli.md)

OKF 与 Marqdo 调研：[OKF 与 Marqdo 调研](../../doc/research/okf-and-marqdo.md)

外联 view 单元示例：[外联 view 单元示例](19-外联视图单元.mq.md)
