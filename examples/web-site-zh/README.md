# web-site-zh

中文作者通过网页、表单与鉴权资源组织站点。首页、关于、写文章、后台与短链详情各自声明路由；父组件组合结构，内容叶子只写 Markdown。主题保留原来的墨色、酒红、金色、字体、粒子、断点和表单状态，不写作者业务 JavaScript 或手写 CSS。

从仓库根使用当前源码 CLI，先准备工程与 Web 插件，再调用真实启动流程：

```sh
target/debug/marqdo call skills/marqdo-dev/index.mq.md prepare_engineering --effects process.exec --fs-root . --require-ok --json
target/debug/marqdo call skills/marqdo-dev/index.mq.md prepare_web --effects plugin --fs-root . --require-ok --json
target/debug/marqdo call examples/web-site-zh/index.mq.md serve --effects plugin,fs.write,network.write --fs-root examples/web-site-zh --require-ok --json
```

打开 http://127.0.0.1:18082/ ；默认数据库位于站点文件根的 data/site-zh.db，重启保留现有文章。admin/demo 仅为本机演示账户。服务阻塞当前进程，停止由调用者负责。

按职责阅读：

- [工程入口](index.mq.md) — 声明首页，组合数据、表单、会话、门禁与监听。
- [布局入口](components/index.mq.md) — 公共侧栏与独立后台结构，终端叶子在 components/content。
- [文章请求资源](components/articles.mq.md) — 当前摘要卡片与真实短链完整正文。
- [共享表单](forms.mq.md) — 实际字段和规则在 forms.tables.md，校验失败保留 CSRF。
- [主题入口](styles/index.mq.md) — 保留原始数据，只映射结构地址并适配包装几何。
- [原生核对工具](verify.mq.md) — 实际比较 412 条原始布局/动画声明并返回 CSS 身份。

```sh
target/debug/marqdo call examples/web-site-zh/verify.mq.md theme_values --effects plugin,fs.read --fs-root examples/web-site-zh --require-ok --json
```

主题生成在 public/generated/theme.css；它是可再生产物，源码为相邻声明表，不应手改生成文件。字体链接保留 Google Fonts 地址，离线使用原声明中的后备字体；本地浏览器验收不声称验证远端字体服务送达。

公共创建表单继续公开，后台页面则有实际角色门禁。共享表单出现在后台不意味着提交端点自动私有。实际 HTTP 回归在 Tree、Bytecode 中使用独占副本站点；真实浏览器验证样式、发布、详情、会话导航、退出与窄屏顺序，避免操作示例已有数据库。

动手流程见英文对应工程的 [DEMO.md](../web-site/DEMO.md)。
