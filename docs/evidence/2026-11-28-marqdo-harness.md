# Marqdo Harness 根消费者验收证据（2026-11-28）

本文件记录主仓库对 `marqdo-app-harness` 的接入证据。应用自身状态与未验证范围以
`apps/harness/docs/mdh/STATUS.md` 为准；本文件不重复、也不改写那些结论。

## 身份

| 对象 | 身份 |
| --- | --- |
| 主仓库候选 | 本提交（`git rev-parse HEAD`） |
| 应用 gitlink | `apps/harness` → `ee482a23c2e0ae3ca4c2b56281ea82e9fb58619d` |
| 应用 origin | `https://git.cflmy.cn/cflmy/marqdo-app-harness.git` |
| DSH 上游 | `4b399c39a6d1479cd84ca591b4d7c88e24d38a96`（`dsh-v0.2.3-alpha.2`） |
| CLI gitlink | `63776735e924ade279cdf59d5c733dc24bfdabbb`（`v3.0.0-rc.6`） |
| CLI 资产 | sha256 `8dc0b006302e03c23451b8ea9f8a0fcdae8bf67c80c546bee2434bf69755bfe3` |
| Std Bundle | tree digest `sha256:db905508dde89763b9d43b47fc5d52c6a1d5445a91f3c86a550ced43e8aac9d7` |

## 根治理校验

| 命令 | 结果 |
| --- | --- |
| `python3 scripts/check-layout.py` | 通过；`app-harness apps/harness d28c5e4051... active` |
| `python3 -m unittest tests.integration.test_layout` | 60 tests PASS；含 app 路径/远端/前缀/release/嵌套/依赖方向负例 |
| `python3 scripts/check-repo-conventions.py` | PASS（38 components）；应用 README/AGENTS/entry 真实存在 |

## 组合调度

- `scripts/integration.sh` 的 active entry 分派对 `app:app-harness:scripts/ci.sh` 使用独立 `app` gate。
- Rust candidate lane 仍只构建一个 CLI/Worker/Bundle 制品，并在安装后写出稳定的
  `.integration-work/app-candidate.json`：归档 SHA-256、CLI source SHA、Bundle digest、
  安装目录、CLI 绝对路径与应用 gitlink。
- `.woodpecker/eco.yml` 增加专用 Node lane（`node:24.22.3-bookworm`），消费同一
  candidate manifest，`pnpm install --frozen-lockfile && pnpm run build` 后运行
  `apps/harness/scripts/ci.sh`；Rust lane 不安装 Node/Electron。
- 未知 active entry 仍以非零退出失败，不静默跳过；应用为 `release = "optional"`，
  其失败不阻断合法 CLI 基础发行，但包含 app 的根候选会如实记录失败。

## 应用侧已复核证据

证据全文见 `apps/harness/docs/mdh/evidence/`。

- 上游基线 `pnpm install --frozen-lockfile` 与 `pnpm run build`：10/10 gates PASS。
- 下游 28 项测试：27 PASS；网络安装测试在 `MDH_TEST_INSTALL=1` 下 PASS（rc.5→rc.6→回退）。
- 真实 CLI 执行 factorial `n=5` → `{"type":"int","decimal":"120"}`；除零 → `failed`；摘要不符 → `incomplete`。
- 真实 `marqdo-knowledge` 编译/查询：`mode=compiled`，命中带精确源路径、摘要、span 与通道。
- `/mdh` 页面与 API：200；cross-site / 错误 Host / 不匹配 Origin → 403。
- `scripts/package.sh`：预构建归档冷组合 PASS（748,901,507 bytes，sha256 `21a532d2949a25bae76a41b2a4e0f08d06f39a084db2f02a1cd1506ad5a24617`）。

## UNVERIFIED

- 父仓 PR#48 的 Woodpecker pipeline 已真实运行；首次 candidate 失败于 app-candidate manifest 把摘要字符串误转为 `Path`，已修复并等待 rerun 通过。Node lane 尚未在该次 pipeline 中执行完成。
- 真实模型两轮任务（任务 A/B）未运行：本机没有模型凭据，没有用 fixture 冒充。
- 浏览器自动化未运行；页面与 API 通过真实 HTTP 请求验证。
- 上游同步演练未运行：镜像 HEAD 等于基线，没有更新的提交。
- 候选归档未签名，未在第二台干净主机或更低 glibc 上冷安装。
