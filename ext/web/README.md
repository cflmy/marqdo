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
# http://127.0.0.1:18081/
# /_form/article  ·  /admin  ·  /admin/articles/new
```

Offline smokes: `tests/ext/web-smoke.mq.md`, `web-form-smoke.mq.md`, `web-admin-smoke.mq.md`.

| Feature | Notes |
|---------|--------|
| Form §5.5 | field/rules tables · `validate` · `submit` · `mount_form` → `GET\|POST /_form/{id}` |
| Admin CRUD | `admin=True` · schema → same form path · new/edit/delete under `/admin/{table}` |
| Route | `` `app`.route path=/about page=`about` `` → `GET /about` |