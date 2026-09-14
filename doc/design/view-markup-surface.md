# View：Markup 表面保真

| | |
|---|---|
| 状态 | **Active** |
| 日期 | 2026-09-14 |
| 相关 | [view.md](view.md) · [bracket-call-modifiers.md](bracket-call-modifiers.md) · [collection-access-link.md](collection-access-link.md) · [markdown-mapping-v0.3.md](markdown-mapping-v0.3.md) |

---

## 1. 目标

`marqdo view` 的 Structure / 表达式展示应**贴近作者写出的 Markup**，而不是内部 AST 的旧拼法。读者从 view 抄回源文件时，应仍是合法、推荐的表面。

## 2. 表面对照

| AST / 概念 | View 优先显示 | 备注 |
|------------|---------------|------|
| `Expr::Index` | `[键](基)` | 嵌套 `[k2]([k1](base))` |
| `InterpPart::Index` | 同上嵌套链接形 | 不再默认 `` `base`[^k] `` |
| `CallExpr` + `bracket_marked` / `pre_modifiers` | `修饰 [callee] 实参…` 或 `[callee] 实参…` | 仅经典 `>` 解析时仍显示 `> callee …` |
| 赋值 / 返回 | 粗体 / 斜体语义即可 | 不强制还原整句叙述 |
| foreach | `- [项](集合)` | 不变 |

## 3. 非目标

- 不把 view 做成第二语法；解析真相仍在 mapping / call-arguments。
- 不删除对脚注取元、旧 `>` 调用的兼容显示需求时，可用次要提示，但默认推荐面优先。

## 4. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-09-14 | 初稿；Index 链接形 + 括号调用修饰表面 |
| 2026-09-14 | `bracket_marked`：无修饰的 `[callee]` 也保真显示 |
