# 开发、验收与发行

## 从工单到集成

1. 在会修改代码的责任仓库寻找或创建工单。记录复现输入、组件 SHA、平台、公共契约、预期/实际结果，以及可执行正负例。涉及其他仓库时，各仓库保留自己负责的工单，在消费者工单标明依赖和回归条件。例：HTTP 的 SSE 分块错误归 HTTP；模型事件拼接错误归 AI；握手或字节流原语错误先由标准库或 CLI 的所有者定位。
2. 在组件仓库提交 PR，运行独立 CI。公共协议改动同时验证真实产生者和消费者，附测试命令、制品摘要、平台、CI URL。组件源码审核通过后推送可由 CI 获取的精确提交。
3. 在主仓库提交集成 PR。先初始化相关组件，再以 detached HEAD 锁定审核过的 SHA 并暂存 gitlink。同一份工具契约或 Stdlib 切换需要同时更新多个组件时，在一个候选中提交整组 gitlink。
4. 基线移动时重验候选组合。留下真实失败证据并路由责任工单。通过同 SHA 的必需门禁后合并，随后关闭集成追踪及已解决的关联工单。

官网内容与托管变更在 marqdo-site 提交，编辑器行为与 VSIX 在 marqdo-vscode 提交。涉及公开 CLI 或 View 契约时沿用跨仓工单流程，消费者用实际发行制品验收。[官网仓库](https://git.cflmy.cn/cflmy/marqdo-site) · [VS Code 仓库](https://git.cflmy.cn/cflmy/marqdo-vscode)

```sh
git submodule update --init
git -C tools/view fetch origin
git -C tools/view checkout --detach <审核通过的提交>
git add tools/view
git diff --cached --submodule=log
```

## 验收

| 层 | 责任与证据 |
| --- | --- |
| C1 组件 CI | 独立构建与测试、授权负例、固定依赖、打包 smoke，由组件仓库维护 |
| C2 公共契约 | 真实 CLI 和 Worker、独立 Conformance 与消费者，覆盖当前协议、失败和正确身份 |
| C3 生态集成 | 结构、公开冷克隆、精确 gitlink、官方基础发行、CLI Tool 和契约消费回归 |

本候选先对全部 active 组件运行集成。Woodpecker 的 `layout` 校验 Git tree，随后初始化公开直连子模块。`candidate` 用官方冻结归档验证冷安装、Conformance、View、Knowledge、Prove 和 Lint。View 的集成测试包含安装、静态核验、显式执行、Observation 和移除。C1 的完整构建、View 的全浏览器矩阵、Prove 的 Lean 后端以及 CLI 的 strict VM 和性能门禁沿用各组件独立流程。

本地运行需要 Linux x86_64/glibc ≥ 2.39、Python 3.11+、jsonschema、Git、Rust、C 编译器和 curl。

```sh
git submodule update --init
python3 scripts/check-layout.py --checkouts
python3 -m unittest discover -s tests/integration
bash scripts/integration.sh
```

集成日志记录候选提交、每个 gitlink、二进制摘要及各检查退出码。对产物默认要求 digest 与来源提交一致。当前候选的冻结制品适配器以正式 RC.4 为基线。更新 CLI 基线时须同步修改其可验证制品入口及相关消费者，并同时审查源提交、锁定摘要和发布平台。

组件更多后才缩小日常影响范围。算法必须用新旧依赖图并集计算反向闭包，再加测试配对与受影响场景。CLI、Stdlib、未知边或集成策略变更保守全量测试。发布全量验证并绑定精确提交、运行环境、依赖与制品摘要。任何必需 FAIL 或 UNVERIFIED 都阻止对应阶段验收。

跨仓候选目前由维护者手动提交。自动化接入时，事件只用于唤醒，记录仓库、完整 SHA、CI URL。服务验证签名、仓库允许列表、真实 CI 结果和提交可达性，按仓库和 SHA 去重。机器人只写候选分支与 PR，维护者负责合并；凭据仅放服务端或平台 Secret，测试任务使用非特权身份。

## 发行与回滚

先验证完整候选组合，再由组件各自发布不可变制品。基础发行包含同源 CLI、Worker 与单一标准库 Bundle；工具和扩展使用各自权威制品与 CLI 的现行安装机制。主仓库发布记录应从 gitlink 和真实制品生成，包含主仓库 SHA、组件 SHA、各自版本、归档摘要、构建环境、平台和测试记录。

发布前在空 HOME、无 checkout 和禁网环境验证基础安装。每个声称支持的平台都要实机证据。失败候选保留日志；发布失败使用新的候选，已有标签和资产保持不可变。回滚用新集成提交恢复上一组通过的 gitlink，安全修复场景先审查回退风险。
