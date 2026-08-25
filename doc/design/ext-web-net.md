# 网络扩展波次：`ext/web` 鉴权 · WebSocket · 标准库网络原语

| | |
|---|---|
| 状态 | **Draft（本波次锁定设计）** |
| 日期 | 2026-08-24 |
| 相关 | [ext-web.md](ext-web.md) · [ext-abi.md](ext-abi.md) · [stdlib-modules.md](stdlib-modules.md) · [ext-cli.md](ext-cli.md) |
| 目标 | 补齐 `ext/web` 登录鉴权（session/cookie）、WebSocket 端点；为网络扩展所需的**基础设施解析原语**适当补强 `lib/net`（标准库）。 |

---

## 0. 一句话

**领域能力（鉴权、WebSocket 服务器）全部走 ABI 插件 `plugins/web`；纯解析基础设施（cookie / multipart 解析、WS 客户端）按需进入标准库 `lib/net`，因为标准库需要支撑扩展库开发。**

---

## 1. 边界判定：什么进标准库 `lib/net`，什么进扩展

### 1.1 跨语言调研结论

| 语言 | 标准库 | 扩展/生态 | 边界准则 |
|------|--------|----------|---------|
| Python | `urllib`（URL/基础 HTTP）、`http.cookies`（**cookie 解析器在标准库**） | `requests`/`httpx`（session/连接池/便利层） | 标准库 = 基础设施原语（解析器）；便利层 = 第三方 |
| Node.js | `http`/`https`/`http2`（低层流式，不解析 body） | express、ws、formidable | 足够 fancy 就不进核心；核心只放几乎人人要用的低层原语；**先 userland 验证再拉进 core** |
| Go | `net/http`（服务器+客户端都在标准库）、`net/http/cookiejar`（**RFC 6265 cookie jar 在标准库**） | `golang.org/x/net`（publicsuffix 等演进性） | HTTP 是「与 stdin 同级」的核心能力；**cookie 解析/管理进标准库**；持久化等演进性能力在外 |
| Rust | 极小 std（无 HTTP） | `http`（词汇）→ `hyper`（传输）→ `reqwest`（便利） | 分层；HTTP 因生态多样不进 std |

**共识**：
1. **cookie 解析/管理（RFC 6265）**在 Python/Go 都是**标准库**（`http.cookies`、`net/http/cookiejar`）。
2. **multipart/form-data 解析**在 Go 是标准库（`net/http`），在 Python 是第三方 `multipart`。
3. **WebSocket** 几乎总在扩展/第三方（Node `ws`、Python `websockets`、Go `gorilla/websocket`），**很少进语言标准库**。
4. **session 是领域级便利**，几乎所有语言都在框架层（express-session、Django session），不在语言标准库。

### 1.2 Marqdo 判定（锁定）

| 能力 | 类型 | 归属 | 理由 |
|------|------|------|------|
| `cookie.parse` / `cookie.parse_response` | 基础设施解析 | **`lib/net`（标准库）** | 对齐 Go/Python 标准库；`ext/web` 鉴权依赖 |
| `multipart.parse` | 基础设施解析 | **`lib/net`（标准库）** | 对齐 Go 标准库；`ext/web` 文件上传依赖 |
| `ws.connect`（客户端） | 基础连接 | **`plugins/web`（ABI）+ `ext/web`** | WS 栈（tokio-tungstenite）已在 web 插件依赖链；核心 `Cargo.toml` 锁定 rustc 1.81 不宜再引入 WS 栈；符合「尽量只 ABI」 |
| `web.session_*`（session 管理） | 领域 | **`plugins/web`（ABI）+ `ext/web`** | session 是框架层便利，不进标准库 |
| `/admin` 登录鉴权 | 领域 | **`plugins/web`（ABI）+ `ext/web`** | 网站领域能力 |
| `web_ws`（服务器端点） | 领域 | **`plugins/web`（ABI）+ `ext/web`** | WebSocket 服务器是网站能力 |

**实现约束（用户要求）**：
- 标准库新增的**解析原语**（cookie/multipart）是纯函数、无网络 I/O，作为 L0.5 宿主原语进 `src/host/`（与 `url_encode` 同模式），由 `lib/net` 薄包装——这符合「标准库需要支持扩展库开发」。
- **涉及网络 I/O 的 WS 能力**（客户端 + 服务器端点）一律进 `plugins/web`（ABI v2），由 `ext/web` / `ext/web/网页.mq.md` 包装。核心运行时（rustc 1.81 锁定）**不再新增** WS 栈依赖。
- **网站领域能力**（session、鉴权中间件）一律进 `plugins/web`（ABI v2），由 `ext/web` 包装。
- **禁止**：为这些能力修改核心语法；禁止在 `src/host/` 塞领域逻辑。

---

## 2. 标准库 `lib/net` 新增（基础设施原语）

### 2.1 `cookie.parse` / `解析Cookie`
- 形参：`text`（`Cookie:` 请求头或 `Set-Cookie:` 响应头）
- 结果：`[{name, value, path, domain, expires, max_age, secure, http_only, same_site}]`
- 行为：RFC 6265 解析；`Set-Cookie` 多 cookie 分条；无属性则字段为空。

**英文（`lib/net.mq.md`）：**
```markdown
## cookie_parse
    + `text`

**> host_cookie_parse text=`text`**
```

**中文（`lib/网络.mq.md`）：**
```markdown
## 解析Cookie
    + `内容`

**> host_cookie_parse text=`内容`**
```

### 2.2 `multipart.parse` / `解析多部分`
- 形参：`body`（文本）、`boundary`
- 结果：`[{name, filename, content_type, value}]`（`value` 为字段值；文件字段用 `filename` + `content_type` + `value`（base64 或文本））
- 行为：解析 `multipart/form-data` 正文；字段名 → value；文件字段带 filename。

**英文：**
```markdown
## multipart_parse
    + `body`
    + `boundary`

**> host_multipart_parse body=`body` boundary=`boundary`**
```

**中文：**
```markdown
## 解析多部分
    + `正文`
    + `边界`

**> host_multipart_parse body=`正文` boundary=`边界`**
```

### 2.3 WebSocket 客户端 — 见 §3.4（`plugins/web` ABI）

WS 客户端为领域能力，放 `plugins/web`（ABI）而非标准库，理由见 §1.2。`lib/net` 不新增 WS 客户端包装，避免标准库依赖 ext 插件。

---

## 3. `plugins/web`（ABI v2）新增：领域能力

### 3.1 Session 存储（`web_session_*`）
插件进程内 `static` 内存存储（`HashMap<session_id, HashMap<String,Value>>`），与 `web_listen` 同生命周期。

| ABI 名 | 形参 | 结果 |
|--------|------|------|
| `web_session_new` | 可选 `ttl_sec` | `{id}`（新 session id） |
| `web_session_set` | `id`, `key`, `value` | `{ok:true}` |
| `web_session_get` | `id`, `key` | 值或 `{ok:false}` |
| `web_session_del` | `id`, `key` | `{ok:true}` |
| `web_session_destroy` | `id` | `{ok:true}` |

### 3.2 `/admin` 登录鉴权（`web_auth_*`）
- 作者面：`web.auth` 类（`网页.鉴权`），形参 `admin_users`（`|用户名|密码|` 表）、可选 `session_ttl`。
- `listen` 时若配置了 auth：`/admin*` 未登录 → 跳转 `/admin/login`；登录页 `POST` → 校验 `admin_users` 表 → 写 session cookie → 重定向。
- 登出：`/admin/logout` → 销毁 session。

| ABI 名 | 形参 | 结果 |
|--------|------|------|
| `web_auth_login` | `username`, `password`, `users`, `session_ttl` | `{ok:true, session_id}` 或 `{ok:false}` |
| `web_auth_check` | `session_id`, `users` | `{ok:true}` 或 `{ok:false}` |
| `web_auth_logout` | `session_id` | `{ok:true}` |

### 3.3 WebSocket 服务器端点（`web_ws_route`）
- 作者面：`app.route_ws` / `应用.路由实时`，形参 `path`、可选 `echo=True`。
- `listen` 时 `{path}` 升级为 WS：`echo=True` 回发原文；否则接收并丢弃（扩展点）。

| ABI 名 | 形参 | 结果 |
|--------|------|------|
| `web_app_route_ws` | `app`, `path`, `echo` | `{app}`（登记 WS 端点） |

### 3.4 WebSocket 客户端（`web_ws_connect`）
- 作者面：`web.ws` / `网页.实时`，形参 `url`、`message`、可选 `headers`、`timeout_sec`。
- 行为：客户端连接一次 WS，发 `message`，收**全部**服务器回文后关闭。**单请求-响应式**（非长驻），与 Marqdo 同步模型一致。

| ABI 名 | 形参 | 结果 |
|--------|------|------|
| `web_ws_connect` | `url`, `message`, `headers`（map）, `timeout_sec` | `{ok:true, messages:[…]}` 或 `{ok:false, error}` |

**英文（`ext/web/web.mq.md`）：**
```markdown
# ws
    + `timeout_sec`=30

## connect
    + `url`
    + `message`
    + `headers`=None
```

**中文（`ext/web/网页.mq.md`）：**
```markdown
# 实时
    + `超时秒`=30

## 连接
    + `地址`
    + `消息`
    + `headers`=None
```

> `plugins/web` 用 `tokio-tungstenite`（与 axum WS 同栈，版本兼容 rustc 1.81）。

---

## 4. `ext/web` 作者面新增

### 4.1 英文面（`web.mq.md`）

```markdown
# auth
    + `users`            # |用户名|密码| 表
    + `session_ttl`=3600

## login
    + `username`
    + `password`

## check
    + `session_id`

## logout
    + `session_id`
```

`# app` 新增 `## route_ws`：
```markdown
## route_ws
    + `path`
    + `echo`=True
```

### 4.2 中文面（`网页.mq.md`）

```markdown
# 鉴权
    + `用户表`
    + `会话时长`=3600

## 登录
    + `用户名`
    + `密码`

## 校验
    + `会话id`

## 登出
    + `会话id`
```

`# 应用` 新增 `## 路由实时`：
```markdown
## 路由实时
    + `路径`
    + `回显`=真
```

### 4.3 用法示例（英文）

```markdown
# main

`admins` =
| 用户名 | 密码 |
|--------|------|
| admin | secret |

*store = > db.open *
*page = > web.page title="Site" *
*app = > web.app page=`page` db=`store` admin=True host=127.0.0.1 port=18081 *
*app = > `app`.route_ws path="/live" echo=True *
*app = > web.auth users=`admins` *
> `app`.listen

*echo = > web.ws.connect url="ws://127.0.0.1:18081/live" message="hi" *
```

---

## 5. form 完善（现有功能验证 + 补测试）

现状已具备（无需改代码，需验证）：
- `action=update`：hidden `id` + `db.update`（form.rs `from_schema`/`submit` 已实现）
- 校验失败页级回显：`submit_and_respond` → `render_page_ex`（http.rs 已实现）

本波次为 update 流程和页级回显补 gold 测试，确认端到端可用。

---

## 6. 测试与验收

| 测试 | 内容 |
|------|------|
| `tests/ext/web-net-smoke.mq.md` | session new/set/get/del + auth login/check/logout + `route_ws`/`app.auth` 登记 + ws 连接失败路径（离线） |
| `tests/lib/net-cookie.mq.md` | `cookie_parse` 解析 |
| `tests/lib/net-multipart.mq.md` | `multipart_parse` 解析 |
| `tests/ext/web-admin-smoke.mq.md` | form `action=update` 全流程 + 校验失败 `errors` 回显 |
| 现有 gold | 全量回归（树遍历 + 字节码） |

验收：`cargo test --test gold` 全绿（除网络 live），双后端一致。

### 6.1 字典操作约束（作者面）

Marqdo 原生字典操作：GFM 横表**构造**、脚注取元 `变量[^键]` **读取**。`ext/web` 作者面**不使用** `json.get` / `json.set` 做字典读取/改写——所有字段读取一律 `self[^键]`（或实例变量 `[^键]`），避免"用 json 标准库做字典操作"的反模式。`json.parse` / `json.stringify` 仅用于文本 ↔ 结构化数据的序列化转换（如 HTTP body 解析），不用于本地字典读写。

### 6.2 相对路径按入口脚本解析（host_query `entry_dir`）

`static_dir`（如 `public`）与相对 db 路径（如 `sqlite:data/site.db`）此前按**进程 cwd** 解析——从仓库根目录跑 `examples/*/index.mq.md` 时，`/static` 404、db 会建到根目录，这是错误行为。

本波次修复：宿主新增 `host_query("entry_dir")`（返回入口 `.mq.md` 所在目录的绝对路径，回退 cwd），`plugins/web` 的 `web_listen` 将相对 `static_dir`、`db_url_of` 将相对 db 路径统一 `entry_dir().join(...)` 到绝对路径。于是无论从仓库根目录还是示例目录运行，静态资源与数据库都落在**脚本目录**下。绝对路径不受影响；`host_query` 不可用时回退旧 cwd 行为。

---

## 7. 非本波次

- Postgres / Redis / S3 驱动（roadmap W4，另行）
- WS 广播房间、多客户端（先 echo 单连接）
- session 持久化到 SQLite / 文件（先内存）
- 复杂权限 / RBAC