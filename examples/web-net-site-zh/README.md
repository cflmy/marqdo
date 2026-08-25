# examples/web-net-site-zh

中文作者面示例：只导入 `ext/web/网页.mq.md`（不用英文 `web.mq.md`），并用 `lib/网络.mq.md` 的解析原语。

```bash
cargo build -p marqdo_plugin_web
cargo run -- examples/web-net-site-zh/index.mq.md
```

从仓库根目录或示例目录运行都可以：`static_dir`（如 `public`）与相对 db 路径（如 `data/site-net-zh.db`）都会基于**入口脚本所在目录**解析，而不是终端的当前目录。

打开 http://127.0.0.1:18084/：

- `/` — SQLite 文章列表 + `/live` WebSocket 实时小部件（`public/live.js`）
- `/about`
- `/new` — 表单 POST 到 `/_form/article`
- `/tools` — `lib/net`（`解析Cookie` / `解析多部分`）的解析演示
- `/admin` — 会话门禁后台，用 `admin` / `secret` 登录
- `/live` — WebSocket 回显端点（首页小部件或 `网页.实时` 的 `连接` 接入）
