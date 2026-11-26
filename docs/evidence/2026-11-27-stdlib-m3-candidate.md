# 标准库源码归位 M3 候选证据（2026-11-27）

父仓工单：[cflmy/marqdo#9](https://git.cflmy.cn/cflmy/marqdo/issues/9)。
本文件记录 **M3 候选组合**（源码所有权切换 + 统一制品输入）的候选证据，不是已完成发行声明。主分支仍保持迁移前组合 `484cb10`，根 PR #12 只有在 CI 通过且 CLI Phase B 合并后才晋升。

## 候选身份

| 对象 | 身份 |
| --- | --- |
| 根候选 | `migration/stdlib-m2-candidate`（CLI Phase B 合并提交 `deadf6e`） |
| CLI Phase B 合并 | `cflmy/marqdo-cli@deadf6edeea6dd0f5abf51c47e80204fd851baf0`（PR #17 fast-forward 合并；Phase B 代码提交 `72f57be`，另含 CI 脚本修复；`stdlib/` 已删除，pin 指向生态 Bundle） |
| 生态 Bundle | `marqdo-stdlib@0.2.0`，source `cflmy/marqdo@48189243f7`，tree digest `sha256:db905508...`，aliases digest `sha256:5f513c0f...`（17 模块 + manifest） |
| 组件固定 | 六个 std 与三个 ext 的已合并候选 SHA（M2 证据同一组） |
| 发行归档（本地候选） | `marqdo-3.0.0-rc.4-x86_64-unknown-linux-gnu.tar.gz`，`sha256:9a926010ffe8e7353ee18b65fc568ced8bbf4b09b7592713f1213e2f3e3d40f9`（CLI `72f57be` + 新 Bundle） |

## 统一制品输入（ECO-02 核心）

根 `scripts/integration.sh` 不再读取 View 的冻结 RC.4 脚本；一次候选内派生全部输入：

1. `scripts/build-stdlib.py --out <work>/stdlib` 由固定组件 gitlink 生成唯一 Bundle；
2. 从根固定的 `cli/` 提交构建 CLI 与 Provider Worker；
3. `cli/tools/release/package.py --source-sha <cli SHA> --version <workspace version> --stdlib-dir <Bundle>` 生成发行归档与 `RELEASE_INFO.json`；
4. `cli/tools/release/smoke.py` 验证归档、`SOURCE_SHA`、stdlib 摘要与语言-only 运行；
5. `scripts/acceptance-stdlib.py --cli <installed> --bundle <Bundle>` 跑组件 entry、显式 Package、组合 fixture 与失败负例；
6. Conformance、View、Knowledge、Prove、Lint 使用同一安装后二进制与相邻 Bundle。

## 本地候选验证

| 门禁 | 命令 | 结果 |
| --- | --- | --- |
| 根布局 | `python3 scripts/check-layout.py --checkouts` | PASS（含 6 个 active stdlib entry 与 9 个固定组件） |
| Bundle | `python3 scripts/build-stdlib.py --out <dir>` | 两次构建一致，digest `sha256:db9055...` |
| CLI Phase B | `cargo fmt` / `clippy -D warnings` / `cargo test --workspace`（显式 Bundle） | fmt/clippy PASS；72 个测试二进制 PASS |
| CLI 工具测试 | `MARQDO_TEST_STDLIB_BUNDLE=<bundle> python3 -m unittest discover -s tools/ci` | 75 tests OK |
| 独立构建 | `python3 tools/ci/independent_build.py` | PASS（无 stdlib 源树、无兄弟 Bundle，语言-only 运行成功，依赖 std 的程序 fail-closed） |
| 发行归档 | `package.py --stdlib-dir <Bundle>` + `smoke.py --archive` | PASS（stdlib 0.2.0 摘要校验、缺失 Bundle fail-closed、language-only 运行） |
| 验收脚本 | `scripts/acceptance-stdlib.py` 本地 M2 版 | PASS（6 组件 + 3 显式 Package + 4 组合 + 2 负例） |

## 未验证 / 待办

- 完整根 CI（`scripts/integration.sh` 含 View/Knowledge/Prove/Lint 消费者）在候选分支执行，结果记录在 PR #12/评论，不以本地部分运行替代。
- 根晋升（合并 CLI PR #17 与根 PR #12、climo gitlink 与制品同一候选）属 MIG-RELEASE。
- 完整产品归档冷安装（docker `--network none`）、多平台、ext-http loopback 领域测试仍待补；如需标记为发行门禁必须另行执行，不得以本文件冒充 PASS。
