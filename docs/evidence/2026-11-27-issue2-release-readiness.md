# #2 基础发行切换：发行门禁候选证据（2026-11-27）

工单：[marqdo#2](https://git.cflmy.cn/cflmy/marqdo/issues/2)。本文件记录**未发布**的本地发行候选验证；它不是 release、没有 tag、没有上传资产。按 `cli/docs/releasing.md`，tag/Release 需要维护者在冻结 SHA 上显式批准，自动化只能预备 RC 与证据。

## 候选身份

| 对象 | 身份 |
| --- | --- |
| 根组合 | `cflmy/marqdo@4a922017088490dfe7be75f76f70fa8fd5bb1c54`（本地验证基线） |
| CLI 源码 | `cflmy/marqdo-cli@deadf6edeea6dd0f5abf51c47e80204fd851baf0`（Phase B，无内置 `stdlib/` 源码树） |
| CLI 同 SHA CI | Woodpecker CLI pipeline 122（push）/121（pull_request）**success** |
| Bundle | `marqdo-stdlib@0.2.0`，tree digest `sha256:db905508dde89763b9d43b47fc5d52c6a1d5445a91f3c86a550ced43e8aac9d7` |
| 归档（验证用，未发布） | `marqdo-3.0.0-rc.4-x86_64-unknown-linux-gnu.tar.gz`，sha256 `377d2787e9cdeeb7c8e70024dacd0e2790fb0752716abeb5c0f72509807644d0` |
| 平台 | Linux x86_64 宿主；冷安装容器 Ubuntu 24.04 / glibc 2.39，`--network none`，无 Cargo |

**版本冲突声明**：该验证归档沿用 Cargo workspace 的 `3.0.0-rc.4` 版本但内容/mtime 组合与已发布 rc.4 不同（sha 不同）。它绝不能上传或作为 rc.4 再发布。正式候选必须先冻结新 SHA 并把版本/README/CHANGELOG/COMPATIBILITY/开发者文档统一提升到 `3.0.0-rc.5`，再重跑全部门禁。

## 已真实执行的本地门禁（PASS）

1. **Bundle 生产**：`scripts/build-stdlib.py` 从固定 gitlinks 生成 Bundle；`digest=sha256:db905508...`。
2. **确定性归档**：`cli/tools/release/package.py` 对同一 `SOURCE_SHA` + 同一 Bundle 连续构建两次，两个 tar.gz sha256 完全一致（`377d2787...`）。
3. **归档 smoke**：`cli/tools/release/smoke.py` PASS；缺少随附 Bundle 时 fail-closed，语言-only 程序可运行。
4. **离线冷安装**：`cli/tools/release/cold-install.sh` 在 Ubuntu 24.04/glibc 2.39、无网络、非 root、无 Cargo 容器内 PASS：`SOURCE_SHA` 与 `--version` 一致，`run`/`check`/`call`/JSONL `run.completed` 正常，移动 `stdlib` 后 `打印` 以 `kernel.unavailable` fail-closed，语言-only `阶乘` 仍可运行。
5. **标准库面**：同一归档 CLI 运行 `std-surface`，text/JSON/FS/path/datetime/table/Console 输出符合预期（实际 `call` 返回数组）。
6. **三个迁出扩展**：`extensions/http`、`extensions/regex`、`extensions/archive` 的 `scripts/test.sh` 使用**归档 CLI + 归档 Bundle** 全部 PASS。
7. **外部 conformance**：`cli/tools/ci/verify_extension_contracts.py --cli <归档 CLI> --release-info <归档 RELEASE_INFO>` 七类结果全部 PASS（plugin/worker/callable/negative-security/foreign-kernel/resource-provider/tool-analysis/package-pinned-git/reference-apps）。
8. **负例**：篡改归档随附 Bundle 的 `stdlib.json.digest` → CLI 退出 2（`stdlib digest mismatch`）；缺失 Bundle → fail-closed。
9. **无重复源码**：CLI 源码树 `git ls-tree HEAD stdlib` 为空；归档内只有单一 Bundle。

## 未完成 / BLOCKED（不是 PASS）

| 门禁 | 状态 | 需要 |
| --- | --- | --- |
| 冻结 release SHA + 版本/文档一致性（rc.5） | NOT STARTED | 维护者决定版本与候选 SHA；本次验证归档版本冲突，不可发布 |
| Strict L1 独立 VM、非 root、same-SHA 证据 | UNVERIFIED | 可用的 disposable VM/隔离 runner；本地 bubblewrap 命名空间不可用 |
| G4 性能（固定节点或维护者批准的 shared-node WARN） | UNVERIFIED | 固定/获批节点与正确性基线 |
| 维护者显式 tag/Release 批准 | NOT STARTED | 按 `cli/docs/releasing.md` 由维护者签署 |
| Windows/macOS | UNSUPPORTED | 本 RC 范围只声明 Linux x86_64 glibc |

## 结论

外部 Bundle、确定性归档、离线冷安装、三个迁出扩展与工具/外部 conformance 已在**同一 CLI+Bundle 组合**上通过；基础发行切换所缺的是新版本冻结、strict VM、性能与维护者发布批准。未创建 tag、未上传资产、未修改根发行身份；在获批前根 CLI/Bundle gitlink 与迁移台账保持现状。
