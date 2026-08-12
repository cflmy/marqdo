# examples/web-site

Canonical English sample for `ext/web` (design §4).

```bash
cargo build -p marqdo_plugin_web
cargo run -- examples/web-site/index.mq.md
```

Then open http://127.0.0.1:18081/ — `/about`, `/new` (form in page slot), `/admin`.
