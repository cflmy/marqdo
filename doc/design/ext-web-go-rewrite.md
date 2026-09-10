# `plugins/web` Go 全量重写（多语言 ABI 生态 · 网页制备完备）

| | |
|---|---|
| 状态 | **Accepted · 实现进行中（W-G0…W-G4 ✅）** |
| 日期 | 2026-09-10 |
| 相关 | [ext-web.md](ext-web.md) · [ext-web-net.md](ext-web-net.md) · [web-net-capabilities.md](web-net-capabilities.md) · [ext-web-drivers.md](ext-web-drivers.md) · [web-assets-and-images.md](web-assets-and-images.md) · [ext-hosting-fill.md](ext-hosting-fill.md) · [ext-abi.md](ext-abi.md) · [ext-cli.md](ext-cli.md) · [web-asgi-servers-and-marqdo.md](web-asgi-servers-and-marqdo.md) |
| 实现目录 | `plugins/web/`（**Go module**；产物仍为 `libweb.so` / `web.dll` / `libweb.dylib`） |
| 作者面 | `ext/web/web.mq.md` · `ext/web/网页.mq.md`（**ABI 名与方法面保持兼容**；非另起一套 API） |
| 验收应用 | 工作区 `anlian`（暗恋见君）：用 Marqdo 前后端脚本完整重写；**PostgreSQL + Redis 外接** |

---

## 0. 一句话

用 **Go** 重写官方网络拓展库原生层（`plugins/web`），在 **C ABI v2 全量兼容** 前提下提供「网页制备」所需的全部服务端能力；`ext/web` 继续用 GFM 表 + `#` 类做作者面；应用（含暗恋见君）的前后端脚本全部用 Marqdo 书写。此举同时验证 **多语言 ABI 插件生态**（首个非 Rust 官方插件）。

**不是**最小 hello 插件；**不是**在旧 Rust 代码上打补丁（对齐 [ext-web.md](ext-web.md) **C5**）。

---

## 1. 动机与目标

### 1.1 为何重写

| 动机 | 说明 |
|------|------|
| 开发/编译节奏 | Rust 插件迭代慢，网络域能力长期「半成品感」来自堆叠依赖与慢反馈，而非语言绝对性能 |
| 多语言生态 | ABI 已是稳定 C 契约；需要第一个 **Go** 官方插件证明「宿主 Rust ≠ 插件必须 Rust」 |
| 网页制备完备 | 目标是支持 **完整动态站点制备**（装配、CRUD、鉴权、实时、上传、SEO、代理、驱动…），并以真实论坛项目验收 |
| 性能与拉起 | Go `net/http` 生态成熟、单二进制友好；论坛级 IO 密集负载下足够「极致可用」 |

### 1.2 成功标准（全部满足才算完成）

1. **ABI 全量对等**：下文 §5 列出的每一个已注册 `web_*` 符号，参数 CSV、JSON 形、语义与现网 gold 一致（中英参数别名保留）。
2. **作者面可用**：现有 `ext/web` 示例与 `tests/ext/web-*`（含 live-server）在换 Go `.so` 后通过；无需作者改业务脚本（除非文档标明的 **G 增强**）。
3. **网页制备完备**：单站点可完成页面装配、样式/头/图、库表、表单、路由、中间件、鉴权/RBAC、上传下载、SEO/RSS/sitemap、WS、缓存、对象存储、反代、invoke——即 W0–W8 + P3 + C0–C4 + H1/H2 能力面。
4. **驱动外接**：`postgres://`、`redis://`、`s3://` 为正式一等能力（与 [ext-web-drivers.md](ext-web-drivers.md) 一致）；SQLite / memory / file 仍支持本地与测试。
5. **暗恋见君可写完**：用 Marqdo 后端脚本 + 浏览器 WASM 前端脚本重写 anlian 核心模块（见 §8）；运行时仅依赖 `marqdo` + `libweb` + **外接 PostgreSQL + Redis**（可选 S3/RustFS）。
6. **安装路径不变**：`marqdo ext add web`、`plugin.native_path name=web`、`MARQDO_WEB_PLUGIN`、产物文件名 `libweb.so` 等与 [ext-cli.md](ext-cli.md) 兼容。

### 1.3 非目标

| 非目标 | 说明 |
|--------|------|
| 改 Marqdo 核心语法 | 仍遵守 ext-web **C1** |
| 把服务器搬进浏览器 WASM | 仍遵守 ADR 0002 |
| 进程内 TLS | HTTPS 由反代；`cookie_secure` 保留 |
| 第三方插件市场 | 仍无 registry |
| 把论坛业务写进 Go 插件 | 帖子/新闻/聊天室规则在 `.mq.md`；插件只提供原语 |
| 强制嵌入式替代 PG/Redis | 本项目明确 **允许并依赖外接** PG/Redis |

---

## 2. 分层架构（锁定）

```text
┌──────────────────────────────────────────────────────────────────┐
│  应用（anlian / 任意站点）                                         │
│  · 后端 .mq.md：路由、业务、装配、invoke 处理函数                    │
│  · 前端 .mq.md：WASM + lib/browser（作者零业务 JS）                 │
└────────────────────────────┬─────────────────────────────────────┘
                             │ import web / 网页
┌────────────────────────────▼─────────────────────────────────────┐
│  ext/web（L1 · Marqdo）                                           │
│  # page # db # app # form # auth # cache # storage # media # ws   │
│  make_style / make_head / make_images / client_embed …           │
└────────────────────────────┬─────────────────────────────────────┘
                             │ lib/plugin → dlopen
┌────────────────────────────▼─────────────────────────────────────┐
│  plugins/web（L0 · Go · C ABI v2）  ← 本文件重写对象                 │
│  net/http + 驱动适配 + HTML 装配 + WS hub + 中间件 …                │
└──────────┬─────────────────┬─────────────────┬───────────────────┘
           │                 │                 │
    PostgreSQL            Redis            S3/MinIO/RustFS
    （外接）               （外接）           （可选外接）
```

| 层 | 语言 | 职责 |
|----|------|------|
| 应用脚本 | Marqdo | 前后端业务、页面表、领域模型 |
| `ext/web` | Marqdo | 双语作者 API；禁止直接 `host_*` |
| `plugins/web` | **Go** | 全部 `web_*` ABI；高性能网络与驱动 |
| `lib/net` | Marqdo + Rust host | **仅**客户端 HTTP / cookie / multipart / markdown **解析**；不进本重写范围（边界不变） |
| 宿主 | Rust | `dlopen`、JSON Value、`host_query`（`entry_dir` / `call_lib_path` …） |

---

## 3. 硬约束（继承 + 本重写追加）

### 3.1 继承 ext-web C1–C7

| ID | 约束 |
|----|------|
| C1 | 不为 web 改核心语法 |
| C2 | 作者面禁止 json 袋胶水拼装页面 |
| C3 | 中英 API 分文件；禁止混排 |
| C4 | 首页 `index.mq.md`；组件可拆 |
| C5 | **禁止在旧 Rust 插件上打补丁演进**；清空/替换后用 Go 重写 |
| C6 | `# db` 完整 CRUD；`select`/`查询`；admin 与表单共用写库 API |
| C7 | 表单字段+规则；**写库前服务端校验** |

### 3.2 本重写追加（G 系列）

| ID | 约束 |
|----|------|
| **G1** | 产物仍为 C ABI v2；`MARQDO_ABI_VERSION = 2`；导出符号名与 [include/marqdo_abi.h](../../include/marqdo_abi.h) 一致 |
| **G2** | **ABI 函数名与参数名向后兼容**：§5 清单中每个 `web_*` 必须注册；参数 CSV 与现 Rust 插件一致（含中文别名） |
| **G3** | Go 使用 `-buildmode=c-shared`；**禁止**把 Go runtime 假设成唯一线程——跨 FFI 调用必须可重入或明确串行化文档 |
| **G4** | `marqdo_plugin_shutdown` 必须停止 HTTP/WS、关闭 DB/Redis 连接、释放 hub |
| **G5** | 字符串跨界：成功时 `*out_json` / 失败时 `*err_msg` 必须用 **host `alloc`** 分配（与现插件相同），由宿主 `free` |
| **G6** | 相对路径（sqlite 文件、静态目录、上传根）一律相对 `host_query("entry_dir")` |
| **G7** | 多语言样板：仓库内保留可复用的 Go ABI 脚手架注释/包结构，供后续其它语言插件参考 |
| **G8** | 金样例不过 = 未完成；切换默认 `libweb` 到 Go 后，既有 `tests/ext/web-*` 必须绿 |

---

## 4. 技术选型（Go）

| 能力 | 选型 | 备注 |
|------|------|------|
| HTTP 服务器 | 标准库 `net/http` | 路由可用轻量自研 mux 或 `chi`/`http.ServeMux`（Go 1.22+ 模式） |
| 中间件 | 自研链式 `func(http.Handler) http.Handler` | 对等 CORS / 安全头 / gzip / body limit / access_log / cache-control |
| WebSocket | `gorilla/websocket` 或 `coder/websocket` | 对等 echo / broadcast / drain；**G 增强**：命名房间（§7） |
| SQLite | `modernc.org/sqlite`（纯 Go）或 `mattn/go-sqlite3`（cgo） | 优先纯 Go 以便交叉编译；FTS5/migrate 行为对齐 |
| PostgreSQL | `jackc/pgx/v5` | `postgres://` / `postgresql://` |
| Redis | `redis/go-redis/v9` | `# cache`；**G 增强**：可选 session/WS 后端 |
| S3 | `aws-sdk-go-v2` 或 MinIO SDK | `s3://`；兼容 RustFS |
| 密码 | `golang.org/x/crypto/argon2` | argon2id 参数对齐现实现 |
| Markdown | `goldmark`（GFM 扩展） | 对等 `pulldown-cmark` 用途（正文 HTML） |
| JSON | `encoding/json` | ABI 边界 |
| 构建 | `go build -buildmode=c-shared -o libweb.so` | 见 §9 |

**不引入**完整 Web 框架（Gin/Echo 业务层）以免作者面旁路；框架能力用 Marqdo `# app` 表达。

---

## 5. ABI 全量清单（必须注册 · 兼容）

> 参数列为注册时 `params` CSV（英；实现须同时接受设计中已有的中文键别名，与现 `lib.rs` 一致）。  
> 状态列：`P0` = 首波必须与 gold 对齐；实现顺序见 §10。

### 5.1 Page / compose / assets / render

| ABI | params（摘要） | 域 |
|-----|----------------|-----|
| `web_page_new` | （空或 title 等，对齐现实现） | page |
| `web_compose_components` | page, components, … | compose |
| `web_compose_main` | page, main, … | compose |
| `web_page_query` | page, query | page |
| `web_page_order` | page, order | page |
| `web_page_link_prefix` | page, prefix | page |
| `web_page_css` | page, css | page |
| `web_page_detail` | page, … | page |
| `web_page_meta` | page, meta | SEO |
| `web_page_head` | page, head | W8 |
| `web_page_images` | page, images | W8 |
| `web_page_paginate` | page, … | content |
| `web_head` | table / … | W8 |
| `web_images` | table / … | W8 |
| `web_app_icons` | app, icons | W8 |
| `web_style` | table, strict? | CSS |
| `web_compose_form` | page, form, target? | form embed |
| `web_render` | page | HTML |

### 5.2 SEO / ops routes

| ABI | 域 |
|-----|-----|
| `web_rss_build` | RSS |
| `web_app_route_rss` | app |
| `web_app_redirect` | app |
| `web_app_error_page` | app |
| `web_app_sitemap` | app |
| `web_app_robots` | app |
| `web_sitemap_build` | sitemap |

### 5.3 Upload / media

| ABI | 域 |
|-----|-----|
| `web_app_upload` | app |
| `web_app_download` | app |
| `web_upload_validate` | media |
| `web_upload_save` | media |
| `web_media_new` | media |
| `web_app_gallery` | app |

### 5.4 Database

| ABI | 备注 |
|-----|------|
| `web_db_new` | `sqlite:` / `postgres://` |
| `web_db_init` | unique / index / fk / 审计时间戳 |
| `web_db_insert` / `select` / `get` / `update` / `delete` / `exec` | CRUD |
| `web_db_query` / `count` | 原生 / 计数 |
| `web_db_migrate` | **SQLite only**（对等） |
| `web_db_fts_create` / `web_db_search` | **SQLite only** |
| `web_db_begin` / `commit` / `rollback` | 事务句柄 |
| `web_db_list_tables` / `web_db_table_info` | admin 内部 |

Where 语义：等值 map、filter 表、OR / IN / BETWEEN、分页 offset/limit——行为对齐现 `db.rs` / `db_pg.rs`。

### 5.5 Cache / storage

| ABI | URL |
|-----|-----|
| `web_cache_new` / `get` / `set` / `del` / `exists` / `ttl` | `memory:` / `redis://` |
| `web_storage_new` / `put` / `get` / `delete` / `list` | `file:` / `s3://` |

### 5.6 App / HTTP / hosting

| ABI | 域 |
|-----|-----|
| `web_app_new` | app |
| `web_app_route` | 动态 `{param}` + `/_part` |
| `web_app_mount_form` | `/_form` |
| `web_app_static` | 静态目录 |
| `web_app_middleware` | `configure`：CORS、安全头、gzip、body_limit、JSON API、access_log、cache_control |
| `web_app_proxy` | 流式反代 / SSE（H1） |
| `web_app_invoke` | HTTP → `lib.member`（H2；经 `host_query call_lib_path`） |
| `web_listen` | 阻塞监听；组装全部路由 |
| `web_app_route_ws` | WS 端点 |
| `web_ws_connect` | 客户端单次请求-响应式 |
| `web_app_auth` | `/admin` 门禁 |
| `web_app_gate` | RBAC 段前缀 |

### 5.7 Form

| ABI |
|-----|
| `web_form_new` / `fields` / `rules` / `validate` / `render` / `submit` / `from_schema` |

规则：`required`、`min:N`、`max:N`、`email`、`url`、`match:field`、`in:a,b,c`。

### 5.8 Session / auth

| ABI |
|-----|
| `web_session_new` / `set` / `get` / `del` / `destroy` |
| `web_auth_new` / `login` / `check` / `logout` |
| `web_password_hash` |

安全对等：argon2id、CSRF、登录限速、CSPRNG session id、cookie Secure（配置）、滑动 TTL、持久化会话（默认 SQLite 会话表；见 §7 Redis 可选）。

### 5.9 HTTP 挂载面（`web_listen` 必须复现）

- `GET /`，`GET /_part/{id}`，`GET|POST /_form/{id}`
- 可选 admin：`{prefix}/`、`/login`、`/logout`、`/{table}`、`/new`、`/{id}/edit`、`/{id}/delete`（C0：`admin=False` 让出 `/admin`）
- 作者 `route` 页 + `{path}/_part/{id}`；动态路径参数
- WS；RSS；upload POST；download GET；gallery；sitemap；`/robots.txt`
- 301/307 重定向；自定义 404/500；proxy；invoke
- 静态与 icons；约定 `/favicon.ico`
- 保留名：`/_form`、`/_part`、静态挂载冲突检测

### 5.10 `host_query` 依赖

| name | 用途 |
|------|------|
| `entry_dir` | 路径沙箱根 |
| `call_lib_path` | invoke / 嵌套 lib 调用（含 `args`） |
| （其它已允许项） | 与 ABI v2 allowlist 一致；不得扩展任意 HostFn |

---

## 6. 作者面（保持 · 不另起炉灶）

英文 `ext/web/web.mq.md` / 中文 `ext/web/网页.mq.md` 现有 `#` 类与 `##` 方法 **保持**：

| 类 | 职责 |
|----|------|
| `# page` / `# 页面` | 装配、meta、head、images、paginate、render |
| `# db` / `# 数据库` + `# txn` | CRUD、事务、migrate/FTS（SQLite） |
| `# form` / `# 表单` | 字段、规则、校验、渲染、提交 |
| `# app` / `# 应用` | route、configure、listen、auth、gate、upload… |
| `# auth` / `# 鉴权` | login/check/logout/hash |
| `# cache` / `# 缓存` | memory/redis |
| `# storage` / `# 存储` | file/s3 |
| `# media` / `# 媒体` | validate/save |
| `# ws` / `# 实时` | connect |
| 模块函数 | `ensure_plugin`、`make_style`、`make_head`、`make_images`、`client_embed`、`text_patch`、`dom_patch`、`list_html` |

**重写默认策略**：先 **只换原生 `.so`**，L1 不动；仅当 §7 G 增强需要新方法时，再 **追加**（不破坏旧方法）。

---

## 7. G 增强（完备网页制备 + anlian；兼容之上增量）

> 以下为 **新增能力**，不替代 §5。作者面用新方法/新 configure 键；旧脚本不受影响。

| ID | 能力 | 动机 | 作者面草案 |
|----|------|------|------------|
| **G-WS1** | **命名房间** WS：`join`/`leave`/`publish`；组名 ASCII（如 `chat.room.{id}`） | anlian 公聊/私聊 | `app.route_ws` 增 `mode=room` + `room_key`；或 `ws.room_*` ABI |
| **G-WS2** | 房间消息钩子：先 `invoke`/落库再广播 | 「先写 PG 再推送」 | configure 或 route 表 `on_message=lib.member` |
| **G-SESS1** | Session 后端可选 `redis://` | 多进程/多实例 | `web_session_new` url 或 auth 配置 `session=redis://` |
| **G-CACHE1** | Redis pub/sub 可选作 WS 扇出后端 | 多进程 WS | `configure ws_backend=redis` |
| **G-API1** | Bearer API Key 校验原语（hash+pepper+scopes） | anlian `apikeys` | `web_api_key_check` + `# auth` 或独立小类；**业务 scopes 仍在 .mq.md** |
| **G-MAIL0** | （可选后续）SMTP 发送原语 | 邮箱验证码 | 不阻塞首版对等；列入 anlian 二期 |

**明确仍属应用层（不进插件）**：反垃圾策略、草稿定时发布约定、好友关系、专题 `topic_bundles` 静态交互内容（静态挂载 + 前端 Marqdo/既有 JS 资产）。

---

## 8. 暗恋见君验收矩阵（检测「能不能写完」）

外部依赖（允许）：**PostgreSQL**、**Redis**；可选对象存储。

| 模块 | Marqdo 后端 | Marqdo 前端 | 插件依赖 | 验收 |
|------|-------------|-------------|----------|------|
| 站点壳 / 首页 | `# page` + db | 局部刷新可选 WASM | listen、static、cache | 首页可开 |
| posts 帖子/评论/板块 | route + form + db | 发帖表单 WASM 可选 | CRUD、CSRF、upload | CRUD+列表详情 |
| news | 同上 | 同上 | 同上 | 列表详情 |
| accounts | auth + session +（二期 mail） | 登录表单 | argon2、session、gate | 注册登录 |
| profiles | db + storage | 头像上传 | storage/upload | 资料页 |
| search | db.search 或 PG 查询 + cache | — | db/cache | 关键词搜索 |
| chat | route_ws + G-WS1/2 + db | WS 客户端脚本 | Redis + WS rooms | 公聊+私聊实时 |
| apikeys | G-API1 + db | 管理页 | — | Bearer 调 API |
| webmaster | sitemap/robots | — | 已有 ABI | `/sitemap.xml` |
| topics | `app.static` → bundles | 既有静态交互 | static | 专题可访问 |

**「写完」定义**：上述模块在 Marqdo 仓库或 `anlian` 旁路目录有可运行 `.mq.md` 站点；gold/手工脚本能走通主路径；不依赖 Django/Channels/Daphne/gunicorn。

---

## 9. 仓库布局与构建

### 9.1 目录（目标）

```text
plugins/web/                 # Go module（替换原 Rust crate）
  go.mod
  go.sum
  README.md
  abi/
    marqdo_abi.h             # 从 include/ 拷贝或 cgo 引用
    bridge.go                # //export marqdo_plugin_* ；register；alloc/free
  internal/
    page/ compose/ render/ form/
    db/          # sqlite + postgres driver
    cache/ storage/ upload/
    session/ auth/ password/ ratelimit/
    httpx/       # listen、router、middleware、admin、proxy、invoke
    ws/          # hub、rooms、client connect
    assets/ rss/ sitemap/ markdown/
    table/       # GFM 表 JSON 形态辅助
  cmd/                        # 可选：非 c-shared 的本地自测
scripts/build-web-plugin.sh   # go build -buildmode=c-shared -o …/libweb.so
```

原 Rust 源码：迁移期可暂存 `plugins/web-rust-archive/` 或 git 历史保留；**默认构建不再 `cargo build -p marqdo_plugin_web`**。

### 9.2 产物与解析

| 平台 | 文件名 |
|------|--------|
| Linux | `libweb.so` |
| macOS | `libweb.dylib` |
| Windows | `web.dll` |

解析顺序仍走 `installed_native_path` / `MARQDO_WEB_PLUGIN`；`find_native_plugin` 增加查找：

- `plugins/web/build/libweb.so`（Go 构建输出）
- `target/{debug,release}/libweb.so`（兼容旧路径，可选 copy）

### 9.3 宿主 / CI 变更要点

| 组件 | 变更 |
|------|------|
| 根 `Cargo.toml` workspace | 移除 `plugins/web` Rust member |
| `tests/gold.rs` | `cargo build -p marqdo_plugin_web` → 调用 `scripts/build-web-plugin.sh` |
| `marqdo ext add web` | 构建回退：有 Go 则 `go build`；Release 仍下发预编译 native zip |
| Release 脚本 | 增加 Go 交叉/本机编译 `libweb` 打包步骤 |
| [ext-cli.md](ext-cli.md) / [ext-abi.md](ext-abi.md) | 注明 web 插件实现语言为 Go |

### 9.4 cgo / runtime 注意

- `marqdo_plugin_init` 内 `register_fn` 注册全部符号；保存 host `alloc`/`free`/`host_query`。
- 长时间 `web_listen`：在独立 goroutine 跑 `http.Server`，ABI 调用阻塞至 shutdown 或按现语义阻塞（**对齐现 Rust：listen 阻塞当前插件调用**）。
- 禁止在 FFI 回调里长时间持有阻碍 GC 的错误模式；JSON 尽快拷贝到 Go string。
- Windows 导出需 `.def` 或 `//export` 完整三个符号。

---

## 10. 实现波次（按文档开发顺序）

| 波次 | 内容 | 退出条件 |
|------|------|----------|
| **W-G0** | 文档（本文）+ ADR + 目录骨架 + ABI bridge 能 `plugin.load` + `demo` 注册探测 | ✅ `tests/ext/web-go-abi-smoke.mq.md`；`web_go_ready` + `web_page_new`；`scripts/build-web-plugin.sh` → `libweb.so` |
| **W-G1** | table 辅助 + `web_style` + page new/compose/render（无 listen） | ✅ `web-smoke` / `web-zh-smoke`（含 compose_form / app_new / app_route 袋） |
| **W-G2** | SQLite db CRUD + where + txn + init | ✅ `web-db-w2-smoke` / `web-db-w6-smoke`（txn·migrate·FTS） |
| **W-G3** | form + mount_form + CSRF 基础 | ✅ `web-form-smoke`（validate/submit/render；CSRF 字段已接，listen 侧硬化见 W-G4/G5） |
| **W-G4** | `web_listen` + route + static + part + middleware + JSON API | ✅ offline：`web-route` / `web-static` / `web-middleware` / `web-part`；`web_listen` 已接线（live 随 W-G5+ hardening） |
| **W-G5** | session/auth/password/rate limit/gate/admin | ✅ offline：`web-security` / `web-c0` / `web-c1`；login rate limit 包已就位（HTTP 接线随 listen hardening） |
| **W-G6** | upload/download/gallery/ETag + assets W8 | `web-upload` / `web-assets` / `web-p3` |
| **W-G7** | SEO RSS sitemap robots redirect error paginate markdown | `web-content` / `web-w7-*` |
| **W-G8** | WS echo/broadcast + access_log | `web-ws-broadcast-*` |
| **W-G9** | Postgres + Redis cache + file/S3 storage | `web-drivers-smoke` |
| **W-G10** | proxy + invoke + nested plugin | `web-proxy-invoke` / hosting live |
| **W-G11** | 定制 C2–C4 shell/layout/style/nav | `web-c2`–`c4` |
| **W-G12** | G 增强 WS rooms + Redis session/pubsub + API key 原语 | 单测 + anlian chat 切片 |
| **W-G13** | 默认切换 Go `libweb`；归档 Rust；更新发版与 ext CLI | 全量 `tests/ext/web-*` 绿 |
| **W-G14** | anlian Marqdo 重写主路径（可分仓或 `examples/anlian-mq/`） | §8 矩阵主路径可演示 |

每波次：**先补 Go 实现 → 跑对应 gold → 再进入下一波**。禁止跨波次留下「注册了但返回 stub 错误」的 ABI（除 W-G0 明确的未实现探测）。

---

## 11. 测试策略

| 类型 | 做法 |
|------|------|
| 单元 | Go `go test ./internal/...`（db where、form rules、middleware） |
| 金样例 | 现有 `tests/ext/web-*.mq.md`；构建脚本产出 `libweb.so` 后跑 |
| Live | 现有 `*-live-server.mq.md` + curl |
| 回归门禁 | CI：安装 Go → `scripts/build-web-plugin.sh` → `cargo test` 相关 web 项 |
| anlian | 独立 smoke：登录、发帖、WS 发言、API key |

---

## 12. 风险与缓解

| 风险 | 缓解 |
|------|------|
| cgo/共享库与宿主线程 | 文档化；listen 独立 goroutine；shutdown 明确 |
| JSON ABI 开销 | 热路径减少往返；装配结果缓存；不在此用「换语言」幻想消除 |
| 行为微差异（HTML/Markdown） | 用同一批 gold HTML 片段断言；差异列 changelog |
| 交叉编译 | 优先 `modernc.org/sqlite`；Release 矩阵 Linux/Windows 先 |
| 双实现并存混乱 | W-G13 前用 `MARQDO_WEB_PLUGIN` 显式指向；文档写清默认 |

---

## 13. 文档与索引更新清单

实现推进时同步：

- [ ] 本文状态行随波次更新
- [ ] [ext-abi.md](ext-abi.md) Demo/Web 节：实现语言 Go
- [ ] [ext-cli.md](ext-cli.md)：构建回退含 `go build`
- [ ] [ext-web.md](ext-web.md)：链到本文；C5 落实说明
- [ ] [web-asgi-servers-and-marqdo.md](web-asgi-servers-and-marqdo.md)：axum → Go `net/http`
- [ ] [doc/README.md](../README.md) 索引行
- [ ] ADR：`doc/adr/0004-web-plugin-go.md`（Accepted）
- [ ] `plugins/web/README.md` 开发者构建说明

---

## 14. 决议摘要

1. **全量重写** `plugins/web` 为 Go，**不是**最小闭环。  
2. **既有网络拓展能力（§5）重写后必须仍可用**；作者面默认兼容。  
3. **PostgreSQL / Redis 外接** 为一等公民；Marqdo 承担应用前后端脚本。  
4. **暗恋见君** 为完备性验收，不是缩小插件范围的借口。  
5. **多语言 ABI 生态** 以本插件为样板（G7）。  
6. 开发严格按 **§10 波次** 推进；本文件为唯一施工图。
