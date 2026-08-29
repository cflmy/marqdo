# 网络能力调研：从「能跑」到「完整项目」

| | |
|---|---|
| 状态 | **Accepted · W0–W7 + P3 完结**（2026-08-28 复核） |
| 日期 | 2026-08-26（初稿）· 2026-08-28（完结复核） |
| 相关 | [ext-web.md](ext-web.md) · [ext-web-net.md](ext-web-net.md) · [ext-abi.md](ext-abi.md) · [stdlib-modules.md](stdlib-modules.md) |
| 目标 | 盘点 `ext/web` + `plugins/web` + `lib/net` 现状，对照主流语言网络栈，给出「开发一个完整 Web 项目」所需能力的差距清单与补强路线 |

---

## 0. 一句话结论

**`ext/web` + `plugins/web` 已完成 W0–W7 + P3 路线图**：中间件/JSON API、数据层（连接池/事务/分页/FTS/迁移/外键/审计时间戳）、安全硬化（argon2/CSRF/会话持久化/限速/RBAC）、内容站点标配（SEO/RSS/Markdown/分页 UI）、上传下载/相册/ETag、sitemap/robots/错误页/重定向、WebSocket 广播与访问日志。足以在 Marqdo 上实现博客/CMS/中小型 API 站点；**未内置**的仅剩标签页模板（D6，可用路由+`db.count` 自建）与应用层反垃圾。边界不变：**纯解析进 `lib/net`，HTTP 服务器与领域能力进 `plugins/web` + `ext/web` 作者面**。


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
| `markdown_parse` / `解析Markdown` | host `net.rs` | ✅ GFM/Markdown → HTML（纯解析，W4c） |

**标准库网络原语已齐备**：客户端 GET/POST/SSE、URL 编码、cookie、multipart、Markdown 解析。服务器侧上传/下载/ETag 在 `plugins/web`（A5/P3）。

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

**关键实现事实（2026-08-28 W7+P3 后）：**

- **中间件 + JSON API**（W1）：`app.configure` 装配 CORS、安全头、gzip、请求体上限、JSON 端点。
- **数据层**（W2/W6/W7/P3）：连接池、事务、分页、OR/IN/BETWEEN、`db.count`、迁移、FTS5 搜索、init 唯一/索引/外键、审计时间戳。
- **安全**（W3s/P3）：argon2、CSRF、SQLite 会话、CSPRNG session id、登录限速、RBAC（`app.gate` + 用户 `role`）。
- **内容站点**（W4c/W7）：SEO `page.meta`、RSS、Markdown 正文、`page.paginate`、sitemap/robots。
- **媒体**（W5/P3）：multipart 上传、下载、`app.gallery`、ETag/Cache-Control。
- **运维向**（W6/W7）：访问日志、自定义 404/500、301/307 重定向、WS 广播；HTTPS 由反代终止 + `cookie_secure`。
- **仍属应用层约定**：标签聚合页模板（D6）、评论反垃圾、草稿/定时发布（表+where 即可）。

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

> **marqdo 列**反映 **2026-08-28 W7+P3 完结** 后状态（见 §3 差距表）。

| 能力 | FastAPI | Express | Flask | axum | marqdo |
|---|---|---|---|---|---|
| 路由（含动态参数） | ✅ | ✅ | ✅ | ✅ | ✅ 动态路由 `/post/{slug}` |
| 中间件管道 | ✅ | ✅ | ✅ | ✅（tower） | ✅ W1 `app.configure` |
| CORS | ✅ 内置 | ✅ 中间件 | ✅ flask-cors | ✅ tower-http | ✅ W1 |
| JSON 请求/响应 | ✅ 原生 | ✅ | ✅ | ✅ | ✅ W1 JSON API 表 |
| 校验（schema/字段） | ✅ Pydantic | 手动(zod) | 手动 | ✅ extractor | ✅ form 校验表 |
| 静态文件 | ✅ Starlette | ✅ | ✅ | ✅ | ✅ `app.static` |
| 模板/渲染 | ✅ Jinja | ✅ | ✅ | ✅ | ✅ GFM 表装配 |
| 会话 | ✅ | ✅ express-session | ✅ | ✅ | ✅ SQLite 持久化（W3s） |
| 鉴权 | ✅ OAuth2/APIKey | ✅ | ✅ | ✅ | ✅ admin + RBAC（P3） |
| 文件上传 | ✅ | ✅ multer | ✅ | ✅ | ✅ W5 upload |
| WebSocket | ✅ | ✅ ws | ✅ Flask-SocketIO | ✅ | ✅ echo + 广播（W6） |
| 安全头 | ✅ | ✅ helmet | ✅ | ✅ tower-http | ✅ W1 |
| 限流 | ✅ | ✅ | ✅ | ✅ | ✅ 登录限速（W3s） |
| 日志 | ✅ | ✅ morgan | ✅ | ✅ | ✅ access_log（W6） |
| gzip 压缩 | ✅ | ✅ | ✅ | ✅ | ✅ W1 |
| 分页 | 手动 | 手动 | ✅ paginate | ✅ | ✅ db + page UI（W2/W4c） |
| 迁移 | ✅ alembic | ✅ | ✅ | ✅ sqlx | ✅ W6 `db.migrate` |
| 测试工具 | ✅ TestClient | ✅ supertest | ✅ | ✅ | gold 测试（`tests/ext/web-*`） |
| HTTPS/TLS | 反代 | 反代 | 反代 | ✅ rustls | 反代 + `cookie_secure`（W7 文档锁定） |

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
| A6 | **HTTPS / TLS**（rustls 终止或反代提示） | ✅ **文档锁定（W7）**：进程内 TLS 不启用；反代终止 HTTPS，并设 `cookie_secure=True` | 文档 + `cookie_secure` | P2 ✅ |
| A7 | **gzip / br 压缩** + **ETag / Cache-Control 缓存头** | gzip ✅（W1）；Cache-Control ✅（W7）；ETag ✅ **已实现（P3，下载/相册 + If-None-Match→304）** | `plugins/web` | P2 ✅ |
| A8 | **访问日志**（请求方法/路径/状态/耗时） | ✅ **已实现（W6b）**：`configure access_log=True` → stderr | `plugins/web` | P2 ✅ |
| A9 | **自定义 404/500 错误页** | ✅ **已实现（W7）**：`app.error_page` / `应用.错误页`；404 fallback + 500 页袋 | `plugins/web` | P2 ✅ |
| A10 | **重定向增强**（permanent/自定义状态码） | ✅ **已实现（W7）**：`app.redirect` → 301/307 | `plugins/web` | P3 ✅ |

### 3.2 类 B：数据层深化（内容站点/社区的根基）

| # | 缺失能力 | 影响 | 归属 | 优先级 |
|---|---|---|---|---|
| B1 | **连接池 / busy_timeout / WAL** | 高并发下 `database is locked`、每请求重连开销 | `plugins/web`（db.rs） | **P0 ✅** |
| B2 | **事务 API**（`begin` / `commit` / `rollback`，或 `with_transaction` 包装） | 批量/多表写无原子性 | `plugins/web` | **P0 ✅** |
| B3 | **结果集型裸查询**（exec 返回 rows） | count / join / group / 子查询完全不可用 | `plugins/web` | P1 ✅ |
| B4 | **分页**（db 层 `limit`+`offset`；页面层上一页/下一页） | 列表页只能限量 | `plugins/web` + `ext/web` | P1 ✅（db 层；页面 UI 属 D4） |
| B5 | **查询表达力**：OR / IN / BETWEEN / IS NULL / LIKE 转义 / 括号组合 | where 只能 AND | `plugins/web` | P1 ✅ |
| B6 | **迁移机制**（schema 版本表 + 迁移脚本） | ✅ **已实现（W6）**：`db.migrate` + `_marqdo_migrations`（SQLite） | `plugins/web` + `ext/web` | P2 ✅ |
| B7 | **索引 / 唯一 / 外键声明**（init 表增强） | ✅ **已实现（W7+P3）**：`唯一`/`索引`/`外键`（`posts.id` / `posts`） | `plugins/web` | P2 ✅ |
| B8 | **聚合辅助**：count / group 便捷 API | 统计面板、标签计数 | `plugins/web` | P2 ✅（`db.count`） |
| B9 | **全文搜索**（SQLite FTS5） | ✅ **已实现（W6）**：`db.fts` + `db.search`（bm25） | `plugins/web`（FTS5 现成） | P2 ✅ |
| B10 | **时间戳 / 审计字段自动维护** | ✅ **已实现（P3）**：表含 `created_at`/`updated_at` 时 insert/update 自动写入 | `plugins/web` | P3 ✅ |

### 3.3 类 C：安全硬化（能上线的前提）

| # | 缺失能力 | 影响 | 归属 | 优先级 |
|---|---|---|---|---|
| C1 | **密码哈希**（argon2/bcrypt/sha+盐，替代明文） | ✅ **已实现（W3s）**：argon2 | `plugins/web` | **P0 ✅** |
| C2 | **CSRF 防护**（session-bound token，写操作校验） | ✅ **已实现（W3s）** | `plugins/web` | **P0 ✅** |
| C3 | **登录限速/失败锁定**（每 IP/用户名 5 次/15min） | ✅ **已实现（W3s）** | `plugins/web` | P1 ✅ |
| C4 | **会话持久化**（SQLite/文件，替代内存） | ✅ **已实现（W3s）** | `plugins/web` | P1 ✅ |
| C5 | **加密安全 session id**（CSPRNG + 碰撞重试） | ✅ **已实现（W3s）** | `plugins/web` | P1 ✅ |
| C6 | **cookie 增强**：`Secure` 标志、签名、滑动过期 | ✅ **已实现**：`cookie_secure` + 滑动 TTL（touch）；cookie 签名未单独做 | `plugins/web` | P2 ✅ |
| C7 | **防暴力破解 / 会话上限 / 后台过期清理** | ✅ 限速 + prune；会话条数硬上限可后续 | `plugins/web` | P2 ✅ |
| C8 | **RBAC / 多角色**（作者/管理员/访客） | ✅ **已实现（P3）**：用户表 `role`/`角色`；`app.gate`；`/admin*` 默认需 `admin` | `plugins/web` + `ext/web` | P3 ✅ |

### 3.4 类 D：内容站点标配（博客/CMS 特有）

| # | 缺失能力 | 影响 | 归属 | 优先级 |
|---|---|---|---|---|
| D1 | **SEO**：每页 `<title>`/`meta description`/OG 标签/`canonical`/结构化数据 | ✅ **已实现（W4c）**：`page.meta` | `ext/web` | **P0 ✅** |
| D2 | **sitemap.xml / robots.txt** 生成 | ✅ **已实现（W7）**：`app.sitemap` / `app.robots` | `plugins/web` + `ext/web` | P1 ✅ |
| D3 | **RSS/Atom 输出**（`content_type=application/rss+xml` 路由） | ✅ **已实现（W4c）**：`route_rss` | `ext/web` + 插件 | P1 ✅ |
| D4 | **分页导航 UI**（上一页/下一页，依赖 B4） | ✅ **已实现（W4c）**：`page.paginate` | `ext/web` | P1 ✅ |
| D5 | **Markdown 渲染**（正文存储为 Markdown，渲染为 HTML） | ✅ **已实现（W4c）**：`lib/net` + 页面装配 | `lib/net` | **P0 ✅** |
| D6 | **标签/分类聚合页**（`/tag/{slug}` 动态路由 + 计数） | 可用路由+`db.count` 自建（未内置模板） | `ext/web` | P2 |
| D7 | **评论系统**（表单 + 审核 + 反垃圾） | ✅ 约定：表+表单+where（W6）；反垃圾属应用层 | `ext/web` | P2 ✅ |
| D8 | **草稿/发布/定时**（状态字段 + 过滤查询） | ✅ 约定：`status` + where（W6） | `ext/web` + db where | P2 ✅ |
| D9 | **图片/附件库**（依赖 A5 上传 + 相册页） | ✅ **已实现（P3）**：`app.gallery` HTML 相册 + A5 上传/下载 | `ext/web` + `plugins/web` | P3 ✅ |

---


## 4. 补强路线（分波次，按「标准库 vs 扩展」边界）

### 4.1 波次建议（每波独立可交付、可回归）

| 波次 | 主题 | 内容 | 对应差距 |
|---|---|---|---|
| **W1** | 服务器基础设施 | 中间件管道（CORS/安全头/压缩/请求体限制）+ JSON API（`application/json` 响应 + DB 查询端点） | A1 A2 A3 A4 A7 ✅ **已完成** |
| **W2** | 数据层深化 | 连接池/busy_timeout/WAL + 事务 API + 结果集裸查询 + 分页(offset) + OR/IN/BETWEEN + 聚合 count | B1 B2 B3 B4 B5 B8 ✅ **已完成** |
| **W3** | 安全硬化 | 密码哈希 + CSRF + 会话持久化 + CSPRNG session id + cookie 增强 + 登录限速/锁定 | C1–C7 ✅ |
| **W4** | 内容站点标配 | SEO 元数据 + sitemap/robots + RSS + Markdown 渲染 + 分页导航 UI + 标签/分类 | D1–D5 ✅；D6 约定 |
| **W5** | 上传与媒体 | multipart 文件上传接收 + 落盘 + 下载 + 图片库 | A5 ✅；D9 相册 P3 ✅ |
| **W6** | 进阶 | 全文搜索(FTS5) + 迁移机制 + 评论系统 + 草稿/发布 + WS 广播 | B6 B9 ✅；WS 广播 + access_log ✅；D7/D8 用表+where ✅ |
| **W7** | 完结打磨 | Cache-Control、自定义 404/500、301 重定向、sitemap/robots、init 唯一/索引、HTTPS 反代文档 | A6 A7 A9 A10 B7 D2 ✅ |
| **P3** | 真·网络扩展补齐 | 审计时间戳、外键、RBAC 门禁、媒体相册、下载 ETag | B10 C8 D9 + ETag/FK ✅ |

### 4.2 边界判定要点（严格遵守既有约束）

1. **进标准库 `lib/net`**（纯解析、无网络 I/O、支持扩展库开发）：
   - `markdown_parse`（GFM/Markdown → HTML，纯解析）→ D5 ✅。
   - `xml_escape` / RSS/Atom 纯文本装配辅助（如需）→ D3（作者面 `route_rss` 已够用）。
   - **不得**放进标准库：JSON API、中间件、上传接收、事务、会话（均涉 I/O 或领域语义）。
2. **进 `plugins/web`（ABI）**：A1–A10、B1–B10、C1–C8、D7–D8 的插件侧 — **均已落地**（2026-08-28）。
3. **进 `ext/web` 作者面（Marqdo）**：D1–D4、A5/D2/D9 等 GFM 表包装 — **均已落地**（中英 `web.mq.md` / `网页.mq.md`）。
4. **已具备、无需新增**：动态路由、静态文件、表单校验/回显、session、admin/RBAC、WebSocket、SSE 客户端、cookie/multipart/Markdown 解析。

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

## 5. 结论

**W0–W7 + P3 已全部落地**（路线图见 [roadmap/ext-web.md](../roadmap/ext-web.md)）。验收覆盖：

- 离线/在线 gold：`tests/ext/web-*-smoke.mq.md`（含 security、drivers、upload、w6、w7、p3）
- 完整示例：`examples/marqdo-blog/`（列表/详情/标签/RSS/后台/上传/WS）
- 作者面：`ext/web/web.mq.md` · `ext/web/网页.mq.md`
- AI 编写指引：`skills/marqdo/`（§ ext/web）

**后续可选（非阻塞）**：标签页内置模板（D6）、cookie 签名、会话条数硬上限、应用层反垃圾、进程内 TLS（仍建议反代）。

**部署对照（Daphne / Uvicorn / 反代）：** 见 [web-asgi-servers-and-marqdo.md](web-asgi-servers-and-marqdo.md) — Marqdo **不能**被 Daphne 托管；生产等价路径是 **反代 → 内嵌 axum `listen`**。

**一句话：`ext/web` 已是可上线级别的 Marqdo 动态站扩展；博客/CMS/中小型 API 可在表格 + 类方法作者面上完整实现。**

---

## 6. 附：博客示例能力对照（`examples/marqdo-blog`）

| 博客功能 | 现状 |
|---|---|
| 文章列表（首页） | ✅ |
| 文章详情（动态路由） | ✅ |
| 标签/分类 | ✅ 路由 + `db.where` + `db.count` |
| 分页 | ✅ `db.paginate` + `page.paginate` |
| 搜索 | ✅ FTS5 `db.search` |
| SEO 元数据 | ✅ `page.meta` |
| RSS | ✅ `app.route_rss` |
| 评论 | ✅ 表 + form + where（反垃圾属应用层） |
| 后台写文章 | ✅ admin CRUD + RBAC |
| 草稿/发布 | ✅ `status` 字段 + where |
| 上传配图 | ✅ `app.upload` + `app.gallery` |
| 用户登录 | ✅ argon2 + CSRF + SQLite 会话 + 限速 |
| sitemap / robots | ✅ W7 |
| 实时终端（WS） | ✅ W6 广播 |

---

## 7. 附：Markdown 正文渲染（D5，已落地）

博客详情页正文经 `lib/net.markdown_parse`（`host_markdown_parse`）解析为 HTML，由 `plugins/web` 渲染层在页面装配时调用（W4c）。作者存储 Markdown 字符串于 DB，展示侧自动渲染 GFM 子集（标题、列表、链接、代码块等）。纯解析留在标准库；**不得**在 `ext/web` 作者面直接调 `host_*`。
