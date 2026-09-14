# Roadmap：lib / ext 代码即文档

| | |
|---|---|
| 状态 | **Active** |
| 日期 | 2026-09-14 |
| 相关 | [lib-code-as-docs.md](../design/lib-code-as-docs.md) · [view-markup-surface.md](../design/view-markup-surface.md) · [bracket-call-modifiers.md](../design/bracket-call-modifiers.md) · [collection-access-link.md](../design/collection-access-link.md) |

---

## 0. 总序（防返工）

1. **方括号标记调用**（设计 + AST + 金样）— **已落地**
2. lib / view 设计文档 — **已写**
3. View 表面（Index 链接形、括号调用）— **已落地**
4. 分波改造 `lib/*` → `ext/*`

旧 `>` / 粗体调用、脚注取元解析均保留。

## 1. 里程碑

| 波 | 内容 | 状态 |
|----|------|------|
| **Call** | `pre_modifiers`；`tests/structure/bracket-call.mq.md`；mapping / call-arguments | **done** |
| **M0 View** | `InterpPart::Index` → `[k](base)`；`call_display` 修饰表面 | **done** |
| **M1** | 试点 `lib/table` · `lib/text` · `lib/json`（中英）+ public/stdlib | **done** |
| **M2** | 其余 `lib/*` 叙述化；禁新脚注取元 | **done** |
| **M3–M4** | `ext/web`、`ext/ai`：脚注→链接取元；叙述；示例可逐步换括号调用 | pending |
| **M5** | quantum / linalg + full gold | pending |

## 2. 验收（出门）

- 括号调用与 `[k](c)` 取元金样并存、不回归。
- 试点库可读：目的句 + 叙述形参 + 一例 + 短实现。
- `cargo test --test gold` 绿。

## 3. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-09-14 | 初稿；Call + M0 标 done |
| 2026-09-14 | M1 试点 table/text/json + public/stdlib 标 done |
| 2026-09-14 | M2 其余 lib 叙述化标 done |
