# 路线图：宿主集成补齐（H0–H4）

| | |
|---|---|
| 状态 | **Active · 规划** |
| 日期 | 2026-09-07 |
| 缺口 | [ext-hosting-gaps.md](../research/ext-hosting-gaps.md) |
| 设计 | [ext-hosting-fill.md](../design/ext-hosting-fill.md) |
| 相关 | [ext-web.md](ext-web.md)（W/C 完结）· [ext-agent-optimize.md](ext-agent-optimize.md) · [agent-streaming.md](agent-streaming.md) |
| 基线 | v0.3.6+ |

本文只跟踪**实现波次**。作者 API、ABI 边界、禁止项以设计文为准。

---

## 0. 目标回顾

让 `ext/web` + `ext/ai` 具备：同域 SSE 中继、HTTP→用户 `##`、MCP Server（至少 stdio）、跨模块 db/corpus 可发现——从而下游无需 Python 旁路进程。

---

## 1. 波次总表

| 波次 | 内容 | 关闭缺口 | 状态 |
|------|------|----------|------|
| **H0** | 文档契约：跨模块 `db` 传 `url`；`corpus_search` 调用前 `ensure_plugin` 说明；缺口索引进 `doc/README` | GAP-05 文档侧 | **done** |
| **H1** | `app.proxy` / `configure proxy=`：上游 SSE/HTTP **流式**转发 | GAP-01 | **done** |
| **H2** | `app.invoke` / `configure invoke=`：请求 → 白名单 `##` → JSON | GAP-02 | **done** |
| **H3** | 跨模块插件对象 `_type` 分发修补 + `corpus_search` 自动 `ensure_plugin` | GAP-05 运行时 | **done** |
| **H4** | `agent.mcp_server` + `serve transport=stdio`（HTTP 可选 H4b） | GAP-03 | **done**（stdio） |
| **H4b** | MCP Streamable HTTP | GAP-03 余量 | **todo**（可选） |
| **—** | 副端口 API | GAP-04 | **won't**：H1+H2 同端口关闭 |

建议顺序：**H1 → H2 → H3 → H4**（H2 的 invoke 内核可被 H4 工具复用，故 H2 不宜晚于 H4）。

---

## 2. 分波工作项

### H1 — 流式代理

1. `plugins/web`：`proxy.rs` + `web_app_proxy`；listen 挂流式 handler。  
2. `ext/web`：`## proxy` + `configure proxy=` 表。  
3. 金样 `web-proxy-sse-smoke`（mock 上游）。  
4. 教程/public 短节：「浏览器同域调 LLM」。

**完成定义：** `curl -N` 同域 path 收到多帧 SSE；密钥仅 env 注入。

### H2 — HTTP invoke

1. 共享或插件内 `invoke_fn`（解析 `库.成员`、映射 body）。  
2. `web_app_invoke` + 白名单。  
3. 金样：临时 `##` 写文件 + JSON 响应；拒绝未允许路径。  
4. 迁移说明：替换 `/store/*` + 外挂 `marqdo run`。

**完成定义：** 单进程 `listen` 下表单/fetch 可触发用户 `##`。

### H3 — db / corpus

1. 复现并修跨模块 `unknown object type db`（设计选项 A）。  
2. `ext/ai` `corpus_search` 路径 `ensure_plugin`。  
3. 金样 `web-db-cross-module-smoke` + corpus CLI run 金样。

### H4 — MCP Server

1. `plugins/agent` stdio JSON-RPC 最小子集：`initialize` / `tools/list` / `tools/call`。  
2. `ext/ai` `# mcp_server` · `## tool` · `## serve`。  
3. 工具调用复用 H2 `invoke_fn`。  
4. 金样夹具；文档对接 Cursor `mcp.json` 示例。

**H4b：** Streamable HTTP 与鉴权；不阻塞 H4 stdio 发版。

---

## 3. 刻意不做

| 项 | 原因 |
|----|------|
| 默认开启任意 `##` 远程执行 | 安全 |
| 中继进 `lib/net` / `src/host` | 边界 |
| 重开 W0–W7 内容站清单 | 已完结 |
| 以副端口为推荐架构 | H1+H2 同端口即可 |
| Dense embedding 默认 | OKF 路线不变 |

---

## 4. 与发版

- 每波可随小版本发布；H1+H2 建议同一版本对外宣传「可去掉 llm_proxy」。  
- H4 可标 `ext/ai` 能力点，与「真 MCP 客户端」分列 Release notes。

---

## 5. 进度勾选（落地时改）

- [x] H0 缺口 + 补齐文档  
- [x] H1 proxy SSE  
- [x] H2 invoke  
- [x] H3 db/corpus  
- [x] H4 mcp_server stdio  
- [ ] H4b mcp http（可选）  
- [x] 研究文 GAP 状态改为 closed（H4b 除外）  
