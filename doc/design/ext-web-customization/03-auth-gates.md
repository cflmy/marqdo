# 解决 3：鉴权 / 门禁可配置且行为一致

| | |
|---|---|
| 问题 | [研究 §3](../../research/ext-web-customization-limits.md#3-鉴权--门禁模型偏固定) |
| 优先级 | **P0** · 波次 **C1**（依赖 [01](01-routes-and-admin-prefix.md) 的 prefix） |
| 触点 | `plugins/web/src/http.rs`（`rbac_middleware`、`path_matches`、login 表单）· `lib.rs`（`web_app_auth` / `web_app_gate`）· `ext/web` |

---

## 1. 目标

1. **未登录策略一致**：可配置 `redirect`（303 登录）或 `forbid`（403），对任意 gate 路径生效，不再「仅 `/admin*` 会跳登录」。
2. **匹配语义清晰**：支持精确路径、前缀、以及排除列表；`/desk` 默认**不**误伤 `/desk/login`。
3. 默认 admin gate 使用 **路径段边界**（`/admin` 与 `/admin/…`），不匹配 `/admin-publish`。
4. 登录 URL、表单 `action`、成功/失败回跳均可配置（与 C0 对齐）。

---

## 2. 作者 API（草案）

```markdown
*应用 = > `应用`.auth users=`用户表` login_path="/desk/login" login_redirect="/desk"*
*应用 = > `应用`.gate path="/desk" roles="admin" match="prefix" on_deny="redirect" exclude="/desk/login"*
```

| 字段 | 说明 |
|------|------|
| `login_path` | 登录页 URL（GET 表单 + POST 处理）；默认 `{admin_prefix}/login` |
| `match` | `exact` \| `prefix`（默认对自定义 gate 建议 `prefix` + `exclude`） |
| `on_deny` | `redirect` \| `forbid`；`redirect` 时跳 `login_path`（可带 `?next=`） |
| `exclude` | 逗号分隔或表列：匹配前缀时放行的路径 |

`auth` 注册默认 gate 时：

- 使用 `{admin_prefix}` + 段边界，**不要** `starts_with("/admin")` 裸前缀。
- 自动把 `login_path` 加入放行列表。

---

## 3. 插件改动点

| 改动 | 说明 |
|------|------|
| `path_matches` | 增加 `exact`；`prefix` 要求下一字符为 `/` 或结尾；可选 exclude |
| `rbac_middleware` | 按 gate 的 `on_deny` 分支；通用放行 `login_path`，不写死 `/admin/login` |
| 登录 HTML | `action={login_path}`；hidden `next` 可选 |
| Session/CSRF | 短期仍用框架实现；文档标明字段名；中期再开放 `login_form` 自定义字段映射（非本波次必做） |

---

## 4. 兼容策略

- 未传新字段：行为对齐现网（`/admin*` → redirect 登录；其它自定义 gate → 403）。
- 新增金样：`/desk` gate + `/desk/login` 可匿名打开。

---

## 5. 验收

| 场景 | 期望 |
|------|------|
| visitor 访问 `/desk`（`on_deny=redirect`） | 303 → `/desk/login`，不是 403 |
| visitor 访问 `/desk/login`（exclude） | 200 登录页 |
| `/admin-publish` 无显式 gate | **不**被默认 admin gate 拦住 |
| 旧站仅 `app.auth` | 仍保护 `/admin*` |

---

## 6. 过渡期 workaround

JS `fetch` 打 `/admin/login` 再 `location.replace('/desk')`；C0+C1 落地后改为同前缀登录并删除 JS 兜底。
