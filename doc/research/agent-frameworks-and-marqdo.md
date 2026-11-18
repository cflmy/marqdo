# 调研：主流智能体开发框架与 Marqdo「代码即文档 / 文档即知识库」优势

| | |
|---|---|
| 状态 | **调研笔记 · 供 `ext/agent` 深入优化参考** |
| 日期 | 2026-08-28 |
| 范围 | 2025–2026 开源/厂商智能体框架形态；对照 Marqdo `ext/ai/agent` 设计 |
| 相关设计 | [ext-agent.md](../design/ext-agent.md) · [ext-agent-plan.md](../design/ext-agent-plan.md) · [ext-agent-parent.md](../design/ext-agent-parent.md) · [okf.md](../design/okf.md) · [stdlib-writeback.md](../design/stdlib-writeback.md) · [stdlib-subtask.md](../design/stdlib-subtask.md) |
| 姊妹调研 | [okf-and-marqdo.md](okf-and-marqdo.md) · **A0–A4 后缺口** [agent-framework-gaps-after-a4.md](agent-framework-gaps-after-a4.md) |
| 外部对照（抽样） | [Langfuse 框架对比（2026-07）](https://langfuse.com/blog/2025-03-19-ai-agent-comparison) · [LangGraph persistence](https://docs.langchain.com/oss/python/langgraph/persistence) · [Microsoft Agent Framework / AutoGen 维护模式](https://github.com/microsoft/autogen) · CrewAI / OpenAI Agents SDK / Claude Agent SDK / Google ADK / LlamaIndex 公开文档 |

---

## 0. 一句话结论

主流智能体框架几乎都在解决同一组工程问题——**编排、工具、状态、记忆、可观测、人机协同**——但它们的「权威真相」默认落在 **Python/TS 对象图 + 会话 transcript + 外部 Store/向量库** 上。

Marqdo 的宪法把权威真相定在 **可执行 Markdown（`.mq.md`）**：人写的程序、模型读的提示、执行后的写回、可复用的任务技能，是**同一份文档的不同生命阶段**。这不是「又一个 Agent SDK」，而是把 **Agent 开发本身做成文档工程**。

由此产生的突出优势（后文展开）：

1. **编排可读 = 提示词不再黑盒**（源码即上下文）  
2. **过程可审 = 写回进源文件，而不是只进遥测平台**  
3. **记忆可 git = 成功经验晋升为任务知识包（OKF），审计=仓库历史**  
4. **复杂任务=子程序，而不是无限加长的 chat**（工作簿 + `subtask`）  
5. **确定步骤固化为代码，不确定步骤才问模型**（父 Plan-and-Move）  
6. **语言与知识形态同构**（与 Google OKF「Markdown 知识包」同向，但多走「可执行」一步）

**优化 `ext/agent` 时应守住这些差异化，而不是向 LangChain 类「隐藏循环 + TOOL: 协议」回退。**

---

## 1. 调研方法与边界

### 1.1 做了什么

| 动作 | 说明 |
|------|------|
| 横向扫盘 | 2026 年主流开源框架：LangGraph / LangChain DeepAgents、CrewAI、OpenAI Agents SDK、Claude Agent SDK、Google ADK、Microsoft Agent Framework（AutoGen 继承者）、Pydantic AI、LlamaIndex、Strands、Mastra / Vercel AI SDK、smolagents 等 |
| 结构抽象 | 抽取「框架如何形成」的共性分层，而不是比 GitHub stars |
| 对照本仓 | 精读 `ext-agent*`、`okf`、`writeback`、`subtask`、parent Plan-and-Move；对照旧 TOOL: 薄循环为何被否定 |
| 找优越点 | 只保留 **Marqdo 结构上必然领先**、而非「以后可以加插件追上」的点 |

### 1.2 不做的

- 不宣称 Marqdo 在吞吐、多模态、托管运维上已全面领先（这些不是当前差异化）。  
- 不把「有没有向量检索」当成胜负手——那是应用层可加能力。  
- 不做厂商营销话术复读；优缺点按**开发者体验与系统真相源**评。

---

## 2. 智能体框架是怎么「形成」的

无论品牌如何包装，成熟框架通常收敛为同一条装配线：

```text
┌─────────────────────────────────────────────────────────────┐
│ 1. Model I/O                                                │
│    补全 / 工具调用 / 流式 / 多模供应商适配                     │
├─────────────────────────────────────────────────────────────┤
│ 2. Agent 单元                                               │
│    指令(system) + 工具表 + 可选角色/人设                      │
├─────────────────────────────────────────────────────────────┤
│ 3. 编排（Orchestration）                                    │
│    图 / 角色队 / 对话群 / 工作流 API / supervisor             │
├─────────────────────────────────────────────────────────────┤
│ 4. 状态与记忆                                               │
│    短：thread checkpoint / 会话历史                          │
│    长：Store / 向量 RAG / 用户偏好库                         │
├─────────────────────────────────────────────────────────────┤
│ 5. 工具与行动面                                             │
│    函数 schema、MCP、沙箱、权限、CodeAct                      │
├─────────────────────────────────────────────────────────────┤
│ 6. 可观测与治理                                             │
│    Trace、评测、护栏、HITL、配额                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.1 三种主流「编排哲学」

| 范式 | 核心隐喻 | 代表 | 可预测性 | 原型速度 |
|------|----------|------|----------|----------|
| **图 / 状态机** | 节点=步骤，边=转移，状态显式 | LangGraph、Google ADK workflow、MAF workflows | 高 | 中–慢（先画图） |
| **角色协作** | Agent=工种，Crew=班组 | CrewAI、早期 AutoGen 群聊 | 中 | 快 |
| **薄循环 + 工具** | 「模型自己决定下一步」 | OpenAI Agents SDK、Claude Agent SDK、Strands、smolagents | 低–中（靠护栏） | 最快 |

2026 年的趋势不是「哪个范式消灭其它」，而是：

- **生产复杂流** → 图（可控、可恢复、HITL）。  
- **内容/研究班组** → 角色。  
- **编码助手 / 生态绑定** → 厂商 SDK 自带 hardened loop。  
- **LangChain DeepAgents** 等开始把「规划器 + 子代理 + 文件记忆」打包成 opinionated harness——说明业界也意识到：纯 chat 不够，需要**文件与子上下文**。

### 2.2 记忆层的行业共识（与 Marqdo 对照的关键）

以 LangGraph 文档为代表（行业事实上的标准划分）：

| 层 | 行业做法 | 作用 |
|----|----------|------|
| **短时** | Checkpointer / thread_id | 会话续跑、崩溃恢复、time-travel |
| **长时** | BaseStore / 向量库 / Mem0·Zep 等 | 跨会话事实、偏好 |
| **检索知识** | RAG corpus | 领域文档，不与会话状态混用 |

常见架构错误：把 checkpointer 当成长时记忆——换 `thread_id` 就「失忆」。

**这一点对 Marqdo 极关键：**  
行业把「记忆」拆成多个后端；Marqdo 默认把**任务级成功经验**落成 **`.mq.md` + agent-kb（OKF）**，用文件系统与 git 当记忆平面——见 §7。

### 2.3 框架「形成产品」的路径

```text
PoC（脚本循环）
  → SDK（Agent + tools）
    → 编排（图 / 多代理）
      → 持久化（checkpoint / store）
        → 平台（托管、评测、护栏、计费）
```

越往后，**编排源码与运行时真相越分离**：提示在字符串常量或 Prompt Hub，状态在 DB，轨迹在 Langfuse，知识在向量库。人要复盘一次失败，往往要开四个系统。

Marqdo 的赌注是：**尽量把真相拉回单一 `.mq.md` 时间线**。

---

## 3. 代表框架解剖（如何形成 + 优缺点）

### 3.1 LangGraph（显式状态图）

**如何形成**

- 把 Agent 步骤建模为 **有向图**：节点读/写共享 state，条件边决定下一跳。  
- 编译为可执行应用；挂 checkpointer → 持久化、HITL interrupt、故障续跑。  
- 长时记忆另挂 Store；生态上接 LangChain 工具与 LangSmith/Langfuse 轨迹。

**优点**

- 生产向可控性最强；分支、重试、并行边清晰。  
- 状态类型化后，测试与回放（time-travel）可行。  
- 云中立：图在你自己的进程里跑。

**缺点**

- 上手成本高：先设计图，再写节点。  
- 「提示词 / 业务说明」仍常散落在 Python 字符串；**图代码 ≠ 给人读的说明书**。  
- 记忆与知识仍依赖外置 Store/RAG；审计链路长。  
- DeepAgents 等上层 harness 说明：光有图仍不够「文件化协作」。

**对 Marqdo 的启示**

- 要学其「显式控制与可恢复」；不要学其「状态默认在对象里」。  
- Marqdo 多步的工作簿修订环 ≈ **把图展开成磁盘上的可执行文档**。

---

### 3.2 CrewAI（角色班组 + Flows）

**如何形成**

- Agent = role / goal / backstory；Crew 协调任务；近年加 **Flows** 把确定性业务逻辑包住自治班组。

**优点**

- 多代理叙事直观，原型极快。  
- 适合内容流水线、调研班组。

**缺点**

- 协作转移大量靠描述，**可预测性弱于图**。  
- 过程真相仍是运行时对象与日志；与「文档即程序」无关。  
- HITL / 企业治理相对弱。

**对 Marqdo 的启示**

- 「多角色」可用多个 `# 智能体` / 多个工作簿表达，但应以**可跑文档**为边界，而不是隐式群聊。

---

### 3.3 AutoGen → Microsoft Agent Framework（MAF）

**如何形成**

- AutoGen：对话对 / 群聊驱动；历史在内存。  
- 2026：AutoGen **维护模式**；能力并入 **Microsoft Agent Framework**（与 Semantic Kernel 合流）：ChatAgent + 多种编排（sequential / concurrent / handoff / group / Magentic）+ 工作流图 + Azure Foundry 治理面。

**优点**

- 企业栈（.NET / Azure）一体；A2A、MCP、护栏、托管轨迹。  
- CodeAct 等模式：让 Agent **写代码执行**而非只吐 JSON——与「固化为代码」方向有共鸣。

**缺点**

- 生产治理深度常绑定云平台；开源核心与托管能力分层。  
- 编排语言仍是通用编程语言；**业务知识与程序不同构**。  
- 对话范式易导致「无限闲聊」式多代理。

**对 Marqdo 的启示**

- CodeAct / 固化步骤：与 Marqdo「确定步骤写成 `##`」同向，但 MAF 写的是 Python；Marqdo 写的是**人与模型共用的 `.mq.md`**。

---

### 3.4 OpenAI Agents SDK / Claude Agent SDK / Google ADK / Strands

**如何形成（共性）**

- 提供 **hardened agent loop**（工具、handoff、权限、会话）。  
- 深度绑定自家模型或云（OpenAI / Anthropic Claude Code harness / Gemini+ADK / Bedrock）。

**优点**

- 最快接入「能干活的助手」；权限与工具路径经过产品锤炼（尤其 Claude Code 同源 SDK）。  
- 官方 tracing、realtime 等增值。

**缺点**

- 编排与提示仍在宿主语言；**跨团队知识复用靠复制仓库或 Prompt 平台**。  
- 供应商锁定风险；换模时常换范式。  
- 「记忆」默认会话式；企业知识仍外挂。

**对 Marqdo 的启示**

- 模型 I/O 应继续放在 `ext/llm`，**不要**在 agent 里重造聊天栈。  
- 不要把「官方 loop」当哲学：Marqdo 的哲学是文档驱动，不是 SDK 驱动。

---

### 3.5 LlamaIndex / Haystack（文档与 RAG 优先）

**如何形成**

- 以索引、检索、查询引擎为中心；Agent 是检索之上的编排层。

**优点**

- 企业文档智能、多格式摄取成熟。  
- 「知识」有一等公民地位。

**缺点**

- 知识默认是 **被检索的被动语料**，不是 **可执行的任务程序**。  
- Agent 编排能力弱于 LangGraph 系；常需再叠一层框架。  
- 索引与源文档易漂移（双源真相）。

**对 Marqdo 的启示（尖锐对比）**

- LlamaIndex：**文档 → 向量 → 检索 → 回答**。  
- Marqdo：**文档 → 可执行语法 → 运行 → 写回 →（可选）晋升为任务知识包**。  
  同一「Markdown 知识」在 Marqdo 里可以是 **会跑的技能**，而不仅是 chunk。

---

### 3.6 Pydantic AI / Mastra / Vercel AI SDK

**如何形成**

- 强调类型安全（Pydantic）或 TS 全栈 DX（Mastra / Vercel）。  
- 把 Agent 当普通后端组件：校验、otel、耐久执行。

**优点**

- 工程化体验好；与现有 Web/API 栈融合。

**缺点**

- 仍是「代码库里的 Agent」；非技术干系人无法直接打开同一份编排当说明书。  
- 知识沉淀路径仍外置。

---

### 3.7 横向对比表（开发者真相源）

| 维度 | 典型框架落点 | Marqdo 落点 |
|------|--------------|-------------|
| 编排定义 | Python/TS 图或类 | `.mq.md` 标题/表/调用 |
| 系统提示 | 字符串 / Prompt Hub | 站立提示写在同一文档 |
| 工具 | 装饰器函数 / MCP | runbook 内 `##` + `subtask` |
| 短时状态 | checkpoint / session | 工作簿文件 + 写回块 + view |
| 长时记忆 | Store / 向量 | agent-kb（OKF）+ git |
| 复杂任务 | 子图 / 子代理上下文窗 | **子 `.mq.md` 程序** `spawn path=` |
| 复盘 | Langfuse trace | `git log` + 打开工作簿 + view output-card |
| 成功复用 | 手工抽 prompt / 存 Store | **固化 + OKF 晋升 / lookup** |

---

## 4. 行业共性痛点（框架越强，痛点越清晰）

下列痛点在 LangGraph、CrewAI、厂商 SDK 文档与实践文章中反复出现；它们正是 Marqdo 要正面打穿的：

### 4.1 提示词与编排分离

- 行为由 system prompt + 隐藏模板决定；code review 审不到「模型真正看见什么」。  
- Prompt 漂移与代码漂移双轨发生。

### 4.2 过程不透明或只对平台透明

- 本地只有最终返回值；细节在 SaaS trace。  
- 合规团队要的「这份文档当时怎么决策的」难以直接交付。

### 4.3 记忆碎片化

- thread / store / RAG / 日志四套系统；「上次同类任务怎么做成的」检索成本高。  
- 成功经验很少自动变成**可再执行资产**。

### 4.4 多步任务变成超长 transcript

- 上下文窗口被工具输出撑爆；DeepAgents 才开始「卸到文件系统」。  
- 说明业界被迫走向 **文件化**——而 Marqdo **一开始就以文件为执行边界**。

### 4.5 确定性逻辑仍反复问模型

- 已验证的步骤下一轮又被重新「推理」，浪费且不稳。  
- 缺少「晋升为普通代码」的一等路径。

### 4.6 Agent 框架与知识格式断裂

- Google OKF 等推动「知识用 Markdown 包交换」，但主流 Agent 框架并不把 **可执行程序** 当作知识页。  
- 结果：知识包给人策展，Agent 另跑一套 Python。

---

## 5. Marqdo 在智能体域的设计思想（必须找准）

### 5.1 宪法重申

| 口号 | 在 Agent 语境下的含义 |
|------|------------------------|
| **代码即文档** | 智能体编排、工具表、站立提示、任务叙述写在同一份 `.mq.md`；打开即读、即可跑 |
| **文档即知识库** | 写回块、工作簿、OKF 任务页都是知识；Agent 与人共用同一交换面 |
| **文档能跑通 = 约定成立** | 金样 / `marqdo run` / view 是验收，不是「旁边另有一份真实现状」 |

旧「TOOL: 黑盒循环」被官方设计明确否定，正是因为它把 Marqdo **降级成了 LangChain 的薄壳**。

### 5.2 两级执行（与业界范式对齐但不抄袭）

| Marqdo | 业界近似 | 关键差异 |
|--------|----------|----------|
| `## step` 单步 | 薄 Agent loop | 注入**本文件源码 + 调用位置 + Skill**；默认写回 |
| `## plan` 多步 | 规划器 + 子代理 | 产物是 **工作簿 `.mq.md`**；经 `subtask` 整文件执行；父以补丁演进 |
| 父 Plan-and-Move | supervisor / orchestrator | 行动面：READ/CALL/DECISION；**禁止整文件 LLM 重写** |
| OKF agent-kb | long-term store | 命中的是 **可再跑的任务技能**，不是抽象 embedding 片段（可再叠加 soft near） |

### 5.3 执行与隔离

- 工具与子工作簿走 `lib/subtask`（`spawn fn=` / `spawn path=`），与主推理隔离——对应业界「子代理独立上下文」，但边界是 **文件与子进程语义**，不是仅换一个 message list。

---

## 6. Marqdo 的必要突出优势（核心章节）

下列优势要求满足：**若取消「可执行 Markdown 为权威」，优势即消失**。这才是「必要」优越点，而不是功能清单攀比。

### 6.1 单一真相源：编排 / 提示 / 结果共址

**优势陈述**  
模型在 `step`/`plan` 中看到的上下文，可以就是**人类正在编辑的那份源码**（及调用点）。站立提示不是隐藏 system blob，而是文档段落。

**为何必要**  
消除「提示词仓库 vs 代码仓库」双源；code review、教学、合规审计面对同一人造物。

**相对框架**  
LangGraph/CrewAI/SDK：提示常在字符串或控制台配置；trace 在另一系统。Marqdo：`view` 在语句下挂 output-card，与源同行。

### 6.2 写回 = 内建的「Notebook 输出单元」式过程记忆

**优势陈述**  
`lib/writeback` 把成功/失败写入 `<!-- marqdo-out … -->`，锚在调用行下。智能体默认写回后，**过程留在源文件**，不是只留在遥测。

**为何必要**  
教学与调试要「看见刚才那一步」；失败成为可读避坑记录；成功可被后续命中。

**相对框架**  
业界靠 Langfuse/LangSmith；本地仓库往往是空的。Marqdo 仓库本身就是过程库（可再导出到 trace，但不依赖它才能复盘）。

### 6.3 复杂任务 = 生成并演进子程序，而不是堆 transcript

**优势陈述**  
`plan` 创建 `.marqdo/agent-runs/*.mq.md`，用 `spawn path=` 跑完整 `# main`。父观察**真实退出码、返回值、写回、源码**，再 FIND/REPLACE 补丁。

**为何必要**  
上下文窗口有限；长工具输出必须卸出。DeepAgents 用「文件系统抽象」补课；Marqdo **语言层原生就是文件程序**。

**相对框架**  
子代理多是「新对话 + 摘要」；Marqdo 子任务是 **可独立 `marqdo run` 的文档**，可被人类直接打开修改后重跑。

### 6.4 确定步骤固化为代码；模型只处理不确定残差

**优势陈述**  
父作为「智能体开发大师」：能写成 `##` 的就固化；子智能体只跑不确定的 `step`。`DECISION: DONE` 可 solidify + OKF 晋升。

**为何必要**  
降本、降方差、形成组织资产。业界常把每次任务重新推理；Marqdo 把「学会的技能」编译回语言。

**相对框架**  
MAF CodeAct 写出的是通用语言脚本；Marqdo 固化的是 **仍保持文档属性的 `.mq.md` 技能**——人不用会 Python 也能读懂并改。

### 6.5 文档即知识库 + OKF：记忆可交换、可策展、可信任

**优势陈述**  
任务知识包（`.marqdo/agent-kb/`）以 OKF 形承载 `Marqdo Task` / skill，`resource:` 必须指向可执行 `.mq.md`。lookup / near / soft_match 走插件，不进核心 host。

**为何必要**  
Google OKF 证明行业需要「Markdown 知识包」；Marqdo 多走一步：**知识页声明的可执行资源真能跑**。记忆不是不可解释的向量命中，而是 **带路径的技能复用**。

**相对框架**  
LlamaIndex 检索散文块；Store 存 JSON 事实。Marqdo agent-kb：**命中 → 再跑同一技能路径**（精确 / 规范化 / near / 父裁决）。

### 6.6 审计与协作默认路径 = Git

**优势陈述**  
工作簿演进、写回、晋升都是文件变更 → `git log` / PR / blame。

**为何必要**  
企业已有代码治理；不必为 Agent 另建「提示治理平台」才能起步。

**相对框架**  
多数框架的状态在 DB；要审计得接专用平台。Marqdo 从第一天就兼容软件工程工作流。

### 6.7 中英双面 API 与「说明书即程序」教学属性

**优势陈述**  
`# 智能体` / `# agent`、`单步`/`step`、`多步`/`plan` 对称；教程文档可以直接是金样。

**为何必要**  
智能体开发教育成本高；Marqdo 把教材和运行物合一（与 quantum/math 扩展同一哲学）。

### 6.8 分层纪律：扩展不脏核心

**优势陈述**  
`ext/**` 禁 `host_*`；agent-kb 在 `plugins/agent`；核心保持瘦。

**为何必要**  
避免「Agent 框架把语言运行时变成巨石」——这是许多 Python Agent 栈的终局病。

---

## 7. 优势地图（一图看懂定位）

```text
                 可预测性 / 生产控制
                        ▲
                        │  LangGraph / MAF workflows
                        │
                        │         ★ Marqdo plan
                        │           (workbook = program)
                        │
     角色班组 ──────────┼────────────── 薄循环 SDK
     CrewAI             │           OpenAI / Claude SDK
                        │
                        │  LlamaIndex (RAG)
                        │
                        └────────────────────────────►
                              文档/知识一等公民程度

★ Marqdo 同时偏右上：控制来自「可执行文档+补丁环」，
  知识来自「同源 Markdown + OKF」，而非向量库单独称王。
```

---

## 8. 诚实边界：什么时候不该硬吹 Marqdo

| 场景 | 更合适的选择 | 原因 |
|------|--------------|------|
| 超大规模并行图、成熟 Python 数据栈 | LangGraph + 现有 infra | 生态与库广度 |
| 强绑定 Azure 治理 / .NET | MAF | 平台一体 |
| 纯文档问答 / 海量非结构化语料 | LlamaIndex 等 | 摄取与检索成熟；Marqdo 可 **调用** 它们作工具，不必重造 |
| 只要一个 ChatGPT 式助手周末原型 | 厂商 Agents SDK | 启动最快 |
| 需要强实时语音 / 多模原生 | 厂商 SDK | 非 Marqdo 当前主场 |

Marqdo 的正确叙事不是「取代所有 Agent 框架」，而是：

> **当你希望智能体开发成果沉淀为组织可维护的文档资产，并且编排本身必须可教、可审、可跑时——选 Marqdo。**

---

## 9. 对 `ext/agent` 深入优化的建议方向（调研导出）

优化应强化 §6 优势，避免回退到「隐藏 TOOL 循环」。

| 优先级 | 方向 | 理由 |
|--------|------|------|
| P0 | **工作簿质量与补丁可靠性**（精确 FIND/REPLACE、固化、观察切片） | 直接决定「子程序演进」是否优于 transcript |
| P0 | **OKF 命中体验**（精确 / near / soft_match 默认真策略、策展 UX） | 落实「文档即知识库」的复用飞轮 |
| P1 | **过程可见**（streaming / view 过程卡，见 [agent-streaming.md](../roadmap/agent-streaming.md)） | 对标 Langfuse，但真相仍写回文档 |
| P1 | **Skill / 源码注入的预算与渐进披露** | 避免上下文爆炸，同时保持「源码即提示」 |
| P2 | **与外部 RAG/MCP 的工具化接入** | 补齐语料检索，但不把向量库升为权威真相 |
| P2 | **评测金样：同类任务二次 plan 应命中 kb 快路径** | 用测试锁住差异化 |
| 避免 | 再引入私有 JSON 工具协议作主路径 | 与宪法冲突 |
| 避免 | 整文件 LLM 重写工作簿 | 已在 parent 设计禁止 |

验收提问（产品自评）：

1. 新人能否只读一份 `.mq.md` 理解 Agent 在干什么？  
2. 失败后能否在 git 与 view 内完成复盘，不强制打开外部 trace？  
3. 成功任务下周能否以 **跑同一技能** 的方式复用，而不是「再聊一遍」？  
4. 确定步骤是否越来越多地变成普通 `##`，模型调用是否下降？

若四问皆是——优化方向正确。

---

## 10. 结论

2026 年的智能体框架市场在 **图控、角色班组、厂商 hardened loop、RAG 知识** 四条线上分化；共同短板是 **真相源碎片化** 与 **成功经验难以变成可执行组织资产**。

Marqdo 用「代码即文档、文档即知识库」把 Agent 开发收束到 **可执行 Markdown 生命循环**：

**书写 → 注入模型 → 执行 → 写回 →（多步）补丁演进 → 固化 → OKF 晋升 → 再命中。**

这不是功能点堆砌，而是架构级选择。深入优化 `ext/agent` 时，应把资源砸在 **工作簿、写回、固化、知识包命中、过程可视** 上，把「文档驱动的智能体开发框架」做成可感知的产品体验——这才是相对 LangGraph / CrewAI / 厂商 SDK **不可替代** 的优越点。

---

## 11. 参考与延伸阅读

### 本仓

- [ext-agent.md](../design/ext-agent.md) — 框架总设计（否定 TOOL 薄循环）  
- [ext-agent-plan.md](../design/ext-agent-plan.md) — 多步工作簿  
- [ext-agent-parent.md](../design/ext-agent-parent.md) — 父 Plan-and-Move  
- [okf.md](../design/okf.md) · [okf-and-marqdo.md](okf-and-marqdo.md)  
- [stdlib-writeback.md](../design/stdlib-writeback.md) · [stdlib-subtask.md](../design/stdlib-subtask.md)  
- [roadmap/agent-streaming.md](../roadmap/agent-streaming.md) · [roadmap/okf-near-match.md](../roadmap/okf-near-match.md) · [roadmap/ext-agent-optimize.md](../roadmap/ext-agent-optimize.md)  
- [agent-framework-gaps-after-a4.md](agent-framework-gaps-after-a4.md) — A0–A4 之后还差什么  

### 外部（抽样，2025–2026）

- Langfuse: *Comparing Open-Source AI Agent Frameworks*（2026-07 更新盘点）  
- LangGraph docs: Persistence（checkpointer vs store）  
- Microsoft: AutoGen maintenance → Agent Framework migration  
- CrewAI: Crews + Flows  
- OpenAI Agents SDK / Claude Agent SDK / Google ADK 公开 README  
- Google OKF SPEC v0.2（知识包形态；与 Marqdo 可执行源对照）
