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

```bash
bash scripts/build-web-plugin.sh
export MARQDO_EXT=$PWD/ext   # if ~/.marqdo/ext is stale
export MARQDO_WEB_PLUGIN=$PWD/plugins/web/build/libweb.so
cd examples/marqdo-blog && marqdo run serve.mq.md
# other terminal:
bash examples/marqdo-blog/smoke.sh
```

| Path | Artifact |
|------|----------|
| `/` | Document home + post cards |
| `/about` | ZH Document |
| `/post/{slug}` | Document detail |
| `/api/ping` | Endpoint JSON |
| `/api/echo` | Endpoint JSON echo |
| `/static/*` | public assets |

Design: [ext-web-artifact.md](../../doc/design/ext-web-artifact.md).
