# Engineering Knowledge Compiler（EKC）

| | |
|---|---|
| 状态 | **Accepted（v1 已落地）** |
| 日期 | 2026-09-28 |
| 提案 | [next/008.md](../next/008.md) |
| 相关 | [okf.md](okf.md) · [catalog-cli.md](catalog-cli.md) · [ext-agent.md](ext-agent.md) |

## 1. 定位

Marqdo 把仓库投影为 **可查询、可验证、可复用** 的工程知识系统。Source of Truth 仍是 `*.mq.md`；`.marqdo/**` 是 GENERATED 投影。

```text
.mq.md ──► EKC ──► Executable + Catalog L0–L4 + Graph + Context Pack
                         │
                         ▼
              Agent preflight (REUSE → ADAPT → CREATE)
```

**Engineering First Information (EFI)**：生成代码前先获得已有能力、约束、决策、失败经验与推荐复用。

## 2. 五层 Catalog

| 层 | 内容 | 落盘 |
|----|------|------|
| L0 Repository | 语言/规模 | `.marqdo/catalog/repository.mq.md` |
| L1 Module | imports/exports/responsibility | `.marqdo/catalog/modules/` |
| L2 Symbol | signature/calls/fingerprint | `.marqdo/catalog/symbols/` |
| L3 Capability | 工程能做什么 | `.marqdo/catalog/capabilities/` |
| L4 Knowledge | Decision/Constraint/Failure/… | `.marqdo/knowledge/{decisions,constraints,…}/` |

既有 `marqdo catalog` 根级 `catalog.yaml` / `index.md` / `modules/` / `concepts/` **保留**；`catalog` 成功后默认再跑 EKC（`--no-knowledge` 可关）。

## 3. 知识类型（OKF `type`）

| type | 含义 |
|------|------|
| `Capability` / `Function` | 能力与符号投影 |
| `Decision` | 为什么这样做 |
| `Constraint` | 必须/禁止 |
| `Failure` / `AntiPattern` | 负面知识 |
| `Pattern` / `Migration` / `Fact` | 模式、迁移、事实 |
| `Marqdo Context Pack` | Agent 上下文制品 |
| `Marqdo Policy` | 复用决议编译结果 |
| `Marqdo Engineering Knowledge` | 总览 / `engineering.yaml` |

未知 type 优雅降级（可读，不专属逻辑）。

## 4. 图边

`implements` · `uses` · `depends_on` · `tested_by` · `documented_by` · `constrained_by` · `decided_by` · `supersedes` · `conflicts_with` · `deprecated_by` · `derived_from` · `failed_by` · `related_to`

落盘：`.marqdo/graph/graph.json` · `edges.json` · `index.json`。

## 5. CLI

```bash
marqdo knowledge [PATH] -o .marqdo          # compile（默认）
marqdo knowledge preflight "task"
marqdo knowledge record REUSE
marqdo knowledge learn --task ... --failure ...
marqdo find "authentication"
marqdo reuse "implement oauth login"
marqdo impact path/to/file.mq.md
marqdo conflicts | stale | duplicate [--fail] | verify
```

统一 `--json`。`duplicate` 默认阈值 `0.75`；`--fail` 作 CI Gate。

## 6. Reuse 协议

三态：**REUSE → ADAPT → CREATE**（硬顺序）。Reuse budget：`min_candidates` / `allow_create_after`。仅 `create_allowed: true` 时允许盲生成。

## 7. Context Pack

`.marqdo/agent/contexts/task-<id>.mq.md`：Task / Existing Capabilities / APIs / Constraints / Decisions / Failures / Recommended / Forbidden。

## 8. Agent

- Plugin：`agent_eng_preflight` / `agent_eng_reuse` / `agent_eng_record`
- Facade：`agent.preflight` · `agent.eng_reuse` · `agent.eng_record`
- `agent.plan … eng_preflight=True` 在 agent-kb 之前跑 EFI；缺知识图时 `missing_knowledge` 放行（兼容纯 agent-kb 测试）

## 9. 指标

`.marqdo/agent/episodes/reuse_metrics.json`：`reuse_ratio` / `adapt_ratio` / `novel_ratio` / `duplication_rate`。

## 10. 原则（五条）

1. Discover before Create  
2. Reuse before Adapt  
3. Adapt before Duplicate  
4. Verify before Promote  
5. Code changes update knowledge  

**不做**：以向量库为知识模型；embedding 仅可选检索实现，默认关闭。Hybrid = lexical + symbol + capability + graph。
