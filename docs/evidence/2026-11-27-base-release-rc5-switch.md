# 基础发行切换：rc.5 发布与根 gitlink 固定（2026-11-27）

父工单：[marqdo#2](https://git.cflmy.cn/cflmy/marqdo/issues/2)。本文件记录实际发布与根组合切换。

## 发布身份

| 对象 | 身份 |
| --- | --- |
| 发布 | [marqdo-cli v3.0.0-rc.5](https://git.cflmy.cn/cflmy/marqdo-cli/releases/tag/v3.0.0-rc.5) |
| tag / 源码 | `v3.0.0-rc.5` → CLI `c44b0f943875d563bb7c170f8ccd3fd56a82360f` |
| CLI 同 SHA CI | pull_request + push pipeline **success**（Woodpecker CLI repo） |
| 归档 | `marqdo-3.0.0-rc.5-x86_64-unknown-linux-gnu.tar.gz`，4,907,427 bytes，sha256 `fd7a313e0ddaf38a9b1901ff2a9aa7b9a0a153e65522a9c0c60f01e2549f583b` |
| 归档资产 | release assets：tar.gz + `.sha256`；下载后 sha256 与本地构建一致 |
| Bundle | `marqdo-stdlib@0.2.0`，digest `sha256:db905508...`，外部显式输入 |
| 根 CLI gitlink | `c44b0f94`（本候选切换；旧 rc.4 源码 `452e9a3` 为回滚目标） |
| 平台 | Linux x86_64 / glibc 2.39（Ubuntu 24.04 容器验证） |

## 发布门禁（同一最终归档上真实执行）

- CLI 发布 SHA 的 Woodpecker pull_request/push pipeline **success**。
- 确定性归档：同一 `SOURCE_SHA` + 同一 Bundle 连续两次 package，sha256 完全一致（`fd7a313e...`）。
- `smoke.py` PASS。
- Ubuntu 24.04 / glibc 2.39、`--network none`、非 root、无 Cargo 冷安装 PASS（缺 Bundle fail-closed；语言-only 可运行）。
- 归档 CLI + 归档 Bundle：三个迁出扩展 http/regex/archive PASS；冻结外部 conformance 七类 PASS。
- 发布资产下载校验：下载归档 sha256 = `fd7a313e...`。
- 根同候选：`check-layout --checkouts`、根集成测试与生态 candidate CI（含既有消费者）以 `cli/c44b0f9` 为输入；结果以根 PR CI 为准。

## 已记录的未验证限制（不冒充 PASS）

- Strict L1 独立 VM/非 root：本发布环境无 disposable VM/命名空间，**UNVERIFIED**；release notes 明示无 PASS。
- G4 性能：paired shared-node 比较 9/9 工作负载、最大 p50 +5.15%、最大 RSS +4.87%，低于阈值，但**不是获批节点 PASS**；release notes 记录为 shared-node WARN 限制。
- 维护者已明确批准本次 rc.5 发布并接受上述两项记录限制。

## 回滚

若不接受 rc.5，恢复根 `cli` gitlink 到 `452e9a3`（rc.4 发布源码）并重跑候选；已发布 tag/资产不可变，不回写。
