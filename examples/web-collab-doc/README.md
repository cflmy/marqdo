# web-collab-doc

这个工程演示同源 WebSocket 房间中的双浏览器记事本。客户端发送完整文本快照，收到的较新消息覆盖本地内容；它不实现 CRDT、离线合并或持久化历史。服务只在本机演示端口提供匿名房间，部署前应配置真实鉴权。

从仓库根目录构建当前浏览器运行时，再启动服务：

```sh
marqdo call skills/marqdo-dev/index.mq.md build_browser 'out="examples/web-collab-doc/static"' --effects plugin,process.exec,fs.write --fs-root . --require-ok --json
marqdo call examples/web-collab-doc/index.mq.md serve --effects plugin,fs.read,network.write --fs-root . --json
```

打开 http://127.0.0.1:18094 两次，在任一窗口编辑文本。另一窗口收到快照后仍可继续本地编辑，空文本也是有效快照。构建缺少兼容 WASM 目标时，先按 [工具链手册](../../skills/marqdo-dev/toolchain.mq.md) 准备工作区编译器，把返回的 compiler 传给 build_browser。

入口 [index.mq.md](index.mq.md) 负责页面发现与房间绑定；[home.mq.md](home.mq.md) 组合说明、编辑器和状态，末端内容与控件分别可读；[客户端](static/client.mq.md) 直接表达输入、消息和在线状态处理。

[开发手册](../../skills/marqdo-dev/index.mq.md) 的 test_browser 使用当前构建运行真实双页交互，覆盖远端更新后的下一次本地输入、空文本与在线状态。设计背景见 [协作设计](../../doc/design/ext-web-collab.md)。
