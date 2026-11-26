# extensions/web（Web 领域聚合目标路径）

该路径是 `kind = "domain"` 的 Web 领域聚合仓库 [marqdo-ext-web](https://git.cflmy.cn/cflmy/marqdo-ext-web) 的挂载路径。当前状态 `planned`：**没有 gitlink，也没有本目录内的实现源码**；本 README 只是根 catalog 的记录说明。

## 边界

- 领域聚合仓库只做治理：唯一设计合同、示例、跨组件集成测试、发布装配与交付协调；**不得**在其中实现 renderer、parser、数据库驱动或业务逻辑，也不得复制子仓库源码。
- 子组件不在根 catalog 登记：[marqdo-web-design](https://git.cflmy.cn/cflmy/marqdo-web-design)（设计解释 Rust library）与 [marqdo-web-preview](https://git.cflmy.cn/cflmy/marqdo-web-preview)（`web` Tool 与离线预览）记录在领域仓库自己的 `components.toml`，激活时随领域 gitlink 一次接入根。
- 唯一作者合同：[WEB-DESIGN](https://git.cflmy.cn/cflmy/marqdo-ext-web/src/branch/main/docs/WEB-DESIGN.md)；领域开发指南：[DEVELOPMENT](https://git.cflmy.cn/cflmy/marqdo-ext-web/src/branch/main/docs/DEVELOPMENT.md)。

## 激活门禁

两个子组件具备真实独立入口与候选提交、领域 catalog 与 gitlink 一致、领域 CI 通过、根 `check-layout.py --checkouts` 递归校验通过、Design/Preview 同候选证据齐全。未满足前保持 `planned`，不创建占位 gitlink。详见 [迁移台账](../../docs/MIGRATION.md) 与 [开发流程](../../docs/DEVELOPMENT.md)。
