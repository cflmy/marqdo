# 标准库 / 扩展库：代码即文档

| | |
|---|---|
| 状态 | **Active** |
| 日期 | 2026-09-14 |
| 相关 | [code-as-docs-gaps.md](code-as-docs-gaps.md) · [bracket-call-modifiers.md](bracket-call-modifiers.md) · [collection-access-link.md](collection-access-link.md) · [stdlib.md](stdlib.md) · [marqdo-dev skill](../../.cursor/skills/marqdo-dev/SKILL.md) · [roadmap](../roadmap/lib-ext-code-as-docs.md) |

---

## 1. 目标

`lib/*` 与 `ext/*` 的 `.mq.md` 应**可当说明书读、可当程序跑**。薄库不写长文；厚领域库（web / quantum / …）仍用叙述承载意图，但数据与调用面与用户程序同一套标记。

## 2. 薄库模板（默认）

每个导出函数 / 方法尽量按此骨架（中英分文件锁步）：

1. **目的句**（一句人话，无标记即可）。
2. **叙述形参**：正文里出现 `` `名` `` / `` `名`=默认 ``（推断签名）；过渡期可保留 `` + `名` ``。
3. **一例**：粗体或 `>` 调用；优先可读表面。
4. **实现**：短；取元用 `[键](集合)`，**禁止新增**脚注取元 `` `m`[^k] ``（旧解析仍可用）。

调用示例（三种等价，试点期任选；新叙述优先括号标记或粗体）：

```markdown
**出 = [put] 入=`表` 键="a" 值=1**

**出 = put 入=`表` 键="a" 值=1**

> put 入=`表` 键="a" 值=1
```

带布尔开关时可用前置修饰：`严格 [parse] text=`源`` → `严格=True`（形参名与修饰语字面一致；核心不剥「的」等词缀）。

## 3. 硬约束

| 要 | 不要 |
|----|------|
| GFM 表作数据；`table.put` / 具名助手 | `json.set` 链当字典构造器 |
| `[k](c)` 取元；`- [项](集合)` foreach | 新代码写 `` m[^k] `` |
| 中英 API 对称（见 [stdlib-i18n.md](stdlib-i18n.md)） | 只改一侧 |
| 一例跑通即文档 | 另开第二套「文档站」手写说明 |

## 4. 波次

见 [roadmap/lib-ext-code-as-docs.md](../roadmap/lib-ext-code-as-docs.md)：先稳定调用表面，再 M1 试点 table/text/json，其后批次 lib → ext。

## 5. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-09-14 | 初稿（随括号调用落地） |
