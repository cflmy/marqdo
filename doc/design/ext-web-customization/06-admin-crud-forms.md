# 解决 6：后台 CRUD 与表单能力边界

| | |
|---|---|
| 问题 | [研究 §6](../../research/ext-web-customization-limits.md#6-后台-crud-与表单能力边界) |
| 优先级 | **P3** · 随 C0 文档化；增强可选 |
| 触点 | `plugins/web/src/http.rs` / `form.rs`（内置 admin）· 站点自研页 + `db.*` · `/_form` |

---

## 1. 目标

1. **产品叙事澄清**：内置 `/admin`（或可配置前缀）= 通用表浏览器，**不是** CMS；关则彻底让路（见 [01](01-routes-and-admin-prefix.md)）。
2. 自研后台的推荐路径写清楚：`page` + `form` + `db` + gate，而不是改插件壳。
3. 对「链接前缀 / new / edit」给出约定或薄辅助，减少前端状态机拼装。

---

## 2. 作者 API / 文档（草案）

### 2.1 推荐架构（文档为主）

| 需求 | 做法 |
|------|------|
| 产品级编辑器、专栏联动 | 自研 `# page` + 组件；写库用 `db.insert`/`update`；鉴权用 `gate` |
| 简单表维护 | 开 `admin=True`（可换前缀） |
| 关内置仍要 CRUD | 自研列表/编辑页；**不要**依赖 disabled 壳 |

### 2.2 可选薄增强（非 CMS）

| 能力 | 说明 |
|------|------|
| `form` 支持 `mode=new\|edit` 查询参数约定 | 文档 + 示例；必要时 `web.form` 读 `id`/`new` |
| `list` 组件 `link_prefix` | 已有则补示例；缺失则加一列「详情路径模板」 |
| Markdown 字段 | 继续「存库 + 前台渲染」；分栏编辑器保持外挂 JS（明确不做进 ABI） |

---

## 3. 插件改动点

| 改动 | 优先级 |
|------|--------|
| C0：disabled 不占路径 | 必需（属 [01](01-routes-and-admin-prefix.md)） |
| 示例 `examples/web-custom-desk/` | 展示自研列表/编辑，零内置 admin |
| form 查询参数助手 | 可选 |

---

## 4. 兼容策略

- 不扩展内置 admin 成 CMS（避免范围爆炸）。
- Tutorial 增加「何时用内置 admin / 何时自研」。

---

## 5. 验收

| 场景 | 期望 |
|------|------|
| 文档 + 示例 desk | 无 `/admin` 也能完成一文增删改 |
| 内置 admin 金样 | 仍绿 |

---

## 6. 过渡期 workaround

自研 `/desk` + 调用 `/_form` 或直接 DB API；复杂编辑器继续外挂 JS。
