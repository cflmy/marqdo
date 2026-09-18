# ext/web：请求上下文与表单字段来源

| | |
|---|---|
| 状态 | **Accepted** |
| 日期 | 2026-09-18 |
| 相关 | [ext-web.md](ext-web.md) §5.5 · 对照 Django `request.user` / URL kwargs |
| 动机 | 禁止扩展库按业务表名（如 `comments`）特判填字段；身份与路由键属于请求上下文 |

---

## 0. 原则

1. **扩展库提供机制，站点表达领域** — 不得出现 `if table == "comments"`。
2. **身份只从 session 来，且服务端盖戳** — 客户端提交的 `author` 等不可信。
3. **资源键从路由参数来** — 页面已在 `/post/{slug}` 时，表单继承该页 `params`，不解析 Referer 猜 slug。
4. **非 `client` 来源的字段默认不渲染** — 服务端在校验前写入；若 POST 带同名键，一律覆盖丢弃。

---

## 1. 请求上下文（Request）

每次处理 HTTP 请求时，框架逻辑上持有：

| 键 | 含义 |
|----|------|
| `user` / `username` | 登录名；访客为空 |
| `params` | 动态路由捕获（如 `{slug}`） |
| `path` | 请求路径 |
| `method` | GET / POST / … |
| `query` | URL 查询（可选） |

与 Django 对应：`request.user`、`request.resolver_match.kwargs`、`request.path`。

页面渲染时已有 `page.params`（`injectParams`）。表单提交时通过隐藏字段 `_mq_params`（JSON）与 `_mq_return`（回跳路径）把**渲染该表单时的页上下文**带回 POST，避免依赖 Referer。

---

## 2. 字段表「来源」列

字段表增加可选列 `来源` / `source`（缺省 = `client`）：

```markdown
| 字段 | 标签 | 类型 | 必填 | 默认 | 来源 |
|------|------|------|------|------|------|
| body | 写下你的想法 | textarea | true | | client |
| author | | | true | | session.username |
| post_slug | | | true | | route.slug |
| created_at | | | false | | now |
```

| 来源值 | 含义 | 渲染 |
|--------|------|------|
| `client` / 空 | 用户填写 | 正常控件 |
| `session.username` / `session.user` / `user` | 当前登录用户名 | 不渲染 |
| `route.NAME` / `param.NAME` / `params.NAME` | `params[NAME]` | 不渲染 |
| `now` / `utc.now` / `server.now` | UTC `YYYY-MM-DD HH:MM:SS` | 不渲染 |

`类型=hidden` 仍可显式使用；与 `来源≠client` 叠加时仍不依赖客户端值（服务端覆盖）。

---

## 3. 提交管线

```
ParseForm → 去掉来源≠client 的 POST 键
         → 按字段来源从 Request 写入
         → validate → insert/update
         → 回跳：优先 _mq_return（同站路径），否则 form.redirect
```

伪造 `author` / `post_slug` 无效。

---

## 4. 页面列表装配（副绑定）

详情页可附加与主主体**不同查询条件**的列表绑定（如评论）：

```markdown
**post = > post.列表装配 主体=`评论卡片` 条件=`评论条件` 排序="-created_at"**
```

条件值同样支持 `{slug}`。渲染在主文章/卡片区之后，无需为子资源单独写 JSON API + 前端过滤。

---

## 5. 非目标

- 不在扩展库内实现「评论」领域类型。
- 不把 Referer 解析当作正式 params 来源（仅兼容期可作 `_mq_params` 缺失时的弱回退，金样不依赖）。
