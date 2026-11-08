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
| `Expr::Occluded` | 正文只读动作说明，实际实现进入侧注 | 每个片段独立关联，不把整句改成一次赋值 |
| 空遮蔽 `()` | 待补充实际逻辑 | 当前空遮蔽返回 None，不伪装成已接通工具 |
| `Expr::Index` / `InterpPart::Index` | 保留经典或脚注取元的兼容说明 | 当前作者程序优先用 access 读取集合；默认链接形是遮蔽 |
| `CallExpr` + `bracket_marked` / `pre_modifiers` | `修饰 [callee] 实参…` 或 `[callee] 实参…` | 仅经典 `>` 解析时仍显示 `> callee …` |
| 赋值 / 返回 | 保留原叙述句子，在真实执行片段上标明语句或返回 | 普通链接与字面量不冒充执行遮蔽 |
| foreach | `- [动作](遍历 集合 为 项)` | 正文显示动作，控制逻辑见侧注；迭代选择器只切换已有运行记录 |

## 3. 非目标

- 不把 view 做成第二语法；解析真相仍在 mapping / call-arguments。
- 不删除对脚注取元、旧 `>` 调用的兼容显示需求时，可用次要提示，但默认推荐面优先。

当前阅读器的实现与验收见 [view-readable-actions.md](view-readable-actions.md)。
可执行知识先展示输入、返回、调用边界、依赖和各项依据；摘要和机器身份按需展开。
源码说明与运行标注分开，展开侧注或知识卡片不会执行代码。

## 4. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-09-14 | 初稿；Index 链接形 + 括号调用修饰表面 |
| 2026-09-14 | `bracket_marked`：无修饰的 `[callee]` 也保真显示 |
