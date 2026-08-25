# examples/web-net-site

Demonstrates the network-extended `ext/web` + `lib/net` surface (design `doc/design/ext-web-net.md`).

```bash
cargo build -p marqdo_plugin_web
cargo run -- examples/web-net-site/index.mq.md
```

Then open http://127.0.0.1:18083/:

- `/` — articles from SQLite plus a live WebSocket widget (`/live`, served by `public/live.js`)
- `/about`
- `/new` — form POST to `/_form/article`
- `/tools` — parsing demos powered by `lib/net`: `net.cookie_parse` and `net.multipart_parse`
- `/admin` — session-gated admin; sign in with `admin` / `secret`
- `/live` — WebSocket echo endpoint (try it from the home widget, or with `web.ws.connect`)

You may run it from the repo root or from this directory: relative `static_dir` (e.g. `public`) and relative db paths (e.g. `data/site-net.db`) resolve against the **entry script's directory**, not the terminal's current working directory.
