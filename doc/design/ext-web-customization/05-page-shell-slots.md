# 解决 5：页面壳与插槽松绑

| | |
|---|---|
| 问题 | [研究 §5](../../research/ext-web-customization-limits.md#5-页面壳与插槽约定僵硬) |
| 优先级 | **P1** · 波次 **C2/C4** |
| 触点 | `plugins/web/src/render.rs`（body class、四区 HTML）· page 槽位装配 · nav 组件表 |

---

## 1. 目标

1. 作者可选**非侧栏栅格**布局，而不必用 CSS 硬怼掉 `has-sidebar`。
2. 侧栏存在 ≠ 强制桌面双列；提供 stacked / bare 等预设。
3. 为「条件导航」留扩展点（先文档 + 简单 hook，完整运行时条件可后置）。

---

## 2. 作者 API（草案）

```markdown
*首页 = > web.page … layout="stacked"*
*应用 = > web.app page=`首页` … layout="bare"*
```

| `layout` | DOM / class | 说明 |
|----------|-------------|------|
| `sidebar`（默认，有 side 槽时） | `body.has-sidebar` | 现网行为 |
| `stacked` | `body.layout-stacked`；side 在 main 上或折叠区 | 单列流式，适合杂志/营销 |
| `bare` | 无 top/side/foot 强制栅格；只渲染 `main`（+ 显式槽） | 登录页、落地页 |
| `rail`（可选） | 保留站点已有 `has-rail` 约定并文档化 | 避免「只有首页有 class」 |

规则：**有 side 组件但 `layout=stacked|bare` 时，不自动加 `has-sidebar`。**

---

## 3. 插件改动点

| 改动 | 说明 |
|------|------|
| `render` 读 `page.layout` / `app.layout` | 决定 body class 与 HTML 骨架 |
| `SHELL_LAYOUT`（见 [02](02-shell-css.md)） | 为 `layout-stacked` / `layout-bare` 提供最小样式；`shell_css=off` 时不注入 |
| Nav 表 | 短期保持声明式；文档说明「条件项用两套 nav 组件 + 媒体查询隐藏」 |
| 中期 hook（非阻断） | `navigate` 已有字段可扩展 `when`/`media` 列（单独 RFC） |

---

## 4. 兼容策略

- 未设 `layout`：有 side → 与现网 `has-sidebar` 相同。
- 金样增加 `layout=bare` 登录页（可与 C0 自研 `/desk/login` 共用）。

---

## 5. 验收

| 场景 | 期望 |
|------|------|
| 有 side + `layout=stacked` | body **无** `has-sidebar`；无 `14rem` 双列（在 full 壳下用 stacked 规则） |
| `layout=bare` | 输出不含 `<aside class="side">`（或 empty 不占栅格） |
| 默认站 | 不变 |

---

## 6. 过渡期 → 已落地

**C2 已落地（2026-09）：** `layout=sidebar|stacked|bare|rail`；有 side 时 `stacked`/`bare` 不再强制 `has-sidebar`。金样同上。
