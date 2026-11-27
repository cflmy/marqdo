# 仓库开发规范内化与防膨胀门禁（2026-11-27）

来源：[marqdo#10](https://git.cflmy.cn/cflmy/marqdo/issues/10)（生态独立 CI/CD、制品发布与跨仓库传播实施规范）中的职责模型、独立 CI 合同、制品/发布事务、晋升与对账要求，经与现有 ARCHITECTURE/DEVELOPMENT/REPOSITORY-TASKS/MIGRATION 对齐后整理为仓库级规范。

## 落地内容

| 类型 | 文件/工具 |
| --- | --- |
| 根规范 | `docs/REPOSITORY-CONVENTIONS.md`：仓库准入、两层治理、职责矩阵、最低文件集、AGENTS 规范、独立 CI/CD、四种事实与 C2→B2→A2 晋升、单一事实源、防膨胀硬禁止清单、审核清单、例外记录 |
| 根约束 | `AGENTS.md` 增加“生态仓库开发规范（防膨胀，强制）”章节与规范链接 |
| 模板 | `docs/templates/AGENTS.component.md`（含 `<!-- marqdo:repo-conventions v1 -->` 标记） |
| 门禁 | `scripts/check-repo-conventions.py`：校验 active 仓 README/AGENTS 标记与链接/真实 entry、planned 不登记 entry、根 catalog 不重复登记领域后代、叶子仓无第二 catalog/.gitmodules、根 .gitmodules 权威远端 |
| 测试 | `tests/integration/test_repo_conventions.py`（6 个正反例），并纳入根 `.woodpecker/eco.yml` layout 阶段 |
| 子仓规范 | 每个 active 仓与已迁入实现的 extension 仓新增/更新 `AGENTS.md`，只写本仓边界并链接根规范 |

## 已传播的仓库（各自 PR 合并且按 fast-forward 合并）

| 仓库 | 规范提交 |
| --- | --- |
| marqdo-cli（根 gitlink，docs-only） | `13affa76dd8aa15d54ac89d580d5e60bff692384` |
| marqdo-extension-conformance | `665c12e9b1c116121cbed82c9014db208587dd58` |
| marqdo-view | `48e530935880989d95b54f81af0944a031543d65` |
| marqdo-knowledge | `6e881000898b0130a4d783c285d8e2967f8d7a0b` |
| marqdo-prove | `e9fce68551726a992167669d76c0bc1f9f0e4d14` |
| marqdo-lint | `aa0bdac7de9ed5aaa96c660657770a6f6dd90ed2` |
| marqdo-std-collections | `f18eec7ada16c5a28dd4f58daa5b1b7d2089b65d` |
| marqdo-std-text | `5b5d3b67c3c114a305a08e3a90c0d3ae008f7c34` |
| marqdo-std-data | `6f79b1345da0444418f5e9cd5a226d3df2a2e2bb` |
| marqdo-std-io | `02d1ad172c077674b6a0c3d0bd7a61092b278b10` |
| marqdo-std-fs | `104741bca034f8ae537389fef7c75c10eaed5b8c` |
| marqdo-std-system | `1561a6b7b6a2eb1f249976aaf6e05941d2db5296` |
| marqdo-ext-http | `6efcbc8c8cf90d4d5da36e923c0df0ac24a416a6` |
| marqdo-ext-regex | `23f9417809fae669bf8e690d56637be8f86fd27a` |
| marqdo-ext-archive | `86dcca0881c52e87cb3671e8cdace64a46f60c85` |
| marqdo-ext-web（领域，含 Design/Preview 子 gitlink 更新） | `70ba4a27e592edab74348e599b6d3882dcdfcbaf` |
| marqdo-web-design | `cbef7b622a96cd7057bd03cb206fe67089843dc4` |
| marqdo-web-preview | `86a260c64634eee263e19e7620f56dd2a86330f8` |

- Design/Preview 的 stale `planned` 声明同步修正为 active，Web 领域 README/AGENTS 的组件状态表同步修正；`web-components` 仍 planned。
- 根候选把上述子仓提交固定进 gitlink；本地 `check-repo-conventions.py` PASS（37 components）、`check-layout.py --checkouts` PASS、完整集成测试 **85 tests OK**；根 CI candidate 为最终门禁。

## 明确未做/后续

- 本 PR 只落“仓库组织/开发规范与门禁”，不实现 #10 的 CI/CD 工作流、发布索引、通知/reconcile 工作流、Woodpecker manual/cron；这些仍按 #10 逐项实施并需平台 M0 实测。
- planned 仓库（core/encoding/net、其余扩展、web-components）不批量生成 AGENTS/CI 模板；按规范在激活时补齐。
- 规范例外仍须在 Issue/ADR 记录原因、责任者、复查日期与退出条件。
