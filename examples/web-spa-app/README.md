# web-spa-app

Marqdo **native SPA**: list → detail → edit with client routes + `scope=memory` store. Zero author JS.

## Run

```bash
marqdo wasm build -o examples/web-spa-app/static
cd examples/web-spa-app
marqdo run index.mq.md
# http://127.0.0.1:18093
```

Design: [ext-web-spa.md](../../doc/design/ext-web-spa.md).
