# `ext/web` 定制限制：开发者不易定制的问题盘点

| | |
|---|---|
| 状态 | **调研锁定（踩坑整理）** |
| 日期 | 2026-09-04 |
| 相关 | [ext-web.md](../design/ext-web.md) · [ext-web-net.md](../design/ext-web-net.md) · [web-net-capabilities.md](../design/web-net-capabilities.md) |
| 解决索引 | [../design/ext-web-customization/](../design/ext-web-customization/) |

结合自研后台、样式迁移与响应式改造中的实际踩坑，整理当前 `plugins/web` + `ext/web` 对站点作者的**强占与难定制点**。本文只陈述问题与证据；逐项解法见解决索引。

---

## 0. 一句话

内置 `/admin` 壳、固定鉴权路径、注入式 `SHELL_CSS` 与四区栅格，让「只写 `.mq.md`」的产品路径在**自研后台 / 品牌主题 / 移动端**上反复撞墙；作者被迫旁路前缀 + `!important` + 前端 JS 兜底。

---

## 1. 路由与路径被框架强占

**现象**

- `/admin` 及 `/admin/*` 整段保留，无法 `app.route` 注册自定义页（错误：`route path … is reserved`）。
- `app.重定向` / `redirect` 目标若落在 reserved 前缀，同样被校验拒绝。
- `后台=False`（`admin=False`）后，访问 `/admin/news` 等仍进内置壳，只显示 *Admin is disabled*，**不能**换成自研页。
- 登录/登出硬绑 `/admin/login`、`/admin/logout`；登录成功硬编码 `Redirect::to("/admin")`，无法配置回跳到 `/desk`。

**代码证据**

| 位置 | 行为 |
|------|------|
| `plugins/web/src/lib.rs` → `normalize_route_path` | `/admin`、`/_form`、`/_part`、`/static` 一律 reserved |
| `plugins/web/src/http.rs` | 始终挂 `/admin…` 路由；disabled 时仍 `admin_shell` + 文案 |
| 同文件 `admin_login_post` | 成功跳转写死 `/admin` |

**结果**

自研后台只能用 `/desk` 等旁路前缀；入口与鉴权路径分裂，还要靠前端 JS（`fetch('/admin/login')` 再 `replace('/desk')`）做跳转兜底。

→ 解决：[01-routes-and-admin-prefix.md](../design/ext-web-customization/01-routes-and-admin-prefix.md)

---

## 2. 内置 `SHELL_CSS` 难以彻底覆盖

**现象**

- 每个页面都会注入框架壳样式，例如 `body.has-sidebar { grid-template-columns:14rem 1fr; … }`、默认卡片/表单/侧栏色。
- 规则**无媒体查询**、选择器优先级不低；站点主题选择器不够强时，小屏双列、侧栏占位等「幽灵布局」反复出现。
- 首页有额外 class（如 `has-rail`）时碰巧盖住，其他页没有同类 class 就暴露——表现为「只修了首页」。
- 实务上经常要 `!important` + 更高选择器，与「干净主题层」冲突。

**代码证据**

| 位置 | 行为 |
|------|------|
| `plugins/web/src/render.rs` → `SHELL_CSS` | 常量内联进每个 HTML |
| 同文件壳渲染 | `<style>{SHELL_CSS}{extra}</style>`，主题只能接在后面 |

→ 解决：[02-shell-css.md](../design/ext-web-customization/02-shell-css.md)

---

## 3. 鉴权 / 门禁模型偏固定

**现象**

- 未登录访问受保护路径：对 `/admin*` 会 **303 → 登录页**；对 `/desk` 等自定义路径往往是 **403 HTML**，行为不一致。
- 门禁匹配偏宽：`path.starts_with` / `path_matches` 使 `/desk` 匹配 `/desk/login`，公开登录页易被误伤。
- 默认鉴权挂上 `/admin*` → `admin`；`/admin-publish` 这类名字也会被当成 admin 域（`starts_with("/admin")`）。
- 会话 cookie、CSRF、登录表单字段均固定在框架实现；难换登录 UI 或改成功落地页。

**代码证据**

| 位置 | 行为 |
|------|------|
| `http.rs` → `rbac_middleware` | visitor + `/admin*` → redirect；其它 gate → 403 |
| `http.rs` → `path_matches` | 前缀匹配，无「精确路径 + 子路径排除」DSL |
| `lib.rs` → `web_app_auth` | 默认塞入 `/admin*` gate |

→ 解决：[03-auth-gates.md](../design/ext-web-customization/03-auth-gates.md)

---

## 4. 样式作者体验受限

**现象**

- 样式表是 GFM 表格；含 `/` 的值必须加引号（如 `"1 / 5"`），否则单元格表达式把 `/` 当除法（曾出现 `grid-column` 变成 `0`）。
- `@keyframes`、复杂选择器、多层媒体查询（`min-width` and `max-width`）表达能力有限，装配顺序要小心。
- 主题 CSS 与 `SHELL_CSS` 同页串联，级联顺序不透明，调试成本高。
- 响应式不能指望框架；壳层几乎不提供移动端抽屉/折叠。

→ 解决：[04-stylesheet-authoring.md](../design/ext-web-customization/04-stylesheet-authoring.md)

---

## 5. 页面壳与插槽约定僵硬

**现象**

- 壳结构固定为 `topnav` / `side` / `main` / `foot` 栅格命名区。
- 侧栏一旦装配，`body` 即 `has-sidebar`，套上框架双列；杂志风单列、抽屉必须自己拆布局。
- 组件表驱动导航，难做运行时条件导航（小屏合并顶栏+侧栏），通常再补 `theme.js`。

→ 解决：[05-page-shell-slots.md](../design/ext-web-customization/05-page-shell-slots.md)

---

## 6. 后台 CRUD 与表单能力边界

**现象**

- 内置 admin 是通用表管理，不是产品级 CMS；**关了又占着 `/admin` 路径**（与 §1 叠加）。
- 表单以声明式字段表为主；Markdown 分栏、专栏元数据联动等只能外挂 JS。
- 列表「链接前缀」等约定简单；编辑/新建状态机常靠前端拼（`?id=`、`?new=1`）。

→ 解决：[06-admin-crud-forms.md](../design/ext-web-customization/06-admin-crud-forms.md)

---

## 7. 静态资源与脚本生命周期

**现象**

- 头资源脚本常在 `<head>` **同步**加载；若脚本假设 DOM 已就绪会直接失败（登录页曾因此不跳转）。
- 缓存靠 `?v=` 手动 bump；无构建管线时易出现「改了 JS 浏览器仍用旧文件」。

→ 解决：[07-assets-script-lifecycle.md](../design/ext-web-customization/07-assets-script-lifecycle.md)

---

## 8. 扩展与部署耦合

**现象**

- 能力依赖 web 插件 ABI / Marqdo 版本（如 0.3.2 起的样式引号约定）；升级要同步 Docker / 本机二进制。
- 定制若依赖改插件源码（放宽 `/admin` reserved、可配置 login redirect），就脱离「只写 `.mq.md`」的产品路径，维护成本陡增。

→ 解决：[08-ext-deploy-coupling.md](../design/ext-web-customization/08-ext-deploy-coupling.md)

---

## 9. 优先级建议（给实现排期）

| 优先级 | 项 | 理由 |
|--------|-----|------|
| P0 | §1 路由/前缀 + §3 鉴权一致 | 阻塞自研后台产品路径 |
| P1 | §2 SHELL_CSS + §5 壳布局 | 阻塞品牌主题与响应式 |
| P2 | §4 样式作者体验 + §7 脚本生命周期 | 降低日常踩坑，不改架构也能缓解 |
| P3 | §6 CRUD 边界 + §8 部署耦合 | 文档/约定/可选增强为主 |

详细验收与实现切片见解决索引 README。
