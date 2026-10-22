# 变更记录：M2 遮蔽语法 `[描述](实现)`

| | |
|---|---|
| 状态 | **Implemented**（模式开关默认仍是 classic） |
| 日期 | 2026-10-22 |
| 里程碑 | M2（[occlusion-syntax.md](occlusion-syntax.md) §13） |
| 改动文件 | `src/ast.rs` · `src/parse/expr.rs` · `src/parse/mod.rs` · `src/interp.rs` · `src/bytecode/compile.rs` · `src/view/render.rs` · `src/main.rs` · `tests/occlusion.rs` |
| 影响面 | 语法（新节点）+ 两个后端 + 渲染 |

---

## 1. 新增语法

```markdown
[描述](实现)
```

- `[]` = **描述**（自由文本，**永不执行**；仅用于检索、统计、过期校验）
- `()` = **实现**（唯一执行体）
- `[]` 为空 → `label.missing`（error，M2.5 落诊断）
- `()` 为空 → 值为 `None`（空载荷桩，M2.5 落 warning）

实测（`MARQDO_MARKUP=occlusion`）：

```
a=2      [加一之后的值](1 + 1)
b=42     [外层]([内层](40 + 2))       ← 嵌套遮蔽
c=True   [某某功能开发中]()             ← 空载荷 → None
```

---

## 2. 关键决策：**模式开关，暂不切换语义**

`[键](集合)` 与 `[描述](实现)` **外形完全相同**。若 M2 直接改语义，全仓 506 处取元
会**静默变成"遮蔽整个集合"** —— 这正是文档里点名的最坏一类变更（同形换义）。

因此 M2 只**实现能力**，不动默认：

| 模式 | `MARQDO_MARKUP` | `[a](b)` 的语义 |
|------|-----------------|-----------------|
| `Classic`（默认） | 未设 / 其他值 | 取元（今天的行为） |
| `Occlusion`（纯度） | `occlusion` / `purity` / `1` / `on` | 遮蔽（描述 + 实现） |

**M7 才迁移语料并翻转默认**，同时对旧形报 `syntax.legacy_index` 硬错误。
这样 M2 可独立验证，且不会出现"半迁移的静默错义"。

---

## 3. 实现要点

### 3.1 AST

```rust
Occluded {
    label: String,
    payload: Option<Box<Expr>>,   // None = 空载荷桩（值 None）
}
```

### 3.2 解析

- `Parser` 增加 `markup: Markup` 字段；`parse_expr_markup(input, prefer_var, markup)`
  供测试/迁移显式指定；CLI 通过 `MARQDO_MARKUP` 设置进程默认。
- 描述扫描：读到 `]` 为止；`\]` 转义字面方括号；跨行即报未闭合。
- 之后必须紧跟 `(`；`()` 内若无内容 → `payload: None`；否则 `parse_or()` 后要求 `)`。
- 嵌套自然成立：payload 是完整表达式，可再含遮蔽。

### 3.3 求值（两个后端一致）

| 位置 | 处理 |
|------|------|
| `interp.rs` `eval_expr` | 有 payload → 求值 payload；无 → `Value::None` |
| `bytecode/compile.rs` | 有 payload → 编译 payload；无 → `Op::None_` |
| `collect_decl_reads` / `collect_reads_expr` | 递归 payload（描述不产生读引用） |
| `view/render.rs` `expr_prec` | 渲染成 `[描述](payload)` / `[描述]()` |

**描述永不参与求值**——守住铁律 1。

---

## 4. 测试

| 层 | 内容 | 结果 |
|----|------|------|
| 单元（8） | 值直通 / 空载荷 / 描述含空格与标点 / `\]` 转义 / 嵌套 / classic 仍是取元 / 缺括号报错 | ✅ |
| 集成（3） | `MARQDO_MARKUP=occlusion` 跑通三个值；不设该变量时旧取元不变；occlusion 模式下旧形**不再**是取元 | ✅ |
| 全量 | `cargo test --release` | **575 passed / 0 failed**（M1 后 565 + 10） |
| 双后端 | tree 与 `--backend bytecode` | 一致 |

---

## 5. 顺带重构

- `Markup` 枚举与 `parse_expr_markup` 抽为显式参数，避免测试依赖全局可变状态
  （全局只由 CLI 入口设置一次）。
- `parse::expr` 由 `mod` 改 `pub mod`，供 `main.rs` 设置模式。

---

## 6. 边界

| 项 | 说明 |
|----|------|
| 描述不参与作用域/类型 | 写什么都不影响执行 |
| 描述里的 `` `实体` `` | M4 才做引用一致性校验 |
| 空载荷 warning | M2.5 落诊断（本提交只有语义） |
| 跨行描述 | 不支持（v1） |
| 旧取元 | 仍可用；M7 迁移后改为硬错误 |

---

## 7. 变更清单

| 文件 | 变更 |
|------|------|
| `src/ast.rs` | 新增 `Expr::Occluded { label, payload }` |
| `src/parse/expr.rs` | `Markup` 枚举 + 全局默认 + `parse_expr_markup` + `parse_occluded` + `Parser.markup` |
| `src/parse/mod.rs` | `pub mod expr`；`collect_reads_expr` 递归 payload |
| `src/interp.rs` | 求值 + 读引用收集 |
| `src/bytecode/compile.rs` | 编译为 payload（或 `None_`） |
| `src/view/render.rs` | 表面渲染 + 输入提示收集 |
| `src/main.rs` | `MARQDO_MARKUP` 环境变量 |
| `tests/occlusion/value.mq.md` · `tests/occlusion/classic-index.mq.md` · `tests/occlusion.rs` | 新增集成测试 |
| `doc/design/change-m2-occlusion.md` | 本文件 |
