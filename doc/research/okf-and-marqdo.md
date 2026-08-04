# 调研：Google Open Knowledge Format（OKF）与 Marqdo

| | |
|---|---|
| 状态 | 调研笔记 |
| 日期 | 2026-08-04 |
| 对象 | [OKF v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) |
| 公告 | [v0.1 介绍](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing) · [v0.2 trust signals](https://cloud.google.com/blog/products/data-analytics/okf-v0-2-adds-trust-signals) |

---

## 1. OKF 是什么

**Open Knowledge Format（开放知识格式）** 是 Google Cloud 于 2026 年 6 月发布的厂商中立开放规范（Apache 2.0）。它把「LLM Wiki」模式形式化为可移植格式：用**目录树中的 Markdown 文件 + YAML frontmatter** 表示组织知识（表结构、指标、API、操作手册等），供人类与 AI Agent 共同阅读、交换与维护。

一句话定位：

> 知识应活在**格式**里，而不是专有服务或散落的非结构化文本里。

### 核心形态

```
bundle/
  index.md                 # 可选：目录索引（渐进披露）
  log.md                   # 可选：变更历史
  concepts/.../*.md        # 每个概念一个 Markdown 文件
```

每个概念文件：

```yaml
---
type: Metric                 # 唯一必填
title: Monthly Active Users
description: ...
tags: [product, growth]
sources: [...]               # v0.2：出处
generated: {...}             # v0.2：谁生成的
verified: {...}              # v0.2：谁验证的
status: stable               # v0.2：生命周期
stale_after: 2026-12-01      # v0.2：过期日
---

# 正文（自由 Markdown）

指标定义为……
```

- **Concept ID** = 相对路径去掉 `.md`
- **链接** = 普通 Markdown 链接，形成可遍历关系图
- **无中心 schema registry**；未知 `type` 必须被优雅降级处理

### 设计三原则（官方表述）

1. **Minimally opinionated** — 只强制互通所需的最小公约；其余留给生产者。
2. **Producer / consumer independence** — 写知识的人（或 Agent）与读知识的 Agent/工具解耦；格式即契约。
3. **Format, not platform** — 不绑定云、数据库、模型或 Agent 框架；`cat` 能读，`git clone` 能分发。

### v0.2 新增：五类信任信号

当知识主要由 Agent **持续生成**时，纯 Markdown 不够。v0.2 把下列问题变成 frontmatter 一等字段：

| 问题 | 机制 |
|------|------|
| 从何而来？ | `sources`（出处 + 可信度信号，不作统一打分） |
| 该信多少？ | `generated` / `verified` → 推导 trust tier（未验证 / 机器确认 / 人工审核） |
| 是否仍真？ | `stale_after`（绝对日期，非相对 TTL） |
| 是否现行？ | `status`: `draft` → `stable` → `deprecated` |
| 数字是否按约定算出？ | 概念类型 `Attested Computation`：executor 跑计算出 receipt，**非 LLM** 的 attester 核验 |

---

## 2. OKF 与 Marqdo：相邻，但不等同

| 维度 | OKF | Marqdo |
|------|-----|--------|
| 本质 | **知识交换格式** | **编程语言** |
| 文件角色 | 描述资产与概念的文档 | 可执行程序 = 可渲染文档 |
| 机器侧 | 解析 metadata、遍历链接、过滤信任 | 解析、类型检查、求值/编译 |
| Markdown 地位 | 载体与呈现层 | **语法表层的一部分**（源即 Markdown） |
| 成功标准 | 跨组织互通、Agent 可策展 | 严肃编程能力 + 阅读体验同时成立 |

OKF 回答：「人和 Agent 如何**共享与信任知识**？」  
Marqdo 回答：「程序如何**本身就是知识**，且仍能运行？」

因此借鉴应落在**表示哲学与工程约束**，而不是照搬「知识目录」产品形态。

---

## 3. 值得借鉴的设计点

### 3.1 后缀与生态搭便车 — 已采纳

OKF 坚持「就是 Markdown」：GitHub 渲染、编辑器预览、静态站、搜索索引零摩擦。

**Marqdo 对应决策：源文件后缀为 `.mq.md`。**

- `.md` 确保通用 Markdown 渲染器开箱可用
- `.mq` 前缀声明这是 Marqdo 源，而非普通文档
- 与 OKF「格式优先于平台」一致：可读性不依赖自研 viewer

### 3.2 最小强制面（Minimally opinionated）

OKF 只强制 `type`；其余可选，未知字段保留不拒绝。

**对 Marqdo 的启发：**

- 语言的「文档面」与「程序面」之间，划清**硬语法**与**散文**
- 第一版语法宜小：标题层级、声明、表达式……先钉死互通/可执行最小集
- 扩展用约定或可选 frontmatter，避免一上来造完整注解宇宙

### 3.3 生成式 YAML / OKF 清单 — 已采纳方向（已修正）

OKF 用 YAML frontmatter 承载「可查询元数据」。Marqdo **借鉴其形态**，但职责不同：

**YAML（OKF 风格知识包）由工具从 `.mq.md` 自动生成，不是人手写的配置文件。**

详见 [`design/generated-yaml-manifest.md`](../design/generated-yaml-manifest.md)。

| | |
|--|--|
| 类比 | 其它语言中由工具维护的依赖描述（如 `Cargo.toml` / `requirements.txt` 所承担的那类角色） |
| 形态 | OKF：概念目录 + YAML frontmatter + Markdown 正文 |
| 用途 | ① 编译器/包管理消费依赖图与导出 ② **自动生成本项目文档**（模块索引、依赖关系等） |
| 权威 | 始终是手写 `.mq.md`；清单可删、可重建 |

```
.mq.md 源（人写）
    → marqdo 工具链
    → 生成 OKF 风格清单（YAML + 概念页）
    → 供编译解析，并渲染为项目目录文档
```

执行方式等运行参数不放进该 YAML 当配置；由 CLI / 子命令决定。  
程序本体（类型、函数）只活在源正文里。

### 3.4 路径即身份，链接即关系

Concept ID = 路径；关系 = Markdown 链接；目录 = 渐进披露（`index.md`）。

**对模块系统的启发：**

- 文件路径可成为模块身份的一部分（类比 OKF concept ID）
- `import` / 交叉引用尽量与 Markdown 链接同构，使「读文档跳转」与「解析依赖」共用一种边
- 大型程序按「书的章节」组织目录，而不是扁平包名墙

### 3.5 生产者 / 消费者分离

同一 bundle：人手写、Agent 生成、管道导出，均可被另一 Agent 或 UI 消费。

**对 Marqdo 工具链：**

- 解析器、渲染器、求值器、LSP 都是**消费者**；源格式是唯一契约
- 允许 Agent 协助编写 `.mq.md`，但语言必须对「纯手写」同样友好
- 不把语义锁进某一 IDE 或云服务

### 3.6 信任与 attestation（中长期）

OKF v0.2 的 Attested Computation：批准的算法 + 确定性核验，防止 Agent「随口报数」。

**远期可借鉴（非 v0 范围）：**

- 文档中嵌入的「可运行示例」是否经过核验执行？
- 渲染出的结果是否带来源与 receipt？
- Agent 生成的 Marqdo 模块如何标注 `generated` / `verified`？

这强化「文档中的数字/行为与真实执行一致」——与 Marqdo「同一份真相」高度同向。

### 3.7 规范本身的克制

OKF 规范强调：一页纸能讲清 v0.1 的互通面；v0.2 只**加词汇、不加强制规则**。

**对 Marqdo 文档文化：**

- ADR 记录否决项
- 语法草案保持短小、可验证的 conformance 描述
- 版本演进优先加法兼容，避免随意破坏「已是有效程序」的源文件

---

## 4. 不应照搬之处

1. **OKF 不是语言** — 其 body 是自由散文；Marqdo body 必须有严格可执行语义。不能停在「Markdown 知识库」。
2. **无中心 type 注册** 适合知识目录；编程语言的类型系统需要更强的一致性与检查规则。
3. **`Attested Computation` 的 executor/attester** 面向数据平台查询核验；Marqdo 若引入类似机制，应长在「测试 / 示例求值 / 再现构建」上，而不是 BigQuery 收据模型。
4. **保留文件名**（`index.md` / `log.md`）可参考，但是否占用需在模块设计里单独 ADR，避免与源文件约定冲突。

---

## 5. 建议纳入 Marqdo 设计议程的议题

来自本次调研、建议在 `design/` 或 `adr/` 中跟进：

1. **`.mq.md` 的双重解析模型** — Markdown CST 与 Marqdo AST 如何共享同一文本、何处分叉。
2. **生成式 OKF 清单的落盘与命令** — 路径、是否提交 git、`sync`/`doc` 行为（方向已采纳，见 [design/generated-yaml-manifest.md](../design/generated-yaml-manifest.md)）。
3. **正文中的依赖声明语法** — import / 链接如何写出，供工具提取进生成清单。
4. **Agent 友好但不 Agent 绑定** — 源格式对 LLM 可写，语义对确定性工具可检。
5. **可渲染结果的 attestation（后期）** — 文档内求值结果的可验证性。

---

## 6. 参考链接

- 规范：[okf/SPEC.md](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)（v0.2）
- 仓库：[GoogleCloudPlatform/knowledge-catalog](https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf)
- 博客：[Introducing OKF](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing)
- 博客：[OKF v0.2 trust signals](https://cloud.google.com/blog/products/data-analytics/okf-v0-2-adds-trust-signals)

---

## 7. 结论

OKF 证明了一条与 Marqdo 同向的产业判断：**人类可读的 Markdown 可以成为机器与 Agent 的一等交换面**，前提是约定足够小、足够稳，并把信任做成可查询字段而非口头保证。

Marqdo 比 OKF 多走一步：Markdown 不仅是知识的容器，更是**程序的表面语法**。我们借鉴 OKF 的形态，用作**从源自动生成的依赖清单与项目目录文档**（而非人手配置）；同时借鉴「格式而非平台」、路径/链接身份、以及（远期）provenance。语言设计必须单独完成——**可渲染不等于可执行，可执行才是 Marqdo 的终点。**
