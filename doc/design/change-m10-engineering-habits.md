# 变更记录：M10 工程化落点（README + agent 提示词）

| | |
|---|---|
| 状态 | **Implemented** |
| 日期 | 2026-10-22 |
| 里程碑 | M10（[occlusion-description-and-search.md](occlusion-description-and-search.md) §5） |
| 改动文件 | `ext/ai/agent.mq.md`（README 一节已在文档阶段落地） |

---

## 1. 改了什么

把「接不了就申报，别假装完成」写进 **agent 的默认提示词**，两处：

### 1.1 `how-to-act` 协议段（每轮都会带）

```
Call ONLY names listed under --- tools --- (whitelist). …
If a capability cannot truly be wired, do NOT fake it silently and never claim it works:
write an occlusion with an explicit status, e.g. [按关键词检索，接口未接，待开发](None).
Stubs must run without error and must state their return value (None is acceptable).
Before finishing, check your own declarations with labels.search.
Tools run via Marqdo subtask (spawn fn / wait).
```

### 1.2 默认 standing（`p`）

```
You are a Marqdo agent. Code is documentation: … not a hidden system prompt.
When you cannot wire a capability, declare it in the description (待开发 / 模拟)
instead of faking success.
```

**为什么这两处**：standing 定基调（每次对话开头），how-to-act 定动作（每轮）。
只写 standing 容易被长上下文冲淡；只写 how-to-act 又缺"为什么要申报"。

---

## 2. 设计要点

| 要点 | 说明 |
|------|------|
| 给**范式**而不是规则 | 直接给出可抄的 `[按关键词检索，接口未接，待开发](None)`，模型不必发明措辞 |
| 给**返回值**要求 | "必须写清返回什么（None 也行）"——对应 `label.empty_payload` 的 warning |
| 给**自查**动作 | `labels.search` 让模型收尾时能自己盘点，而不是靠人提醒 |
| 用**英文**写 | 与既有 standing / 协议段语言一致（模型对英文指令更稳） |

---

## 3. 验证

| 项 | 结果 |
|----|------|
| `marqdo check ext/ai/agent.mq.md`（occlusion 模式） | `contract check OK` |
| 提示词文本落入模块 | `grep` 命中 2 处（standing 与 how-to-act 各 1） |
| 全量回归 | 587 passed / 0 failed（与 M9 相同，本改动不动逻辑） |

---

## 4. 边界

- 提示词是**建议**，不是门禁；违反不会被拒绝，但会在描述/诊断里暴露。
- `labels.search` 需要 M7 翻转默认后对 classic 文档才有意义；提示词先就位。

---

## 5. 变更清单

| 文件 | 变更 |
|------|------|
| `ext/ai/agent.mq.md` | how-to-act 协议段与默认 standing 各追加申报约定 |
| `doc/design/change-m10-engineering-habits.md` | 本文件 |
