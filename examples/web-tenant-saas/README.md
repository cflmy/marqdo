# web-tenant-saas

从仓库根目录运行：

```sh
marqdo call examples/web-tenant-saas/index.mq.md serve --effects plugin,fs.write,network.write --fs-root examples/web-tenant-saas --json
```

打开 http://127.0.0.1:18092，比较页面里的 Acme 与 Beta 查询链接。缺少 t 参数时返回 400；返回的 JSON 包含 rows 列表。首次空库初始化两篇笔记，重启保留已有记录。

[工程入口](index.mq.md) 串联 [数据库初始化](db/index.mq.md)、[租户选择](../../ext/security/tenant.mq.md) 与 [JSON 查询资源](../../ext/data/api.mq.md)。页面内容位于 [intro.md](intro.md)，无原生 HTML 壳。

这是允许匿名切换租户的本机过滤演示。生产访问隔离还需要鉴权与成员关系检查。回归测试 web_handbook 覆盖两种后端的租户过滤、缺少参数与用户记录重启保留。
