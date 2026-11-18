# 方案：智能体 / 大模型优化方向（代码即文档 · OKF · 沙盒）

| | |
|---|---|
| 状态 | **Accepted · O1–O5 locked** |
| 日期 | 2026-10-16 |
| 调研 | [ai-agent-llm-optimize-2026-10.md](../research/ai-agent-llm-optimize-2026-10.md) |
| 路线 | [ext-ai-optimize.md](../roadmap/ext-ai-optimize.md) |
| 相关 | [ext-ai-sandbox.md](ext-ai-sandbox.md) · [ext-agent.md](ext-agent.md) · [okf.md](okf.md) · [engineering-knowledge.md](engineering-knowledge.md) · [ext-agent-optimize.md](../roadmap/ext-agent-optimize.md) · [ai-stack.md](../roadmap/ai-stack.md) |

---

## 0. 方案立场

1. **强化差异化**：可执行 `.mq.md` 为权威；OKF 为复用；沙盒为证明；LLM 为探索。  
2. **不向隐藏 TOOL: 循环 / 向量权威回退。**  
3. 下列方向均为**可行**；推荐波次见 §3，最终裁剪由你选型。  
4. **不改** `doc/next/*`；落地时另开 roadmap 切片。

---

## 1. 架构愿景（闭环）

```text
目标 / 任务
    │
    ▼
EFI / OKF lookup ──hit──► llm_free / marqdo 执行 ──► writeback
    │ miss
    ▼
small-llm 路由 / 父分解
    │
    ▼
生成或修补 .mq.md（代码即文档风格）
    │
    ▼
sandbox.open → sandbox.run（caps）→ diff
    │
    ├─ 失败 → writeback error / 隔离，不 promote
    └─ 成功 → 人审或策略 → commit → solidify / OKF promote
                              │
                              ▼
                         下次 REUSE（降本）
```

---

## 2. 可行方向全集

每个方向标注：解决的问题（Pn）、依赖、风险、粗粒度成本。

### A. 沙盒 × Agent 闭环

| ID | 方向 | 解决 | 做什么 | 依赖 | 风险 | 成本 |
|----|------|------|--------|------|------|------|
| **A1** | `step`/`plan` 默认经 sandbox 试跑再写实树 | P1 | 生成落 upper；绿才 `commit` | sandbox | 破坏现有金样默认 | 中 |
| **A2** | Promote / solidify 强制 sandbox proof | P1 | OKF 写入前要求 proof fixture 绿 | sandbox+OKF | 过严挡复用 | 中 |
| **A3** | Sandbox profile 表（caps 宣告） | P1/P3 | GFM 表声明 net/plugin/fs | sandbox | 表方言膨胀 | 低 |
| **A4** | Guest 风格门禁（表优先、禁 json 胶水） | P5 | commit 前 lint / preflight | sandbox+parser | 误杀合法 json.parse | 中 |
| **A5** | 会话 diff → view 审阅卡 | P1/P4 | HITL 看 CoW diff | view+sandbox | UI 工时 | 中高 |
| **A6** | 多会话并行 CoW（fan-out 实验） | — | 每假设一 session | sandbox | 隔离/合并复杂 | 高 |

### B. OKF / EKC / 技能层

| ID | 方向 | 解决 | 做什么 | 依赖 | 风险 | 成本 |
|----|------|------|--------|------|------|------|
| **B1** | OKF 技能元数据：适用条件 / fixtures / sandbox_profile | P2 | frontmatter + 目录约定 | okf | 与 catalog 字段冲突 | 中 |
| **B2** | Progressive disclosure（索引轻、正文按需 READ） | P2/P6 | 对齐 Anthropic Skills 层级 | agent A3 | 实现重复 | 中 |
| **B3** | Skill Lift 评测（有/无技能配对） | P4 | harness 指标 | B1 评测 | 样本少噪声大 | 中 |
| **B4** | 失败技能隔离（quarantine）+ 回归用例跟随血缘 | P1 | ForgeAgent 式 | OKF | 状态机复杂 | 中高 |
| **B5** | near-match / soft_match 产品化默认开 | P6 | 已有 roadmap | okf-near | 误复用 | 低中 |
| **B6** | EKC preflight 默认硬门槛（REUSE→ADAPT→CREATE） | P6 | step 前强制 EFI | EKC | 拖慢探索 | 中 |
| **B7** | Capability 包与 `llm_free` 资源双向链接 | P2 | catalog L3↔任务包 | EKC+OKF | 图维护 | 中 |

### C. 大模型 / 路由 / 成本

| ID | 方向 | 解决 | 做什么 | 依赖 | 风险 | 成本 |
|----|------|------|--------|------|------|------|
| **C1** | 步骤级路由（lookup / patch / sandbox_run / net） | P3 | 扩展 route policy 表 | routing | 策略过拟合 | 中 |
| **C2** | 总成本计量（含推理 token 膨胀）写回文档 | P7 | usage→writeback/metrics | llm result | 噪声 | 低中 |
| **C3** | 用强模型「反思」优化 standing / policy `.mq.md`（类 GEPA） | P3/P5 | 离线优化环，产物仍是文件 | llm | 需评测集 | 高 |
| **C4** | 高频 OKF 技能蒸馏/微调小模型（编译进权重） | — | 长期研究 | 数据+算力 | 运维重；偏离「源码权威」 | 很高 |
| **C5** | Named model 绑定任务类型（fast=路由，reasoning=分解） | P3 | 已有 named handles 加深 | llm | 配置漂移 | 低 |
| **C6** | Prompt document 强制：无参 ask 必须来自 type:prompt 文件 | P5 | 硬化 005 | llm | 破坏脚本式调用 | 低 |

### D. 代码即文档（产出质量）

| ID | 方向 | 解决 | 做什么 | 依赖 | 风险 | 成本 |
|----|------|------|--------|------|------|------|
| **D1** | 官方「可读 Marqdo」Skill 强化（表/链接取元/禁混名） | P5 | skills/marqdo 加厚 | skill | 模型仍违规 | 低 |
| **D2** | AST/结构 lint：json.set 链、混 EN/ZH API | P5 | `marqdo lint` 或 sandbox 钩子 | parser | 误报 | 中 |
| **D3** | 工作簿补丁只允许表驱动 / 命名槽写回 | P5 | 收紧 A0 补丁面 | agent | 表达力下降 | 中 |
| **D4** | 示例库：sandbox+OKF+表风格 canonical demos | P4/P5 | examples/ | 文档 | 维护 | 低 |

### E. 评测与产品化

| ID | 方向 | 解决 | 做什么 | 依赖 | 风险 | 成本 |
|----|------|------|--------|------|------|------|
| **E1** | 完成 Wave B1 harness（constitution / writeback / call_site） | P4 | 已规划 | agent | — | 中 |
| **E2** | 指标面板：ReuseRate / ProofRate / TokenCost | P4/P7 | metrics 写回或 catalog | agent | 虚荣指标 | 中 |
| **E3** | Live 金样矩阵（sandbox 开/关 × 路由后端） | P4 | CI optional | CI | flaky | 中 |
| **E4** | 教程：「只读一份 .mq.md 理解 Agent」 | — | doc/tutorial | 文档 | — | 低 |

### F. 明确不优先 / 需单独论证

| ID | 方向 | 理由 |
|----|------|------|
| **F1** | 第二套 Agent Framework（Python/TS 对象图权威） | 违宪 |
| **F2** | 向量库升为记忆权威 | 违宪；可作证据工具 |
| **F3** | 默认通用 shell Agent | 与 Marqdo-only 沙盒战略冲突 |
| **F4** | 整库 `ext/web` 进默认沙盒 | 已否决 |
| **F5** | 人人 microVM 本地默认 | DX 过重；留给 hosted |

---

## 3. 执行顺序（O1→O5 locked）

| 波次 | 方向组合 | 为何先做 |
|------|----------|----------|
| **O1** | A1 轻量（opt-in `sandbox=True`）+ A3 + D1 + D4 | 立刻接上刚落地的沙盒，风险可控 |
| **O2** | A2 + B1 + B3 + E1 | 验证→晋升→可度量 Skill Lift |
| **O3** | C1 + C2 + B6 | 降本与 EFI 硬门槛 |
| **O4** | A4 + D2 + A5 | 质量门禁与人审 |
| **O5** | B4 + C3 | 失败隔离与离线提示优化 |
| **O∞** | C4 / A6 | 研究项，单独立项 |

切片实现见 [ext-ai-optimize.md](../roadmap/ext-ai-optimize.md)。

---

## 4. 验收原则（任一波次）

- 金样：有/无沙盒、有/无 OKF hit、路由后端切换。  
- 指标：至少报告 ReuseRate 或 TokenCost 之一相对基线变化。  
- 文档：`.mq.md` 示例仍可读为 Markdown（表/链接取元，少 json 胶水）。  
- 红线：不改 `doc/next/*`；ext 不调 `host_*`。

---

## 5. 落地

O1–O5 已锁定；实现见 [roadmap/ext-ai-optimize.md](../roadmap/ext-ai-optimize.md)。**仍不改** `doc/next/*`。
