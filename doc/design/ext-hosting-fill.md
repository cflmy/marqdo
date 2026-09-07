# 设计：宿主集成缺口补齐（`ext/web` · `ext/ai`）

| | |
|---|---|
| 状态 | **Accepted · H1–H4 stdio 已落地**（H4b HTTP 可选） |
| 日期 | 2026-09-07 |
| 缺口盘点 | [ext-hosting-gaps.md](../research/ext-hosting-gaps.md) |
| 路线图 | [ext-hosting.md](../roadmap/ext-hosting.md) |
| 边界 | [ext-web-net.md](ext-web-net.md) · [ext-abi.md](ext-abi.md) · [ext-agent.md](ext-agent.md) |
| 目标 | 用**扩展库一等 API**关闭 GAP-01–05，取代下游 Python 旁路 |

---

## 0. 一句话

**服务器中继与「请求→`##`」进 `ext/web`；MCP Server 与 corpus 可发现进 `ext/ai`；标准库只保留已有客户端 SSE。** 作者用 GFM 表声明路由/工具，不写 JS、不启第二语言运行时。

---

## 1. 分层与禁止项

| 层 | 职责 | 落点 |
|----|------|------|
| 作者面 | 声明路径、上游、函数名、工具表 | `ext/web/web.mq.md` · `ext/ai/agent.mq.md`（+ 中文对偶） |
| ABI | axum 流式代理、每请求 interp 调用、MCP 传输 | `plugins/web` · `plugins/agent` |
| 标准库 | 不变：缓冲客户端 SSE / 解析 | `lib/net`（**不**加中继） |
| 核心 host | **禁止**新增 agent/web 领域 `host_*` | 见 Skill 硬规则 9 |

**禁止：**

1. 为中继在浏览器手写业务 JS。  
2. `json=` 继续塞「任意 `##`」语义（保持 DB-only；新能力用独立装配名）。  
3. MCP Server 把向量库升为权威真相（工具结果可带 `authority=workbook`）。  
4. 无白名单的「任意文件系统路径 + 任意 `##`」远程触发。

---

## 2. GAP-01 · `app.proxy` 流式反向代理

### 2.1 作者 API

```markdown
*应用 = > `应用`.proxy path="/v1/chat/completions" upstream=$OPENAI_BASE_URL*
*应用 = > `应用`.proxy path="/llm/stream" upstream="https://api.example.com" strip_prefix="/llm" stream=True*
```

或装配表（一次挂多条）：

```markdown
*应用 = > `应用`.configure proxy=`代理表`*

| path | upstream | stream | strip_prefix | headers_from_env |
|------|----------|--------|--------------|------------------|
| /proxy/chat | $OPENAI_BASE_URL | True | | OPENAI_API_KEY=Authorization |
```

| 参数 | 默认 | 含义 |
|------|------|------|
| `path` | 必填 | 浏览器同域路径 |
| `upstream` | 必填 | 上游根或完整 URL；支持 `$ENV` |
| `stream` | `True` | `True`：响应按上游 `Content-Type` 管道转发（含 `text/event-stream`） |
| `strip_prefix` | 空 | 转发前去掉本站前缀 |
| `methods` | `POST` | 允许方法 |
| `headers_from_env` | 空 | `ENV=Header-Name` 注入（密钥不进前端） |
| `timeout_ms` | 插件默认 | 上游超时 |

中文：`## 代理` / `流式=` / `上游=` / `去前缀=`。

### 2.2 ABI / 插件行为

| 步骤 | 行为 |
|------|------|
| 注册 | `web_app_proxy` 写入 `AppState` 路由表 |
| 请求 | 读 body + 过滤危险 hop-by-hop 头；合并 env 注入头 |
| 上游 | `reqwest`/`hyper` 流式客户端（插件内，不进核心） |
| 响应 | 复制状态码与 `content-type`；`body` 为 stream；SSE 不整段缓冲 |
| CORS | 仍走既有 `configure cors=`（同域调用时浏览器不依赖上游 CORS） |

### 2.3 与 `lib/net.http_post_sse` 关系

| API | 角色 |
|-----|------|
| `http_post_sse` | **出站消费者**（CLI/agent 侧收齐 events） |
| `app.proxy` | **入站中继**（浏览器 ← Marqdo ← 上游） |

二者互补，不互相替代。

### 2.4 验收

- 金样：mock 上游发两帧 SSE，浏览器侧（或 `curl -N`）收到两帧且中间无全缓冲延迟门槛。  
- 密钥仅出现在服务端 env 注入头。  
- 金样名建议：`tests/ext/web-proxy-sse-smoke.mq.md`。

---

## 3. GAP-02 · `app.invoke` / `configure invoke=`（HTTP → `##`）

### 3.1 作者 API

```markdown
*应用 = > `应用`.invoke path="/api/沉淀" method=POST fn="run.沉淀"*
```

表形式：

```markdown
*应用 = > `应用`.configure invoke=`调用表`*

| path | method | fn | body | return |
|------|--------|-----|------|--------|
| /api/note | POST | run.写出 | json | json |
| /api/search | GET | db.recent | query | json |
```

| 列 / 参数 | 含义 |
|-----------|------|
| `path` / `method` | HTTP 路由 |
| `fn` | `库.成员`（入口模块已 `import`）或 `模块路径##标题`（白名单前缀内） |
| `body` | `json` \| `form` \| `query` \| `raw` → 映射到 `##` 形参（同名优先；其余进 `payload` 袋） |
| `return` | `json`（默认）\| `text` \| `status` |

中文：`## 调用` / `函数=` / `正文=` / `返回=`。

### 3.2 安全默认（锁定）

1. **仅**解析入口 workbook 已导入的 `bind` 路径，或 `invoke_allow=` 显式前缀列表。  
2. 单请求超时 + 可选并发上限（防 listen 线程饿死）。  
3. 默认不启；作者显式 `invoke` / `configure invoke=`。  
4. 错误：`{ok:false, error}` + HTTP 4xx/5xx；不把内部路径栈默认回传生产模式。

### 3.3 ABI / 插件行为

| 步骤 | 行为 |
|------|------|
| 注册 | `web_app_invoke` 存 `(path, method, FnRef)` |
| 命中 | 解析 body → Value 袋；`call_lib_path` / 等价「同步调用已加载模块的 `##`」 |
| 返回 | Value → JSON（复用现有 host JSON 通路）；侧效（写盘）在同一解释器事务语义内尽最大努力 |

**不**把 `json=` DB 路由改语义；`invoke` 是平行装配。

### 3.4 与 compose 期 `call_lib_path` 关系

| 时机 | API |
|------|-----|
| 装配页 | 现有 compose / `call_lib_path` |
| **每个 HTTP 请求** | 本设计 `invoke` |

### 3.5 验收

- `POST` JSON → 用户 `##` 写入临时文件 + 返回 `{ok:true,…}`。  
- 未导入路径 → 4xx，不执行。  
- 金样：`tests/ext/web-invoke-smoke.mq.md`。

---

## 4. GAP-03 · MCP Server（`ext/ai`）

### 4.1 作者 API

```markdown
*服务 = > agent.mcp_server name="marqdo-qd"*
*服务 = > `服务`.tool name="qd_search" fn="求道.搜索" description="关键词检索"*
*服务 = > `服务`.tool name="qd_capture" fn="求道.捕捉"*
*服务 = > `服务`.serve transport=stdio*
# 或
*服务 = > `服务`.serve transport=http host=127.0.0.1 port=7433*
```

工具表：

| name | fn | description |
|------|-----|-------------|
| qd_search | 求道.搜索 | … |
| qd_list | 求道.列出 | … |

### 4.2 传输

| transport | 用途 |
|-----------|------|
| `stdio` | Cursor / Claude Desktop 拉起 |
| `http` | Streamable HTTP（与官方 MCP 演进对齐；首版可先 SSE+POST 子集） |

### 4.3 与 A4 fixture 客户端关系

| API | 方向 |
|-----|------|
| `mcp_list_tools` / `mcp_call` | **出站**（Agent 调外部 MCP；现 fixture） |
| `mcp_server` + `serve` | **入站**（外部调 Marqdo） |

客户端「真 MCP」仍见 agent 缺口文；本设计先交 Server，fixture 可保留测客户端。

### 4.4 ABI

- `plugins/agent`：`agent_mcp_server_*`（注册工具、stdio loop、可选 HTTP）。  
- 工具实现：复用 GAP-02 同款「调用 `##`」内核（可抽共享 `invoke_fn`），避免两套分发。  
- 返回包装：可含 `authority=workbook`。

### 4.5 验收

- 金样：stdio 夹具发 `tools/list` + `tools/call`，断言 JSON-RPC 形结果。  
- 可选：`tests/ext/agent-mcp-server-smoke.mq.md`。

---

## 5. GAP-04 · 同寿副端口

### 5.1 关闭策略（优先）

GAP-01 + GAP-02 挂在**同一** `app.listen` → **不实现副端口**，文档宣布 GAP-04 关闭。

### 5.2 备选 API（仅当必须隔离）

```markdown
*应用 = > `应用`.listen host=127.0.0.1 port=18081 also_port=7432 also_routes=`中继表`*
```

`also_routes` 仅允许 `proxy` / `invoke` 已注册子集。实现成本高，**默认不做**。

---

## 6. GAP-05 · 跨模块 `db` 与 corpus 可发现

### 6.1 `db` 句柄（扩展约定 + 运行时）

**作者约定（立即文档化）：**

1. 跨模块优先传 **`url` 字符串**，调用方 `*store = > web.db url=…*` 再 `select`。  
2. 若传对象：必须保留 `_type` 为插件注册名（与 `web.db` 构造一致）。

**运行时修补（H 波次）：**

| 选项 | 做法 | 选用 |
|------|------|------|
| A | 跨模块返回插件对象时规范 `_type` + 全局方法表查找 | **首选**（一次修好所有插件对象） |
| B | `db.open` 只返回 `{url}` 朴素 map，禁止「裸 select」跨文件 | 文档契约；体验较差 |

金样：模块 A `open` → 返回 → 模块 B `select` 成功。  
`tests/ext/web-db-cross-module-smoke.mq.md`。

### 6.2 `corpus_search` 可发现

```markdown
## corpus_search
+ `query`
+ `root`=".marqdo/agent-kb"

* > plugin.load name=agent *
** > agent_corpus_search query=`query` root=`root` **
```

或在 `# agent` / `## corpus_search` 入口统一 `ensure_plugin`（与现 `agent` 构造对齐），保证 **纯 `marqdo run` + import agent** 即可用。

**不**把 `agent_corpus_search` 搬进 `src/host/`。

### 6.3 验收

- 无显式手工 ABI 名的情况下，`marqdo run` 金样调用 `agent.corpus_search` 成功。  
- 跨模块 `db.select` 金样绿。

---

## 7. 扩展库表面清单（落地时改这些文件）

| 文件 | 增补 |
|------|------|
| `ext/web/web.mq.md` · `网页.mq.md` | `## proxy` / `## invoke`；`configure` 增 `proxy`/`invoke` 列 |
| `plugins/web/src/http.rs` · `middleware.rs` · 新 `proxy.rs` / `invoke.rs` | 路由注册与流式/调用 |
| `ext/ai/agent.mq.md` · `智能体.mq.md` | `# mcp_server` · `## tool` · `## serve`；`corpus_search` ensure_plugin |
| `plugins/agent/` | MCP Server 传输 + 共享 invoke |
| `tests/ext/web-proxy-sse-smoke.mq.md` 等 | 见上 |
| `doc/roadmap/ext-web.md` | 链到 H 波次（不重开 W0–W7） |
| `public/` / Skill | 发版时补「同域 LLM 代理 / HTTP 调用 ##」短节 |

**不**新增 `lib/run` 作为核心依赖：用户业务 `##` 仍在其自己的 `lib/*.mq.md`；Marqdo 只提供 **invoke 管道**。

---

## 8. 迁移：下游旁路 → 扩展 API

| 旁路（下游） | 替换为 |
|--------------|--------|
| `llm_proxy.py` `/proxy/stream` | `app.proxy` / `configure proxy=` |
| `llm_proxy.py` `/store/*` + `marqdo run` | `app.invoke` → 用户 `##` |
| `mq_bridge.py` 仅过滤 | 用户 `##` 内 `table`/`re` 过滤，或修复后的 `corpus_search` |
| `qdagent_mcp.py` | `agent.mcp_server` + `serve` |
| `mq.sh` 双进程 | 单 `marqdo run index.mq.md`（proxy+invoke 同端口） |

---

## 9. 开放点（实现前拍板）

| # | 问题 | 默认倾向 |
|---|------|----------|
| 1 | `invoke` 的 `fn` 是否允许 `path.mq.md##标题` 字面量 | 允许但必须 `invoke_allow` 前缀 |
| 2 | MCP HTTP 首版范围 | stdio 先交；HTTP 跟一波 |
| 3 | 代理是否缓冲「非 SSE」大 body | 非流式可限长缓冲；SSE 禁止整段缓冲 |
| 4 | GAP-05 选项 A vs B | **A** |

---

## 10. 完成定义（整包）

1. 无 Python 旁路即可：浏览器同域 SSE 对话 + HTTP 触发沉淀 `##` +（可选）MCP stdio 暴露同一批 `##`。  
2. 金样覆盖 proxy / invoke / cross-db / corpus / mcp-server（stdio）。  
3. `doc/research/ext-hosting-gaps.md` 各 GAP 标为关闭或降级为「备选副端口不做」。  
4. 发版说明指向 `ext/web` · `ext/ai` 新作者面，不指向 `scripts/*.py`。
