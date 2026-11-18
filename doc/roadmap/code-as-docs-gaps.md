# 路线图：代码即文档叙述面摩擦（G1–G4）

| | |
|---|---|
| 状态 | **Done**（实现已合入工作树；金样绿） |
| 日期 | 2026-09-14 |
| 设计 | [code-as-docs-gaps.md](../design/code-as-docs-gaps.md) |
| 样例 | `examples/code-as-docs/index.mq.md` |

---

## 里程碑

### M1 — 文档与金样夹具

- [x] 设计文档记录 G1–G4 与启发式  
- [x] 本路线图  
- [x] `tests/markup-v03/` 四缝金样 + `gold.rs` `markup_v03_*`

### M2 — G1 模块级 `---`

- [x] `parse_classified`：顶层 frame 行 skip；顶层 Comment 引言 skip  
- [x] 函数体内 `---` / `***` 改为叙述跳过（不再收束；空返回收束）  
- [x] 金样：`module-hr.mq.md`

### M3 — G3 浮点面量与运算

- [x] `Literal::Num(f64)`  
- [x] `parse_primary` 识别 `digits.digits`（及 `.5` / `3.` 形态）  
- [x] `eval_binary` + bytecode Int/Num 提升（Int/Int `/` 仍为整除）  
- [x] 金样：`float-discount.mq.md`

### M4 — G2 叙述强调启发式

- [x] `scan_prose_line`：非代码形粗体、非返回形行内斜体 → 丢弃  
- [x] 保持 `**n=n+1**`、`**加一 37**`、整行 `*n*`、「返回*n*」  
- [x] 更新 Skill：允许装饰性强调；可执行仍看形状  
- [x] 金样：`soft-emphasis.mq.md`

### M5 — G4 动态脚注键

- [x] AST：`IndexKey::{Lit,Var}`（`` [^`ident`] ``）  
- [x] 解析、求值、bytecode  
- [x] 金样：`dynamic-footnote.mq.md`  
- [x] 回写 `examples/code-as-docs`（浮点折扣 + 动态键）

### M6 — 出门

- [x] `examples/code-as-docs` 展示四缝修复后的写法  
- [x] `cargo test --test gold` 绿  
- [x] 设计验收覆盖（见金样）  

---

## 顺序建议

G1（小）→ G3（中，Literal/运算）→ G2（启发式，依赖金样钉行为）→ G4（AST 面）→ 回写样例。

---

## 修订历史

| 日期 | 说明 |
|------|------|
| 2026-09-14 | 初稿 |
| 2026-09-14 | 实现完成并勾选 |
