<!-- marqdo:repo-conventions v1 -->
# AGENTS.md — <repo>

本文件约束在本仓库工作的维护者与 AI Agent。全局规则唯一来源：
[marqdo 仓库开发规范](https://git.cflmy.cn/cflmy/marqdo/src/branch/main/docs/REPOSITORY-CONVENTIONS.md)。
本文件只写本仓边界，不复制全局规范或合同正文。

## 1. 本仓定位与唯一权威

- catalog ID：`<id>`；父级：`<root|domain>`；角色：`<core|stdlib|extension|tool|test|domain>`；状态：`<planned|active>`。
- 职责：<scope>。
- 本仓拥有：<唯一实现/合同/制品/测试>。
- 本仓不拥有：<其它仓的实现/合同/调度/发布>。
- 权威输入/合同：<e.g. 真实 CLI 公开 Analysis 3、MARQDO_BIN 绝对路径、WEB-DESIGN 合同、外部 Bundle>。

## 2. 必做 / 禁止

| 必做 | 禁止 |
| --- | --- |
| 只在本仓职责内修改，先开本仓工单 | 实现其它仓库功能、复制 parser/实现/合同 |
| 用真实 CLI/制品与独立期望测试 | mock 或字符串检查冒充真实验收/PASS |
| 依赖用固定 SHA/摘要，显式输入 | 依赖父仓 checkout、兄弟仓相对路径、浮动分支 |
| 未运行写 `UNVERIFIED` | 伪造 entry/版本/CI 徽章/gitlink/发布资产 |
| 记录 SHA、制品摘要、平台、命令、退出码 | 提交密钥、凭证、构建产物或本地绝对路径 |
| 跨仓变更走 C2→B2→A2 晋升 | 自动 tag/release/合并，或直接改父仓/兄弟仓 |

- 不新增第二 catalog/manifest/lock/version 表/发布流程；不建空壳仓或为单功能拆仓。
- 不在本仓登记其它仓库的后代路径；治理树最多两级。

## 3. 构建、测试与证据

- CI 入口：`<entry>`（非交互、失败非零、不打印秘密、只写声明输出；构建与发布权限分离）。
- 输入：<MARQDO_BIN / MARQDO_STDLIB_DIR / lock/manifest；绝对路径并验证身份>。
- 测试：真实正例 + 错误/授权/边界负例；本地可复现；默认禁网。
- 证据：候选 SHA、依赖/制品 SHA-256、平台/工具链、命令与退出码、日志位置、测试范围；FAIL/UNVERIFIED 不得写成 PASS。
- 交付：<source archive | tool bundle | domain evidence | none>；发布仅在受保护 tag/批准工作流后，不自动发布。

## 4. 变更与工单路由

- 本仓缺陷在本仓开工单；上游公共合同/协议缺陷路由到 `<repo>`；作者面/领域语义在领域仓 ADR。
- 新仓库、新公共合同、新目录层级或破坏性语义：先在根/领域开 Issue/ADR，记录责任、消费者、退出条件。
- 跨仓组合：组件 PR+CI → 领域 PR/验收 → 根 PR/递归与冷安装证据；父级只更新固定 gitlink 与证据。
