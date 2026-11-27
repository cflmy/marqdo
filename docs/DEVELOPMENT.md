# 开发、验收与发行

生态采用**逐层候选晋升**：功能仓库修实现，领域聚合仓库固定组合并做领域验收，主仓库固定整个递归树并做生态验收。任何一层失败都停在该层，不让未验证组合进入受保护的分支；回退根 gitlink 即可复现上一个已接受组合，不重写子仓库历史。

## 责任仓库与工单

1. 先确定唯一责任仓库：功能行为在功能仓库；领域合同、示例与组合在领域聚合仓库；gitlink、跨领域集成、发行组合与治理在主仓库。拿不准时按“第一个违反公开合同的所有者”定责，而不是按错误最后显示的仓库。
2. 在责任仓库找到或建立工单，按 `.gitea/ISSUE_TEMPLATE/change.md` 记录：责任组件与类型、影响消费者、根/领域/组件 SHA、CLI 与依赖制品摘要及平台、最小 fixture 与命令、实际与预期结果、权威合同链接、修复范围、验收命令。
3. 跨仓库阻塞在上游仓库开工单，并在消费者工单记录回归条件；不同问题拆分，重复发现追加证据。任何维护者或 AI 都可以读其它仓库定位，但修改必须进入对应责任仓库的独立提交。

## 分层候选流程

以 Web 领域为例，一般组件同理：

```text
功能提交 C2（功能仓库，通过组件 CI）
  -> 领域 PR 将 gitlink 更新到 C2，通过领域 CI，形成 B2
  -> 主仓库 PR 将领域 gitlink 更新到 B2，通过根递归校验与受影响组合验收，形成 A2
```

- 子模块的 detached HEAD 只用于复现与集成；开发应在子仓库独立创建分支后推送，禁止 `git submodule update --remote` 在发布/验收中追踪浮动分支。
- 同一份公共合同切换需要多个组件同步变化时，在一个候选组合中提交整组 gitlink，配对候选联调通过后再晋升；不允许主分支出现半套协议。
- 领域激活（planned → active）的门禁：两个功能仓具备真实 `entry` 与候选提交，领域 `components.toml` 与 gitlink 一致，领域 CI 和根递归校验通过，Design/Preview 等有依赖关系的组件获得同候选证据。已发布的开发提交可先作为 `planned` 子模块固定。

## 递归校验与组合检查

```sh
git submodule update --init --recursive
python3 scripts/check-layout.py            # 根层：catalog/gitlink/远端/白名单
python3 scripts/check-layout.py --checkouts # 递归：各 active 领域固定提交、全局图与依赖
python3 -m unittest discover -s tests/integration
bash scripts/integration.sh                # 生态候选：真实 CLI 制品 + 组件验收
```

- 校验器读取 Git tree 中固定提交的 catalog，不以工作区改动替代候选；`--checkouts` 需要子模块已初始化，缺失、错误 HEAD、脏工作区、错误入口都明确 FAIL/UNVERIFIED，不静默跳过。CI 先验证并检出直属子模块，再用 `--domain-paths` 校验领域的子模块远端，最后逐个领域检出直属子库。
- 没有 active 领域时，新校验规则同样接受合法的直属功能组件，这不是历史协议兼容。
- 未知的 active 入口、未声明的 gitlink、目录与 catalog 不一致都必须让候选失败。`planned` 不进入验收计数。
- 根 `eco` 在 PR、`main` 推送和显式手动运行时执行。PR 始终保留 `layout` 状态。仅改动根 `README.md` 或 `docs/evidence/` 下的 Markdown 记录时，自动运行只执行轻量结构校验和脚本测试。混合改动、其他路径、缺失变更列表和手动运行都执行递归检出及完整生态验收。

## 三层验收

| 层 | 责任与证据 |
| --- | --- |
| C1 组件 CI | 责任组件独立检出、独立构建与测试、授权负例、固定依赖、打包 smoke；由组件仓库维护 |
| C2 公共契约 | 真实 CLI 和 Worker、独立 Conformance 与消费者，覆盖当前协议、失败和正确身份 |
| C3 生态集成 | 递归结构校验、公开冷克隆、精确 gitlink、官方基础发行、CLI Tool 与契约消费回归 |

三层不是重复跑三次测试：同一候选、同一制品、同一环境的有效证据可以复用；身份不匹配必须重跑。每个 active 组件提供独立验收入口（建议 `scripts/ci.sh` 或 `scripts/integration.sh`），从显式传入的 `MARQDO_BIN`、Bundle/依赖制品位置和隔离输出目录读取输入，不自行拉取浮动 CLI 或其它组件 main。

## 统一 CLI 与制品输入

- 生态候选使用**固定、可验证**的 CLI/Worker/Stdlib 候选制品；输入必须绑定 CLI gitlink SHA、目标平台、归档/二进制/Bundle digest、工具链与锁。
- 标准库 Bundle 的唯一生产者是根仓库的 `scripts/build-stdlib.py`：它从根提交的精确组件 gitlink 读取各 active `kind = "stdlib"` 组件的 `modules/`，校验发布路径（只发布 `.mq.md`、拒绝 symlink/子模块/大小写与 NFC 冲突、`tests/` 不入库），生成现行 `stdlib.json`（tree digest + aliases digest），配置在 `scripts/stdlib-bundle.json`。版本变化必须显式（`--baseline-manifest` 拒绝同版本不同摘要）。组件仓继续拥有源码与测试，根不复制模块。
- 组合验收入口是根仓库的 `scripts/acceptance-stdlib.py`：只接受显式 `--cli` 与 `--bundle`（或按根 gitlink 构建），运行各 active 组件的 `entry`、跨仓组合 fixture（别名、Table 切分、默认 Bundle smoke）和必须失败的负例；提供 `--pack-tool` 时用 CLI 自己的打包器做两次确定性打包并核对摘要。CLI 自身打包请使用 Phase A 之后的 `--stdlib-dir` 输入，不再默认读取 CLI 源码树。
- 统一制品输入已落地（ECO-02 核心）：`scripts/integration.sh` 不再从 View 的冻结脚本取 CLI 制品，而是由同一根候选派生全部输入——根唯一生产者 `scripts/build-stdlib.py` 按固定 gitlink 生成 Bundle；`cli/` 固定提交构建 CLI 与 Worker；`cli/tools/release/package.py --stdlib-dir <Bundle>` 生成发行归档；随后 Conformance、View、Knowledge、Prove、Lint 与根 `acceptance-stdlib.py` 全部使用同一安装后二进制与相邻 Bundle。任何消费者都不能决定 CLI 版本，也不再有浮动或隐式制品。
- 组件验收只读取传入制品，不自行下载未知版本；开发目录覆盖入口（如 `MARQDO_STDLIB_DIR`）不得当作发行行为。缺失、摘要不符或平台不符必须拒绝。

## 影响范围与调度

根 `scripts/impact.py` 比较已接受根提交与候选递归展开的 `(global_id, parent, remote, SHA, state)`：领域指针变化展开到子组件身份，不能只标记父节点；变更叶子加入集合后，取 `requires` 新旧图并集的反向传递依赖闭包，再加所在祖先领域及 `checks_with` 固定点（协同验收不是构建先决）。领域无法展开（缺对象/检出）时结果标 UNVERIFIED 并**全量**扩大范围，不静默跳过；`--fail-on-unverified` 可把该情形变成硬失败。`integration.sh` 将报告写入候选证据（changed/affected/checks_with/fallback），正式发行仍执行所有 active 组件与完整测试。

## 工单路由与状态

| 现象 | 责任归属 |
| --- | --- |
| 组件表头正确但 Design 未识别 | web-design（若 Analysis 的真实 AST 正确） |
| Link 中 `not_loaded` 错误处理 | web-design；若 CLI candidate 违反公开 resolver 合同则 CLI |
| 手机布局溢出、场景实例串扰 | web-preview |
| 特定设计模板不符合合同 | web-components（激活后）或作者项目，不修改核心 renderer 迎合坏模板 |
| Web 设计约定出现冲突 | marqdo-ext-web 协调，Design/Preview 各自提交 |
| 真实生产框架渲染/业务错误 | 目标应用仓库，不归 Web 原型 |
| SSE 或网络传输错误 | Ext-Net 的 HTTP/WS 或 CLI Resource 真正责任方 |
| 固定 SHA 组合构建失败 | 若各组件公开制品正确，由所属聚合/根修复装配，否则责任组件 |

状态流：`open → triaged → in-progress / blocked → component-verified → integrated`。可用现有 Git 平台标签与关联链接，不设计新工单数据库；阻塞项不能以 mock PASS 关闭。组件 PR 合并后可关闭组件修复工单，但消费者/领域或生态集成任务保留到真实候选组合通过。

## 自动化权限

先实现人工触发与 PR/CI 门禁，不开发自动 Agent 管理平台。候选通知的授权校验由根 `scripts/candidate-notify.py` 承担：只接受规范化通知（仓库、类型、head/base SHA、CI 状态与 URL），校验仓库在根图远端允许列表内、提交在本地对象库可达、组件候选的 base 必须是 accepted 根图当前固定的 SHA、CI 为允许主机上的 success 且必须是 https；按 `(repository, head SHA)` 去重，拒绝重放，accepted 基线变化时标记 `revalidate` 并要求重跑门禁；影响范围复用 `scripts/impact.py`，无法确定时全量。校验器只输出计划（`auto_promote=false`、`auto_merge=false`、仅创建候选分支需人工评审），不合并、不推送、不扩权。Webhook 只作通知（记录仓库、完整 SHA、CI URL）；上级通过 API 验证存在性、审批与权限后再创建候选 PR，不得自动合入或提升权限。

经过授权的 webhook 入口由根 `scripts/webhook-adapter.py` 实现：先以仓库 webhook secret 校验 `X-Gitea-Signature`（HMAC-SHA256）原始 body，再只接受 `push`（非删除）与 `pull_request`（opened/synchronized/reopened）事件；仓库必须来自 accepted 根图的远端允许列表，base 使用 accepted 图当前固定 SHA（payload base 只作 hint），CI 聚合要求至少一个状态且全部 success、URL 为允许 CI 主机上的 https。校验通过后交给同一 `candidate-notify.py` 做可达性/去重/重放/基线重验与影响分析，输出计划（不合并、不推送、不建 PR）。通知丢失的恢复由同一脚本 `--reconcile` 承担：扫描允许列表仓库的开放 PR 与 head statuses，对未入账的成功候选执行同一验证（幂等状态文件，重复运行只报 duplicate）；API transient 错误有限退避重试，401/403 立即失败，不把触发成功当作下游已处理。外部不可信 PR、未批准 gitlink 或脚本不得在拥有跨仓库写权限/发版密钥的 runner 中运行；区分只读 checkout/test token、受限 issue/PR token、受保护发布权限。日志不得输出凭证、完整 Analysis 或生产 API key。

正式发布对账使用 `python3 scripts/reconcile-releases.py --plan --published-after <ISO-8601 时间>`。从固定根及领域 tree 解析受管仓库，核对 Gitea tag、`release-evidence.json`、同 SHA Woodpecker 组件 CI 和资产字节摘要。根 `reconcile` 工作流从当前受保护的 `main` 串行运行，使用 `gitea_reconcile_token` 以专用非管理员账号写追踪工单。`historical-baseline` 表示已登记历史输入，`blocked` 表示发布验证未通过，`tracked` 表示已记录经验证的发布。消费者验收和父级接受均需各自的 CI 与 PR 证据。

对已验证的发布，`--plan` 计算直接 active 消费者的输入摘要。受保护 `--apply` 按发布和消费者登记一条任务，基线变化时在原任务中记录新输入。输入附精确资产、依赖、消费者基线和根追踪编号。下游独立 CI 及批准后的晋升基于各自的真实结果。

## 发行与回滚

- 组件可独立 SemVer 发布；领域可发布经过验证的组合制品；根生态 release 固定整个 gitlink 树。独立版本不代表维护多套现行协议。
- 每份证据记录根及递归组件 SHA、制品摘要、真实命令、退出码、环境/工具链、日志、测试范围与未验证项。复用现有 manifest/evidence 模式，不复制成新的权威锁。
- Git tree 是源码身份权威；Cargo.lock/`marqdo.lock`/现行 Bundle manifest 是对应依赖权威。记录不可变归档校验和、可信来源及获取方法；摘要校验不是发布者身份认证。
- 基础发行包含同源 CLI、必需 Worker 与单一标准库 Bundle；不使用浮动 branch/标签名作为发布依据。发布失败保留日志，使用新候选；已有标签和资产保持不可变。回滚用新集成提交恢复上一组通过的 gitlink。

## Web 领域开发约束

- 唯一作者合同是 `marqdo-ext-web` 的 `docs/WEB-DESIGN.md`；Design 是唯一解释实现，Preview 不另加解析规则。合同变更在该领域走 ADR，并让 Design/Preview 以配对候选一次切换。
- Design 只消费公开 CLI `check --emit` 的 Analysis 3，不复制 Markdown/语言解析器、不依赖 CLI 私有 Rust crate；Preview 只消费 Design 的公开 crate/固定 revision，`,...` 不通过 `../design` 长期路径依赖，最终发布以 `Cargo.lock` 固定真实依赖身份。
- 首版不迁移 WASM、不新增 CLI 语义、不要求旧 plugin 回调；模拟状态、原型标识与“未实现真实功能”必须对作者可见。
- 真实验收使用真实浏览器 `file://` 制品、真实 CLI 正负例与真实冷安装；原型测试 PASS 不能当作生产测试 PASS。

## 本地环境

需要 Linux x86_64/glibc ≥ 2.39、Python 3.11+、Git、Rust、C 编译器和 curl；`integration.sh` 另需 jsonschema 与冻结 CLI 归档。详细命令与组件验收入口见各仓库 `README.md` 与 [各仓库任务](REPOSITORY-TASKS.md)。

## 迁移台账

标准库与扩展的现状、历史归属和切换顺序以 [迁移台账](MIGRATION.md) 为准。当前 CLI 内置标准库仍是基础发行的现行权威；`marqdo-stdlib` 聚合与九个功能仓库的迁移完成前，不得声称 Stdlib 拆分完成，也不得先删旧实现造成产品回归。
