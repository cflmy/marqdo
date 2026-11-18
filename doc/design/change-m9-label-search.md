# 变更记录：M9 两级词表检索（`labels` CLI + MLSP `labels.search`）

| | |
|---|---|
| 状态 | **Implemented** |
| 日期 | 2026-10-22 |
| 里程碑 | M9（[occlusion-description-and-search.md](occlusion-description-and-search.md) §3 · [occlusion-syntax.md](occlusion-syntax.md) §13） |
| 改动文件 | `src/label.rs` · `src/main.rs` · `src/mlsp.rs` · `tests/occlusion.rs` |
| 影响面 | 新增 CLI 子命令与 MLSP 方法（**不改语法**） |

---

## 1. 两级词表

| 级 | 在哪 | 作用 |
|----|------|------|
| **第一级：内置** | `label::BUILTIN_GROUPS`（数据常量） | 默认查询 + LLM 范例 + 跨项目可比 |
| **第二级：参数** | `--grep` / `grep` | 替换或并集，项目自己的说法照样能搜 |

内置六组：`未完成`（待开发/未开发/开发中/未实现/待实现/未接/接口未接/TODO）、
`假数据`（模拟/假数据/桩/占位/stub/mock）、`示例`、`实验`、`废弃`、`待校验`。
`pending` = `未完成 ∪ 假数据`。

**词表是数据不是语法**：扩充它不改任何解析规则。

---

## 2. CLI

```bash
marqdo labels .                              # 默认：pending 口径（内置两组）
marqdo labels . --grep "联调"                 # 替换为自定义词
marqdo labels . --grep "联调" --also-builtin   # 自定义 ∪ 内置
marqdo labels . --group 示例,实验              # 选内置组
marqdo labels . --count                       # 只给数字（CI/棘轮用）
marqdo labels . --json                        # 一个 JSON 数组
```

输出（人类）：

```
a.mq.md:3  [main] 按关键词检索，接口未接，待开发  ← 待开发,未接,接口未接
a.mq.md:4  [main] 客户留存率，模拟              ← 模拟
label search: 2 命中（1 文件，词=[…]）
```

`--json`：

```json
[{"file":"a.mq.md","line":3,"unit":"main","label":"…","words":["待开发"],"has_payload":true}]
```

---

## 3. MLSP

```json
{"id":1,"method":"labels.search",
 "params":{"source":"…"|"path":"…","grep":["联调"],"also_builtin":true}}
```

返回：

```json
{"ok":true,"words":[…],"count":2,"hits":[{"line":3,"unit":"main","label":"…",
  "words":["待开发"],"has_payload":true}]}
```

语义与 CLI 完全一致（同一份 `label::search`）。

---

## 4. 检索语义（v1）

| 项 | 规则 |
|----|------|
| 匹配 | **子串**（CJK 无词边界，子串最稳） |
| 多词 | OR |
| 大小写 | 英文不敏感（`TODO`/`todo` 等价） |
| 误判 | 必然存在（`[不属于待开发范围]` 会命中）——只影响计数，输出带描述原文可人工判断 |

不做否定语法；留作 v1.1 可选项。

---

## 5. 测试

| 层 | 内容 | 结果 |
|----|------|------|
| 集成（4） | pending 计数=2；`--grep` 替换=1；`--also-builtin` 并集=3；MLSP `labels.search` 返回 hits | ✅ |
| 全量 | `cargo test --release` | **587 passed / 0 failed**（M4 后 583 + 4） |

---

## 6. 边界

| 项 | 说明 |
|----|------|
| 只在 occlusion 模式产生命中 | classic 模式下没有 `Expr::Occluded`；M7 翻转默认后自然可用 |
| 解析失败的文件跳过 | 那是 `check` 的职责，检索不应因一个坏文件整体失败 |
| 不做棘轮 | `--baseline` / `--fail-on-increase` 留给 M8（门禁） |

---

## 7. 变更清单

| 文件 | 变更 |
|------|------|
| `src/label.rs` | `WordGroup` / `BUILTIN_GROUPS` / `PENDING_GROUPS` / `group_words` / `parse_words` / `matched_words` / `Hit` / `search` |
| `src/main.rs` | `labels` 子命令 + `collect_mq_files` 递归扫描 |
| `src/mlsp.rs` | `labels.search` 方法 + 分发 |
| `tests/occlusion.rs` + `tests/occlusion/labels.mq.md` | 4 个集成测试 |
| `doc/design/change-m9-label-search.md` | 本文件 |
