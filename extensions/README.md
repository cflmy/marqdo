# 领域扩展与领域聚合

本目录覆盖按需安装的领域能力。功能组件默认 `planned`，开发在独立责任仓库进行，验收后按治理规则接入；聚合仓库不实现子组件业务，根 catalog 只登记 `kind = "domain"` 的聚合本身，不重复登记其后代路径。

## 领域聚合（planned）

| 路径 | 仓库 | 说明 |
| --- | --- | --- |
| [web](web/README.md) | [marqdo-ext-web](https://git.cflmy.cn/cflmy/marqdo-ext-web) | Web 原型设计领域聚合：唯一设计合同、示例、组合验收与交付协调；子组件 [marqdo-web-design](https://git.cflmy.cn/cflmy/marqdo-web-design)、[marqdo-web-preview](https://git.cflmy.cn/cflmy/marqdo-web-preview) 在领域 `components.toml` 登记 |

## 独立组件（planned）

- [marqdo-ext-math](math/README.md) · 数值算法、统计、符号数学及绘图
- [marqdo-ext-linalg](linalg/README.md) · 复数、矩阵、分解及数值线性代数
- [marqdo-ext-quantum](quantum/README.md) · 量子电路、态向量、采样与模拟
- [marqdo-ext-http](http/README.md) · HTTP、URL、SSE 与报文生命周期
- [marqdo-ext-websocket](websocket/README.md) · WebSocket 握手、帧与连接语义
- [marqdo-ext-ai](ai/README.md) · 模型协议、流式输出与工具调用数据
- [marqdo-ext-mcp](mcp/README.md) · MCP 协议、传输、发现和调用
- [marqdo-ext-crypto](crypto/README.md) · 成熟密码学实现的封装
- [marqdo-ext-database](database/README.md) · SQL 参数绑定、事务和驱动连接
- [marqdo-ext-storage](storage/README.md) · 本地和对象存储接口
- [marqdo-ext-cache](cache/README.md) · TTL、缓存和失效
- [marqdo-ext-security](security/README.md) · 应用身份、会话及权限模型
- [marqdo-ext-browser](browser/README.md) · 浏览器宿主、WASM 适配、DOM 和事件
- [marqdo-ext-document](document/README.md) · Markdown 结构与显式文档编辑
- [marqdo-ext-archive](archive/README.md) · 压缩、安全归档和解包
- [marqdo-ext-engineering](engineering/README.md) · 工程操作、文件补丁与产物
- [marqdo-ext-regex](regex/README.md) · 正则匹配、捕获、扫描和替换
- [marqdo-ext-datetime](datetime/README.md) · IANA 时区、DST 和高级日历

[组件目录](../components.toml) · [各仓库任务](../docs/REPOSITORY-TASKS.md) · [架构边界](../docs/ARCHITECTURE.md)
