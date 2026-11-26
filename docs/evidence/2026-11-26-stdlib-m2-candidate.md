# 标准库源码归位 M2 候选证据（2026-11-26）

父仓工单：[cflmy/marqdo#9](https://git.cflmy.cn/cflmy/marqdo/issues/9)。本文件记录 **M2 候选组合**的真实执行结果，是候选证据，不是已发布声明。根主分支仍保持迁移前组合（`484cb10`）；本候选只有在 CLI Phase A 与六个接收仓 PR 合并、且完整生态门禁通过后才可晋升。

## 候选身份

| 对象 | 身份 |
| --- | --- |
| 根候选分支 | `migration/stdlib-m2-candidate`（本文档提交前树 `8fa728b302fa29caf8268a758eb4b0e90431d780`） |
| 根主分支基线 | `484cb10d66db9b92f0527499642ae6abddb3ef29`（未改动） |
| CLI gitlink（未变） | `cflmy/marqdo-cli@452e9a32bd9e40dba427d4180691c58bab74449e`（RC.4 组合） |
| CLI Phase A 候选（仅用于打包/验证，PR #12） | `cflmy/marqdo-cli@f8b82d414328d92b31865fed05e68223f7bee917` |
| std-text | `cflmy/marqdo-std-text@c2a7d327e5d1914db8952562614707c112a07da1`（PR #2） |
| std-system | `cflmy/marqdo-std-system@053ddb0e59dc016b9c7833f115d14fa04e721d48`（PR #2） |
| std-io | `cflmy/marqdo-std-io@e18b0b1b40fcc6adcb42b5ab9fe1c9c04e9b8263`（PR #2） |
| std-fs | `cflmy/marqdo-std-fs@63bceae58736f13770c7eead02535726d08c19d0`（PR #2） |
| std-data | `cflmy/marqdo-std-data@848af7a357ceda2cfc20721b93406cf4be705889`（PR #2） |
| std-collections | `cflmy/marqdo-std-collections@490765226866ac74b7365471c202e7638187bff8`（PR #2） |
| ext-http（显式 Package） | `cflmy/marqdo-ext-http@368b33d241`（PR #2，合并后精确 SHA 以根 gitlink 为准） |
| ext-regex（显式 Package） | `cflmy/marqdo-ext-regex@6e7e633798`（PR #2） |
| ext-archive（显式 Package） | `cflmy/marqdo-ext-archive@aba6b60934`（PR #2） |

`std-core`、`std-encoding`、`std-net` 保持 `planned`，不加入 Bundle。HTTP/Regex/Archive 已迁为显式 Package（`index.mq.md` + `marqdo.yaml`，消费者经 `@alias` 导入），不进入默认 Bundle；其源码候选已合并到各自仓库 main，根候选固定对应提交并执行显式 Package 消费门禁（`kind = "explicit-package"`）。

## 唯一 Bundle 制品

`scripts/build-stdlib.py` 从上述根 gitlink 组合，两次干净构建逐字节一致：

| 字段 | 值 |
| --- | --- |
| name / version | `marqdo-stdlib` / `0.2.0`（内容变化显式升版本；摘要算法不变） |
| tree digest | `sha256:db905508dde89763b9d43b47fc5d52c6a1d5445a91f3c86a550ced43e8aac9d7` |
| aliases digest | `sha256:5f513c0fdbb3c1e9829be3c9fa23a2d1e89cb8510073fb5eb1e8e2f5265a83bb`（与迁移前一致） |
| 模块数 | 17（仅模块；`tests/`、`demo.mq.md`、`net/http`、`regex`、`compress` 均不在默认 Bundle） |
| 打包归档 | `marqdo-stdlib-0.2.0.tar.gz`，`sha256:212cca32f10ecfa0cdedfa976f7726db5e01cdb0e664a41ae26d4b34a8fd7048`（CLI Phase A `tools/pack-stdlib.py` 两次输出逐字节一致） |

## 门禁结果

| 门禁 | 命令 | 结果 |
| --- | --- | --- |
| 根布局 | `python3 scripts/check-layout.py` | PASS |
| 递归布局 | `python3 scripts/check-layout.py --checkouts` | PASS（33 个固定子树；6 个 active stdlib entry 存在且工作区干净） |
| 根集成测试 | `python3 -m unittest discover -s tests/integration` | Ran 56 tests，OK（含 9 个 Bundle 生产者契约测试） |
| 组件独立验收 | `MARQDO_BIN=<RC.4 由 452e9a3 构建> MARQDO_STDLIB_DIR=<新 Bundle> scripts/test.sh`（各仓） | 6/6 PASS，共 17 个 case（collections 1、text 2、data 3、io 4、fs 2、system 5） |
| 跨仓组合 | `scripts/acceptance-stdlib.py --cli <CLI> --bundle <新 Bundle>` | PASS：官方 `print`/`打印` 别名、Table 值操作+GFM 跨模块、默认 Bundle smoke |
| Table 等价性 | 组合 fixture 终值与迁移前原 `stdlib/tests/table.mq.md` 终值 | 逐字节一致 |
| 显式 Package 消费 | `scripts/acceptance-stdlib.py` 内 http/regex/archive 各自 `scripts/test.sh`（临时消费者工程 + `marqdo lock` + `@alias` 导入） | PASS（原领域测试终值与迁移前基线一致） |
| 失败负例 | 缺显式 Bundle、篡改模块摘要 | 均按预期失败（exit 1），不静默回退 |
| 相邻安装离线运行 | 把新 Bundle 放到 CLI 二进制旁 `stdlib/`，不设 `MARQDO_STDLIB_DIR` | PASS（默认离线安装路径可用） |
| CLI 打包 | Phase A `pack-stdlib.py --stdlib-dir <新 Bundle>` ×2 + 篡改输入 | 确定性 PASS；篡改拒绝 |

CLI 身份（sha256）：由 `452e9a3` 构建的 RC.4 二进制 `sha256:0377b855...`；Phase A 合并 master（`f8b82d4`）二进制 `sha256:6ede3b9e...`。两者对同一 Bundle 的组件与组合门禁结果一致。

## 未验证 / 待办

- 完整 `scripts/integration.sh` 的既有消费者门禁（Conformance/View/Knowledge/Prove/Lint）本轮**未执行**：该流程仍取冻结 RC.4 制品，本次未替换为统一制品输入（ECO-02，marqdo#7）；不能把本次 PASS 当作生态发行验收。
- CLI Phase B（删除仓内官方标准库源树、pin 切换）未做，等待 CLI PR #12 合并与本候选组件 PR 合并。
- HTTP/Regex/Archive 已成为显式 Package 且组件内测试通过；本候选的根组合门禁只验证了独立 Package 行为，尚未验证真实应用项目的 `git` 依赖锁定/同步与发布分发，也尚未从默认 Bundle 删除后验证既有 `std:` 用户的迁移提示（按合同不保留 fallback）。
- 未做完整产品归档冷安装（新 Bundle + 新 CLI 发行归档）与多平台证据；Windows/macOS 未验证。
- 根主分支未改动，回退方式为放弃本候选分支，迁移前组合 `484cb10` 仍可检出。

## 复现

```sh
git checkout migration/stdlib-m2-candidate
git submodule update --init --recursive
python3 scripts/check-layout.py --checkouts
python3 -m unittest discover -s tests/integration
python3 scripts/build-stdlib.py --out build/stdlib
python3 scripts/acceptance-stdlib.py --cli <verified-marqdo> --work build/accept
# 可选：--pack-tool <cli>/tools/pack-stdlib.py（Phase A 之后）
```
