# `marqdo view` — 源结构浏览器

| | |
|---|---|
| 状态 | 已定（实现中） |
| 日期 | 2026-08-04 |
| 相关 | [examples-and-tests.md](examples-and-tests.md) · [pipeline-debug.md](pipeline-debug.md) · AST（`src/ast.rs`） |

---

## 1. 动机

`.mq.md` 既是文档也是程序。需要一种**可读渲染**，而不仅是纯文本或 `run` 的 stdout：

- 按语法树展示函数、分支、循环、调用、返回；  
- 同页展示**执行结果**；  
- 支持打开**单个文件**或**文件夹**（扫描全部 `.mq.md` + 结构索引）。

---

## 2. CLI

```bash
marqdo view [PATH] [--port PORT] [--host HOST] [--no-open]
```

| 参数 | 默认 | 说明 |
|------|------|------|
| `PATH` | `.` | `.mq.md` 文件，或目录（递归扫描 `*.mq.md`） |
| `--port` | `7429` | HTTP 端口 |
| `--host` | `127.0.0.1` | 仅本地 |
| `--no-open` | 关 | 不尝试打开系统浏览器 |

示例：

```bash
marqdo view examples/structure/hello.mq.md
marqdo view examples
marqdo view examples/keywords --port 8080
```

启动后打印 `http://127.0.0.1:7429/`，阻塞直至 Ctrl+C。

---

## 3. 信息架构

### 3.1 目录模式

```
┌─────────────┬──────────────────────────────────┐
│ 索引         │  选中文件 / 欢迎页                 │
│ structure/  │                                  │
│  hello      │  [源码结构] [执行结果] [原始文本]   │
│  branch     │                                  │
│ keywords/   │                                  │
└─────────────┴──────────────────────────────────┘
```

- 侧栏：相对 `PATH` 的目录树；只列出 `.mq.md`。  
- 点击文件：主区切换为该文件的三栏/三节视图。

### 3.2 单文件模式

无侧栏列表（或仅当前文件）；主区同「选中文件」视图。仍提供「执行」结果。

---

## 4. 主区三节

### 4.1 源码结构（AST 渲染）

不依赖浏览器 Markdown 猜测语义，而用服务端已解析的 AST：

| AST 节点 | 视觉 |
|----------|------|
| `Function` | 卡片：标题层级、形参芯片、子函数嵌套；**中间的注释行仍展示**（来自源行分类，不在 AST 内） |
| `Stmt::*` | 绑定 / 返回 / 调用 / 分支 / 循环；表达式用**表面语法**（如 `` `x` > 0 ``），不用 Debug |

样式：清晰分区，避免「通用仪表盘」堆砌；结构层级一眼可读。

### 4.2 执行结果

服务端对该文件跑与 `marqdo run` 相同的管线，**捕获 stdout/stderr** 与退出状态，展示在「输出」区。  
含 `input` 的程序：view 中标注「需交互，未在 view 执行」或提供只读说明（v0：跳过/失败信息）。

### 4.3 原始文本

等宽展示源文件；可选行分类着色（Code / Comment / Blank），与 `--dump-lines` 一致。

---

## 5. HTTP API（实现细节）

| 方法 | 路径 | 含义 |
|------|------|------|
| `GET` | `/` | 索引 HTML |
| `GET` | `/file?path=…` | 单文件页（path 相对根，防 `..` 逃逸） |
| `GET` | `/api/tree` | JSON 文件树 |
| `GET` | `/api/file?path=…` | JSON：`{ source, ast_html 或 ast_json, stdout, stderr, ok }` |

v0 可全部服务端渲染 HTML，API 可选。

安全：根目录锁死为 CLI 给定的 `PATH`（若为文件则其父目录为根、仅一文件）；拒绝根外路径。

---

## 6. 依赖

本地同步 HTTP 即可（如 `tiny_http`），无强制前端构建；HTML/CSS/少量 JS 内嵌于 Rust。

---

## 7. 验收

1. `marqdo view examples/structure/hello.mq.md` 显示结构 + `Hello World!` 输出。  
2. `marqdo view examples` 侧栏含 `structure/` 与 `keywords/` 分组。  
3. 分支/循环样例在结构区可区分臂与循环体。  
4. 路径逃逸被拒绝。
