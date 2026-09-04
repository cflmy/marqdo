# 解决 2：`SHELL_CSS` 可关 / 可分层

| | |
|---|---|
| 问题 | [研究 §2](../../research/ext-web-customization-limits.md#2-内置-shell_css-难以彻底覆盖) |
| 优先级 | **P1** · 波次 **C2** |
| 触点 | `plugins/web/src/render.rs`（`SHELL_CSS`、`<style>` 拼接）· page/app 元数据 · `ext/web` 样式装配 |

---

## 1. 目标

1. 站点主题可选择：**不要**框架栅格/卡片默认，或只要 reset 级变量。
2. 壳 CSS 与主题 CSS 的级联顺序**可声明、可文档化**。
3. 默认仍注入完整壳（兼容旧站）；新主题显式 `shell_css=off` 即可干净起步。

---

## 2. 作者 API（草案）

```markdown
*应用 = > web.app page=`首页` … shell_css="minimal"*
# 或页面级覆盖
*页 = > `页`.meta …*   # 或 page 选项 shell_css=
```

| 值 | 行为 |
|----|------|
| `full`（默认） | 现网 `SHELL_CSS` 全文 |
| `minimal` | 仅 `:root` 变量 + `body` margin/字体；**不含** `has-sidebar` 双列栅格与卡片表单皮肤 |
| `off` / `none` | 不注入框架壳样式；只输出站点 `style` / 外链 CSS |

级联顺序锁定为文档约定：

1. （可选）框架 `SHELL_CSS`
2. 站点全局 style 表 / `site.css`
3. 页面级 style

禁止再出现「主题写在 extra，却被同特异性壳规则在小屏打穿且无媒体查询」而无开关可关的情况。

---

## 3. 插件改动点

| 改动 | 说明 |
|------|------|
| 拆分 `SHELL_CSS` | `SHELL_VARS` / `SHELL_LAYOUT` / `SHELL_WIDGETS` 三段；`full`=`all`，`minimal`=vars(+极简 layout) |
| `render` 读 `app.shell_css` / `page.shell_css` | 页面覆盖应用 |
| 为 `has-sidebar` 布局加基础媒体查询 | 即使 `full`，小屏改为单列（减轻幽灵双列）；`minimal`/`off` 不负责响应式 |

---

## 4. 兼容策略

- 默认 `full`：视觉与现网一致。
- Tutorial 增加一节「品牌主题请用 `shell_css=minimal|off`」。

---

## 5. 验收

| 场景 | 期望 |
|------|------|
| `shell_css=off` + 站点只写单列 grid | 响应里**无** `grid-template-columns:14rem` |
| `minimal` | 有 CSS 变量；无 `.content.cards article` 边框默认（或等价皮肤段） |
| 默认示例站 | HTML 仍含完整壳 CSS |

---

## 6. 过渡期 workaround

继续用更高特异性 + `!important`；实现后主题应删除这类补丁，并在回归里断言壳字符串不出现。
