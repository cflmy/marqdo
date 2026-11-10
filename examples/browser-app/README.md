# browser-app：可读源码中的浏览器工程

从 [index.mq.md](index.mq.md) 开始阅读和执行，再沿 [组件入口](components/index.mq.md) 查看布局。首页拆成列表与计时，网络页再拆成输出、批量请求和 WebSocket 控件；内容在 content 中保留普通 Markdown，控件叶子使用原生 View 节点。

浏览器程序是 [static/app.mq.md](static/app.mq.md)。事件表、参数与返回契约就在回调旁边；操作用可读标签表达，列表文本始终保留为数据。样式入口为 [styles/index.mq.md](styles/index.mq.md)，旁边的字面表保留旧页面全部 59 条声明，并给出 View 包装层的几何适配。

| 能力 | 实际行为与边界 |
|---|---|
| 列表 | 原生记录保留独立 ID；重复内容可单独删除，特殊字符使用 text，删除按钮使用 children |
| 草稿 | localStorage 保存输入框文本；完成回调确认保存，缺失值与空草稿分别处理；条目不跨刷新保存 |
| 历史 | 三个面板使用真实 pathname；popstate 只反映现有位置，不重复 push；直接打开 /about 或 /net 返回首页 |
| 计时 | 同名计时器替换，停止保留已累计次数 |
| 批量请求 | 两个公开 UUID 请求可能失败；输出每项真实状态、响应体或错误 |
| WebSocket | 本地 /live 原生回声资源；展示实际打开、消息、错误与关闭回调 |
| 剪贴板与下载 | 导出当前列表文本；即时提示仅确认已发起，浏览器策略仍可能阻止完成 |

在仓库根目录构建当前官方运行时，再启动原生服务器。需要已经可用的 Rust/WASM 工具链和本仓库工程、网络插件；缺少 WASM 标准库时先阅读 [工作区工具链手册](../../skills/marqdo-dev/toolchain.mq.md)，并将返回的 compiler 传给 build_browser。生成资源与编译样式不作为业务源码提交。

```sh
marqdo call skills/marqdo-dev/index.mq.md build_browser 'out="examples/browser-app/static"' --effects plugin,process.exec,fs.write --fs-root . --require-ok --json
marqdo call examples/browser-app/index.mq.md serve --effects plugin,fs.write,network.write --fs-root examples/browser-app --require-ok --json
```

打开 http://127.0.0.1:18095。控件等待实际启动成功后启用；缺失资源返回明确失败，不能把加载说明当作已运行。服务只监听本机，示例不引入数据库、认证或持久条目存储。

开发验证同样通过可执行手册完成。下列浏览器任务原生复制当前运行时，并运行真实页面交互；它覆盖重复项、字面文本、草稿、历史、计时、导出内容、受控批量请求失败与本地 WebSocket 收发。Node/Playwright 是自动化测试的外联边界，文件读写与任务编排仍由 Marqdo 执行。

```sh
marqdo call skills/marqdo-dev/index.mq.md check 'path="examples/browser-app/index.mq.md"' --effects plugin --require-ok --json
marqdo call skills/marqdo-dev/index.mq.md test_browser --effects plugin,process.exec,fs.write --fs-root . --require-ok --json
```

后续修改先阅读 [编程规范](../../doc/library-development.mq.md)，再修改一个职责的组件或回调。需要新的浏览器宿主能力时扩充正式 bridge 与标准库契约，保留真实浏览器验收；不在父组件中重新拼接 HTML 或业务 JavaScript。
