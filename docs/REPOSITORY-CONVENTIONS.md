# 仓库开发规范（生态级）

本文件把 [marqdo#10](https://git.cflmy.cn/cflmy/marqdo/issues/10) 与既有架构/开发/迁移文档中的仓库组织要求整理为**可执行的仓库规范**，供主仓库、领域聚合仓和功能仓共同遵守。它不新建第二套权威：仓库身份仍以 `components.toml` + Git tree/gitlink + `.gitmodules` 为准，合同仍以各仓现行规范为准，流程细节以 [DEVELOPMENT.md](DEVELOPMENT.md)、[ARCHITECTURE.md](ARCHITECTURE.md)、[REPOSITORY-TASKS.md](REPOSITORY-TASKS.md)、[MIGRATION.md](MIGRATION.md) 为准。

## 1. 目的：防膨胀，而不是少文件

建立子仓库的唯一理由是形成**明确职责、独立开发闭环和可追溯协作**；不是为了减少单仓文件数量。任何新增仓库、目录层级、合同、清单、构建入口或发布线都必须证明：

1. 存在唯一责任所有者，且没有现成仓库能承担；
2. 有独立构建/测试/交付或独立合同边界，而不是"按功能/按文件"拆分；
3. 至少一个真实消费者或明确的默认发行需求（新增标准库还须满足 ARCHITECTURE 的准入门槛）；
4. 有可验证的激活路径（entry、测试、证据、CI），不产生空壳仓库；
5. 不引入第二套 parser、manifest、catalog、lock、版本表、制品服务或发布流程。

**治理树最多两级 gitlink**：根 → 领域聚合（可选）→ 功能组件。禁止第三级；禁止用目录复制模拟仓库；禁止为单个功能、页面、算法、表格或实验创建仓库。

## 2. 仓库准入与状态

- 新仓库必须先在根仓库开 Issue/ADR，记录责任、消费者、退出条件，并在 `components.toml` 登记 `planned`；不得先建空仓再补理由。
- `planned` 仓库允许挂载真实开发提交，但**不计完成率**、不登记 `entry`、不要求发布制品；只能有 README/必要源码/最小测试，不得批量复制模板、CI 徽章、占位实现或假入口。
- 从 `planned` 激活必须同时具备：真实实现、README（职责/构建/依赖/平台）、`AGENTS.md`（本仓边界与路由）、可执行 CI 入口、真实正反例、交付/发布方式、候选证据；并在父级 catalog 登记 `entry`。
- 只有规范/fixture 的仓库允许源码归档交付，不强制二进制；但必须有确定性归档与证据。
- 应用（`kind = "app"`）只允许在根层登记，ID 为 `app-<name>`、路径为 `apps/<name>`、`release = "optional"`。应用依赖 active 叶组件，不能被其它组件依赖；激活时必须登记真实 `entry` 与独立 CI。
- `active` 仓库不得回退为"文档占位"；若失去责任，必须记录替代仓库与迁移/归档计划。

## 3. 角色与禁止事项

| 所有者 | 应做 | 禁止 |
| --- | --- | --- |
| 功能仓库（core/stdlib/extension/tool/test） | 自身实现、测试、制品、发布、消费上游候选、修复本仓缺陷 | 实现其它仓库功能；依赖父仓或兄弟仓 checkout；复制其它仓实现/parser/合同；偷偷替换上游逻辑 |
| 应用仓库（app） | 下游产品装配、生态安装/更新、文稿执行与知识复用；独立 CI/发行 | 发布公共语言/运行时合同；复制生态生产源码；被基础组件依赖 |
| 领域聚合仓（`kind=domain`） | 领域合同、组合、领域候选 gitlink、领域验收、通知父仓 | 重复叶子实现；第二套调用协议；登记后代到根 catalog |
| 应用仓（`kind=app`） | 产品装配、生态安装、文稿执行、私人知识与 UI；消费公开合同 | 复制 CLI/stdlib/ext/tool 源码；成为其它组件依赖；进入 base 发行；自建第二套解析器/锁 |
| 根仓库 | 展开依赖、传播协调、全局候选、递归集成、正式发行、滞后报告 | 实现子组件业务；批量直接改子仓代码；成为每个仓的构建前置 |
| CLI | 语言/通用能力、公开协议、CLI/Worker/基础产品打包 | 生态调度代码；反向依赖 stdlib 源码；物理拆仓 |
| Conformance | 独立公共契约运行验收 | 取代组件自测；伪造未支持平台的 PASS |

## 4. 每个 active 仓库的最低文件集

```text
<repo>/
├── README.md                 # 职责、构建、测试、依赖、支持平台、状态
├── AGENTS.md                 # 本仓边界、禁止项、输入/输出、工单路由（含规范标记）
├── scripts/ci.sh             # 非交互构建+检查+真实功能测试；失败非零；不打印秘密
├── scripts/package.sh        # 有制品时：生成候选并验证冷使用
├── scripts/upstream.sh       # 有外部依赖时：消费候选/生成最小依赖更新
├── tests/ 或 fixtures/       # 真实正反例
├── .woodpecker/              # 独立远程 CI；发布门禁与普通 CI 分离
└── docs/ 或 spec/            # 公开接口/合同/证据
```

允许复用现有脚本名（如 `scripts/test.sh`、`scripts/acceptance.sh`、`scripts/package-tool.sh`），但 `entry` 必须真实存在、可本地执行、非交互、默认无网络写副作用。构建产物、缓存、日志只写声明输出（如 `MARQDO_CI_OUT`），不得污染源码树；不得提交 `target/`、`dist/`、浏览器产物、凭证或本地绝对路径。

## 5. `AGENTS.md` 规范

每个 active 仓库必须有 `AGENTS.md`，以 `<!-- marqdo:repo-conventions v1 -->` 开头，并至少包含：

1. 本仓定位、catalog ID、父级、唯一权威合同/输入；
2. 必做/禁止边界（不复制、不越界、不建第二目录/清单/合同/发布线）；
3. 构建/测试入口、实际依赖输入、证据要求（SHA、摘要、平台、命令、退出码、日志；未运行 = UNVERIFIED）；
4. 工单路由（上游合同缺陷去哪、跨仓组合如何晋升）；
5. 指向根规范：`https://git.cflmy.cn/cflmy/marqdo/src/branch/main/docs/REPOSITORY-CONVENTIONS.md`。

模板见 [templates/AGENTS.component.md](templates/AGENTS.component.md)。`AGENTS.md` 只写本仓边界，不复制全局规范正文；合同正文仍只在唯一权威仓维护。

## 6. 独立 CI/CD 与证据

- 每个 active 功能仓必须能在**全新临时目录独立 clone**、无父仓、无兄弟源码、无隐式 PATH/缓存的情况下完成声明的构建与测试；可从权威远端下载固定 SHA/摘要的依赖，冷安装阶段禁网。
- 依赖 CLI 的组件用绝对 `MARQDO_BIN` 并验证相应 `SOURCE_SHA`/制品；需要外部 Bundle 时用显式 `MARQDO_STDLIB_DIR`，不得用它替代最终发行默认 Bundle 检查。
- 工具链、镜像、锁文件、target、CI 脚本变化必须进入缓存 key；cache hit 不是 PASS；测试脚本与持有发布密钥的脚本分离。
- 普通 push/PR 只做检查与回归；发布仅消费**准确同 SHA** 的受保护 tag/批准工作流；不得用历史 PASS、浮动分支或 tag 名代替。
- 正式发布必须记录：tag→完整 source SHA、真实 CI 证据、资产 basename/size/sha256、实际依赖身份、平台。发布后通知失败只重试通知，不重复发布或覆盖已公开资产。

## 7. 四种事实与晋升

必须分别记录，禁止推导：

1. 组件提交已通过本仓 CI；
2. 组件 release 资产已完整发布；
3. 消费者已对特定候选组合验证/接入；
4. 领域或根主分支/正式发行已接受该组合。

晋升顺序：组件提交 C2（组件 PR + 组件 CI）→ 领域提交 B2（领域 PR 更新子 gitlink + 领域验收）→ 根提交 A2（根 PR 更新领域 gitlink + 递归布局/受影响组合/冷安装）。先推送子仓提交，再更新父级 gitlink；父级 PR 只改计划中的 gitlink/catalog/证据，不夹带子仓源码。禁止自动合并、自动提升或把通知成功当作下游已处理。

## 8. 单一事实源

| 信息 | 唯一来源 |
| --- | --- |
| 源码组合 | 根 Git tree 的 gitlink（递归）+ `.gitmodules` 权威远端 |
| 仓库职责/依赖/影响 | 各级 `components.toml` 的 `kind/state/entry/requires/checks_with` |
| 组件真实依赖 | 各仓 Cargo.lock/Cargo.toml、`marqdo.yaml`/`marqdo.lock`、现行 Tool/Bundle manifest |
| 发布身份与资产 | 权威 Gitea Release/tag、发布证据与 SHA-256 |
| CI 结果 | Woodpecker 真实 pipeline/日志/同 SHA 证据 |
| 推进/缺陷/搁置 | Gitea Issue/PR/评论 |

禁止手工维护"全生态最新版本表"、第二套 catalog、重复 contract、复制 lock/manifest、把组件 SHA 写进 catalog。报告、影响分析、候选计划、发布索引只做派生视图，且不成为新锁。

## 9. 硬禁止（防膨胀清单）

- 不因文件多、模块多、页面多、功能多而拆仓；不为单个功能/实验/平台建仓。
- 不建空壳仓、占位仓、镜像仓、docs-only 副本、重复服务或第二套包管理/制品服务/消息队列/Issue 数据库。
- 不复制 Markdown/语言/表格 parser 或其它仓实现；不并列维护第二份现行合同。
- 不在根/领域仓库实现子组件业务；不把子仓源码复制到父仓。
- 不让子仓依赖父仓 checkout、兄弟仓相对路径或 `../design` 类发布依赖。
- 不新增未在 catalog 的路径/仓库层级；不在根 catalog 登记领域后代。
- 不伪造 entry、CI 徽章、版本、gitlink、PASS 或发布资产；未运行写 UNVERIFIED。
- 不自动 tag/release/合并；不覆盖已公开 tag/资产；不使用未审查分支发布。
- 不提交密钥、凭证、完整 Analysis、生产配置或本地绝对路径。
- 不为 planned 仓库批量生成模板/脚本；激活时按最低集补齐。

## 10. 审核清单

**新仓/激活**：唯一职责？现有仓为何不能承担？谁消费？entry/测试/证据/发布方式齐全？planned 未计入完成率？根 catalog 与 `.gitmodules` 一致？

**子仓 PR**：只改本仓？未复制/越界？合同变更在唯一权威仓？依赖以固定身份验证？证据可复现？没有假 PASS/未验证声明？

**领域/根晋升**：只改计划 gitlink/catalog/证据？固定完整 SHA？递归布局通过？受影响消费者同候选通过？冷安装/回滚已记录？

**发布**：tag 指向冻结 SHA？同 SHA CI 真实成功？资产可下载且摘要一致？旧资产未被覆盖？通知与后续接入有追踪？

## 11. 工具与检查

- `python3 scripts/check-repo-conventions.py`：校验 active 仓最低文件集/AGENTS 标记/entry/catalog 一致性/无后代重复登记/叶子仓无子模块/无第二 catalog。
- `python3 scripts/check-layout.py --checkouts`：治理树、远端、路径、全局 ID、依赖 DAG 与固定 checkout。
- `python3 -m unittest discover -s tests/integration`：布局、影响分析、候选、Webhook/对账、仓库规范测试。
- 根 `.woodpecker/eco.yml` 的 layout 阶段执行上述检查；任何规则变更加测试，不删检查放行。

## 12. 例外与记录

任何偏离本规范的例外（例如临时 planned 实现、平台限制、无法运行的 strict/perf 门禁）必须在工单/ADR 中记录：原因、责任者、复查日期、退出条件；不得以"先这样"长期沉积。规范本身变更走根仓库 PR + Issue/ADR，并同步模板与检查器。
