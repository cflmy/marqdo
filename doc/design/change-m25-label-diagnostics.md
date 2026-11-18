# 变更记录：M2.5 遮蔽标签诊断（`label.missing` / `label.empty_payload`）

| | |
|---|---|
| 状态 | **Implemented** |
| 日期 | 2026-10-22 |
| 里程碑 | M2.5（[occlusion-syntax.md](occlusion-syntax.md) §13，诊断规范 §10） |
| 改动文件 | **新增 `src/label.rs`** · `src/check.rs` · `src/lib.rs` · `tests/occlusion.rs` |
| 影响面 | `marqdo check`（新增两类诊断） |

---

## 1. 两条诊断

| code | 级别 | 触发 | 文案要点 |
|------|------|------|---------|
| `label.missing` | **error** | `[](实现)` 描述为空 | 「给这段实现写一句人话，或改用不带遮蔽的写法」 |
| `label.empty_payload` | **warning** | `[描述]()` 载荷为空 | 「这里没有可执行内容」，建议写 `(None)` 或补实现 |

实测：

```
$ MARQDO_MARKUP=occlusion marqdo check empty-label.mq.md
empty-label.mq.md:3:1: 遮蔽缺少描述：`[](…)` —— 给这段实现写一句人话，或改用不带遮蔽的写法
contract check: 1 条诊断（1 error）      # exit 1

$ MARQDO_MARKUP=occlusion marqdo check empty-payload.mq.md
empty-payload.mq.md:3:1: 空载荷遮蔽：`[某某功能开发中]()` —— 这里没有可执行内容
contract check: 1 条诊断（0 error）      # exit 0（warning 不影响退出码）
```

JSON 出口（自动继承既有 schema，含 `doc_quote`）：

```json
{"code":"label.empty_payload","severity":"warning","span":{"line":3,"col":1},
 "doc_quote":"某某功能开发中","message":"空载荷遮蔽：…","suggestion":"写清桩的返回值（如 `(None)` 或 `(0)`）…"}
```

---

## 2. 实现

### 2.1 新增 `src/label.rs`

- `OcclusionSite { label, has_payload, line, unit }`
- `collect(module)`：遍历函数体与嵌套语句/表达式，收集所有遮蔽点
  （描述只读，**不产生任何读引用**）
- `check(module, path)`：产出上面两条诊断

### 2.2 接入 `check_module`

```rust
walk(module, &module.functions, path, &known, &mut out);
out.extend(crate::label::check(module, path));   // M2.5
```

设计上把"标签诊断"独立成模块，M4（引用一致性）与 M9（检索）都往这里加，
`check` 只做一行汇总。

---

## 3. 测试

| 层 | 内容 | 结果 |
|----|------|------|
| 集成（3 新） | 空描述 → exit 1 + 文案；空载荷 → exit 0 + 文案；干净样例 → 无诊断 | ✅ |
| 全量 | `cargo test --release` | **578 passed / 0 failed**（M2 后 575 + 3） |

反向验证：三条断言分别验证"会响"与"不会误响"（干净样例必须无输出）。

---

## 4. 边界

| 项 | 说明 |
|----|------|
| 描述内容不校验 | 写什么都可以；M4 才校验反引号引用 |
| 空载荷仅在 occlusion 模式可能出现 | classic 模式下 `()` 空是语法错 |
| warning 不影响退出码 | 与既有 `Severity` 语义一致 |

---

## 5. 变更清单

| 文件 | 变更 |
|------|------|
| `src/label.rs` | **新增**：遮蔽点收集 + 两类诊断 |
| `src/lib.rs` | `pub mod label;` |
| `src/check.rs` | `check_module` 追加 label 诊断 |
| `tests/occlusion.rs` + `tests/occlusion/{empty-label,empty-payload,clean}.mq.md` | 新增 3 个集成测试与样例 |
| `doc/design/change-m25-label-diagnostics.md` | 本文件 |
