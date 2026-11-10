# web-spa-app

这个工程演示列表、详情、编辑和新建之间的浏览器路由。客户端用原生记录保存条目，以稳定 id 选择内容；刷新页面后恢复演示数据，不提供持久化存储。用户输入通过文本效果和结构化列表显示。

从仓库根目录先构建浏览器运行时，再启动可调用服务：

```sh
marqdo call skills/marqdo-dev/index.mq.md build_browser 'out="examples/web-spa-app/static"' --effects plugin,process.exec,fs.write --fs-root . --require-ok --json
marqdo call examples/web-spa-app/index.mq.md serve --effects plugin,fs.read,network.write --fs-root . --json
```

打开 http://127.0.0.1:18093。构建需要兼容的 WASM 工具链；缺少时按 [工具链手册](../../skills/marqdo-dev/toolchain.mq.md) 准备工作区编译器，并把返回的 compiler 传给 build_browser。不要用旧版运行时替代当前源码的构建。

工程入口 [index.mq.md](index.mq.md) 负责发现和服务；[components/index.mq.md](components/index.mq.md) 逐层组合导航和视图面板；[客户端源码](static/client.mq.md) 说明事件、路由和状态更新。修改内容从 Markdown 叶子开始，修改布局在组件中完成。

[开发手册](../../skills/marqdo-dev/index.mq.md) 的 test_browser 使用当前构建验证实际列表编辑、浏览器后退、输入转义和移动端布局。设计背景见 [SPA 设计](../../doc/design/ext-web-spa.md)。
