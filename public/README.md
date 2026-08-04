# 用户文档（可执行 · 可下载）

本目录即面向访客的 Marqdo 介绍：每个 `.mq.md` 既是文稿也能 `marqdo run`，并随仓库 / 静态站一起下载。

| 路径 | 内容 |
|------|------|
| [`00-welcome.mq.md`](00-welcome.mq.md) | 首页 |
| [`structure/`](structure/) | 基本结构 |
| [`keywords/`](keywords/) | 关键字与内置 `print` |
| [`features/`](features/) | 特性迭代说明 |

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
