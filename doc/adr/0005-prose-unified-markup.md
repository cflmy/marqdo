# ADR 0005：叙述统一的 Markup 映射（粗体即代码 · 斜体即返回 · 形参体推断）

| | |
|---|---|
| Status | **Accepted** |
| Date | 2026-09-13 |
| Related | [markdown-mapping-v0.3.md](../design/markdown-mapping-v0.3.md) · [markdown-mapping.md](../design/markdown-mapping.md) · [return-hr-and-code-surface.md](../design/return-hr-and-code-surface.md) · [roadmap/markup-v0.3.md](../roadmap/markup-v0.3.md) |

## Context

- Marqdo 的产品承诺是「代码即文档 / 文档即代码」：`.mq.md` 打开是文章，运行是程序。
- v0.2（[markdown-mapping.md](../design/markdown-mapping.md)）为消歧做了硬隔离：无标记 = 注释；`*…*` = 语句；`**…**` = 返回；调用必须 `>`；形参必须 `` + `名` ``。
- 该隔离使叙述与程序**视觉割裂**；且与 AI/Markdown 习惯相反（模型偏好用 `**` 强调「要做的事」）。
- 产品方重新审视后，希望合法化如下写法：

```markdown
## 加一函数
对于输入变量`n`,我们执行操作**n=n+1**接着返回*n*。

# main
> 加一函数 37
或者可以直接这么调用**加一函数 37**
```

## Decision

1. 采纳 **Markup 映射 v0.3** 为下一版语言宪法草案，全文见 [markdown-mapping-v0.3.md](../design/markdown-mapping-v0.3.md)。
2. **角色对调**：`**…**` = 代码段（含可省略 `>` 的调用）；`*…*` = 返回值。
3. **叙述中的** `` `名` ``：**首次未赋值 = 声明变量**；未使用不报错；**仅当可执行面读取时升为形参**（见设计 §4.4 · §8.2）。不设「提及性」专用语法。
4. **形参默认由函数体/叙述推断**（升参）；`` + `名` `` 仅作迁移兼容。
5. **`>` 调用面保留**（可选）。
6. **空返回**用标记而非关键字：推荐 `---` / `***`；显式 `*None*` / `*无*`；整行 `**`；兼容 `****`（设计 §7.2）。
7. **不做 v0.2 双模式兼容**（摸索阶段允许破坏性变更）；旧 `.mq.md` 须改写。
8. 金样例门禁：`tests/markup-v03/`；存量 suite 随后迁移。

## Consequences

### Positive

- 叙述与代码共用 Markdown 习惯，文档可读性上升。
- 更贴合 AI 生成粗体「操作」的偏好，降低错用标记。
- 去掉形参表仪式，函数更像「带名字的说明段落」。

### Negative / Risks

- **破坏性**：与 v0.2 金样例、Skill、教程、大量 `lib/`/`ext/` 源码不兼容，必须分波迁移或双模式。
- 段内扫描与形参推断增加解析/语义复杂度——空返回与升参规则已在设计 §7.2 / §8.2 钉死；其余 §12 项用金样例收口。
- view/catalog/LSP 高亮规则需同步。

### Neutral

- `#` / `##` 对象体系、标准库、插件 ABI、关键字最小集不变。
- 输出仍是 `print` / `打印`，不是粗体/斜体。
