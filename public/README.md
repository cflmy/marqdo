# 用户文档（可执行 · 可下载 · 中英双语）

本目录即面向访客的 Marqdo 介绍：每个 `.mq.md` 既是文稿也能 `marqdo run`，并随仓库 / 静态站一起下载。

**双语约定：** 拉丁文件名 → 英文叙述与英文内置；中文文件名 → 中文叙述与中文编程（`打印`、`长度` …）。成对放在同一目录（如 `structure/01-hello.mq.md` 与 `structure/01-你好.mq.md`）。标准库按路径选 API：`lib/text.mq.md` vs `lib/文本.mq.md`。

| 路径 | 内容 |
|------|------|
| [`00-welcome.mq.md`](00-welcome.mq.md) / [`00-欢迎.mq.md`](00-欢迎.mq.md) | 首页（英 / 中） |
| [`structure/`](structure/) | 基本结构（英 / 中并列） |
| [`keywords/`](keywords/) | 关键字与内置（英 / 中并列） |
| [`features/`](features/) | 特性迭代（英 / 中并列） |
| [`lib-import.mq.md`](lib-import.mq.md) / [`lib-导入.mq.md`](lib-导入.mq.md) | 导入 `lib/text` / `lib/文本` |

**不放** `errors/`：失败样例只在 `tests/errors/`，避免用户站出现红色执行失败。

开发文档（语言设计等）在 [`doc/`](../doc/)，与本目录分离。

## 浏览与生成 HTML

```bash
cargo run -- view public
cargo run -- view output public -o public
# 或
./scripts/build-public.sh
powershell -File ./scripts/build-public.ps1
```

生成的 `index.html` 与 `pages/` 默认被 ignore；CI 构建后推到 `gh-pages`。详见 [user-site.md](../doc/design/user-site.md)。
