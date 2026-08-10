# `marqdo view` — 源结构浏览器与静态文档导出

| | |
|---|---|
| 状态 | 已定（v0.0.2 方向） |
| 日期 | 2026-08-04 |
| 相关 | [examples-and-tests.md](examples-and-tests.md) · [generated-yaml-manifest.md](generated-yaml-manifest.md) · 视觉参考 [id.cflmy.cn](https://id.cflmy.cn) |

---

## 1. 动机

`.mq.md` 既是文档也是程序。需要：

1. **本地浏览**：按 AST 展示结构 + 执行结果；  
2. **静态导出**：同一套渲染写成 HTML，作为**用户文档站点**（勿手写第二套站点）；  
3. **启动快**：样式极简，无外链字体/厚主题，避免 view 冷启动慢。

---

## 2. 视觉（Apple HIG / Docs 极简）

对齐 [Apple HIG](https://developer.apple.com/design/human-interface-guidelines) 的 **clarity / deference / white space**，并参考常见 docs 布局（sticky 侧栏 + 居中主栏，见 Nextra / Docusaurus 类站点）：**系统灰底、白内容面、浅分隔线、系统字体**。作者站 [id.cflmy.cn](https://id.cflmy.cn) 仍作克制感参考，但不强行纯黑描边。

| 规则 | 要求 |
|------|------|
| 色板 | 页底 `#f5f5f7`；内容面 `#ffffff`；正文 `#1d1d1f`；次要字 `#6e6e73`；分隔 `#d2d2d7`；选中侧栏项反白 |
| 布局 | 壳层 `max-width ≈ 1680px`；侧栏 ≈280px；主内容 `max-width ≈ 76rem`；Structure 旁函数大纲；**≤800px 侧栏收起** |
| 默认页 | 打开目录时 **直接展示排序后的第一个 `.mq.md`**（实时 `/` 与静态 `index.html` 同），不要求用户先从欢迎页点选 |
| 字体 | 仅系统栈：`-apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif`；等宽 `ui-monospace, SFMono-Regular, Menlo, Consolas, monospace` |
| 外链 | **禁止** Google Fonts / CDN 样式；CSS 全部内联于 HTML |
| 装饰 | 无多层阴影、无渐变、无发光；圆角 ≈12px；结构区为分组列表（底部分隔），非堆叠厚卡片 |
| 诊断路径 | 展示时去掉 Windows `\\?\` 前缀，并优先用相对 view 根的路径 |

验收：首屏无外网字体请求；宽屏主栏明显宽于早期 `58rem` 方案（现约 `76rem` / 壳 `1680px`）。

---

## 3. CLI

### 3.1 实时浏览

```bash
marqdo view [PATH] [--port PORT] [--host HOST] [--no-open]
```

| 参数 | 默认 | 说明 |
|------|------|------|
| `PATH` | `.` | `.mq.md` 或目录 |
| `--port` | `7429` | |
| `--host` | `127.0.0.1` | |
| `--no-open` | 关 | 不打开浏览器 |

### 3.2 静态导出（文档生成）

```bash
marqdo view output [PATH] -o OUT_DIR [--no-exec]
```

| 参数 | 默认 | 说明 |
|------|------|------|
| `PATH` | `.` | 与 `view` 相同扫描范围 |
| `-o` / `--out` | （必填） | 输出目录（不存在则创建） |
| `--no-exec` | 关 | 不跑程序，执行区显示「skipped」 |

**产物布局：**

```
OUT_DIR/
  index.html                 # 默认 = 排序后第一个 .mq.md 的完整页（侧栏高亮该项）
  pages/
    <相对路径>.html          # 每个 .mq.md 一页；路径中的 `/` 保留为子目录
                             # 例：structure/hello.mq.md → pages/structure/hello.mq.md.html
```

- 页内链接一律**相对路径**，可用任意静态服务器或 `file://`（同目录结构下）打开。  
- 用户文档推荐流程：`marqdo view output public -o public`，将生成物发布到 `gh-pages`（见 [user-site.md](user-site.md)）。  
- **执行权限**：与 `marqdo run` 一致（导入即用）；view/导出仅对 `exit`/`sleep` 做软化，避免拖垮进程。详见 [stdlib-modules.md](stdlib-modules.md) §6。  
- 金样例浏览仍可用：`marqdo view output tests -o /tmp/mq-tests`（含 errors，勿当对外用户站）。  
- 渲染逻辑与实时 `view` **共用**同一套 HTML 构件（结构 / 执行 / 源码），禁止两套皮肤分叉。

示例：

```bash
marqdo view output public -o public
marqdo view output tests/structure/hello.mq.md -o /tmp/hello-doc
```

---

## 4. 信息架构

侧栏文件索引 + 主区：结构（AST+注释；旁侧函数大纲可搜索）· 执行 · 源码。表达式为表面语法。  
**断点调试**不在 view 内，见 [`marqdo debug`](view-debug.md)。

---

## 5. HTTP（实时模式）

| 方法 | 路径 | 含义 |
|------|------|------|
| `GET` | `/` | 默认打开第一个 `.mq.md`（无文件时为空提示） |
| `GET` | `/file?path=…` | 单文件页 |
| `GET` | `/api/tree` | JSON 树（可选） |
| `GET` | `/api/events?path=…` | SSE（`text/event-stream`）：订阅进程内 EventBus（`reasoning` / `delta` / `round` / `decision` / `done` / `error`） |
| `POST` | `/api/run` | JSON `{path, stdin?}`：后台 `run_file`；事件进 bus；结束补 `done`/`error`（不替代页内整页 stdout） |
| `POST` | `/api/foreign-run` | 外联代码块试跑 |

根目录锁死；拒绝 `..` 逃逸。默认页加载仍整页执行；Stream 面板为增强路径（`EventSource` + Run）。

---

## 6. 验收

1. 实时 `view`：HIG 风极简、无外链字体、结构与执行正确；打开目录即见首个文件；函数大纲可搜索跳转。  
2. `view output public -o public`（或临时目录）生成可点的 `index.html`（首文件）与各 `pages/…`。  
3. 静态页中执行区与 `marqdo run` 一致（未加 `--no-exec` 时）；失败诊断路径无 `\\?\`。  
4. 冷启动无明显外网依赖。  
5. 调试用 `marqdo debug`（默认端口 7430），与 view 页面分离。
