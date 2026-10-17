# 外联内核 · 文档会话 · 决策记录

| | |
|---|---|
| 状态 | **Decided（由用户授权代为拍板）** |
| 日期 | 2026-10-17 |
| 范围 | 外联库（`lib/foreign`）· 外联内核 · 文档会话 · view notebook 交互 |
| 相关 | [foreign-kernel.md](../design/foreign-kernel.md) · [notebook-session.md](../design/notebook-session.md) · [notebook-syntax.md](../design/notebook-syntax.md) · [foreign-v2.md](../roadmap/foreign-v2.md) · [notebook-session.md](../roadmap/notebook-session.md) |
| 核心表面 Δ | **无**——不新增 Markdown 标记；只使用既有 `#` / `##` / 表格 / 反引号 |

> 本文件记录「为什么这样设计」。实现细节分别落在设计文档与路线图中；本文件不重复 API 表。

---

## 0. 背景与授权

用户的核心诉求：

1. Marqdo 的本质是 **代码即文档、文档即知识库**；`.mq.md` 天生兼容 Markdown / OKF 知识库。
2. 外联库的初衷是**对标 Jupyter**：让文档与代码并行有用；但 Jupyter 有 Git 难跟踪、隐藏状态、execution_count 噪声等问题，Marqdo 要解决并超越。
3. `view` 模式天然适合这种交互（已有执行按钮、结构视图、writeback、trace）。
4. 之前外联库功能太单一；接下来必须支持：
   - 多个代码块共享外联状态；
   - 单步执行；
   - 切换哪些外联内核共享状态；
   - Marqdo 的状态/变量被外联使用；
   - 外联的状态/变量被 Marqdo 使用。
5. 用户要求：**安全第一、效果最好、功能最强悍、不降级**；充分测试；每份讨论落成详细文档；文档提交并推送后再开发。
6. 用户强调：**设计语法不要用代码思维**，语法要天然可读。

本文档 = 对该授权的决策记录。后续实现若与本文冲突，先改本文并说明理由。

---

## 1. 关键结论

### 结论 A：外联库不是“跑 Python 的函数”，而是多语言内核运行时

`lib/foreign` 的定位升级为：

> **文档运行时与外部语言运行时之间的内核/会话层**：启动内核、执行代码单元、持久化状态、双向类型桥、生命周期与安全。

它不等同于 `subtask`（一次性并发任务），也不等同于 `writeback`（输出持久化）。

### 结论 B：Jupyter 级体验需要两层，不是一层

| 层 | 名称 | 责任 |
|---|---|---|
| **F** | Foreign Kernel Runtime | 内核进程、语言发现/注册、JSON 桥、会话、安全、记录 |
| **D** | Document Session | 文档会话、cell index、kernel 分配、单步、状态面板、执行日志、view 交互 |

上一版 `foreign-v2` 只覆盖 F。共享状态 / 单步 / 切换内核属于 D。两层分开设计、分开验收。

### 结论 C：Hybrid 执行模型

- **Program mode（规范模式）**：`marqdo run` / CI / 可复现执行。文档是程序；外联通过显式 `foreign.session_*` / `call` 编排；结果可重复。
- **Notebook mode（交互模式）**：`view` 中按 cell 执行、单步、切换内核、查看状态。适合探索，不默认写入文档。
- 两者共享同一个内核运行时与同一套 cell 索引；Notebook mode 是 Program mode 的**交互镜头**，不是替代品。

理由：如果让 cell 执行取代文档程序语义，Marqdo 就退化成 Jupyter；如果只有 Program mode，就失去 Jupyter 的即时反馈。Hybrid 同时保留两者。

### 结论 D：命名内核；同内核共享，跨内核/跨语言显式传值

- 一个文档可以有多个命名内核：`py:分析`、`py:绘图`、`js:前端`。
- 同一内核的 cell 共享全局状态。
- 不同内核（即使同语言）默认隔离。
- 不同语言不共享进程内存；只能通过 JSON / 文件 / DB 显式交换。
- 默认每个文档、每种语言一个 `default` 内核；命名内核按需开启。
- 跨文档默认隔离；全局内核以后再做。

### 结论 E：三种单步，分阶段实现

| 单步 | 含义 | 版本 |
|---|---|---|
| **Cell step** | 执行下一个代码单元，状态保留 | v2 必做 |
| **Marqdo statement step** | 逐条执行 Marqdo 语句，查看变量 | v2 复用现有 debugger |
| **Foreign line step** | 进入 Python/Node 逐行 | v2 不做；后续接 DAP/pdb/debugpy |

- 外联 cell 在 statement step 中是原子操作。
- “Step into foreign” 是独立项目，不能混入首版。

### 结论 F：显式状态桥优先，不做隐式自动镜像

- 默认通过 `session_set` / `session_get` / `call input` / `call result` 交换，JSON 拷贝。
- 不默认把 Marqdo 变量自动注入 Python，也不默认把 Python 变量自动写回 Marqdo。
- 原因：命名冲突、Secret 泄漏、类型丢失、隐式状态、不可复现、大对象拷贝。
- 后续可选 `mirror=True`：只同步 `mq_` 前缀、白名单类型、按值拷贝，且默认关闭。

### 结论 G：不新增围栏元数据；用文档原生的“内核表 / 会话表”

- 拒绝 ` ```python kernel=分析 `、`name=`、YAML kernel 配置、魔法注释。
- 使用既有绑定表 + GFM 表格（放在 `# main` 体内，与 cell 同作用域）：
  - `内核表` / `kernels`
  - `会话表` / `session`
- 表格是**声明**，不是自动执行；只有 Run / Step 触发。
- 解析器不变，核心表面零增长；view/session 读取这两个约定绑定。

理由：`kernel=` 是 IDE 配置思维，破坏“代码即文档”；绑定表是文档目录/执行计划，人可读、机器可解析，且不把文档拆成多个对象作用域。

### 结论 H：输出默认临时，writeback 显式；状态不写进 `.mq.md`

- Notebook mode 的输出默认只在 view 中显示，不修改文档。
- 需要持久化时显式 writeback（沿用 `<!-- marqdo-out … -->`），可 Git 跟踪、可忽略。
- 执行日志放 `.marqdo/sessions/…`（默认 git 忽略）。
- 不把 execution_count / kernel 状态 / base64 输出写进 `.mq.md`。

### 结论 I：首批会话语言 Python + Node

- Python：Jupyter 的主场景，优先支持。
- Node：v1 已有基础，且当前开发环境可用。
- 15 个语言预设继续用于**一次性执行**；R / Julia / Ruby 等的持久会话通过自定义 bootstrap 逐步加入，不阻塞首版。

### 结论 J：v2 不做“外联回调 Marqdo 函数”

- 状态/值双向：✅
- Marqdo 代码作为源码文本传给外联：✅（外联按自己的语言执行）
- 外联代码调用 Marqdo `##` 函数：❌ v2；后续做 callback RPC（控制通道 + 重入 + 死锁处理）。
- 原因：重入解释器、线程安全、死锁、安全面都远超首版范围；先把数据/状态桥做稳。

### 结论 K：安全与测试是一等公民

- 所有 spawn / probe / session 入口检查 `allow_exec`。
- 进程组超时清理；有界 IO；stdin 写线程；输出上限；并发上限。
- 每文档生命周期，关闭即 kill；跨文档不共享。
- 测试：node + bash 为最小环境主路径；python 条件覆盖；单元测试覆盖切分/模板/JSON；金样例覆盖记录/超时/会话/并行/状态桥。

---

## 2. 逐问逐答（保留原始讨论颗粒度）

### Q1：外联库是做什么的？

**答**：它是文档的多语言执行总线，负责：

1. 内核生命周期（启动/重启/中断/关闭/切换）；
2. cell 执行（源码 → kernel → stdout/stderr/result）；
3. 状态持久化（同一 kernel 的 cell 共享命名空间）；
4. 类型桥（Marqdo Value ↔ JSON ↔ 语言对象）；
5. 环境发现与配置（probe/register/venv/config）；
6. 记录与安全（耗时、退出码、截断、cap、清理）。

它不负责：文档语法（Language）、UI 布局（Document）、知识目录（Catalog）。这些由相邻层提供。

### Q2：是否支持外联库共享状态？

**答**：支持，但严格分层：

- 同一 kernel：共享全局变量。
- 同一语言不同 kernel：隔离。
- 不同语言：不共享内存，只能显式传值。
- Marqdo ↔ kernel：显式桥。
- 跨文档：默认隔离。

### Q3：能否单步执行？

**答**：能，但要定义清楚：

- cell step：v2 做。
- Marqdo statement step：现有 debugger 已有，v2 接入 view。
- foreign line step：v2 不做；后续 DAP。

### Q4：能否切换哪几个外联库共享状态？

**答**：能，用命名内核：

- `py:分析` 下所有 cell 共享；
- `py:绘图` 隔离；
- 每个 cell 有 kernel 归属；
- 切换归属不迁移状态，需要重跑；
- 跨语言用显式桥。

### Q5：Marqdo 的代码/状态能否被外联使用？外联的状态/变量能否被 Marqdo 使用？

**答**：

- Marqdo 值 → 外联：`session_set` / `call input` / `session_eval input`；JSON 拷贝。
- 外联值 → Marqdo：`session_get` / `call` 返回值；赋值。
- Marqdo 代码 → 外联：作为源码文本传递；外联按自己的语言执行，不是 Marqdo 语义。
- 外联代码 → Marqdo 函数：v2 不做，后续 callback RPC。
- 自动镜像：v2 不做，后续可选。

### Q6：如何超越 Jupyter？

**答**：不是“功能更多”，而是把 Jupyter 的隐藏面变成显式面：

| Jupyter | Marqdo |
|---|---|
| `.ipynb` JSON 混入输出/execution_count | `.mq.md` 源码干净；输出可选 writeback |
| 隐藏 kernel 状态 | 内核 + 文档会话 + 状态面板 + journal |
| 一个 notebook 一个 kernel | 命名内核 + 每 cell 归属 + 显式桥 |
| 只有 cell step | cell step + statement step |
| 手动 Run All | restart & run all + dirty/stale + replay |
| 无知识目录 | OKF / catalog；cell 是可发现知识单元 |

---

## 3. 被拒绝的方案

| 方案 | 为什么拒绝 |
|---|---|
| 围栏元数据 ` ```python kernel=分析 ` | Language 层变更；需 ADR；把文档语法变成 IDE 配置；破坏可读性 |
| YAML `kernels:` 配置 | 代码思维；与“文档即知识”冲突；表格已能表达 |
| 魔法 HTML 注释 `<!-- kernel: 分析 -->` | 不可见、易漂移、读者不可见；违背可读性 |
| 自然语言 prose 解析（“把 xs 送给分析内核”） | 非确定性；误触发；违背“叙述默认安全” |
| 隐式自动镜像变量 | 命名冲突、Secret 泄漏、类型丢失、不可复现 |
| 一个文档一个 kernel | 无法做多语言/隔离实验，正是 Jupyter 的局限 |
| 把状态/输出写进 `.mq.md` | Git 噪声；正是要解决的 Jupyter 问题 |
| cell 自动执行 | 违背“未标记即叙述/未调用即数据”；破坏安全默认 |
| v2 做 foreign→Marqdo callback | 重入/死锁/安全复杂；应先稳定数据桥 |

---

## 4. 已锁定的实现边界

### v2 必做

- F0 安全执行内核 + `invoke` 记录 + `exec` 门禁 + 有界 IO + 进程组超时。
- F1 语言注册/发现：15 预设、`probe`、`register`、`unset`、配置、venv。
- F2 JSON 桥：`call` / `run_many` / `output` / `result`。
- F3 持久内核：Python / Node `session_open/eval/close/list`。
- F3.5 状态桥：`session_set/get/names`。
- D1 文档会话服务端：cell index、kernel registry、journal、open/close。
- D2 view notebook：内核选择、Run/Step/Run All/Restart、状态面板、dirty/stale。
- D3 Marqdo statement step 接入 view。
- D4 语法：`内核表` / `会话表` 解析与校验。
- D5 测试、文档、catalog、CHANGELOG。

### v2 不做（明确记录，避免“降级”误解）

- Foreign line-level step（接 DAP 是后续项目）。
- Foreign → Marqdo 函数 callback。
- 跨语言共享对象引用 / 共享内存。
- 全进程快照（只做变量级 JSON 快照）。
- 跨文档全局内核。
- 自动安装依赖。
- 浏览器 wasm 外联。

这些不是“砍掉”，而是**独立阶段**；v2 先把内核/会话/桥做稳。

---

## 5. 风险与对策

| 风险 | 对策 |
|---|---|
| 会话协议脆弱（原生 fd 输出、缓冲） | bootstrap 用原始 stdout；Python `-u`；Node 保存原始 write；超时 kill；文档标注限制 |
| 状态桥隐式化导致不可复现 | 默认显式；`mirror` 后续且默认关 |
| 多内核进程泄漏 | 每文档生命周期；Drop kill；进程组；session_close；测试 |
| 大对象 JSON 拷贝慢 | v2 明确拷贝语义；后续 Arrow/Parquet/文件句柄 |
| `内核表` / `会话表` 名称冲突 | 文档声明为保留绑定名；后续可用 frontmatter 指针重命名 |
| 表格行/单元不一致 | 明确诊断：未声明内核、重复单元、语言不匹配、未入会话 |
| 交互执行顺序混乱 | journal + dirty/stale + Run All canonical + 执行序号 |
| 安全（Secret/环境） | Secret 不进 JSON；env 显式交付；状态面板打码；exec cap |
| 测试环境无 Python | node/bash 主路径；python 条件测试；协议单测 |

---

## 6. 术语表（统一中英文）

| 中文 | English | 含义 |
|---|---|---|
| 外联内核 | foreign kernel | 外部语言进程 |
| 外联会话 | foreign session | 内核实例 + 命名空间 |
| 文档会话 | document session | Marqdo 状态 + 内核注册表 + journal |
| 代码单元 | cell | 绑定名的代码围栏 |
| 内核归属 | kernel assignment | cell → kernel |
| 状态桥 | state bridge | Marqdo 值 ↔ 内核变量 |
| 执行日志 | execution journal | 执行顺序/耗时/结果/hash |
| 新鲜 | clean | 源码未变且已成功执行 |
| 过期 | dirty | 源码变了，结果可能过期 |
| 失效 | stale | 上游依赖变了，需要重跑 |
| 规范执行 | canonical run | 重启后按会话表顺序 Run All |

---

## 7. 决策时间线

1. 用户提出外联库对标 Jupyter、共享状态、单步、切换、双向使用。
2. Agent 阅读 `src/host/foreign.rs`、`src/view`、`src/debug`、`src/host/subtask.rs`、`lib/foreign`、设计文档。
3. Agent 回答：需要 F + D 两层；单步分三种；共享分三层；显式桥。
4. 用户授权 Agent 拍板：安全、最强、不降级、充分测试、文档先行、提交推送后开发。
5. Agent 锁定本文件 §1 的 A–K 结论，进入文档编写与实现。

---

## 8. 后续变更流程

- 任何与本文冲突的实现，先改本文并写明「变更理由 / 影响 / 迁移」。
- 新增核心语法（任何新标记）必须走 `doc/adr/` + `core-surface.md`；本文所有语法均为既有标记的组合。
- 新增会话语言、跨语言数据平面、callback RPC，各自开独立设计文档，不塞进本文件。
