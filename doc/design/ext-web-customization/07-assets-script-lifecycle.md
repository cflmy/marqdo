# 解决 7：静态资源与脚本生命周期

| | |
|---|---|
| 问题 | [研究 §7](../../research/ext-web-customization-limits.md#7-静态资源与脚本生命周期) |
| 优先级 | **P2** · 波次 **C3** |
| 触点 | `web_page_head` / `web_head` · `render.rs` head 输出 · static 挂载 · [web-assets-and-images.md](../web-assets-and-images.md) |

---

## 1. 目标

1. 默认脚本加载方式**不假设**作者手写 IIFE 抢在 DOM 前执行成功；提供 `defer` / `module` 一等约定。
2. 静态资源变更可发现：文档化 `?v=`，并提供可选自动指纹或 listen 时 mtime 查询参数。

---

## 2. 作者 API（草案）

Head 资源表扩展列（向后兼容）：

| 列 | 说明 |
|----|------|
| `rel` / `关系` | 已有；`script` / `module` |
| `defer` / `推迟` | `真` → `<script defer>`（**建议默认对 script 为真**） |
| `async` / `异步` | 可选 |
| `version` / `版本` | 非空则自动追加 `?v=` |

```markdown
`头` =
| 关系 | 地址 | 推迟 | 版本 |
|------|------|------|------|
| script | /static/theme.js | 真 | 2026-09-04 |
| module | /static/desk.js | 真 | 3 |
```

全局：`app.asset_version="2026-09-04"` 为所有 static 链接统一 bump（可选）。

---

## 3. 插件改动点

| 改动 | 说明 |
|------|------|
| head 渲染 | 尊重 defer/async；`module` → `type="module"`（已有则核对） |
| 默认 | 新建 scaffold 的 script 行带 `defer` |
| （可选）`asset_version` | listen 时重写 head / 页面内 static URL |
| 文档 | 「禁止在同步 head 脚本里直接 `getElementById` 主节点；用 defer 或 `DOMContentLoaded`」 |

---

## 4. 兼容策略

- 旧表无 `defer` 列：保持现网同步行为（或仅对 scaffold/新文档默认 defer，避免静默改变旧站时序）。
- 更安全：缺省仍同步；**Skill 与 scaffold 默认写 defer=真**。

---

## 5. 验收

| 场景 | 期望 |
|------|------|
| defer 脚本读 DOM | 登录跳转逻辑在 DOM 就绪后执行 |
| `version` 列 | HTML 中出现 `theme.js?v=…` |
| 旧示例无新列 | 仍通过 |

---

## 6. 过渡期 workaround

脚本改 `type=module` 或放 body 末；手动 `?v=` bump；登录逻辑用 `DOMContentLoaded`。
