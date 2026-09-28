# examples/web-client-site

SSR **Document Artifact** + `web.client_embed` auto-mount（零作者 JS）。ADR 0007。

```bash
bash scripts/build-web-plugin.sh
export MARQDO_EXT=$PWD/ext   # 若本机 ~/.marqdo/ext 未同步
marqdo wasm build -o examples/web-client-site/static
cargo run -- run examples/web-client-site/serve.mq.md
```

- `index.mq.md` — `type: web` 文档页  
- `serve.mq.md` — `web.serve` + `static_dir`  
- `static/client.mq.md` — `lib/browser` wire（不变）
