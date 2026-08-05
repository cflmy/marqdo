# 用户文档（可执行 · 可下载 · 中英双语）

本目录即面向访客的 Marqdo 介绍：每个 `.mq.md` 既是文稿也能 `marqdo run`，并随仓库 / 静态站一起下载。

**双语约定：** 拉丁文件名 → 英文叙述与英文内置；中文文件名 → 中文叙述与中文编程。成对放在同一目录。标准库：拉丁路径用英文 API，中文路径用中文 API；JSON 仅 `lib/json.mq.md`。

| 路径 | 内容 |
|------|------|
| [`00-welcome.mq.md`](00-welcome.mq.md) / [`00-欢迎.mq.md`](00-欢迎.mq.md) | 首页（英 / 中） |
| [`structure/`](structure/) | 基本结构 |
| [`keywords/`](keywords/) | 关键字与 L0 内置 |
| [`stdlib/`](stdlib/) | **标准库**（导入方式 + text/表/文件/时间/系统/json/网络） |
| [`features/`](features/) | 特性迭代 |
| [`lib-import.mq.md`](lib-import.mq.md) / [`lib-导入.mq.md`](lib-导入.mq.md) | 快速导入示例（text） |

**不放** `errors/`：失败样例只在 `tests/errors/`。

开发文档在 [`doc/`](../doc/)。

## 浏览与生成 HTML

```bash
cargo run -- view public
cargo run -- view output public -o public
```

详见 [user-site.md](../doc/design/user-site.md)。
