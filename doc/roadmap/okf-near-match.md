# OKF 相近任务命中

| | |
|---|---|
| 状态 | **A 规范句 + B aliases + C 父裁决（soft_match）+ D 本地 n-gram `near` 已落地**；E dense embedding 仍非目标 |
| 日期 | 2026-08-11 |
| 相关 | [okf.md](../design/okf.md) §7 · [ext-agent-plan.md](../design/ext-agent-plan.md) §4.2 · [ext-agent.md](../design/ext-agent.md) |
| 触发观察 | `man-test`：相近 goal 生成两份独立 resource |

---

## 1. 问题

v1 agent-kb **精确命中仍是默认快路径**：

1. `normalize_goal`：去首尾空白、合并连续空白（**不做**近义改写）。  
2. `sig` = 规范化字符串的稳定短哈希；`slug` = 可读路径名。  
3. `agent_kb_lookup`：精确 → aliases → canonicalize → **本地 n-gram `near`（默认开）**；仍 miss 且 `plan soft_match=True` 时父裁决 REUSE/NEW（候选为排序 near 列表）。

历史痛点（已由 A/B/C/D 缓解）：

| goal（调用方输入） | 旧行为 |
|--------------------|--------|
| `你是一个智能体，帮我规划明天的行程` | 独立 slug，与下方不命中 |
| `帮我规划明天的行程` | 另一份 resource |
| `帮我规划明天行程` | 又一份（近句碎片） |

调用方直觉：「这不是同一件事吗？」——现可用 canonicalize、默认 `near_match`，或显式 `soft_match` 父裁决。

---

## 2. 为何精确指纹仍优先

| 理由 | 说明 |
|------|------|
| 假阳性代价高 | 错复用旧工作簿比多建一个文件更糟 |
| 可审计 | `sig` 可复算；`near` 留下 `match`/`score` |
| 哲学 | 多轮 = 子文件接力 + OKF 索引，不是聊天式模糊检索 |

**结论：** 精确命中长期保留为**默认快路径**；`near` 是可关闭的第四趟（`near_match=False`）。

---

## 3. 已落地层

### A. 规范化后再精确命中 — **已落地**

`agent_kb_canonicalize`：剥站立前缀、去尾标点；`match: canonical`。

### B. 别名 / 标签表 — **已落地**

Task FM `aliases:`；`match: alias`。

### C. 父智能体软裁决 — **已落地（默认关）**

`plan soft_match=True`：`agent_kb_near_match` 排序候选（含 `score`；空则退回 `list_tasks`）→ 父 `DECISION: REUSE` + `SLUG:` 或 `DECISION: NEW`。

### D. 本地字符 bigram + 余弦 — **已落地（默认开）**

| | |
|--|--|
| 做法 | 稀疏词法向量化（**不是** embedding 模型）：canonicalize 后对 goal 与 Task `title\|description\|aliases` 建字符 bigram 词袋，算余弦 |
| API | `agent_kb_near_match`；`agent_kb_lookup` 第四趟 |
| 参数 | `near_match`/`近义命中` 默认 `True`；`near_threshold`/`近义阈值` 默认 `0.78` |
| 痕迹 | `match: near` + `score`；`cache=soft-hit`；可写 alias |

术语：这是 **sparse lexical near-match**，勿宣传为向量库 / embedding 检索。

### E. Dense embedding（可选远期）

远程 / 本地 dense 向量与 ANN **仍非本仓默认目标**；仅当 D 在真实包体量下证据不足时再评估。

---

## 4. 演进顺序

```text
精确 sig（永不删除）
    → A 规范句 + B aliases（已落地）
    → C soft_match 父裁决（已落地，默认关）
    → D 本地 n-gram near（已落地，默认开）
    → E dense embedding（仅有证据再开）
```

验收原则：

1. `near_match=False` 时与无 near 行为一致；`force`/`optimize` 仍跳过 reuse。  
2. 软/近义命中留下 `cache=soft-hit` / `match` / 可选 `score`。  
3. 假阳性可被 `force=True` / 显式 `workbook=` / 调高 `near_threshold` 覆盖。  
4. 实现仍在 **agent 插件 + `ext/ai`**，不进核心 `src/host`。

---

## 5. 调用方建议

- 固定规范 goal；站立语可由 canonicalize 剥掉。  
- 近句碎片依赖默认 `near_match`；字面差更大时开 `soft_match=True`。  
- 策展：合并重复 task，次要措辞写入 `aliases`（O4）。

---

## 6. 与现有路线图的关系

| 条目 | 关系 |
|------|------|
| [okf.md](../design/okf.md) O2 / O5 | 精确 reuse + A/B/C/D；E dense 仍待 |
| [okf.md](../design/okf.md) O4 | `kb query` / 策展可继续扩 aliases |
| [ext-agent-plan.md](../design/ext-agent-plan.md) | 多轮仍只走子文件接力；软命中只影响「选哪个子文件」 |
