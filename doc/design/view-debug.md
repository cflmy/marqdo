# View 调试（`marqdo debug`）

| | |
|---|---|
| 状态 | **v1（独立 debug 页）** |
| 日期 | 2026-08-05 |
| 相关 | [next-phase.md](../roadmap/next-phase.md) P3 · [view.md](view.md) · [pipeline-debug.md](pipeline-debug.md) |

## 分工

| 命令 | 角色 |
|------|------|
| `marqdo view` | 文档浏览器：Structure + Execution + Source；函数大纲与搜索；**无断点** |
| `marqdo debug` | 调试宿主（默认端口 **7430**）：IDE 式布局；**仅 live** |

静态 `gh-pages` / `view output` **不能**真断点。

## Debug 布局（对齐常见调试器）

参考 VS Code / Chrome DevTools / Jupyter debugger：

```
┌ Toolbar: Start · Continue (F5) · Step (F10) · Stop · status ─┐
├ Files ─┬─ Structure (gutter BP) + outline ─┬─ Variables      ┤
│        │                                    ├─ Breakpoints   │
│        │                                    └─ Call stack    │
├────────┴────────────────────────────────────┴────────────────┤
│ Debug Console (session stdout)                               │
└──────────────────────────────────────────────────────────────┘
```

- 暂停粒度：语句起点（树遍历）。
- 无断点时 Start 停在第一句。
- Structure 为断点面（非 Source 行）。

## API

与此前相同：`POST /api/debug/start|continue|step|stop|breakpoints`（仅 debug 服务）。

## View 大纲

Structure 旁（或窄屏下方）函数树：`#fn-{line}` 锚点 + 搜索框过滤 `data-fn` / `data-fn-path`。
