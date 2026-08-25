# 第 14 章：网络与 Web 扩展

Marqdo 的网络能力分两层：**标准库 `lib/net`**（HTTP 客户端、cookie / multipart 解析）与**官方扩展 `ext/web`**（网页应用、数据库、表单、登录鉴权、WebSocket）。分层的依据是「代码即文档」的可读性——基础、通用、低依赖的进标准库；领域性强、依赖重、只在 Web 场景才用的进扩展。

## 14.1 概览

| 能力 | 位置 | 典型函数 |
|------|------|----------|
| HTTP(S) 客户端 | `lib/net` | `http_get` / `http_post` / `http_request` |
| Cookie 解析 | `lib/net` | `cookie_parse` |
| multipart/form-data 解析 | `lib/net` | `multipart_parse` |
| 网页应用 / 路由 / 静态 | `ext/web` | `web.page` / `web.app` |
| SQLite 数据库 | `ext/web` | `web.db` |
| 表单 | `ext/web` | `web.form` |
| 登录鉴权（session） | `ext/web` | `web.auth` / `app.auth` |
| WebSocket 客户端 / 端点 | `ext/web` | `web.ws` / `app.route_ws` |

```bash
# 运行使用 ext/web 的程序前，先安装扩展
marqdo ext add web
```

## 14.2 HTTP 客户端（`lib/net`）

导入 `lib/net.mq.md`（中文 `lib/网络.mq.md`），用 `http_get` / `http_post` 发请求：

```markdown
---
import net:lib/net.mq.md
---

# main

*resp = > net.http_get url="https://api.example.com/status"*
> print text=`resp`[^status]
```

`http_post` 默认以 JSON 提交，可用 `content_type=` / `headers=` 覆盖：

```markdown
*body = > net.http_post url="https://api.example.com/echo" body={"msg":"hi"}*
> print text=`body`
```

> **字典操作**：返回值是字典（如 `resp`），直接取元 `resp[^键]`，不要用 `json.get`。

### Cookie 解析

`cookie_parse` 把 `Cookie` 请求头或 `Set-Cookie` 响应头解析成列表：

```markdown
*req = > net.cookie_parse text="session=abc123; theme=dark"*
> print text=`req`[^1][^name]     # session
> print text=`req`[^1][^value]    # abc123

*resp = > net.cookie_parse text="id=42; Path=/; HttpOnly; SameSite=Lax" is_response=True*
> print text=`resp`[^1][^http_only]   # True
```

### multipart 解析

`multipart_parse` 解析 `multipart/form-data` 正文（给定 `boundary`）：

```markdown
*parts = > net.multipart_parse body=`body` boundary="----WebKitFormBoundary"*
*field = parts[^1]*
> print text=`field`[^name]
```

## 14.3 网页应用（`ext/web`）

`ext/web` 用类 + 方法装配页面。导入 `web:ext/web/web.mq.md`（中文 `网页:ext/web/网页.mq.md`）。

### 页面、组件、主体

```markdown
---
import web:ext/web/web.mq.md
---

# main

`首页` =

| 组件 | 样式 |
|------|------|
| nav.`导航` | shell.`顶栏` |

*page = > web.page title="我的站点" intro="<h1>你好</h1>"*
*page = > `page`.compose_components components=`首页`*
*html = > `page`.render*
> print text=`html`
```

### 应用 + 路由 + 监听

```markdown
*app = > web.app page=`page` host=127.0.0.1 port=18081*
*app = > `app`.route path="/about" page=`关于`*
> web_listen app=`app`
```

`listen` 提供 `/`、路由页、表单端点 `/_form/{id}`、可选静态目录与 `/admin`。

### 数据库（SQLite）

```markdown
*store = > web.db url="sqlite:site.db"*
> `store`.init name=articles fields=`字段表`
> `store`.insert table=articles rows=`数据`
*rows = > `store`.select table=articles limit=10*
- [行](rows)
  > print text=`行`[^title]
```

## 14.4 登录鉴权（session / cookie）

给 `/admin*` 加登录门禁：`app.auth` 挂用户表，未登录重定向到 `/admin/login`。

```markdown
`管理员` =

| 行 | 用户名 | 密码 |
|----|--------|------|
| 1 | admin | secret |
| 2 | 站长 | pw123 |

*app = > web.app page=`page` admin=True*
*app = > `app`.auth users=`管理员` session_ttl=3600*
> web_listen app=`app`
```

`session_ttl` 是会话有效期（秒），过期后重新登录。

**独立鉴权工具** `web.auth`：登录、校验、登出（不依赖页面），返回 `{ok, session_id, username}`：

```markdown
*auth = > web.auth users=`管理员` session_ttl=3600*
*login = > `auth`.login username="admin" password="secret"*
1. `login`[^ok]
  > print text=登录成功：`login`[^username]
2. *
  > print text=登录失败

*sid = login[^session_id]*
*check = > `auth`.check session_id=`sid`*
> print text=`check`[^username]

> `auth`.logout session_id=`sid`
```

## 14.5 WebSocket

`app.route_ws` 注册端点，`web.ws.connect` 单次请求–响应：

```markdown
*app = > `app`.route_ws path="/live" echo=True*
> web_listen app=`app`
```

客户端（另开终端）：

```markdown
*ws = > web.ws timeout_sec=30*
*echo = > `ws`.connect url="ws://127.0.0.1:18081/live" message="hi"*
1. `echo`[^ok]
  > print text=`echo`[^messages]
2. *
  > print text=连接失败：`echo`[^error]
```

## 14.6 字典操作：用表格与取元，不用 `json.get`/`json.set`

Marqdo 用 **GFM 表格**构造字典、用 **脚注取元 `变量[^键]`** 读取字典——这是语言原生能力，是「代码即文档」的体现。**不要**用 `json.get` / `json.set` 读写本地字典（那只用于文本 ↔ 结构化的序列化，如 `json.parse` / `json.stringify`）。

构造字典（横表，≥2 列 + 1 行）：

```markdown
`配置` =

| 主机 | 端口 |
|------|------|
| 127.0.0.1 | 18081 |
```

读取字典（取元）：

```markdown
> print text=`配置`[^主机]     # 127.0.0.1
> print text=`配置`[^端口]     # 18081
```

读取函数返回的字典字段，同样直接取元：

```markdown
*login = > `auth`.login username="admin" password="secret"*
> print text=`login`[^ok]         # 而不是 json.get value=`login` key="ok"
```

## 14.7 下一步

你已经掌握 Marqdo 的网络与 Web 能力。继续探索：

- [第 13 章：命令行工具](./13-命令行工具.md) —— `marqdo ext` 管理扩展
- 设计文档：[`doc/design/ext-web-net.md`](../design/ext-web-net.md) —— 本波次网络扩展的完整设计与边界判定
