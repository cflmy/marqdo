# 解决 1：路由与 `/admin` 前缀可让出

| | |
|---|---|
| 问题 | [研究 §1](../../research/ext-web-customization-limits.md#1-路由与路径被框架强占) |
| 优先级 | **P0** · 波次 **C0** |
| 触点 | `plugins/web/src/lib.rs`（`normalize_route_path`、`web_app_new`、redirect）· `plugins/web/src/http.rs`（admin 路由挂载、login redirect）· `ext/web/web.mq.md` / `网页.mq.md` |

---

## 1. 目标

1. 关闭内置后台后，**整段前缀可被站点路由占用**（含自研 `/admin`）。
2. 开启内置后台时，前缀**可配置**（默认仍 `/admin`），登录成功/登出落地页可配置。
3. `app.route` / `app.redirect` 只禁止**当前生效**的框架保留段，不再永久霸占 `/admin`。

---

## 2. 作者 API（草案）

```markdown
*应用 = > web.app page=`首页` db=`库` admin=False*

# 或：仍用内置 CRUD，但挂到旁路前缀 + 自定义回跳
*应用 = > web.app page=`首页` db=`库` admin=True*
*应用 = > `应用`.auth users=`用户表` admin_prefix="/desk" login_redirect="/desk" logout_redirect="/desk/login"*
```

| 参数 | 默认 | 含义 |
|------|------|------|
| `admin` / `后台` | `False` | `False`：**不注册**任何内置 admin HTTP 路由 |
| `admin_prefix` / `后台前缀` | `"/admin"` | 内置 CRUD + 登录页挂载根；仅 `admin=True` 时生效 |
| `login_redirect` / `登录回跳` | `{admin_prefix}` | 登录成功 303 目标 |
| `logout_redirect` / `登出回跳` | `{admin_prefix}/login` | 登出后目标 |

中文别名与英文一一对应，写进 `网页.mq.md`。

---

## 3. 插件改动点

| 改动 | 说明 |
|------|------|
| `normalize_route_path(app)` | reserved 列表改为：`/_form`、`/_part`、`/static` + **当前** `admin_prefix`（仅当 `admin=True`） |
| `listen` 路由表 | `admin=False` → **不** `.route("/admin…")`；为真则用 `admin_prefix` 拼路径 |
| `admin_login_post` / `admin_logout` | 使用配置的 redirect，禁止写死 `"/admin"` |
| `web_app_redirect` | 校验目标时用同一套动态 reserved，允许指向已让出的 `/admin` |
| disabled 文案路径 | 删除「挂路由但只显示 Admin is disabled」；无路由则 404 或落入站点自定义页 |

---

## 4. 兼容策略

- 缺省参数下：现有 `examples/web-site`、`web-net-site`、`marqdo-blog` 行为不变（仍 `/admin` + 跳 `/admin`）。
- 文档明确：`admin=False` 后若仍要登录能力，用 §3 的通用 `auth`/`gate`，不必再依赖内置 admin 壳。

---

## 5. 验收

| 金样 / 手工 | 期望 |
|-------------|------|
| `admin=False` + `route path="/admin/news"` | 注册成功；GET 渲染站点页，**不是** Admin is disabled |
| `admin_prefix="/desk"` | 内置登录在 `/desk/login`；成功跳 `login_redirect` |
| 默认 `admin=True` 无新参数 | 与现网一致 |
| `redirect` 到已让出的 `/admin` | 允许 |

---

## 6. 过渡期 → 已落地

~~继续用 `/desk` 旁路 + JS 调 `/admin/login`。~~

**C0 已落地（2026-09）：** `admin=False` 可直接 `route path="/admin/…"`；或 `admin=True` + `admin_prefix="/desk"` + `login_redirect` / `logout_redirect`。见金样 `tests/ext/web-c0-admin-prefix.mq.md`。
