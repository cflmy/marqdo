# OKF 相近任务命中（未来规划）

| | |
|---|---|
| 状态 | **A 规范句 + B aliases + C 父裁决（soft_match）已落地**；E 向量仍待 |
| 日期 | 2026-08-11 |
| 相关 | [okf.md](../design/okf.md) §7 · [ext-agent-plan.md](../design/ext-agent-plan.md) §4.2 · [ext-agent.md](../design/ext-agent.md) |
| 触发观察 | `man-test`：相近 goal 生成两份独立 resource |

---

## 1. 问题

v1 agent-kb **默认只做精确命中**（仍是默认快路径）：

1. `normalize_goal`：去首尾空白、合并连续空白（**不做**近义改写）。  
2. `sig` = 规范化字符串的稳定短哈希；`slug` = 可读路径名。  
3. `agent_kb_lookup`：精确 → aliases → **canonicalize 后再精确/别名**；仍 miss 且 `plan soft_match=True` 时父裁决 REUSE/NEW。

历史痛点（已由 A/B/C 缓解）：

| goal（调用方输入） | 旧行为 |
|--------------------|--------|
| `你是一个智能体，帮我规划明天的行程` | 独立 slug，与下方不命中 |
| `帮我规划明天的行程` | 另一份 resource |

调用方直觉：「这不是同一件事吗？」——现可用 canonicalize（剥站立前缀）或显式 `soft_match` 父裁决。

---

## 2. 为何 v1 坚持精确指纹

| 理由 | 说明 |
|------|------|
| 假阳性代价高 | 错复用旧工作簿比多建一个文件更糟（静默答错题） |
| 可审计 | `sig` 可复算；命中不依赖黑盒相似度阈值 |
| 体量假设 | 单仓 agent-kb 通常不大；调用方可固定规范 goal |
| 哲学 | 多轮 = 子文件接力 + OKF 索引，不是聊天式模糊检索 |

**结论：** 精确命中应长期保留为**默认快路径**；相近命中只能是**显式可选层**，且必须可关闭、可解释。

---

## 3. 目标

在 **不破坏** 精确 `sig` 契约的前提下：

1. **已落地**：A 规范句、B aliases、C `soft_match` 父裁决（默认关）。  
2. **仍待**：E 向量（仅有证据再开）。

非目标（仍不做为默认）：

- 把 agent-kb 做成通用向量知识库 / RAG 产品。  
- 用不可审计的静默相似度覆盖精确 `sig`。  
- 单文件对话日志 / 状态表来「记住」同主题（与 [ext-agent-plan.md](../design/ext-agent-plan.md) §4.2 冲突）。

---

## 4. 候选方案（不必上向量库）

相近命中 **不等于** 必须上向量检索。按与 Marqdo 契合度排序：

### A. 规范化后再精确命中 — **已落地**

| | |
|--|--|
| 做法 | `agent_kb_canonicalize`：剥站立前缀、去尾 `？`/`?`/`。`，再 `normalize_goal`；lookup 第三趟 |
| 落地 | 插件 + `match: canonical`；`cache=soft-hit` |

### B. 别名 / 标签表 — **已落地**

| | |
|--|--|
| 做法 | Task FM `aliases:`；lookup 第二趟精确匹配 |
| 落地 | `match: alias`；promote / `agent_kb_add_alias` |

### C. 父智能体软裁决（miss 后）— **已落地（默认关）**

| | |
|--|--|
| 做法 | `plan soft_match=True`：`agent_kb_list_tasks` → 父 `DECISION: REUSE` + `SLUG:` 或 `DECISION: NEW` |
| 落地 | REUSE → 现有 hit 分支；可选 append alias |

### D. 字面近似（弱，仅拼写容错）

| | |
|--|--|
| 做法 | 编辑距离、共用字符 n-gram、去标点后再比 |
| 优点 | 零模型、便宜 |
| 风险 | 中文近义（「行程 / 一日游」）基本无效；易假阳性 |
| 建议 | 最多作 D 的辅助，不单独作为语义层 |

### E. 向量检索（可选远期）

| | |
|--|--|
| 做法 | 对 task `description` / 规范句做 embedding，ANN Top-K，再经阈值或父模型确认 |
| 优点 | 开放域措辞更稳 |
| 风险 | 依赖外部分数与阈值；运维与可审计性差；偏离「Markdown 即索引」 |
| 建议 | **仅当** A–C 在真实包体量下不够时再评估；默认关闭 |

---

## 5. 演进顺序

```text
精确 sig（永不删除）
    → A 规范句 + B aliases（已落地）
    → C soft_match 父裁决（已落地，默认关）
    → E 向量（仅有证据再开）
```

验收原则（仍适用）：

1. 默认行为与精确路径一致（`soft_match` 默认 False；canonicalize 无模型、低风险）。  
2. 软命中留下 `cache=soft-hit` / `match` 痕迹。  
3. 假阳性可被 `force=True` / 显式 `workbook=` 覆盖。  
4. 实现仍在 **agent 插件 + `ext/ai`**，不进核心 `src/host`。

---

## 6. 调用方建议

- 固定规范 goal 字符串；站立语可由 canonicalize 剥掉，但仍宜少拼。  
- 续跑同一主题时传 `workbook=` 或已知 resource 路径。  
- 近义但字面差大时显式 `soft_match=True`（多一次父 LLM）。  
- 策展：合并重复 task，次要措辞写入 `aliases`。

---

## 7. 与现有路线图的关系

| 条目 | 关系 |
|------|------|
| [okf.md](../design/okf.md) O2 / O5 | 精确 reuse + A/B/C 软命中；E 向量仍待 |
| [okf.md](../design/okf.md) O4 | `kb query` / 策展可继续扩 aliases |
| [ext-agent-plan.md](../design/ext-agent-plan.md) | 多轮仍只走子文件接力；软命中只影响「选哪个子文件」 |
