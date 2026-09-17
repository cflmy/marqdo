# web-collab-doc

Two-browser collaborative notepad over WebSocket room (last-write-wins text).

```bash
marqdo wasm build -o examples/web-collab-doc/static
cd examples/web-collab-doc
marqdo run index.mq.md
# open http://127.0.0.1:18094 twice
```

Design: [ext-web-collab.md](../../doc/design/ext-web-collab.md).
