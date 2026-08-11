# ext/web

Official dynamic website extension: **GFM tables + class methods**.  
Design: [`doc/design/ext-web.md`](../../doc/design/ext-web.md).

| Surface | Import | Classes |
|---------|--------|---------|
| English | `> ext/web/web.mq.md` | `page` · `db` · `app` · `form` · `style` |
| Chinese | `> ext/web/网页.mq.md` | `页面` · `数据库` · `应用` · `表单` · `样式` |

```bash
cargo build -p marqdo_plugin_web
# optional install into ~/.marqdo/ext
marqdo ext add web
```

Canonical sample (design §4): [`examples/web-site/`](../../examples/web-site/).

```bash
cargo run -- run examples/web-site/index.mq.md
# http://127.0.0.1:18081/  ·  /admin
```

Offline assemble smoke: `tests/ext/web-smoke.mq.md`.

**Form / validate / submit** (§5.5): field table + rules + `mount_form` → `GET|POST /_form/{id}`.
