# 领域扩展与领域聚合

本目录覆盖按需安装的领域能力。功能组件默认 `planned`，通过 Git 子模块固定独立仓库的开发提交，完成验收后晋升；聚合仓库不实现子组件业务，根 catalog 只登记 `kind = "domain"` 的聚合本身，不重复登记其后代路径。

## 领域聚合（planned）

| 路径 | 仓库 | 说明 |
| --- | --- | --- |
| `web/` | [marqdo-ext-web](https://git.cflmy.cn/cflmy/marqdo-ext-web) | Web 原型设计领域聚合：唯一设计合同、示例、组合验收与交付协调；子组件 [marqdo-web-design](https://git.cflmy.cn/cflmy/marqdo-web-design)、[marqdo-web-preview](https://git.cflmy.cn/cflmy/marqdo-web-preview) 在领域 `components.toml` 登记 |

## 独立组件（planned）

- [marqdo-ext-math](https://git.cflmy.cn/cflmy/marqdo-ext-math) · 数值算法、统计、符号数学及绘图
- [marqdo-ext-linalg](https://git.cflmy.cn/cflmy/marqdo-ext-linalg) · 复数、矩阵、分解及数值线性代数
- [marqdo-ext-quantum](https://git.cflmy.cn/cflmy/marqdo-ext-quantum) · 量子电路、态向量、采样与模拟
- [marqdo-ext-http](https://git.cflmy.cn/cflmy/marqdo-ext-http) · HTTP、URL、SSE 与报文生命周期
- [marqdo-ext-websocket](https://git.cflmy.cn/cflmy/marqdo-ext-websocket) · WebSocket 握手、帧与连接语义
- [marqdo-ext-ai](https://git.cflmy.cn/cflmy/marqdo-ext-ai) · 模型协议、流式输出与工具调用数据
- [marqdo-ext-mcp](https://git.cflmy.cn/cflmy/marqdo-ext-mcp) · MCP 协议、传输、发现和调用
- [marqdo-ext-crypto](https://git.cflmy.cn/cflmy/marqdo-ext-crypto) · 成熟密码学实现的封装
- [marqdo-ext-database](https://git.cflmy.cn/cflmy/marqdo-ext-database) · SQL 参数绑定、事务和驱动连接
- [marqdo-ext-storage](https://git.cflmy.cn/cflmy/marqdo-ext-storage) · 本地和对象存储接口
- [marqdo-ext-cache](https://git.cflmy.cn/cflmy/marqdo-ext-cache) · TTL、缓存和失效
- [marqdo-ext-security](https://git.cflmy.cn/cflmy/marqdo-ext-security) · 应用身份、会话及权限模型
- [marqdo-ext-browser](https://git.cflmy.cn/cflmy/marqdo-ext-browser) · 浏览器宿主、WASM 适配、DOM 和事件
- [marqdo-ext-document](https://git.cflmy.cn/cflmy/marqdo-ext-document) · Markdown 结构与显式文档编辑
- [marqdo-ext-archive](https://git.cflmy.cn/cflmy/marqdo-ext-archive) · 压缩、安全归档和解包
- [marqdo-ext-engineering](https://git.cflmy.cn/cflmy/marqdo-ext-engineering) · 工程操作、文件补丁与产物
- [marqdo-ext-regex](https://git.cflmy.cn/cflmy/marqdo-ext-regex) · 正则匹配、捕获、扫描和替换
- [marqdo-ext-datetime](https://git.cflmy.cn/cflmy/marqdo-ext-datetime) · IANA 时区、DST 和高级日历

[组件目录](../components.toml) · [各仓库任务](../docs/REPOSITORY-TASKS.md) · [架构边界](../docs/ARCHITECTURE.md)
