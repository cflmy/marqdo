# `ext/web` 定制修复：解决文档索引

| | |
|---|---|
| 状态 | **设计草案（待排期实现）** |
| 日期 | 2026-09-04 |
| 问题盘点 | [../../research/ext-web-customization-limits.md](../../research/ext-web-customization-limits.md) |
| 相关 | [../ext-web.md](../ext-web.md) · [../ext-web-net.md](../ext-web-net.md) · [../../roadmap/ext-web.md](../../roadmap/ext-web.md) |

对应研究文 §1–§8，每项一份解决文档：**目标 → 作者 API → 插件改动点 → 兼容策略 → 验收**。

实现时优先 **不破坏现有金样**（`tests/ext/web-*`、`examples/web-*`）；新能力用显式 opt-in 参数打开。

---

## 文档一览

| # | 主题 | 文档 | 建议优先级 |
|---|------|------|------------|
| 1 | 路由与 `/admin` 前缀可让出 | [01-routes-and-admin-prefix.md](01-routes-and-admin-prefix.md) | P0 |
| 2 | `SHELL_CSS` 可关 / 可层 | [02-shell-css.md](02-shell-css.md) | P1 |
| 3 | 鉴权门禁可配置且行为一致 | [03-auth-gates.md](03-auth-gates.md) | P0 |
| 4 | 样式表作者体验 | [04-stylesheet-authoring.md](04-stylesheet-authoring.md) | P2 |
| 5 | 页面壳与插槽松绑 | [05-page-shell-slots.md](05-page-shell-slots.md) | P1 |
| 6 | 后台 CRUD / 表单边界 | [06-admin-crud-forms.md](06-admin-crud-forms.md) | P3 |
| 7 | 静态资源与脚本生命周期 | [07-assets-script-lifecycle.md](07-assets-script-lifecycle.md) | P2 |
| 8 | 扩展与部署耦合 | [08-ext-deploy-coupling.md](08-ext-deploy-coupling.md) | P3 |

---

## 波次建议（实现用）

| 波次 | 内容 | 依赖 |
|------|------|------|
| **C0** | 可配置 `admin_prefix`；`admin=False` 时不挂内置路由；`login_redirect` / `logout_redirect` | — |
| **C1** | gate：统一未登录策略；精确/前缀匹配；登录页自动放行；默认 gate 不再误伤 `/admin-*` | C0 |
| **C2** | `shell_css=off\|minimal\|full`；主题在壳前或隔离层；可选 `layout=bare` | — |
| **C3** | 样式：文档 + 校验警告；可选 raw CSS 块；脚本 `defer`/`type=module` 默认 | — |
| **C4** | 壳：`layout` 预设（sidebar / stacked / bare）；条件导航钩子（后续） | C2 |

路线图入口可在 [ext-web.md](../../roadmap/ext-web.md) 增补「定制波次 C0–C4」行，落地后再改状态。

---

## 产品原则（解法共同约束）

1. **`.mq.md` 仍是真相源** — 能声明的不进隐藏默认；新开关写在 `app` / `page` / `listen` 参数表。
2. **内置 admin 是可选产品**，不是不可替换的内核路由命名空间。
3. **默认行为保持兼容** — 现有 tutorial / 示例零改动仍能跑；新站显式打开定制。
4. **禁止**为单站需求永久硬编码第二套旁路（如永远推荐 `/desk` + JS fetch 登录）。
