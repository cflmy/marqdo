# web-net-site

本工程保留文章列表、关于页、文章表单、Cookie/Multipart 工具页、登录后的原生管理后台，以及 /live WebSocket 回显。页面递归组合到 Markdown 和原生控件；浏览器交互由 public/client.mq.md 表达。

从仓库根目录准备当前 CLI 与工程插件后，使用同一份开发手册构建 Web 和浏览器运行时：

```sh
target/debug/marqdo call skills/marqdo-dev/index.mq.md prepare_engineering --effects process.exec --fs-root . --require-ok --json
target/debug/marqdo call skills/marqdo-dev/index.mq.md prepare_web --effects plugin --fs-root . --require-ok --json
target/debug/marqdo call skills/marqdo-dev/index.mq.md build_browser 'out="examples/web-net-site/public/generated"' --effects plugin,process.exec,fs.write --fs-root . --require-ok --json
target/debug/marqdo call examples/web-net-site/index.mq.md serve --effects plugin,fs.write,network.write --fs-root examples/web-net-site --require-ok --json
```

打开 http://127.0.0.1:18083。默认数据库位于站点文件根的 data/site-net.db，重启保留已有文章。缺少兼容 WASM 目标时，先按 [工具链任务](../../skills/marqdo-dev/toolchain.mq.md) 准备工作区编译器，再把返回的 compiler 传给 build_browser。

- [布局入口](components/index.mq.md) 组合导航、侧栏、主体和页脚，叶子在 components/content。
- [文章表单](forms.mq.md) 统一定义字段与服务端规则；校验失败后保留真实 CSRF，成功回到文章列表。
- [工具页](pages/tools.mq.md) 展示真实网络解析结果，不请求远端服务。
- [客户端](public/client.mq.md) 保留按钮与 Enter 发送、去除首尾空白、忽略空输入及连续回显日志。
- 管理后台登录使用本机演示账号 admin / secret。部署时替换账号与访问策略。

请求资源沿用五列 View；数据库查询随每次请求更新，表单引用解析当前会话。详情见 [请求资源设计](../../doc/design/web-view-resources.md)。

开发入口的 test_browser 使用实际当前 WASM 验证回显、表单重试、新增文章、后台登录与窄屏布局。HTTP 回归 web_handbook 在 Tree 与 Bytecode 两个后端启动临时站点，验证数据和鉴权，避免操作用户数据库。
