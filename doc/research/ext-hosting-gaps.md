# 调研：宿主集成缺口（浏览器中继 · HTTP→`##` · MCP Server · 同寿副端口 · 跨模块 DB/检索）

| | |
|---|---|
| 状态 | **调研笔记 · 缺口盘点（H1–H4 stdio 已落地；H4b HTTP 可选）** |
| 日期 | 2026-09-07 |
| 触发 | 下游应用（对话站 / 求道类 Agent）在真实集成中被迫旁路 Python 脚本 |
| 基线版本 | **v0.3.6+**（宿主波次 H1–H4 见 [ext-hosting.md](../roadmap/ext-hosting.md)） |
| 相关设计 | [ext-web.md](../design/ext-web.md) · [ext-web-net.md](../design/ext-web-net.md) · [web-net-capabilities.md](../design/web-net-capabilities.md) · [ext-agent.md](../design/ext-agent.md) · [ext-abi.md](../design/ext-abi.md) |
| 邻近缺口 | [agent-framework-gaps-after-a4.md](agent-framework-gaps-after-a4.md)（真 MCP **客户端**） |
| 补齐设计 | [ext-hosting-fill.md](../design/ext-hosting-fill.md) |
| 实现路线 | [ext-hosting.md](../roadmap/ext-hosting.md) |

---

## 0. 一句话结论

`ext/web` 已能做**内容站 / CRUD / JSON-DB API / WS**；`ext/ai` 已能做**工作簿 Agent + 客户端 SSE 消费**。下游要把「浏览器同域流式 LLM」「请求触发任意 `##`」「MCP Server 被 IDE 拉起」合进**单一 Marqdo 进程**时，现有扩展**没有一等装配**——只能外挂缓冲代理 / CLI 子进程。缺口集中在 **扩展库作者面 + `plugins/web|agent` ABI**，不是语法层，也不是再开一套标准库。

**原则：** 旁路脚本只证明需求；关闭缺口必须把能力沉进 `ext/*` + `plugins/*`（可选极薄 `lib/` 编排），禁止把领域逻辑塞进 `src/host/`。

---

## 1. 已有基线（不是缺口）

| 能力 | 现状 | 文档 / 金样 |
|------|------|-------------|
| CORS / 安全头 / gzip / body limit | `app.configure` | `tests/ext/web-middleware-smoke.mq.md` |
| JSON **DB select** 路由 | `configure json=` | `plugins/web/src/middleware.rs` |
| `/_form/*` SQL CRUD | `mount_form` | `plugins/web/src/form.rs` |
| 客户端缓冲 SSE | `lib/net.http_post_sse` → `ext/llm.complete stream=` | `tests/lib/net-openai-sse.mq.md` |
| 执行过程 SSE（非 LLM 中继） | `marqdo view` `/api/run` | [view.md](../design/view.md) §5 |
| Agent 证据工具（fixture MCP） | `corpus_search` / `mcp_list_tools` / `mcp_call` | A4；[agent-framework-gaps-after-a4.md](agent-framework-gaps-after-a4.md) |
| 同模块 `# db` CRUD | `ext/web` `# db` | `examples/web-site/db/` |

---

## 2. 缺口总表

| ID | 缺口 | 需要 | Marqdo 现状 | 典型旁路 | 解除条件 |
|----|------|------|-------------|----------|----------|
| **GAP-01** | 浏览器 → 上游 LLM **同域流式中继** | 绕过对方 CORS；**SSE 原样转发** | **`app.proxy` / `configure proxy=`**（H1） | ~~`llm_proxy.py`~~ | ✅ 已关闭 |
| **GAP-02** | HTTP 处理函数可调用用户 `##` | 请求内执行用户 `##`，返回 JSON | **`app.invoke` / `configure invoke=`**（H2） | ~~外挂 `marqdo run`~~ | ✅ 已关闭 |
| **GAP-03** | **MCP Server** 宿主 | IDE 拉起暴露工具 | **`agent.mcp_server` + `serve` stdio**（H4）；HTTP=H4b | ~~`qdagent_mcp.py`~~ | ✅ stdio；HTTP 可选 |
| **GAP-04** | 同寿「第二监听端口」 | 与 `listen` 同寿中继 | 并入主端口（H1+H2） | ~~`mq.sh` 双进程~~ | ✅ won't |
| **GAP-05** | 跨模块 `db`；纯 run 用 corpus | 跨文件 `select`；CLI 检索 | 传递归查找 + `corpus_search` ensure_plugin（H3） | ~~db/index 导出~~ | ✅ 已关闭 |

旁路脚本名来自下游集成仓库；**本仓库 `scripts/` 不含这些文件**——它们是需求证据，不是上游实现。

---

## 3. 分项说明

### 3.1 GAP-01 · 同域 SSE 中继

**问题**  
浏览器页面跑在 `ext/web` 源上时，直连 OpenAI 兼容上游会撞 CORS；`lib/net.http_post_sse` 只在 **CLI/宿主侧缓冲整段流**，不能把字节边收边推回浏览器。

**现状边界**

| 已有 | 缺什么 |
|------|--------|
| `app.configure cors=` | 解决的是**本站对外** CORS，不是替上游开 CORS |
| `http_post_sse` → `{status, events}` | **客户端消费**，非服务器中继 |
| `app.download` 对象流 | 存储下载，非上游 SSE |
| view `/api/run` SSE | Marqdo **执行事件**，非 LLM token 流 |

**解除形态（产品）**  
作者声明：`path` + `upstream`（或 env）+ 可选 header 白名单；监听端对浏览器返回 `Content-Type: text/event-stream`，chunk 原样或按 OpenAI SSE 帧转发。

### 3.2 GAP-02 · 路由 → `##`

**问题**  
对话沉淀、表单后写 `data/runs/*.mq.md`、触发 `lib/run.沉淀` 等，需要**同进程**执行用户函数。今天只有：

1. 页面装配期 `call_lib_path`（compose，非每请求）；  
2. `/_form` / `json=` 的 **DB 专用** handler。

**解除形态（产品）**  
声明式：`path` · `method` · `fn=lib.member` 或 `file=… ## name` · body→形参映射 · 返回值序列化为 JSON（错误码约定）。安全默认：仅允许入口模块已导入路径；禁止任意路径注入。

### 3.3 GAP-03 · MCP Server

**问题**  
A4 解决的是 Agent **调用** MCP 工具（且仍是 fixture）。下游要把 Marqdo 工作簿 **暴露为** MCP Server（stdio / Streamable HTTP），IDE 拉起后调 `qd_*`。

与 [agent-framework-gaps-after-a4.md](agent-framework-gaps-after-a4.md)「真 MCP 客户端」正交：本缺口是 **Server 宿主**。

**解除形态（产品）**  
`ext/ai`（或 `ext/mcp`）提供：`server tools=` 表（name → `.mq.md`/`##`）；`serve stdio=` / `serve http port=`；工具结果仍可带 `authority=workbook`。

### 3.4 GAP-04 · 同寿副端口

**问题**  
在 GAP-01/02 未并入主端口前，下游用第二端口跑代理。`app.listen` 同步阻塞，无「同进程再绑一个 TcpListener」的作者 API；只能 shell 并起另一运行时。

**解除形态（产品）**  
优先：GAP-01/02 挂在同一 `listen` → **本缺口关闭**。备选：`app.attach` / `listen also=` 同 tokio 运行时附加服务（仅当必须隔离路径树）。

### 3.5 GAP-05 · 跨模块 DB 句柄与 corpus 可发现性

**问题**

1. **类型分发：** 插件对象依赖 `_type` / 方法表；跨模块传回的 store 若丢类型或未注册方法，报 `unknown object type db`。同文件 `db/index.mq.md` 内 `## recent` 再导出 JSON 是现行绕过。  
2. **`agent_corpus_search`：** 注册在 `plugins/agent`；普通 `marqdo run` 未 `plugin.load` / 未构造 `# agent` 时不可用。导入 `ext/ai/agent.mq.md` 文本本身不保证原生插件已加载。

**解除形态（产品）**  
句柄：跨模块返回值保留 `_type=db` 且方法分发查全局/插件表（修运行时或约定「只返回 url，调用方再 `web.db`」二选一，见补齐设计）。corpus：`plugin.load` 自动路径，或 `ext/ai` 导出独立 `## corpus_search` 并在调用前 `ensure_plugin`。

---

## 4. 归属判定（标准库 vs 扩展）

沿用 [ext-web-net.md](../design/ext-web-net.md) / [web-net-capabilities.md](../design/web-net-capabilities.md) 边界：

| 能力 | 归属 | 理由 |
|------|------|------|
| SSE **中继**路由、HTTP→`##`、副端口 | **`plugins/web` + `ext/web`** | 服务器结构 / 网站领域；axum 已在依赖链 |
| MCP **Server**、corpus 可发现 | **`plugins/agent` + `ext/ai`** | Agent/工具宿主；禁止塞 `src/host/` |
| 缓冲 SSE 客户端、解析 | 已有 **`lib/net`** | 不新增中继到 stdlib |
| 任意路径 shell 出去跑 `marqdo` | **刻意不做进核心** | 旁路脚本可留在下游；上游给一等 API 后废弃 |

---

## 5. 优先级建议

| 优先级 | ID | 理由 |
|--------|-----|------|
| **高** | GAP-01 + GAP-02 | 去掉 Python 中继的主路径；浏览器站 + 沉淀闭环 |
| **高** | GAP-03 | IDE 生态入口；与「真 MCP 客户端」可并行设计、分波交付 |
| **中** | GAP-05 | 减少 `db/index` 与 bridge 胶水；影响面偏语言/插件加载 |
| **低（可随 01/02 关闭）** | GAP-04 | 并入主端口后自然消失 |

---

## 6. 明确「不是缺口」

| 项 | 说明 |
|----|------|
| 内容站 W0–W7+P3 | 已完结；本清单不重开上传/RSS/RBAC |
| Mid M1–M6 | 通用原语已够；本清单不是再开 `lib/re` 一类 |
| 隐藏 JSON TOOL 主循环 | 与 Agent 宪法冲突 |
| Dense embedding 默认化 | 仍见 [okf-near-match.md](../roadmap/okf-near-match.md) |
| 浏览器手写业务 JS 代理 | 与 WASM 路线「作者零 JS」冲突；中继应在服务端 `ext/web` |

---

## 7. 与其它文档

| 文档 | 关系 |
|------|------|
| [ext-hosting-fill.md](../design/ext-hosting-fill.md) | **补齐设计（作者 API + ABI）** |
| [ext-hosting.md](../roadmap/ext-hosting.md) | 实现波次 H0–H4 |
| [ext-web.md](../roadmap/ext-web.md) | 站点主线完结；本缺口为后续宿主波次 |
| [agent-framework-gaps-after-a4.md](agent-framework-gaps-after-a4.md) | 客户端真 MCP；与 GAP-03 Server 互补 |
| [web-asgi-servers-and-marqdo.md](../design/web-asgi-servers-and-marqdo.md) | 反代模型；中继仍在 Marqdo `listen` 内 |
