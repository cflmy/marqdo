# 网络能力调研：从「能跑」到「完整项目」

| | |
|---|---|
| 状态 | **调研结论（Draft）** |
| 日期 | 2026-08-26 |
| 相关 | [ext-web.md](ext-web.md) · [ext-web-net.md](ext-web-net.md) · [ext-abi.md](ext-abi.md) · [stdlib-modules.md](stdlib-modules.md) |
| 目标 | 盘点 `ext/web` + `plugins/web` + `lib/net` 现状，对照主流语言网络栈，给出「开发一个完整 Web 项目」所需能力的差距清单与补强路线 |

---

## 0. 一句话结论

**当前 `plugins/web` 已覆盖「单页渲染 + SQLite 表单 CRUD + 简易 admin 门禁 + echo WebSocket + 中间件管道（CORS/安全头/gzip/请求体限制）+ JSON API + 数据层深化（连接池/事务/分页/查询表达力）」，足以支撑中小型内容站点与前后端分离 API；但距「完整项目」（博客/CMS/社区）仍缺四类能力：③ 安全硬化（密码哈希 / CSRF / 限流 / 会话持久化）；④ 内容站点标配（SEO / 搜索 / RSS / 分页导航）；① 中余下上传下载 / HTTPS / 日志 / 错误页。** 全部按「基础设施进 `lib/net`，领域能力进 `plugins/web` + `ext/web`」的既有边界补强。


---

## 1. 现状盘点（2026-08-26 实测）

### 1.1 `lib/net`（标准库，L0.5 宿主原语薄包装）

| 能力 | 归属 | 现状 |
|---|---|---|
| `http_get` / `获取` | host `net.rs` (ureq) | ✅ GET，返回 `{status, body}` |
| `http_post` / `提交` | host `net.rs` | ✅ POST，默认 JSON CT |
| `http_request` / `请求` | host `net.rs` | ✅ 通用 method+body+headers |
| `http_post_sse` / `提交流式` | host `net.rs` | ✅ OpenAI 兼容 SSE 流式消费 |
| `openai_sse_parse` / `解析流式` | host `net.rs` | ✅ 离线 SSE 解析 |
| `url_encode` / `编码地址` | host `net.rs` | ✅ URL 编码 |
| `cookie_parse` / `解析Cookie` | host `net.rs` | ✅ RFC6265 子集（请求头/响应头） |
| `multipart_parse` / `解析多部分` | host `net.rs` | ✅ multipart/form-data 解析 |

**标准库网络原语已比较齐备**：客户端 GET/POST/SSE、URL 编码、cookie、multipart 都有。缺的是**响应元信息（响应头/Set-Cookie 取出）、上传（multipart 文件）与下载（二进制）**。

### 1.2 `plugins/web`（ABI v2，领域能力）

**服务器侧已实现：**

| 域 | ABI | 能力 |
|---|---|---|
| 页面 | `web_page_new` · `web_compose_components` · `web_compose_main` · `web_page_query` · `web_page_order` · `web_page_link_prefix` · `web_page_css` · `web_page_detail` | 页面装配、列表/排序、CSS、详情页 |
| 样式 | `web_style` | GFM 表 → CSS（`make_style`/`样式装配`） |
| 表单 | `web_form_new/fields/rules/validate/render/submit/from_schema` · `web_compose_form` | 字段表、校验表、渲染、提交、schema→form |
| 数据库 | `web_db_new/init/insert/select/get/update/delete/exec/table_info/list_tables` | SQLite CRUD + where + order + limit |
| 应用 | `web_app_new/route/mount_form/static/auth` · `web_app_route_ws` | 路由、静态目录、admin、鉴权、WS 端点 |
| 会话 | `web_session_new/set/get/del/destroy` | 内存 TTL 会话 |
| 鉴权 | `web_auth_login/check/logout/new` · `web_app_auth` | `/admin` 登录门禁 |
| WebSocket | `web_ws_connect`（客户端）· `web_app_route_ws`（服务器） | 单请求-响应式客户端 + echo 服务器 |

**关键实现事实（决定差距）：**

- **中间件层已落地**（W1）：`app.configure`/`应用.装配` 通过 GFM 表装配 CORS、安全响应头、gzip 压缩、请求体上限、JSON API 路由，`监听` 时统一挂到 axum `Router`（`middleware.rs`）。「配置即数据、装配即函数」。
- **JSON API 已落地**（W1）：`|路径|方法|表|条件|排序|上限|` 表声明端点，响应 `application/json`，支持 DB 查询 + 排序 + 上限。
- **数据层深化已落地**（W2）：进程级**连接池**（WAL + `busy_timeout` + `foreign_keys`）+ **事务** API（`db.事务` → `txn.insert/提交/回滚`，`txn` 句柄带 `_type` 走 Marqdo 类分发）+ **结果集裸查询**（`db.query` 返回 `{rows, count}`）+ **分页**（`select`/`paginate` 支持 `offset`，返回 `{rows, total}`）+ **查询表达力**（where 支持 OR 组 / `in` / `between` / `is null` / `like`）+ **聚合计数**（`db.count`）。
- **无文件上传**：无 multipart extractor，`multipart_parse` 只在标准库侧做了纯解析，服务器没接收。
- **会话为进程内内存**：重启即失效、多 worker 不共享；session id 非加密强度；密码明文存储比对。
- **无 HTTPS**：仅 `TcpListener::bind` HTTP。
- **无迁移机制**：schema 版本表 + 迁移脚本缺失；索引/唯一/外键声明待增强（init 表增强）。
- **WebSocket 仅 echo**：无广播/房间/多客户端/业务分发。
- **尚无自定义 404/500 错误页、无访问日志、无 ETag/Cache-Control 缓存头、无重定向增强。**

---


## 2. 主流网络栈对照（联网调研 2026-08）

### 2.1 分层共识（决定「标准库 vs 扩展」边界）

| 语言 | 标准库网络能力 | 扩展/框架能力 | 边界准则 |
|---|---|---|---|
| Python | `urllib`（HTTP 客户端）、`http.cookies`（cookie 解析器）、`http.server`（简易服务器）、`email` 多部分解析 | `requests`/`httpx`（连接池/session）、`FastAPI`/`Flask`（中间件/校验/模板） | 解析器进标准库；便利层/框架层在第三方 |
| Node.js | `node:http`（低层流式，**不解析 body**）、`node:https` | express、fastify、ws、formidable | 低层原语进核心；fancy 功能 userland 验证后再进 core |
| Go | `net/http`（服务器+客户端**都在标准库**）、`net/http/cookiejar`（RFC6265）、`net/smtp` | `golang.org/x/net`（演进性）、gin/echo | HTTP 与 stdin 同级的核心能力；cookie 解析进标准库 |
| Rust | 极小 std（无 HTTP） | `hyper`→`reqwest`→`axum` | 分层；HTTP 生态多样不进 std |

**对本项目的三处判定（与 [ext-web-net.md](ext-web-net.md) 一致并延伸）：**

1. **纯解析原语**（cookie / multipart / url）→ `lib/net` 标准库。✅ 已达成。
2. **HTTP 服务器结构能力**（JSON / 中间件 / 上传下载 / HTTPS / 缓存头）→ `plugins/web`（ABI）。这是 axum 本身就在依赖链里的，不需要新增核心依赖。
3. **领域能力**（session / auth / 分页 / 搜索 / RSS / SEO / 限流）→ `plugins/web` + `ext/web` 作者面。

### 2.2 「完整 Web 框架」能力清单（FastAPI / Express / Flask / axum 共通项）

| 能力 | FastAPI | Express | Flask | axum | 说明 |
|---|---|---|---|---|---|
| 路由（含动态参数） | ✅ | ✅ | ✅ | ✅ | 三框架全支持；marqdo 已有动态路由 `/post/{slug}` |
| 中间件管道 | ✅ | ✅ | ✅ | ✅（tower） | **marqdo 缺失** — 最根本缺口 |
| CORS | ✅ 内置 | ✅ 中间件 | ✅ flask-cors | ✅ tower-http | 缺失 |
| JSON 请求/响应 | ✅ 原生 | ✅ | ✅ | ✅ | 缺失 |
| 校验（schema/字段） | ✅ Pydantic | 手动(zod) | 手动 | ✅ extractor | marqdo 有 form 校验表（够用） |
| 静态文件 | ✅ Starlette | ✅ | ✅ | ✅ | ✅ 已有 |
| 模板/渲染 | ✅ Jinja | ✅ | ✅ | ✅ | marqdo 用 GFM 表装配（自研） |
| 会话 | ✅ | ✅ express-session | ✅ | ✅ | 有（内存，待持久化） |
| 鉴权 | ✅ OAuth2/APIKey | ✅ | ✅ | ✅ | 有（admin 门禁，待硬化） |
| 文件上传 | ✅ | ✅ multer | ✅ | ✅ | 缺失 |
| WebSocket | ✅ | ✅ ws | ✅ Flask-SocketIO | ✅ | 有（echo，待广播） |
| 安全头 | ✅ | ✅ helmet | ✅ | ✅ tower-http | 缺失 |
| 限流 | ✅ | ✅ | ✅ | ✅ | 缺失 |
| 日志 | ✅ | ✅ morgan | ✅ | ✅ | 缺失 |
| gzip 压缩 | ✅ | ✅ | ✅ | ✅ | 缺失 |
| 分页 | 手动 | 手动 | ✅ paginate | ✅ | marqdo 缺失（db 层无 offset） |
| 迁移 | ✅ alembic | ✅ | ✅ | ✅ sqlx | 缺失 |
| 测试工具 | ✅ TestClient | ✅ supertest | ✅ | ✅ | 缺失（依赖 gold 测试） |
| HTTPS/TLS | 反代 | 反代 | 反代 | ✅ rustls | 缺失（通常反代承担） |

---


## 3. 差距分析：按「完整项目」四类需求

### 3.1 类 A：结构化 HTTP 基础设施（做 API / 前后端分离 / SPA 支撑）

| # | 缺失能力 | 影响 | 归属 | 优先级 |
|---|---|---|---|---|
| A1 | **JSON 请求/响应**（`Json` extractor、`application/json` 响应、body 大小限制） | ✅ **已实现（W1）**：`接口` 表声明 JSON API 端点；`请求体上限` 配置 body 限制 | `plugins/web` | **P0 ✅** |
| A2 | **中间件管道**（`app.configure` / `应用.装配`：CORS、安全头、压缩、请求体限制统一挂载） | ✅ **已实现（W1）**：`middleware.rs` 装配 GFM 表并 `app.layer` 挂到 axum | `plugins/web` | **P0 ✅** |
| A3 | **CORS 支持**（`Access-Control-Allow-Origin` 可配） | ✅ **已实现（W1）**：`跨域` 表 `|允许来源|方法|头|暴露头|凭证|` | `plugins/web` | P1 ✅ |
| A4 | **安全响应头**（CSP / X-Frame-Options / X-Content-Type-Options / HSTS / Referrer-Policy / 去 `X-Powered-By`） | ✅ **已实现（W1）**：`安全` 表 `|头|值|` | `plugins/web` | P1 ✅ |
| A5 | **文件上传接收**（multipart extractor → 校验类型/大小 → 落盘）与**下载**（`Content-Disposition`） | ✅ **已实现（W5）**：`app.upload` / `app.download`；`file:` storage；离线 + curl live 金样 | `plugins/web` | P1 ✅ |
| A6 | **HTTPS / TLS**（rustls 终止或反代提示） | 生产部署安全 | `plugins/web`（或文档指引反代） | P2 |
| A7 | **gzip / br 压缩** + **ETag / Cache-Control 缓存头** | gzip ✅ **已实现（W1，`压缩=真`）**；ETag/Cache-Control 待补 | `plugins/web`（tower-http 现成） | P2 |
| A8 | **访问日志**（请求方法/路径/状态/耗时） | 排障、审计 | `plugins/web`（tower-http TraceLayer） | P2 |
| A9 | **自定义 404/500 错误页** | 用户体感 | `plugins/web` | P2 |
| A10 | **重定向增强**（permanent/自定义状态码） | 301 SEO | `plugins/web` | P3 |

### 3.2 类 B：数据层深化（内容站点/社区的根基）

| # | 缺失能力 | 影响 | 归属 | 优先级 |
|---|---|---|---|---|
| B1 | **连接池 / busy_timeout / WAL** | 高并发下 `database is locked`、每请求重连开销 | `plugins/web`（db.rs） | **P0 ✅** |
| B2 | **事务 API**（`begin` / `commit` / `rollback`，或 `with_transaction` 包装） | 批量/多表写无原子性 | `plugins/web` | **P0 ✅** |
| B3 | **结果集型裸查询**（exec 返回 rows） | count / join / group / 子查询完全不可用 | `plugins/web` | P1 ✅ |
| B4 | **分页**（db 层 `limit`+`offset`；页面层上一页/下一页） | 列表页只能限量 | `plugins/web` + `ext/web` | P1 ✅（db 层；页面 UI 属 D4） |
| B5 | **查询表达力**：OR / IN / BETWEEN / IS NULL / LIKE 转义 / 括号组合 | where 只能 AND | `plugins/web` | P1 ✅ |
| B6 | **迁移机制**（schema 版本表 + 迁移脚本） | ✅ **已实现（W6）**：`db.migrate` + `_marqdo_migrations`（SQLite） | `plugins/web` + `ext/web` | P2 ✅ |
| B7 | **索引 / 唯一 / 外键声明**（init 表增强） | 完整性、查询性能 | `plugins/web` | P2 |
| B8 | **聚合辅助**：count / group 便捷 API | 统计面板、标签计数 | `plugins/web` | P2 ✅（`db.count`） |
| B9 | **全文搜索**（SQLite FTS5） | ✅ **已实现（W6）**：`db.fts` + `db.search`（bm25） | `plugins/web`（FTS5 现成） | P2 ✅ |
| B10 | **时间戳 / 审计字段自动维护** | 博客发布日期、更新时间 | `plugins/web` | P3 |

### 3.3 类 C：安全硬化（能上线的前提）

| # | 缺失能力 | 影响 | 归属 | 优先级 |
|---|---|---|---|---|
| C1 | **密码哈希**（argon2/bcrypt/sha+盐，替代明文） | 任何人读到库即得全部口令 | `plugins/web`（argon2 在依赖链） | **P0** |
| C2 | **CSRF 防护**（session-bound token，写操作校验） | 表单 POST 可被跨站伪造 | `plugins/web` | **P0** |
| C3 | **登录限速/失败锁定**（每 IP/用户名 5 次/15min） | 登录接口可被爆破 | `plugins/web` | P1 |
| C4 | **会话持久化**（SQLite/文件，替代内存） | 重启全员登出、多 worker 失效 | `plugins/web` | P1 |
| C5 | **加密安全 session id**（CSPRNG + 碰撞重试） | 会话劫持 | `plugins/web`（getrandom） | P1 |
| C6 | **cookie 增强**：`Secure` 标志、签名、滑动过期 | HTTPS 下泄露、篡改、长会话断线 | `plugins/web` | P2 |
| C7 | **防暴力破解 / 会话上限 / 后台过期清理** | 内存膨胀、滥用 | `plugins/web` | P2 |
| C8 | **RBAC / 多角色**（作者/管理员/访客） | 多作者站点 | `plugins/web` + `ext/web` | P3 |

### 3.4 类 D：内容站点标配（博客/CMS 特有）

| # | 缺失能力 | 影响 | 归属 | 优先级 |
|---|---|---|---|---|
| D1 | **SEO**：每页 `<title>`/`meta description`/OG 标签/`canonical`/结构化数据 | 搜索引擎收录 | `ext/web`（页面表扩展，无需插件） | **P0** |
| D2 | **sitemap.xml / robots.txt** 生成 | SEO 刚需 | `ext/web` + `web_page_*` 辅助 | P1 |
| D3 | **RSS/Atom 输出**（`content_type=application/rss+xml` 路由） | 订阅、聚合 | `ext/web` + 新的 XML 装配 | P1 |
| D4 | **分页导航 UI**（上一页/下一页，依赖 B4） | 列表页体验 | `ext/web` | P1 |
| D5 | **Markdown 渲染**（正文存储为 Markdown，渲染为 HTML） | 博客正文 | `lib/net`/标准库（纯解析，与 cookie 同模式） | **P0** |
| D6 | **标签/分类聚合页**（`/tag/{slug}` 动态路由 + 计数） | 博客结构 | `ext/web`（依赖 B8 计数） | P2 |
| D7 | **评论系统**（表单 + 审核 + 反垃圾） | 社区化 | `ext/web`（表单已具备） | P2 |
| D8 | **草稿/发布/定时**（状态字段 + 过滤查询） | 内容管理 | `ext/web` + db where | P2 |
| D9 | **图片/附件库**（依赖 A5 上传 + 相册页） | CMS 必备 | `ext/web` | P3 |

---


## 4. 补强路线（分波次，按「标准库 vs 扩展」边界）

### 4.1 波次建议（每波独立可交付、可回归）

| 波次 | 主题 | 内容 | 对应差距 |
|---|---|---|---|
| **W1** | 服务器基础设施 | 中间件管道（CORS/安全头/压缩/请求体限制）+ JSON API（`application/json` 响应 + DB 查询端点） | A1 A2 A3 A4 A7 ✅ **已完成** |
| **W2** | 数据层深化 | 连接池/busy_timeout/WAL + 事务 API + 结果集裸查询 + 分页(offset) + OR/IN/BETWEEN + 聚合 count | B1 B2 B3 B4 B5 B8 ✅ **已完成** |
| **W3** | 安全硬化 | 密码哈希 + CSRF + 会话持久化 + CSPRNG session id + cookie 增强 + 登录限速/锁定 | C1–C7 |
| **W4** | 内容站点标配 | SEO 元数据 + sitemap/robots + RSS + Markdown 渲染 + 分页导航 UI + 标签/分类 | D1–D6 |
| **W5** | 上传与媒体 | multipart 文件上传接收 + 落盘 + 下载 + 图片库 | A5 D9 ✅ **A5 done**（D9 相册 UI 后续） |
| **W6** | 进阶 | 全文搜索(FTS5) + 迁移机制 + 评论系统 + 草稿/发布 + WS 广播 | B6 B9 ✅；D7/D8 用表+where；WS 广播后续 |

### 4.2 边界判定要点（严格遵守既有约束）

1. **进标准库 `lib/net`**（纯解析、无网络 I/O、支持扩展库开发）：
   - `markdown_parse`（GFM/Markdown → HTML，纯解析）→ D5。
   - `xml_escape` / RSS/Atom 纯文本装配辅助（如需）→ D3。
   - **不得**放进标准库：JSON API、中间件、上传接收、事务、会话（均涉 I/O 或领域语义）。
2. **进 `plugins/web`（ABI）**：A1–A10、B1–B10、C1–C8、D7–D8 的插件侧；tower-http 已提供 CORS/压缩/安全头/Trace 等现成 layer，`axum` 提供 `Json`/`Multipart`/`Nest` 等 extractor，**均不新增核心运行时依赖**（已锁定 rustc 1.81）。
3. **进 `ext/web` 作者面（Marqdo）**：D1–D4 中纯装配/元数据/分页 UI 部分，以及所有插件能力的 GFM 表包装（中英双面）。
4. **已具备、无需新增**：动态路由、静态文件、表单校验/回显、session 增删查、admin 门禁、WebSocket 单连接 echo、SSE 客户端、cookie/multipart 解析。

### 4.3 作者面示例（示 W4 落地形态，保持「代码即文档」）

**SEO 元数据（扩展页面表）：**

```markdown
`meta` =

| 键 | 值 |
|---|---|
| `title` | 文章标题 |
| `description` | 摘要 |
| `og_type` | article |
| `canonical` | /post/{slug} |
```

页面装配时并入 `<head>` —— 仍是「表 + 函数装配」，不引入 `json.*` 袋。

**RSS（扩展路由，响应头 `content_type=application/rss+xml`）：**

```markdown
*feed = > 页面.列出表 rows=`文章` 分页=20 排序="published desc"*
*xml = > 文本.拼装 xs=`rss 片段们`*
*`app` = > `应用`.路由 路径="/feed.xml" 内容=`xml` 类型="application/rss+xml"*
```

---

## 5. 结论与建议顺序

1. **W1（中间件 + JSON）已落地**：`app.configure`/`应用.装配` 用 GFM 表装配 CORS/安全头/gzip/请求体限制/JSON API，经 gold 测试（离线 smoke + 在线 live + curl 实测）与 blog 示例回归验证。
2. **W2（数据层）已落地**：连接池/事务/分页/查询表达力/计数经 `ext_web_db_w2_smoke` gold 测试（离线）全绿——事务提交/回滚后 `count` 正确、`IN/BETWEEN/OR` 过滤、`paginate` 返回 `{rows,total}`、裸 SQL 结果集。`database is locked` 与无法分页的实际痛点已解决。
3. **W3 安全硬化**：上线前必须完成密码哈希 + CSRF + 会话持久化，否则「完整项目」仅能演示。
4. **W4 内容标配**：让博客示例真正「完整」——SEO/Markdown/分页/RSS 是博客系统的门面。
5. **W5/W6**：按项目实际需要取舍（上传、搜索、评论、迁移）。

**一句话：W1（中间件 + JSON）+ W2（事务 + 分页 + 查询表达力）已补齐；离「完整项目」最近的敲门砖变成「密码哈希 + CSRF」和「SEO/Markdown/RSS」两块。这两块补齐后，博客/CMS/社区类站点即可在 `ext/web` 上完整实现；其余为锦上添花。**

---

## 6. 附：现有能力自测对照表（对博客示例逐项）

| 博客功能 | 现状 | 需要的补强 |
|---|---|---|
| 文章列表（首页） | ✅ 已实现 | — |
| 文章详情（动态路由） | ✅ 已实现 | — |
| 标签/分类 | ⚠️ 可做（db where） | 计数聚合（B8 ✅ 已有 `db.count`） |
| 分页 | ✅ db 层（`paginate` 返回 `{rows,total}`） | 页面导航 UI（D4） |
| 搜索 | ❌ | B9（FTS5） |
| SEO 元数据 | ⚠️ 可手写 | D1 系统化 |
| RSS | ❌ | D3 |
| 评论 | ⚠️ 表单可做 | 审核/反垃圾 |
| 后台写文章 | ✅ admin CRUD | 草稿/发布（D8） |
| 上传配图 | ❌ | A5 + D9 |
| 用户登录 | ⚠️ admin 门禁 | 密码哈希/CSRF/限流（W3） |


---

## 7. 附：正文渲染现状（决定 D5 的必要性）

博客详情页正文由 `plugins/web/src/render.rs` 的 `render_article_body` 渲染，但它是**「Markdown-ish」极简子集**，仅支持：

- 段落（`\n\n` 分段）
- `# ` / `## ` 标题（统一渲染为 `<h2>`）
- ` ``` ` 围栏代码块（转义后进 `<pre>`）

**不支持**：有序/无序列表、行内代码、粗体/斜体、链接、图片、表格、引用块、代码高亮。  
**结论**：博客正文的 Markdown 渲染是真实缺口 → D5（`markdown_parse` 纯解析原语进 `lib/net` 标准库，与 `cookie_parse`/`multipart_parse` 同模式，由 `plugins/web` 渲染层或 `ext/web` 作者面调用）优先级 **P0**。
