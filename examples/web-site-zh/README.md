# examples/web-site-zh

中文作者面示例：只导入 `ext/web/网页.mq.md`（不用英文 `web.mq.md`）。

```bash
cargo build -p marqdo_plugin_web
cargo run -- examples/web-site-zh/index.mq.md
```

打开 http://127.0.0.1:18082/ — `/about`、`/new`、`/admin`、`/static/site.css`。
