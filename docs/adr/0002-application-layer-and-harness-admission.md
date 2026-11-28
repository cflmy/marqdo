# ADR 0002：应用层（app）准入与 Marqdo Harness

- 状态：已采纳（2026-11-28），实施工单为 `marqdo#25`（Application / P0）。
- 现行权威：`docs/ARCHITECTURE.md`、`docs/REPOSITORY-CONVENTIONS.md`、`components.toml`、`.gitmodules` 与 Git tree gitlink。本 ADR 记录决策，不构成第二套规范。
- 影响仓库：`marqdo`（根治理）、`marqdo-app-harness`（新应用仓库）。CLI、Std、Ext、Tool 的现行合同不因本 ADR 改变。

## 背景

生态此前只有 `core/stdlib/extension/tool/test/domain` 六种组件角色。Marqdo Harness
（MQDH）是一个**产品消费者**：它组合已发布的 CLI、单一 Bundle、Tool 与 Ext，向最终
用户提供安装入口、文稿执行入口、私人知识库和成果复用。它既不发布语言/运行时合同，
也不被任何基础组件依赖。直接复用 `tool` 或 `extension` 会把产品应用错误地放进
`requires` 图，导致 CLI 基础发行被应用反向阻塞。

## 决策

### 1. 新增根层 `kind = "app"`

- 只允许出现在根治理层（`depth == 0`），不允许登记在领域聚合仓库内。
- ID 固定为 `app-<name>`，路径固定为 `apps/<name>`，权威远端固定为
  `https://git.cflmy.cn/cflmy/marqdo-<id>.git`（即 `marqdo-app-harness.git`）。
- `release = "optional"`：应用不属于 CLI 基础发行，不进入 `base` 装配。
- 应用是根 leaf：仓库自身不得再登记 `components.toml`、`.gitmodules` 或任何
  gitlink。父仓只固定其提交，不读取其内部子模块。
- `requires` 只能指向 active 的 `core/stdlib/extension/tool` 叶组件；不得指向
  `domain`、`test` 或另一个 `app`。任何其它 kind 都不得以 app 为依赖目标。
- `checks_with` 仍只表示协同验收，不是安装依赖。
- 首期只登记一个应用，不建立 `marqdo-app` 聚合仓库；未来第二个独立应用出现时
  再以新 ADR 评估是否聚合。

### 2. `app-harness` 的上游与产品身份

- 应用仓库为 `cflmy/marqdo-app-harness`，路径 `apps/harness`。
- 保留 DSH 的 Git 历史、MIT `LICENSE` 与第三方声明；`origin` 为 Gitea 权威远端，
  `upstream` 为 `https://git.cflmy.cn/cflmy/deepseek-harness.git`（DSH 权威镜像）。
  GitHub 官方仓库只作为审计与回馈来源，不进入父仓 `.gitmodules`。
- 发布名、包名（`@marqdo/*`）、命令（`mqdh`）、应用标识和发行地址与 DeepSeek 官方
  DSH 完全分离；不得复用官方上传工作流或凭证。
- 首版支持平台以真实证据为准：Linux x86_64 / glibc ≥ 2.39；Node 24.22.3、
  pnpm 11.28.5（由选定 DSH 提交与锁文件决定）。Web 默认 loopback。

### 3. 产品装配

- 唯一正式产品入口为 `mqdh` 启动器，唯一产品 profile 为 `mqdh`，组合为
  `dsh-base` + `dsh-web-app` + 下游 `@marqdo/mqdh-bundle`。
- 下游产品包集中在 fork 的 `packages/marqdo/*`，不散改上游包；必须修改的上游文件
  逐项记录在 `docs/mqdh/upstream-delta.md`。
- DSH 的 Node PTC 模型入口在产品 profile 中关闭；面向模型的文稿执行只经由
  `marqdo_program` 一条组合入口。上游 PTC 源码保留用于同步与基线，不作为第二个
  用户可选执行模式。
- 应用运行数据使用独立 `MQDH_HOME`；DSH 会话放在 `MQDH_HOME/harness`，不覆盖用户
  已有的 DSH 数据。

### 4. 晋升门槛

1. M0：登记 planned 条目并固定真实初始提交；`app-harness` 仍不计完成率，不登记
   `entry`。
2. MQDH-01..09 完成应用自身实现、独立 CI、冷安装与真实候选证据后，应用仓库登记
   `entry = "scripts/ci.sh"` 并推送。
3. MQDH-10：根 PR 只更新 gitlink、catalog、集成调度与证据；应用加入 `requires` 的
   组件必须已 active。根 Node lane 使用与 CLI/Bundle/Tool 相同的候选身份运行真实
   应用验收。
4. 应用激活前不得成为 CLI 独立发行的前置；应用失败不得阻断合法的 CLI 基础发行，
   但包含 app 的根生态候选必须如实记录失败，禁止静默跳过。

## 后果

- `scripts/check-layout.py` 增加 app kind、路径/远端/依赖方向校验与正反例；
  `kinds` 与 `release` 的既有约束保持不变。
- `scripts/integration.sh` 增加显式 app 分派，传入真实 CLI/Bundle/Tool 候选身份；
  未知 active 项继续失败。
- `docs/ARCHITECTURE.md`、`docs/REPOSITORY-TASKS.md`、`docs/REPOSITORY-CONVENTIONS.md`
  登记应用层职责与边界；根 README 给出应用入口。
- 应用仓库必须遵守仓库规范最低集（README、AGENTS、`scripts/ci.sh`、正反例、
  独立 CI、证据），并在 `AGENTS.md` 保留 `marqdo:repo-conventions v1` 标记。

## 非目标

公共知识共享插件、知识市场、多租户/云托管、桌面签名、独立 TUI、完整 host-binding
PTC 与自动发布用户成果均不在本 ADR 与首版范围内；它们需要独立工单与 ADR。
