# ext/web

Official dynamic website extension. Design: [`doc/design/ext-web.md`](../../doc/design/ext-web.md).

```bash
cargo build -p marqdo_plugin_web
marqdo ext add web
```

```markdown
---
> ext/web/web.mq.md
---

# main
> web.ensure_plugin
*`page` = > web.page title=Hi *
*`app` = > web.app page=`page` *
> `app`.listen
```

Scaffold: `> web.scaffold dest=./myapp`

Multipage site pack (compose + table CSS + `/_part`): see [`examples/web-site/`](../../examples/web-site/).
