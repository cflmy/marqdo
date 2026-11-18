# 变更记录：M5 隐式遮蔽（`>` 调用上方的叙述）

| | |
|---|---|
| 状态 | **Implemented**（**默认关闭**，`--implicit-labels` 开启） |
| 日期 | 2026-10-22 |
| 里程碑 | M5（[occlusion-syntax.md](occlusion-syntax.md) §11.2） |
| 改动文件 | `src/label.rs` · `src/check.rs` · `src/main.rs` · `tests/occlusion.rs` |
| 影响面 | `marqdo check` 新增可选检查（默认不改变输出） |

---

## 1. 规则

> **`>` 调用上方的叙述段**就是它的"描述"；段内的反引号实体必须被这次调用用到
> （或能解析到模块/内置）。

```markdown
按 `税率` 与 `档位` 计算应缴税额：      ← 描述

> 计税 档位=2 金额=100                  ← 实现用到了 `档位`，但没有 `税率`
```

```
impl.mq.md:5:1: 调用上方的描述提到 `税率`，但这次调用没有用到它
```

段落边界：紧邻上一行起，遇空行（允许**一个**分隔空行，因为 blockquote 前通常空一行）、
标题、代码行（`**`/`>`/`-`/`|`/数字开头）即止。

---

## 2. 关键决策：**默认关闭**

这不是洁癖，是实测结论：

| 阶段 | lib + ext/ai 上的警告数 |
|------|------------------------|
| 初版（任何反引号都算断言） | **6 条**（全部误报：`` `## open` ``、`` `{ path, kind }` ``、`` `source=` ``、`` `exec` ``/`` `net` ``） |
| 加"实体名形状"过滤 | 2 条 |
| **再加默认关闭** | **0 条** |

理由：**描述一个调用可以用任意措辞**，这比"遮蔽"松散得多；
机械校验必然模糊，默认开启就会制造疲劳（违背 [warning-fatigue-control.md](warning-fatigue-control.md)）。
所以它和 L1 不同：L1 是"能证明才报"，隐式检查是"觉得可疑就报"——后者必须 opt-in。

---

## 3. 实体名过滤（降低误报）

反引号内容必须**像实体名**才算断言：

```rust
首字符：字母 / `_` / 非 ASCII（不能是数字）
其余：字母数字 / `_` / `.` / 非 ASCII 且非空白
```

于是 `` `## open` ``、`` `{ path, kind }` ``、`` `source=` ``、`` `path=` `` 全部被排除。

---

## 4. 用法

```bash
marqdo check 文件.mq.md                     # 默认：不含隐式检查（零噪音）
marqdo check 文件.mq.md --implicit-labels    # 开启
```

MLSP 侧留了同义参数位（`validate` 的 `implicit_labels`），M8 一并接。

---

## 5. 测试

| 层 | 内容 | 结果 |
|----|------|------|
| 集成（3） | 默认静默；opt-in 报不一致；opt-in 对一致描述仍静默 | ✅ |
| 全量 | `cargo test --release` | **593 passed / 0 failed**（M6 后 590 + 3） |

**反向验证**：`implicit_labels_off_by_default` 与 `..._stays_quiet_when_consistent`
两条都在证明"不乱响"。

---

## 6. 边界

| 项 | 说明 |
|----|------|
| 只查 `>` 语句调用 | 表达式内调用没有"上方叙述"这回事 |
| 只认反引号 + 实体形状 | 普通散文永远不参与 |
| 不查 payload 语义 | 只查"提到的实体是否被用到" |

---

## 7. 变更清单

| 文件 | 变更 |
|------|------|
| `src/label.rs` | `CallSite` · `call_sites` · `paragraph_above` · `is_entity_ref` · `check_implicit` |
| `src/check.rs` | `check_path_opts(path, implicit_labels)`（`check_path` 保持默认关闭） |
| `src/main.rs` | `check --implicit-labels` |
| `tests/occlusion.rs` + `tests/occlusion/implicit-*.mq.md` | 3 个集成测试 |
| `doc/design/change-m5-implicit-occlusion.md` | 本文件 |
