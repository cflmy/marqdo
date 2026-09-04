# web-client-site（路线 D）

服务端 `ext/web` + 浏览器 WASM 会话；**作者零业务 JS**。

```bash
# 仓库根
cargo build -p marqdo --release
cargo build --release -p marqdo_plugin_web
./target/release/marqdo ext add web

# 构建 WASM + bridge 到本示例 static/
./target/release/marqdo wasm build -o examples/web-client-site/static
# 保留 client.mq.md（wasm build 只写入 wasm/bridge）

cd examples/web-client-site
marqdo run index.mq.md
# 打开 http://127.0.0.1:18090/
```

`index.mq.md` 用 `web.client_embed source="/static/client.mq.md"` 注入自启脚本；交互在 `static/client.mq.md`。
