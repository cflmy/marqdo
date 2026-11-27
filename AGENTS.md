# Marqdo 维护指引

1. 从 [各仓库任务](docs/REPOSITORY-TASKS.md)、[组件目录](components.toml) 与 [架构边界](docs/ARCHITECTURE.md) 确定**唯一**责任仓库：功能缺陷在功能仓库，领域合同/组合在领域聚合仓库，gitlink、跨领域集成、发行组合与治理在主仓库。主仓库和聚合仓库不实现子组件业务。
2. 治理树最多两级：根 → 领域聚合（`kind = "domain"`）→ 功能组件。每个组件只有一个父级，全局 ID 与权威远端唯一，领域聚合仓库只做治理、合同、示例和组合验收。已发布的 `planned` 仓库可以挂载真实 gitlink。激活时登记实际 `entry`，只有通过验收的 active 组件计入完成率。
3. 找到或建立责任仓库的工单，按 `.gitea/ISSUE_TEMPLATE/change.md` 记录目标、当前提交、最小复现、公共契约、影响消费者和验收命令。跨仓库阻塞在上游开关联工单，并在消费者记录回归条件。不要按错误最终显示的仓库定责，找第一个违反公开合同的所有者。
4. 在责任仓库独立修改、构建和测试。需要调整公共语义时，在 CLI 提交 ADR；调整 Web 作者面语义时，只在 `marqdo-ext-web` 的 [WEB-DESIGN 合同](https://git.cflmy.cn/cflmy/marqdo-ext-web/src/branch/main/docs/WEB-DESIGN.md) 走 ADR，并与实际消费者（Design/Preview）同候选验证；不得并列维护第二份现行规范或旧协议 fallback。
5. 逐层晋升：功能提交 C2 通过组件 CI → 领域 PR 更新 gitlink 至 C2 并通过领域 CI（B2）→ 主仓库 PR 更新领域 gitlink 至 B2 并通过根递归校验与受影响组合验收（A2）。先推送子仓库提交，再在主仓库提交精确 gitlink；区分组件 CI、公共契约 CI 和生态 CI 证据。以 [开发流程](docs/DEVELOPMENT.md) 为准。
6. 新接入组件先核对公开 API、直接依赖、独立入口和实际测试。空仓在 [迁移台账](docs/MIGRATION.md) 中保持 `planned`；Web 领域的 Design/Preview 子组件记录在 `marqdo-ext-web` 自己的 `components.toml`，根目录不登记其后代路径。
7. 记录 FAIL 和 UNVERIFIED 的具体环境、命令和日志。只有实际通过且匹配候选 SHA 的证据才可用于集成与发行；模拟、mock 或原型测试不能冒充生产验收。
8. Web 边界硬约束：不新增 CLI 核心语法或 Web 专属关键字；不复制 Markdown/语言解析器；默认不允许 HTML 注入、业务 JS 或真实网络/数据库/认证；预览是有限确定性解释，生产实现由成熟框架在目标应用仓库承担。

## 生态仓库开发规范（防膨胀，强制）

本文件与 [docs/REPOSITORY-CONVENTIONS.md](docs/REPOSITORY-CONVENTIONS.md) 共同约束所有仓库；后者是仓库组织、准入、最低文件集、AGENTS 模板、CI/发布/晋升与防膨胀清单的详细规范。做任何跨仓、建仓、激活、拆目录、加合同/清单/发布线之前必须先读它。

- **禁止膨胀**：不为单个功能/页面/算法/实验建仓；不建空壳/镜像/docs-only 仓；治理树最多两级 gitlink；禁止第三级、重复 catalog、第二套 contract/manifest/lock/parser/发布服务。
- **每个 active 仓最低集**：README、AGENTS.md（含 `<!-- marqdo:repo-conventions v1 -->` 与根规范链接）、真实 `entry`、正反例、可复现证据、独立 CI；planned 仓库不登记 entry、不批量生成模板、不计完成率，激活时补齐。
- **单一事实源**：源码组合=gitlink+.gitmodules；职责/依赖=catalog；依赖=各仓锁/manifest；发布=Gitea tag/资产；CI=真实 pipeline；推进=Issue/PR。不手工维护“最新版本表”。
- **四种事实分开**：组件 CI 通过、release 发布、消费者接入、父级接受互不推导。晋升固定顺序 C2（组件）→ B2（领域）→ A2（根），先推子仓再更 gitlink，父级 PR 不夹带子仓源码。
- **证据纪律**：记录 SHA、制品摘要、平台、命令、退出码、日志范围；未运行写 `UNVERIFIED`；不伪造 entry/版本/CI/PASS/发布资产；不提交密钥或构建产物。
- **检查入口**：`scripts/check-repo-conventions.py`、`scripts/check-layout.py --checkouts`、`tests/integration`；CI layout 阶段强制执行，规则变更必须同步测试与模板。
