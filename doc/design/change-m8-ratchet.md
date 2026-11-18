# 变更记录：M8 债务棘轮与呈现

| | |
|---|---|
| 状态 | **Implemented** |
| 日期 | 2026-10-22 |
| 里程碑 | M8（[warning-fatigue-control.md](warning-fatigue-control.md) §4 棘轮 · [occlusion-syntax.md](occlusion-syntax.md) §13） |
| 改动文件 | `src/main.rs` · `tests/ratchet.rs` |
| 影响面 | `labels` 新增 `--baseline` / `--fail-on-increase` |

---

## 1. 棘轮规则

```bash
# CI：未完成项只许减少，不许增加
marqdo labels . --baseline .marqdo/pending.json --fail-on-increase
```

| 情形 | 行为 | 退出码 |
|------|------|--------|
| 台账不存在 | **建立**（写入当前计数），提示 `已建立 N` | 0 |
| 当前 > 台账 | 失败，提示 `未完成项增加了 K` | **1** |
| 当前 ≤ 台账 | 通过；若减少则**收紧**台账（`N → M`） | 0 |

**为什么"减少即收紧"**：否则债务可以先降后升，棘轮形同虚设。
收紧后台账记住新的低水位，回退到旧数量同样会失败（测试第 4 步验证）。

台账格式：

```json
{"count": 1, "words": ["待开发", "未开发", "…"]}
```

记录词集是为了让"口径变了"可被审计——换词集时会看到 `words` 变化。

---

## 2. 为什么用棘轮而不是硬门禁

| 方案 | 老债 | 新债 | 结论 |
|------|------|------|------|
| 硬门禁（必须为 0） | 阻塞一切 | 拦住 | 存量债立刻红一片，逼人关检查 |
| **棘轮（只拦增量）** | 不阻塞 | 拦住 | ✅ 债务可见且不恶化 |
| 无门禁 | 不阻塞 | 不拦 | 债务无声增长 |

这与 [warning-fatigue-control.md](warning-fatigue-control.md) 的结论一致：
**老账不追、新账必拦**。

---

## 3. 呈现（已具备）

| 面 | 表现 |
|----|------|
| `expr_display` / `view` | 遮蔽渲染为 `[描述](实现)`，桩渲染为 `[描述]()`，实测 `view output` 页面含 `access(` |
| `labels` | 人类行 `文件:行 [函数] 描述 ← 命中词`；`--json` 一个数组 |
| `check` | `label.*` 三类诊断（missing / empty_payload / ref_dangling） |
| MLSP | `labels.search` · `session.summary` |

**没有做桩徽章**：桩已经是 `[描述]()` 这个显眼形状，再加装饰是噪音；
需要计数时用 `labels --count`。

---

## 4. 测试

| 层 | 内容 | 结果 |
|----|------|------|
| 集成（1 个四步用例） | 建账 → 增加失败 → 减少收紧 → 回退再次失败 | ✅ |
| 全量 | `cargo test --release` | **605 passed / 0 failed**（M7 后 604 + 1） |

---

## 5. 变更清单

| 文件 | 变更 |
|------|------|
| `src/main.rs` | `labels --baseline` / `--fail-on-increase`；建账/收紧/违约逻辑 |
| `tests/ratchet.rs` | 四步棘轮用例 |
| `doc/design/change-m8-ratchet.md` | 本文件 |
