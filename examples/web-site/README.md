# examples/web-site

Canonical **English** sample for `ext/web` — WWW Demo walkthrough site.

**Hands-on script:** [DEMO.md](./DEMO.md)

## Authoring rules (checkable)

| Surface | How it is authored |
|---------|-------------------|
| Routes / forms / admin | GFM tables + class methods |
| Look-and-feel | Host shell CSS + **style tables** |
| Author business JavaScript | **none** |
| Author hand-written `.css` | **none** |

```bash
cargo build -p marqdo_plugin_web
marqdo run examples/web-site/index.mq.md
```

http://127.0.0.1:18081/ — `/about`, `/new` (validation), `/admin` (list + create).

```bash
find examples/web-site \( -name '*.js' -o -name '*.css' \) | wc -l   # 0
```

Chinese twin: [`examples/web-site-zh/`](../web-site-zh/).
