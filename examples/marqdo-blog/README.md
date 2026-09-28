# examples/marqdo-blog

Marqdo **Web Artifact** 示例（ADR 0007）：`.mq.md` 即文档、即路由、即页面。

## 运行

```bash
bash scripts/build-web-plugin.sh
# 若 ~/.marqdo/ext 仍是旧 compose_* 面：export MARQDO_EXT=$PWD/ext
cargo run -- run examples/marqdo-blog/serve.mq.md
```

打开 http://127.0.0.1:18081/：

| 路径 | 来源 |
|------|------|
| `/` | `index.mq.md`（`type: web` / 本例中文 `serve` 扫描） |
| `/about` | `pages/about.mq.md`（`类型: 网页`） |
| `/api/ping` | `api/ping.mq.md`（`type: endpoint`） |

## 设计（代码即文档）

- **Document** — frontmatter `type`/`类型` + Markdown 正文。
- **Endpoint** — `type: endpoint` + 可执行正文。
- **发现** — `网页.服务` / `web.serve root=` 扫描目录内 Artifact。
- **Resource** — DB 等见 `ext/data`（本示例入口以 Document 为主）。

规范：[ext-web-artifact.md](../../doc/design/ext-web-artifact.md) · ADR [0007](../../doc/adr/0007-web-document-endpoint.md)。
