# 核心表面清单（core-surface）

| | |
|---|---|
| 状态 | **Accepted · 核心语法封闭集合（CI 守卫锁定）** |
| 日期 | 2026-09-22；2026-10-23 校正 v2.1 默认链接语义 |
| 守卫 | `tests/core_surface.rs` ⇄ `src/parse/mod.rs` `CORE_CONSTRUCTS`（改动任一侧不同步 → CI 红） |
| 相关 | [markdown-mapping-v0.3.md](markdown-mapping-v0.3.md)（语义宪法）· [layers.md](layers.md)（准入戒律）· [three-problems.md](../roadmap/three-problems.md) 问题一 · [three-problems-plan.md](../roadmap/three-problems-plan.md) T1.1/T1.3 |

> **戒律总纲**：核心标记是**封闭集合**（约 19 个构造）。新增任何一行必须走 ADR（见 [layers.md](layers.md) 准入戒律）并回答「为什么现有标记不够」；生态需求的默认答案是**进 ext**。本表 id 列与守卫测试强一致：改表不同步 `CORE_CONSTRUCTS`（或反之）、样例文件缺失，一律 CI 红。

> **版本边界**：构造 ID 是历史守卫标识，不是固定不变的语义说明。`link-index` ID 暂保留以匹配注册表，但默认 `[]()` 已是描述包裹 payload；旧索引语义仅属于显式 Classic 兼容路径。未来步骤/知识投影见[文档原生指导](document-native-development-guide.md)，不要把目标能力当作当前已实现。核心数量之外，还要审查上下文例外和作者负担。

## 1. 行构造

| id | 外形 | 语义 | 戒律 | 样例 |
|----|------|------|------|------|
| `narrative` | 无标记文字 | 叙述（不执行） | 段落内延续到空行；普通叙述不触发执行；未来知识引用投影也不改变此权限边界 | `tests/markup-v03/soft-emphasis.mq.md` |
| `blank` | 空行 | 段落边界 | 叙述分段的唯一手段；不得被赋予其它语义 | `tests/markup-v03/inc-prose.mq.md` |
| `heading-unit` | `#` / `##` | 对象 / 函数单元 | 标题即单元边界，按深度嵌套；正文禁止用 `##` 做版式（会变成函数） | `tests/markup-v03/decl-ident-only.mq.md` |
| `call-line` | `>` | 调用行 | 输出等副作用 = **普通函数**，不是架构标记 | `tests/markup-v03/dual-call.mq.md` |
| `branch-list` | 体内 `1.` 递增 | 分支 | 自 `1.` 起序号递增构成一个判断；重出 `1.` 开新判断；`N. *` = else；**不是所有有序列表都是 if** | `tests/markup-v03/greet-default.mq.md` |
| `loop-list` | `-` | 循环 | 条件形 `` - `条件` `` 或遍历形 `- [说明](遍历 集合 为 项)`；描述不是绑定变量名 | `tests/structure/loop.mq.md` |
| `param` | `` + `名` `` | 形参（兼容形） | 首选叙述 `` `名` `` 升参（宪法 §8）；`+` 形仅兼容保留 | `tests/markup-v03/greet-default.mq.md` |
| `table` | GFM 表 | 集合 | 几何即类型（[tables-maps-footnotes.md](../roadmap/tables-maps-footnotes.md)）；**绑定 = 数据，未绑定 = 文档表** | `tests/structure/collection.mq.md` |
| `fence` | ` ```lang ` | 外联 | 语言块原样外联，不进 Marqdo 语义 | `tests/lib/foreign-python.mq.md` |
| `hr` | 单行 `---` 或 `***` | 叙述分隔 | **不收束函数**（返回才收束） | `tests/markup-v03/module-hr.mq.md` |
| `writeback` | `<!-- marqdo-out … -->` | 持久化输出 | 只读回放，不执行 | `tests/lib/writeback-get.mq.md` |

## 2. 内联标记

| id | 外形 | 语义 | 戒律 | 样例 |
|----|------|------|------|------|
| `ident` | `` `名` `` | 声明 / 引用 | 首次未赋值 = 声明；叙述与程序共用同一视觉习惯 | `tests/markup-v03/decl-ident-only.mq.md` |
| `bold-code` | `**…**` | 代码（赋值 / 调用 / 表达式） | 粗体内代码判别序：赋值 → 调用 → 表达式；不取消调用行、返回等其它显式执行入口 | `tests/markup-v03/multi-bold.mq.md` |
| `italic-return` | `*…*` | 返回 | 改变控制流（**返回是架构**）；返回即结束函数体 | `tests/markup-v03/inc-prose.mq.md` |
| `empty-return` | `*None*` / `****` | 空返回 | 返回 `None` 并结束函数体 | `tests/markup-v03/empty-return-none.mq.md` |
| `link-index` | `[描述](表达式)` | 描述包裹 payload（历史 ID） | 默认 Occlusion；取元可写 `[说明](access(集合, 键))`，不按描述取键；循环载荷见上 | `tests/occlusion/access-and-loop.mq.md` |
| `bracket-call` | `修饰… [函数] 实参…` | 方括号标记调用 | 前置修饰 → `名=True`；紧邻 `](` 的默认语义为 payload 遮蔽，不是旧索引 | `tests/structure/bracket-call.mq.md` |
| `footnote-index` | `集合[^键]` | 兼容取元 | 保留既有能力；新代码优先 `access(...)`，需要解释时包裹描述，不再推荐旧 `[键](集合)` | `tests/structure/footnote-index.mq.md` |
| `soft-emphasis` | `*仅*` / `**说明**` | 忽略 | 非代码形强调**不执行**——默认安全的关键消歧规则 | `tests/markup-v03/soft-emphasis.mq.md` |

## 3. 词法大类（LineKind，`src/lex/mod.rs`）

行级入口只有四个大类：`Blank` / `Comment`（叙述段落，含软强调与行内 `` `名` ``）/ `Writeback` / `Code`（`# * > + - | ~ $` 或数字开头）。本清单第 1 节的行构造是在 `Code`（及 `Comment`/`Writeback`）内部的**语义细分**；第 2 节内联标记由 `src/parse/prose.rs` 行内扫描产出。两层合起来 = 核心表面全集。

## 4. 范围外（进 Language 层，但不在本守卫）

frontmatter / Artifact Metadata（元信息）、`import` 行（模块装载）、以及 Metadata 内的 **Binding Expression**（`${ns.name}`，见 [binding.md](binding.md) · [ADR 0006](../adr/0006-artifact-metadata-binding.md)）属于 Language 层的其它语言面，随 [layers.md](layers.md) 准入流程登记，**暂不进本清单守卫**（不增 CORE_CONSTRUCTS 计数）。Phase 2 将已绑定 metadata **提升为入口普通变量**（仍非新标记）。如需一并守卫，走 ADR 扩表。
