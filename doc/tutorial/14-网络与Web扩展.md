# 第 14 章：网络与 Web 扩展

> **现行作者面（2026-09-28 · ADR 0007）**：Document / Endpoint / Resource —— 见 [ext-web-artifact.md](../design/ext-web-artifact.md)。  
> 下文若仍出现 `compose_*` / `app.configure` / `web.db` 同文件混写，视为**历史示例**；DB → `ext/data`，鉴权 → `ext/security`，实时 → `ext/net`。

Marqdo 的网络能力分两层：**标准库 `lib/net`**（HTTP 客户端、cookie / multipart 解析）与**官方扩展 `ext/web`**（Document/Endpoint facade + 拆分后的 data/security/net）。分层的依据是「代码即文档」。

## 14.1 概览

| 能力 | 位置 | 典型 API |
|------|------|----------|
| HTTP(S) 客户端 | `lib/net` | `http_get` / `http_post` / `http_request` |
| Cookie / multipart | `lib/net` | `cookie_parse` / `multipart_parse` |
| Document / Endpoint | `ext/web` | `type: web\|endpoint` · `web.serve` · `web.render` · `web.inspect` |
| View 装配 | `ext/web` | `|type|slot|value|attrs|style|` → `web.render` |
| 数据库 | `ext/data` | `db` CRUD / txn |
| 表单 | `ext/data` | `form` |
| 鉴权 / RBAC / OIDC | `ext/security` | `auth` · `rbac` · `oidc` |
| WebSocket | `ext/net` | `websocket` |

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

**resp = > net.http_get url="https://api.example.com/status"**
> print text=[status](`resp`)
```

`http_post` 默认以 JSON 提交，可用 `content_type=` / `headers=` 覆盖：

```markdown
**body = > net.http_post url="https://api.example.com/echo" body={"msg":"hi"}**
> print text=`body`
```

> **字典操作**：返回值是字典（如 `resp`），用链接取元 `[键](resp)`，不要用 `json.get`。

### Cookie 解析

`cookie_parse` 把 `Cookie` 请求头或 `Set-Cookie` 响应头解析成列表：

```markdown
**req = > net.cookie_parse text="session=abc123; theme=dark"**
> print text=[name]([1](`req`))     # session
> print text=[value]([1](`req`))    # abc123

**resp = > net.cookie_parse text="id=42; Path=/; HttpOnly; SameSite=Lax" is_response=True**
> print text=[http_only]([1](`resp`))   # True
```

### multipart 解析

`multipart_parse` 解析 `multipart/form-data` 正文（给定 `boundary`）：

```markdown
**parts = > net.multipart_parse body=`body` boundary="----WebKitFormBoundary"**
**field = [1](parts)**
> print text=[name](`field`)
```

## 14.3 网页应用（Artifact · ADR 0007）

站点是一堆 `.mq.md`：**Document**（`type: web`）与 **Endpoint**（`type: endpoint`）。入口调用 `web.serve` 扫描目录并监听。完整设计见 [ext-web-artifact.md](../design/ext-web-artifact.md)；示例 [marqdo-blog](../../examples/marqdo-blog/)。

### Document 首页

```markdown
---
type: web
title: 我的站点
route: /
method: GET
data_source: posts
data_order: -created_at
import web:ext/web/web.mq.md
---

# 我的站点

欢迎。下方卡片列表由 `data_source` 在服务时从数据库加载。
```

### Endpoint

```markdown
---
type: endpoint
method: GET
path: /api/ping
request: none
response: json
---

# main

`out` =

| ok | service |
|----|---------|
| True | demo |

*out*
```

### 服务入口

```markdown
---
import web:ext/web/web.mq.md
import posts:db/posts.mq.md
---

# main

**store = > posts.open url="sqlite:data/site.db"**
> web.serve root="." host="127.0.0.1" port=18081 db=`store` static_dir="public"
```

离线预览可用 View 表 + `web.render`：

```markdown
`nodes` =

| type | slot | value | attrs | style |
|------|------|-------|-------|-------|
| title | title | Marqdo | | title |

**html = > web.render nodes=`nodes` title="preview"**
> print text=`html`
```

### 数据库（`ext/data`）

```markdown
---
import data:ext/data/db.mq.md
---

# main

**store = > data.db url="sqlite:site.db"**
> `store`.init name=articles fields=`字段表`
> `store`.insert table=articles rows=`数据`
**rows = > `store`.select table=articles limit=10**
```

## 14.4 鉴权（`ext/security`）

HTTP 门禁与登录走 `ext/security`（`auth` / `rbac` / `oidc`），再用 `web.use` 挂中间件表；勿再写 `app.configure`。独立脚本可用：

```markdown
---
import rbac:ext/security/rbac.mq.md
---

# main

**gate = > rbac.rbac**
**r = > `gate`.can permissions="desk:access" needed="desk:access"**
> print text=[allowed](`r`)
```

## 14.5 WebSocket（`ext/net`）

实时能力在 `ext/net`（如 `websocket`），与 Document 站点并列导入，不再挂在 `web.app` 的 junk drawer 上。

## 14.6 字典操作：用表格与取元，不用 `json.get`/`json.set`

Marqdo 用 **GFM 表格**构造字典、用 **链接取元 `[键](变量)`** 读取字典——这是语言原生能力，是「代码即文档」的体现。**不要**用 `json.get` / `json.set` 读写本地字典（那只用于文本 ↔ 结构化的序列化，如 `json.parse` / `json.stringify`）。

构造字典（横表，≥2 列 + 1 行）：

```markdown
`配置` =

| 主机 | 端口 |
|------|------|
| 127.0.0.1 | 18081 |
```

读取字典（取元）：

```markdown
> print text=[主机](`配置`)     # 127.0.0.1
> print text=[端口](`配置`)     # 18081
```

读取函数返回的字典字段，同样直接取元：

```markdown
**login = > `auth`.login username="admin" password="secret"**
> print text=[ok](`login`)         # 而不是 json.get value=`login` key="ok"
```

## 14.7 下一步

你已经掌握 Marqdo 的网络与 Web 能力。继续探索：

- [第 13 章：命令行工具](./13-命令行工具.md) —— `marqdo ext` 管理扩展
- 设计文档：[`doc/design/ext-web-net.md`](../design/ext-web-net.md) —— 本波次网络扩展的完整设计与边界判定
