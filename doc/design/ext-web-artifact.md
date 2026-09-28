# 官方扩展：Web Artifact（Document / Endpoint / Resource）

| | |
|---|---|
| 状态 | **Accepted · 破坏性（取代 page/app God Object 作者面）** |
| 日期 | 2026-09-28 |
| ADR | [0007-web-document-endpoint.md](../adr/0007-web-document-endpoint.md) |
| 讨论稿 | [doc/next/007.md](../next/007.md)（superseded） |
| 安装 | `marqdo ext add web`（`web` / `网页`） |
| 相关 | [binding.md](binding.md) · [ext-web.md](ext-web.md) · [stdlib-i18n.md](stdlib-i18n.md) · [layers.md](layers.md) |

---

## 0. 一句话

> **Document describes the page. Table describes the data. Endpoint describes the interaction. Metadata describes the runtime. Marqdo executes all of them.**

---

## 1. 三原语

| Artifact | Metadata `type` / `类型` | 含义 |
|----------|--------------------------|------|
| **Document** | `web` / `网页` | Markdown 正文 + View → HTML；`route`/`路由` 注册 declared HTTP |
| **Endpoint** | `endpoint` / `端点` | 可执行正文为 handler；`path`+`method` 注册 API |
| **Resource** | （绑定/导入） | `ext/data` · `ext/security` · 未来 LLM 等，经 metadata / import 声明 |

核心装载识别 `type`/`类型`（见 `src/artifact/`）；渲染与 listen 在 `ext/web` + `plugins/web`。

---

## 2. 目录

```text
ext/web/
  web.mq.md · 网页.mq.md     # facade: page route serve render inspect
  page.mq.md · route.mq.md · component.mq.md · dom.mq.md · client.mq.md
ext/net/     http.mq.md · websocket.mq.md · url.mq.md
ext/data/    db.mq.md · cache.mq.md · storage.mq.md
ext/security/ auth.mq.md · oidc.mq.md · rbac.mq.md
```

作者入口通常：

```text
site/
  index.mq.md          # type: web · route: / · 或 # main → web.serve
  pages/*.mq.md        # type: web
  api/*.mq.md          # type: endpoint
  data/                # sqlite runtime (gitignore)
```

---

## 3. Document（`type: web`）

```markdown
---
type: web
title: Marqdo Blog
route: /
method: GET
data_source: posts
data_order: -created_at
import web:ext/web/web.mq.md
import data:ext/data/db.mq.md
---

# Marqdo

Welcome to the executable web.
```

| 键 (EN) | 键 (ZH) | 说明 |
|---------|---------|------|
| `type` | `类型` | `web` / `网页` |
| `route` | `路由` | 路径，可含 `{param}` |
| `method` | `方法` | 默认 `GET` |
| `title` | `标题` | 文档标题 / `<title>` |
| `data_source` | `数据源` | 表名 |
| `data_order` | `排序` | 如 `-created_at` |
| `data_where` | `条件` | `col=val,col2=val2` |

正文 Markdown 经 `dom` 变为 DOM Artifact，再 `web.render`。可选 View 表：

```markdown
`nodes` =

| type | slot | value | attrs | style |
|------|------|-------|-------|-------|
| text | kicker | AI Native | | eyebrow |
| title | title | Marqdo | | title |
```

`page.render nodes=` 是**唯一**主装配路径。禁止 `compose_main` / `compose_intro` 等碎片 DSL。

---

## 4. Endpoint（`type: endpoint`）

```markdown
---
type: endpoint
method: POST
path: /api/posts
request: json
response: json
auth: member
import data:ext/data/db.mq.md
---

**post = > data.db.insert …**

*post*
```

| 键 (EN) | 键 (ZH) | 说明 |
|---------|---------|------|
| `type` | `类型` | `endpoint` / `端点` |
| `path` | `路径` | URL |
| `method` | `方法` | GET/POST/… |
| `request` | `请求` | `json` / `form` / `none` |
| `response` | `响应` | `json` / `text` / `html` |
| `auth` | `鉴权` | 角色名或 `none` |

正文在请求时执行；返回值按 `response` 序列化。Result 协议见 §8。

**Handler 约定（锁定）**：Endpoint 文件用 `# main` 作为请求处理器。请求体/query/path 参数经 `run_artifact` 以 Metadata Binding 覆盖注入（与 `--bind` 同源），正文用已提升的普通变量名。无 `# main` 时返回 Artifact Result 摘要（`ok`/`value`/`error`），适合作文档型 ping。

---

## 5. Facade API（EN / ZH）

| EN | ZH | 作用 |
|----|-----|------|
| `web.page` | `网页.页面` | 从 metadata/正文/nodes 构造 Document 句柄 |
| `web.route` | `网页.路由` | 注册 declared Document 或 Endpoint |
| `web.serve` | `网页.服务` | `root=` 扫描注册 + listen |
| `web.render` | `网页.渲染` | Document → HTML（离线/预览） |
| `web.inspect` | `网页.检视` | `{declared, system}` 路由表 |

Middleware 一等：`web.use` / `route.use`，表驱动；**禁止** `configure(cors, security, …)` 垃圾抽屉。

---

## 6. 发现算法（锁定）

`web.serve root=DIR host=… port=…`：

1. 递归枚举 `DIR` 下 `*.mq.md`（跳过 `data/`、隐藏目录、`node_modules`）。
2. 解析文件头 Metadata；无 `type`/`类型` 或非 web/endpoint → 跳过。
3. Document → declared route（缺 `route` 则相对 `root` 的路径映射，`index.mq.md` → `/`）。
4. Endpoint → declared path+method（`path`/`路径` 必需）。
5. System routes（`/_part/*`、`/_form/*`、静态等）由 runtime 挂载，仅出现在 `inspect.system`。
6. 调用 native listen。

---

## 7. Table 规则

- **View**：仅 `|type|slot|value|attrs|style|`（及列名 ZH 别名：`类型|槽|值|属性|样式`）。
- **Resource schema**：`|字段|类型|可空|` 等 — 专用且文档化，不是函数私有方言。
- **禁止** `|属性|值|样式|` / `|front|back|css|` / `|组件|样式|` 作为页面主装配（已删除）。
- Raw HTML/CSS = escape hatch（`dom.raw` / `page.css`），不是默认语言。

---

## 8. Result

所有易失败的 Resource / Endpoint 边界返回 map：

| 键 | 类型 | 说明 |
|----|------|------|
| `ok` | bool | 成功 |
| `value` | any | 成功载荷 |
| `error` | text \| None | 失败信息 |

---

## 9. 硬约束（取代旧 C1–C7 作者面）

| # | 约束 |
|---|------|
| A1 | Document/Endpoint 由核心识别 `type`；装配在 ext（ADR 0007） |
| A2 | 禁止 `json.set` 袋胶水；Table = Data |
| A3 | 中英分文件；同文件禁止混 API |
| A4 | 禁止作者 `ensure_plugin` / `host_*` |
| A5 | 禁止隐式用户 route；system 须 `inspect` 可见 |
| A6 | 禁止恢复 `compose_*` / `app.configure` / god page 变异堆 |
| A7 | 能力只许迁移/提升；删除须本设计明示 + tests 同步 |
| A8 | 设计未锁定处停工提问，禁止自行发明 DSL |

---

## 10. 与旧 ext-web.md

旧「页面表 + compose_* + app.route」作者面 **废止**。历史能力矩阵见 [web-net-capabilities.md](web-net-capabilities.md)；迁移后以本文件与 ADR 0007 为准。
