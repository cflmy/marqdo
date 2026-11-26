# Marqdo 维护指引

1. 从 [各仓库任务](docs/REPOSITORY-TASKS.md)、[组件目录](components.toml) 和 [架构边界](docs/ARCHITECTURE.md) 确定唯一责任仓库。主仓库处理 gitlink、集成测试、发行组合与治理。
2. 找到或建立责任仓库的工单，记录目标、当前提交、最小复现、公共契约和验收命令。跨仓库阻塞在上游开关联工单，并在消费者记录回归条件。
3. 在责任仓库独立修改、构建和测试。需要调整语义或公共协议时，通过 CLI ADR 评审，与实际消费者同候选验证。
4. 推送子仓库提交后，再在主仓库提交精确 gitlink。区分组件 CI、公共契约 CI 和生态 CI 的证据。以 [开发流程](docs/DEVELOPMENT.md) 为准。
5. 新接入组件先核对公开 API、直接依赖、独立入口和实际测试。空仓在 [迁移台账](docs/MIGRATION.md) 中保持 planned。
6. 记录 FAIL 和 UNVERIFIED 的具体环境、命令和日志。只有实际通过且匹配候选 SHA 的证据才可用于集成与发行。
