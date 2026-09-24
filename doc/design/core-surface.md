# 核心表面清单（core-surface）

| | |
|---|---|
| 状态 | **Accepted · 核心语法封闭集合（CI 守卫锁定）** |
| 日期 | 2026-09-22 |
| 守卫 | `tests/core_surface.rs` ⇄ `src/parse/mod.rs` `CORE_CONSTRUCTS`（改动任一侧不同步 → CI 红） |
| 相关 | [markdown-mapping-v0.3.md](markdown-mapping-v0.3.md)（语义宪法）· [layers.md](layers.md)（准入戒律）· [three-problems.md](../roadmap/three-problems.md) 问题一 · [three-problems-plan.md](../roadmap/three-problems-plan.md) T1.1/T1.3 |

> **戒律总纲**：核心标记是**封闭集合**（约 19 个构造）。新增任何一行必须走 ADR（见 [layers.md](layers.md) 准入戒律）并回答「为什么现有标记不够」；生态需求的默认答案是**进 ext**。本表 id 列与守卫测试强一致：改表不同步 `CORE_CONSTRUCTS`（或反之）、样例文件缺失，一律 CI 红。

## 1. 行构造

| id | 外形 | 语义 | 戒律 | 样例 |
|----|------|------|------|------|
| `narrative` | 无标记文字 | 叙述（注释） | 段落内延续到空行；**默认安全**——永远不存在「叙述被执行」的路径 | `tests/markup-v03/soft-emphasis.mq.md` |
| `blank` | 空行 | 段落边界 | 叙述分段的唯一手段；不得被赋予其它语义 | `tests/markup-v03/inc-prose.mq.md` |
| `heading-unit` | `#` / `##` | 对象 / 函数单元 | 标题即单元边界，按深度嵌套；正文禁止用 `##` 做版式（会变成函数） | `tests/markup-v03/decl-ident-only.mq.md` |
| `call-line` | `>` | 调用行 | 输出等副作用 = **普通函数**，不是架构标记 | `tests/markup-v03/dual-call.mq.md` |
| `branch-list` | 体内 `1.` 递增 | 分支 | 自 `1.` 起序号递增构成一个判断；重出 `1.` 开新判断；`N. *` = else；**不是所有有序列表都是 if** | `tests/markup-v03/greet-default.mq.md` |
| `loop-list` | `-` | 循环 | 条件形 `` - `条件` `` 或遍历形 `- [项](集合)` | `tests/structure/loop.mq.md` |
| `param` | `` + `名` `` | 形参（兼容形） | 首选叙述 `` `名` `` 升参（宪法 §8）；`+` 形仅兼容保留 | `tests/markup-v03/greet-default.mq.md` |
| `table` | GFM 表 | 集合 | 几何即类型（[tables-maps-footnotes.md](../roadmap/tables-maps-footnotes.md)）；**绑定 = 数据，未绑定 = 文档表** | `tests/structure/collection.mq.md` |
| `fence` | ` ```lang ` | 外联 | 语言块原样外联，不进 Marqdo 语义 | `tests/lib/foreign-python.mq.md` |
| `hr` | 单行 `---` 或 `***` | 叙述分隔 | **不收束函数**（返回才收束） | `tests/markup-v03/module-hr.mq.md` |
| `writeback` | `<!-- marqdo-out … -->` | 持久化输出 | 只读回放，不执行 | `tests/lib/writeback-get.mq.md` |

## 2. 内联标记

| id | 外形 | 语义 | 戒律 | 样例 |
|----|------|------|------|------|
| `ident` | `` `名` `` | 声明 / 引用 | 首次未赋值 = 声明；叙述与程序共用同一视觉习惯 | `tests/markup-v03/decl-ident-only.mq.md` |
| `bold-code` | `**…**` | 代码（赋值 / 调用 / 表达式） | 只有粗体进入执行；判别序：赋值 → 调用 → 表达式 | `tests/markup-v03/multi-bold.mq.md` |
| `italic-return` | `*…*` | 返回 | 改变控制流（**返回是架构**）；返回即结束函数体 | `tests/markup-v03/inc-prose.mq.md` |
| `empty-return` | `*None*` / `****` | 空返回 | 返回 `None` 并结束函数体 | `tests/markup-v03/empty-return-none.mq.md` |
| `link-index` | `[键](集合)` | 取元 / foreach | **主取元语法**；与 bracket-call 靠 `]` 后是否紧邻 `(` 消歧 | `tests/lib/table-collections.mq.md` |
| `bracket-call` | `修饰… [函数] 实参…` | 方括号标记调用 | 前置修饰 → `名=True`；与 link-index 消歧同上 | `tests/structure/bracket-call.mq.md` |
| `footnote-index` | `集合[^键]` | 过渡取元 | **仅兼容**；新代码一律写 link-index | `tests/structure/footnote-index.mq.md` |
| `soft-emphasis` | `*仅*` / `**说明**` | 忽略 | 非代码形强调**不执行**——默认安全的关键消歧规则 | `tests/markup-v03/soft-emphasis.mq.md` |

## 3. 词法大类（LineKind，`src/lex/mod.rs`）

行级入口只有四个大类：`Blank` / `Comment`（叙述段落，含软强调与行内 `` `名` ``）/ `Writeback` / `Code`（`# * > + - | ~ $` 或数字开头）。本清单第 1 节的行构造是在 `Code`（及 `Comment`/`Writeback`）内部的**语义细分**；第 2 节内联标记由 `src/parse/prose.rs` 行内扫描产出。两层合起来 = 核心表面全集。

## 4. 范围外（进 Language 层，但不在本守卫）

frontmatter / Artifact Metadata（元信息）、`import` 行（模块装载）、以及 Metadata 内的 **Binding Expression**（`${ns.name}`，见 [binding.md](binding.md) · [ADR 0006](../adr/0006-artifact-metadata-binding.md)）属于 Language 层的其它语言面，随 [layers.md](layers.md) 准入流程登记，**暂不进本清单守卫**（不增 CORE_CONSTRUCTS 计数）。Phase 2 将已绑定 metadata **提升为入口普通变量**（仍非新标记）。如需一并守卫，走 ADR 扩表。
