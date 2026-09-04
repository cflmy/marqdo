# 解决 4：样式表作者体验

| | |
|---|---|
| 问题 | [研究 §4](../../research/ext-web-customization-limits.md#4-样式作者体验受限) |
| 优先级 | **P2** · 波次 **C3** |
| 触点 | 表单元格表达式（T5）· `ext/web` style 装配 · 教程 / Skill · 可选校验 |

---

## 1. 目标

1. 含 `/`、`,`、括号的 CSS 值**不易被静默算坏**。
2. 复杂选择器 / `@keyframes` / 多层 media 有**官方推荐写法**（表内 vs 外链 vs raw 块）。
3. 与壳 CSS 的层叠关系在文档中一句话写清（并链到 [02](02-shell-css.md)）。

---

## 2. 作者 API / 约定（草案）

### 2.1 立即固化的文档约定（可不改代码）

| 规则 | 说明 |
|------|------|
| 含 `/` 的值必须引号 | `"1 / 5"`、`"image/png"`（已有；Skill/教程加醒目反例） |
| 复杂皮肤优先外链 | `static/theme.css` + `page.head` / `app.icons` 同类资源表 |
| `@keyframes` / 长 media | 放外链 CSS，不塞进 GFM 单元格 |
| 调试 | `shell_css=off` 后只看主题文件，避免同页双源 |

### 2.2 产品增强（C3）

| 能力 | 说明 |
|------|------|
| `style` 装配时对可疑单元格告警 | 未加引号且含 `/` → listen/assemble 警告或硬错误（可开关） |
| 可选 `##` raw CSS 块绑定 | 例如页面附属 ````css` 具名块，**不经**表表达式求值 |
| 文档示例 | `min-width` + `max-width` 外链表；引用 [table-cell-expressions.md](../table-cell-expressions.md) |

---

## 3. 插件 / 核心改动点

| 改动 | 说明 |
|------|------|
| web style 编译 | 可选 `strict_css_cells=True`：裸 `/` 报错 |
| （可选）`web_page_css_raw` | 把文本 CSS 追加到页面，跳过 T5 |
| Skill `marqdo` / tutorial 14 | 增加「样式踩坑」小节 |

---

## 4. 兼容策略

- 默认不硬错误（避免打碎旧表）；新 scaffold 默认 `strict_css_cells` 或文档强制引号。
- 不把完整 CSS 语法塞进表格方言——复杂内容走文件。

---

## 5. 验收

| 场景 | 期望 |
|------|------|
| 文档反例 `grid-column` 裸 `1 / 5` | 在 strict 模式下失败或警告 |
| 外链 `theme.css` + `shell_css=off` | 无壳栅格串扰 |
| 现有 web 示例 | 默认模式仍绿 |

---

## 6. 过渡期 workaround

全部敏感值加引号；响应式与 keyframes 只写在 `static/*.css`；小屏问题优先关壳（[02](02-shell-css.md)）再调主题。
