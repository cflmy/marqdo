# Official agent development framework (`ext/ai/agent`)

| | |
|---|---|
| Status | **Accepted · redesign**（取代「thin TOOL: loop」方案） |
| Date | 2026-08-07 |
| Related | [ext-llm.md](ext-llm.md) · [ext-cli.md](ext-cli.md) · [ext-abi.md](ext-abi.md) · [**ext-agent-plan.md**](ext-agent-plan.md)（**多步锁定设计**） · [**okf.md**](okf.md)（**OKF / 任务知识包**） · [module-namespace.md](module-namespace.md) · [objects.md](objects.md) · [stdlib-writeback.md](stdlib-writeback.md) · [stdlib-subtask.md](stdlib-subtask.md) · [markdown-mapping.md](markdown-mapping.md) |

## 0. 为什么要改掉旧方案

旧版把智能体做成「黑盒对话循环」：隐藏提示拼装、`TOOL:<name>` 协议、host 历史袋、一次（或几次）模型往返。这是对 LangChain 类框架的平移，**背离了 Marqdo 的初衷**。

Marqdo 的宪法是 **代码即文档、文档即知识库**：

| 旧方案的问题 | Marqdo 应有的形态 |
|--------------|-------------------|
| 提示词藏在框架字符串里，与编排源码分离 | 提示、工具、任务、结果都在 `.mq.md` 里，人与模型同读同写 |
| 执行过程是不透明的 chat transcript | 思考与行动写回文档；`view` / `debug` 可跟可审 |
| 复杂任务与简单任务同一套循环 | **单步**与**多步（工作簿）**两级模型 |
| 成功经验留在进程内存 | **自写回**做成果缓存与失败知识；同类任务可复用 |

本文件锁定新方向。仓库里现有的 `ext/ai/agent.mq.md` / `智能体.mq.md`（TOOL: 薄循环）视为 **待替换实现**，以本文为准。

---

## 1. 我们在建什么

一个 **文档驱动的智能体开发框架**：编排本身就是可读的 `.mq.md`；模型吃的是这份文档与调用位置；写回让思考过程可控、可调试、可沉淀。

**不是** 外挂任务队列、**不是** 官方捆绑领域工具、**不是** 第二套聊天栈（模型 I/O 仍用 `ext/ai/llm`）。

| 在范围内 | 在范围外（应用 / runbook） |
|----------|---------------------------|
| `# 智能体` / `# agent` 构造 | 领域工具：`## 查天气`、`## 分配任务` … |
| `## 单步` / `## step` | 业务表格、队列、工单流程 |
| `## 多步` / `## plan`（生成并驱动工作簿） | 具体分解策略模板库（可放用户仓） |
| 上下文：源码、调用位置、工作簿路径、Skill | 第二套 HTTP / 聊天协议 |
| 与 `lib/writeback`、**`lib/subtask`（工具执行通道）**、**`lib/plugin` + ABI v2 `plugins/agent`** 协作 | 把领域技能塞进官方 `ext/ai/`；在 `ext` 内直调 `host_*` |

---

## 2. 核心理念：编排可读，过程可写回

1. **提示词与代码不分离**  
   站立提示、工具表、任务叙述都写在 `.mq.md` 里。框架把**当前模块源码**与**调用位置**交给模型，而不是另起一套 system prompt 黑盒。

2. **把 Marqdo 源码交给智能体**  
   每次推理可注入：入口 / 工作簿 `.mq.md` 全文（或相关切片）、`host_call_site`（路径、函数、行）、`skills/marqdo`（写 / 读 `.mq.md` 的语法纪律）。模型输出应优先是 **可解释的决策** 与 **对文档的结构化改动意图**，而不是私有 JSON 协议。

3. **自写回 = 思考轨迹 + 成果缓存**  
   成功或失败都写回文档（见 [stdlib-writeback.md](stdlib-writeback.md)）。成功块可被后续同类任务命中，避免重复失败；失败块成为可读的「避坑记录」。`marqdo view` 在语句下展示 output-card，调试面可对齐同一份真相。

4. **相对其他框架的优势**  
   无需外挂向量库 / 状态机才能「记住」某次子任务已成功：记忆就是仓库里的 Markdown。审计、合规、复盘 = `git log` + 打开工作簿。

---

## 3. 两级执行模型

### 3.1 模式 A — 单步（原子任务）

| | |
|---|---|
| **输入** | 小任务描述 + 必要上下文（工具表、站立提示、源码切片） |
| **过程** | 一次（或有限次）推理；可选调用 **runbook 内** 的工具函数；结果写回 |
| **输出** | 原子结果（文本 / 表 / 句柄字段） |
| **适用** | 问答、单一工具、确定性一步 |

公开方法（中 / 英）：`## 单步` / `## step`。

示意（语法外形；实现以落地代码为准）：

```markdown
---
> ext/ai/智能体.mq.md
> ext/ai/大模型.mq.md
> lib/自写回.mq.md
> lib/时间.mq.md
---

## 获取时间

*`u` = > 时间.此刻秒 *
**> 时间.格式化 秒=`u` 格式=%Y-%m-%d**

# main

*`模型` = > 大模型.大模型 *
`工具表` =
| 工具 |
|------|
| 获取时间 |

*`助手` = > 智能体 大模型=`模型` 工具=`工具表` 站立提示=你是可调试的 Marqdo 助手。 *

*`结果` = > `助手`.单步 任务=今天日期是什么？请在需要时调用工具。 *
> 打印 内容=`结果`
```

单步约定：

- 模型看到的上下文 **包含本文件源码与调用位置**，因此「有哪些 `##` 工具」对模型是可见的；行动约定写在提示里（人可读），例如单独一行 `CALL:<工具名>` / `调用：<工具名>`，最终答案则直接正文回复。
- 工具仍是 **runbook 里的 `##` 函数**（v1 以零参为主；可经 `args=` 扩展）。
- **调用工具走 [`lib/subtask`](stdlib-subtask.md)**：白名单校验通过后 `spawn fn=<名>`（中文面 `启动 函数=`）→ `wait` / `等待`，而不是进程内直接 `call_fn`。这样工具与主推理隔离在子任务语义下，并可与并行、超时、探测同一套主机能力对齐。
- 结束后用自写回记录：任务摘要、选用工具、结果或错误。

### 3.2 模式 B — 多步 / 复杂执行（工作簿）

> **完整锁定设计**见 [**ext-agent-plan.md**](ext-agent-plan.md)。以下为摘要。

| | |
|---|---|
| **输入** | 复杂目标 |
| **过程** | 父智能体**创建（或续跑）一份工作簿 `.mq.md`** → 子文件大模型配置与父一致 → **`lib/subtask` `spawn path=`** 运行该文件 → 读结果与写回 → **改代码 / 改写回后再执行**（有轮次上限） |
| **输出** | 汇总 map（含 `result`）+ 磁盘上的工作簿（代码与 `marqdo-out` 均可演化） |
| **适用** | 需分解、多情形、可固化步骤、可复用子成果的任务 |

公开方法：`## 多步` / `## plan`。

流水线要点：

1. **创建工作簿** — 默认 `.marqdo/agent-runs/`；内容为任务的详细可执行步骤（导入、工具、`# main`、子智能体）。  
2. **模型配置继承父智能体** — 共享 env / 等价 llm 句柄；v1 不静默换模。  
3. **子任务执行整文件** — 不是父进程内嵌解释。  
4. **工作簿内推荐多个 `# 智能体`**；也允许一个智能体多次 `单步`；**已确定步骤应固化为返回答案的普通代码**（多轮 = 多文件接力 + OKF，见 [ext-agent-plan.md §4.2](ext-agent-plan.md)）。  
5. **持续更新** — 代码修订 + 写回注释刷新（替换不累积空行）。  
6. **`单步` 默认自动写回**（`写回=` / `writeback=` 可关），父侧再汇总写回。  
7. **父上下文** — 与单步同级的源码 / Skill / 调用位置 / 历史，外加工作簿全文与子写回。

示意：

```markdown
*`报告` = > `编排`.多步 目标=调研并汇总三篇资料的要点 *
> 打印 内容=`报告`
```

工作簿概念形状与修订循环细节 → [ext-agent-plan.md](ext-agent-plan.md)。
## 4. 分层与仓库布局

```text
ext/
  ai/
    llm.mq.md / 大模型.mq.md     # 模型 I/O
    agent.mq.md / 智能体.mq.md   # 本框架（按本文重写）
```

| 层 | 职责 |
|----|------|
| **应用 runbook** | 领域 `##` 工具、任务表、是否人工确认分解 |
| **`ext/ai/agent`** | 单步 / 多步、工作簿生成、子智能体编排、写回约定 |
| **`ext/ai/llm`** | 补全 / 聊天 |
| **`lib/writeback`** | 持久化输出块 |
| **`lib/subtask`** | **工具调用与（多步时）子智能体并行的执行通道**；见 [stdlib-subtask.md](stdlib-subtask.md) |

安装：[`ext-cli.md`](ext-cli.md) — `marqdo ext add agent` / `add llm`（`.mq.md` + `native/libagent.so`）。导入：`> ext/ai/智能体.mq.md`。

**原生插件（ABI v2）**：`ext/ai/agent` 经 `lib/plugin` 加载；优先 `> plugin.native_path name=agent`，否则按序查找：

1. `MARQDO_AGENT_PLUGIN`（显式 `.so` / `.dylib` 路径）  
2. `CARGO_TARGET_DIR/{debug,release}/libagent.so`（或 `.dylib`）  
3. cwd `target/{debug,release}/`  
4. 可执行文件旁 `libagent.*` 或 `target/debug/`  
5. `MARQDO_EXT/native/libagent.*`（`marqdo ext add agent` 安装）

本地开发须先 `cargo build -p marqdo_plugin_agent`（或 `ext add agent` / 设 `MARQDO_AGENT_PLUGIN`）。详见 [ext-cli.md](ext-cli.md) · [ext-abi.md](ext-abi.md)。

**分层纪律（硬规则，勿破）**：

1. `ext/**/*.mq.md` **禁止**出现任何 `host_*` / `host_` 调用。  
2. Agent 专属能力（含 OKF agent-kb：`agent_goal_sig` / `agent_kb_lookup` / `agent_kb_promote` / …）只进 **`plugins/agent` 注册名**，由 `ext/ai/agent` 在 `plugin.load` 后调用。  
3. **禁止**向 `src/host/`（`HostFn` 表）添加 agent / OKF 领域原语，以免核心包体膨胀。通用 L0.5 仅保留 fs/json/subtask 等标准库底层。  
4. 进程退出用 `sys.exit` / `系统.退出`，不用裸 `exit`。

官方 `ext/ai/` 用 `lib/*` 包装或插件注册名；见 [module-namespace.md](module-namespace.md) §8 · [ext-abi.md](ext-abi.md)。

---

## 5. 公开 API（目标契约）

| 中文 | 英文 | 角色 |
|------|------|------|
| `# 智能体` | `# agent` | 构造：`大模型`/`model`，`工具`/`tools`，可选 `站立提示`/`standing` |
| `## 单步` | `## step` | 原子执行；注入源码 + 位置；可选工具；返回结构化 map；**默认自动写回**（`写回=` / `writeback=` 可关） |
| `## 多步` | `## plan` | 建/续工作簿 → subtask 跑文件 → 修订循环 → 汇总（见 [ext-agent-plan.md](ext-agent-plan.md)） |
| `## 分解` | `## decompose` |（可公开或内部）目标 → 子任务表 |
| `## 清空历史` | `## clear_history` | 若仍保留会话式辅助状态则清空；**不以隐藏 chat 袋为真相源**——真相在文档写回 |

构造参数（v1）：

| 参数 | 说明 |
|------|------|
| `大模型` / `model` | `ext/ai/llm` 句柄 |
| `工具` / `tools` | 工具名表（单列文本，或多列且含 `工具`/`tools`/`name`） |
| `站立提示` / `standing` | 写在文档里的常驻说明（可空） |

`## 单步` 参数：`任务` / `task`；可选 `写回` / `writeback`（默认真）、深度、是否强制不用缓存等。  
`## 多步` 参数：`目标` / `goal`；可选 `工作簿目录`、`工作簿`（续跑路径）、`最大深度`、`最大轮次`、`确认分解` —— 详见 [ext-agent-plan.md §9](ext-agent-plan.md)。

模块内辅助函数（组装上下文、解析模型回复、匹配写回缓存）**不是**稳定对外契约，可随实现调整。

---

## 6. 工具表与子任务调用

工具 **不是** 框架内置能力。在 runbook 定义 `##`，再用表登记名称：

```markdown
`工具表` =
| 工具 |
|------|
| 获取时间 |
| 读取摘要 |
```

- 单列表 → 名称列表。  
- 多列表 → 从列 `工具` / `tools` / `name` 取名。  
- 名称须对应当前模块（或导入可见）的 `##`。  
- **不要** 用 `parse text=["fn"]` 列工具（未加引号的标识符会被当成调用）。

### 6.1 调用路径（锁定）

```text
模型回复含 CALL:<名> / 调用：<名>
        ▼
host_tool_allowed（工具表白名单）
        ▼
lib/subtask：spawn fn=<名>  →  wait
        ▼
工具返回值写回上下文 / 自写回
```

| | |
|---|---|
| 英文面 | `> spawn fn=`名`` → `> wait id=`…`` |
| 中文面 | `> 启动 函数=`名`` → `> 等待 id=`…`` |
| 可选 | `args=` / `参数=` 传给带形参的工具；多工具并行时多次 `spawn` 再 `wait_all` / `等待全部` |

**非目标：** 以裸 `call_fn` 作为框架默认工具通道（应用仍可直接 `call_fn`，但官方智能体走子任务库）。

领域逻辑永远留在应用文件，不进官方 `ext/ai/`。

---

## 7. 上下文注入（每次单步 / 子步）

必须可注入（实现可切片，但不得默默丢掉「代码即文档」）：

| 片段 | 来源 |
|------|------|
| 站立提示 | 构造参数 / 文档正文 |
| 任务 / 目标 | 方法实参 |
| 工具表（人可读） | 构造参数 |
| **当前 `.mq.md` 源码** | 插件 `agent_module_source`（经 ABI v2 `host_query`） |
| **调用位置** | 插件 `agent_call_site` |
| **Marqdo skill** | 插件 `agent_marqdo_skill` |
| Marqdo Skill | `skills/marqdo/`（`MARQDO_SKILL` 可覆盖） |
| 相关写回块 | `lib/writeback` 的 `get` / `list` |

禁止：把整份 Skill + 全文源码藏进不可审查的二进制提示且不在文档中留迹。框架拼装的提示应能在调试时落盘或写回（至少摘要）。

---

## 8. 自写回与缓存语义

依赖 [stdlib-writeback.md](stdlib-writeback.md)：`record` / `写回` → `<!-- marqdo-out … -->`。

### 8.1 单步返回值与默认写回

`单步` / `step` 返回结构化 map，并在 **`写回` / `writeback` 默认真** 时自动 `record` 到命名槽（成功 `ok` / 失败 `error`）。关闭写回则行为与早期「纯返回值」一致，由编排方自行 `writeback.record`。

| 字段 | 何时 | 含义 |
|------|------|------|
| `status` | 总是 | `ok` / `error` |
| `task` | 总是 | 本步任务 |
| `decision` | 总是 | 模型首轮输出（含 CALL 行等） |
| `tool` / `tool_result` | 成功且调了工具 | 工具名与子任务返回 |
| `result` | 成功 | 最终答复 |
| `error` | 失败 | 错误信息 |

```markdown
*`out` = > `助手`.step task=… *
*`回复` = > json.get value=`out` key=result *
# 默认已写回；若 writeback=False 则需自行 record：
# > writeback.record value=`body` key=ok
```

命名槽 `ok` / `error` 互不覆盖；锚点为调用行。多步工作簿内依赖此默认写回，避免靠提示词强制落盘 —— 见 [ext-agent-plan.md §7](ext-agent-plan.md)。

**命中规则（后续缓存）**：读 `key=ok` 且非占位时可复用。失败知识读 `key=error`。  
用户可 `clear key=ok` / `clear key=error` 强制重跑。

复杂执行中，子结果写回优先落在 **工作簿** 文件；父 `多步` 维护汇总写回。

### 8.2 Live 金样例

[`tests/ext/agent-run-live.mq.md`](../../tests/ext/agent-run-live.mq.md)：`agent.step`（**默认写回** `ok`/`error`）+ 子任务工具。凭证：`tests/ext/.env`；`> llm.load_env path=.env` 相对**源文件目录**。需已构建 agent 原生库（见 §4）。
---

## 9. 安全与边界

| 风险 | 对策 |
|------|------|
| 无限多步递归 | `最大深度`（默认小，如 2）；叶任务强制单步 |
| 工具越权 | 仅工具表白名单；子智能体可传更窄的表 |
| 工作簿爆炸 | 统一目录、命名含时间 / id；可选总超时 |
| 缓存过期 | 签名 + 时间戳；手动清块 |
| 模型乱改仓库 | 写回 API 约束写入形态；多步生成文件限于工作簿目录 |

---

## 10. 相对旧 API 的迁移

| 旧（废弃方向） | 新 |
|----------------|----|
| 唯一的 `## 执行` / `## run` + 隐藏 TOOL: 循环 | `## 单步` / `## 多步` |
| host 历史袋为真相 | 文档写回 + 工作簿为真相；历史袋至多辅助 |
| 提示全在框架字符串 | 站立提示与任务在 `.mq.md`；源码注入 |
| `tests/ext/agent-smoke` 测 TOOL 上下文拼接 | 重写为单步离线 / 写回 / 工作簿骨架测试 |

兼容：可短期保留 `## 执行` 作为 `## 单步` 的别名，但文档与金样例迁到新名后删除别名。

---

## 11. 实现路线（文档锁定后）

| 阶段 | 内容 | 状态 |
|------|------|------|
| D0 | 本文 Accepted；废弃 thin TOOL 设计叙述 | **本文件** |
| D1 | 重写 `ext/ai/agent.mq.md` / `智能体.mq.md`：`单步` + 源码/位置注入 + **子任务调工具** + **结构化返回值**；运行时经 **ABI v2 agent 插件**（禁 ext 直调 host） | **done** |
| D2 | `多步`：按 [ext-agent-plan.md](ext-agent-plan.md)（观察→精确补丁；禁止整文件重写）；**D2a–D2f 主路径已落地** | **done** |
| D3 | 金样例：离线单步 / 写回；**live 单步**（`tests/ext/agent-run-live.mq.md`）；多步骨架（可不打网） | **live done** |
| D4 | `view` / `debug` 对工作簿与写回块的导航体验 | TODO |
| O2 | OKF 任务知识包：`plan` reuse/promote（[okf.md §7](okf.md)） | **done** |

### 非目标

- 在官方 `ext/ai/` 捆绑领域工具或工单技能  
- 在 `ext/` 根目录摊平 `.mq.md`  
- 平行再造一套 LLM HTTP 栈  
- 以不可读的私有二进制协议作为编排主界面  

---

## 12. 一句话

**智能体框架必须让编排看起来像文档、跑起来像程序、留痕像知识库。**  
单步解决原子问题（默认写回）；多步让父智能体编写并修订子 Marqdo 工作簿、经子任务执行、把确定步骤固化为代码——详见 [ext-agent-plan.md](ext-agent-plan.md)。

流式呈现思考与中间输出（不替代返回值 / 写回）见规划 [agent-streaming.md](../roadmap/agent-streaming.md)。
