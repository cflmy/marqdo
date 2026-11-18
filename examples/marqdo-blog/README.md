# examples/marqdo-blog

Marqdo **Web Artifact** blog (ADR 0007).

## Layout

```text
index.mq.md          # type: web · / · data_source: posts
pages/about.mq.md    # 类型: 网页 · /about
pages/post.mq.md     # type: web · /post/{slug} · data_where
api/ping.mq.md       # type: endpoint · GET /api/ping
api/echo.mq.md       # type: endpoint · POST /api/echo
db/posts.mq.md       # ext/data schema + seed + open
public/              # static (mounted at /static)
serve.mq.md          # web.serve + db open
smoke.sh             # curl checks against a running serve
```

## Run

从仓库根使用当前 CLI 和开发手册准备原生插件，再调用启动函数：

```sh
target/debug/marqdo call skills/marqdo-dev/index.mq.md prepare_web --effects plugin --fs-root . --require-ok --json
target/debug/marqdo call examples/marqdo-blog/serve.mq.md main --effects plugin,fs.write,network.write --fs-root examples/marqdo-blog --require-ok --json
```

默认数据库按授权站点根解析到 data/blog.db，重启保留已有文章。准备工程插件见开发入口的 prepare_engineering；验收使用独占副本，不删除现有数据库。


| Path | Artifact |
|------|----------|
| `/` | Document home + post cards |
| `/about` | ZH Document |
| `/post/{slug}` | Document detail |
| `/api/ping` | Endpoint JSON |
| `/api/echo` | Endpoint JSON echo |
| `/static/*` | public assets |

Design: [ext-web-artifact.md](../../doc/design/ext-web-artifact.md).
