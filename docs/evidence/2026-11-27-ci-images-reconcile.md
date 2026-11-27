# 共享 CI 镜像、单仓 CI 与发布对账（2026-11-27）

来源：[marqdo#10](https://git.cflmy.cn/cflmy/marqdo/issues/10)。本文件记录本轮 CI/CD 基础设施实现与可验证结果。

## 共享运行时镜像（Gitea Container Registry）

组件不再各自安装工具链或重复下载/编译 CLI。根 `ci/images/` 定义并发布不可变镜像：

| 镜像 | digest | 内容 | 消费方 |
| --- | --- | --- | --- |
| `git.cflmy.cn/cflmy/marqdo-ci-shell:3.0.0-rc.5` | `sha256:8184439a201d...` | Ubuntu 24.04/glibc 2.39、git/curl/python3/build 工具、已发布 CLI+Bundle（`MARQDO_BIN`/`MARQDO_STDLIB_DIR` 预置） | stdlib、extension、conformance、reconcile |
| `git.cflmy.cn/cflmy/marqdo-ci-rust:3.0.0-rc.5` | `sha256:73481a353391...` | shell 镜像 + Rust 1.100（rustfmt/clippy）+ 预热 cargo cache | Design/Preview 与 Rust Tool |
| `git.cflmy.cn/cflmy/marqdo-ci-web:3.0.0-rc.5` | `sha256:0cccda995d77...` | Ubuntu 24.04 + Node 18/Playwright 1.49 Chromium（npmmirror 二进制源，`channel: chromium`）+ CLI/Bundle + 打包 preview web Tool | Web domain 浏览器验收（pipeline 1 **success**） |

- 镜像通过 `ci/images/build.sh [--push]` 从**已发布 rc.5 CLI 归档**（sha256 固定）与固定预览包构建；标签不可变，按 CLI 版本发布；公开匿名拉取，CI 不需要 registry push 凭据。
- 单仓工作流因此只剩本仓入口命令；例如 stdlib/extension/conformance 为：
  ```yaml
  steps:
    test:
      image: git.cflmy.cn/cflmy/marqdo-ci-shell:3.0.0-rc.5
      commands: [sh scripts/test.sh]
  ```
  Rust 组件为 `marqdo-ci-rust` + `scripts/ci.sh`。CLI/根仍保留各自专用镜像与门禁（CLI Windows 交叉、根 Playwright/Docker 冷安装），属有限例外。

## 实际流水线结果（2026-11-27 本机观测）

| lane | 结果 |
| --- | --- |
| stdlib ×6 / extension ×3 / conformance（shell 镜像） | push/PR/manual lane **success**（例：conformance #12、std-text #12、collections #12、data #11、fs #11、io #11、system #11、ext-http #11、ext-regex #11、ext-archive #11） |
| Design / Preview（rust 镜像） | pipeline 6/7 **success** |
| knowledge / lint / prove（rust 镜像） | pipeline 22 / 12 / 17 **success** |
| View（rust 构建 + Playwright 浏览器两步） | pipeline 24 **success**（rc.5 CLI 适配） |
| Web domain（web 镜像：三案例 + 生产 API/Chromium E2E） | pipeline 1 **success** |
| 根 release 对账 | manual #71 **success**（创建 rc.5 追踪 Issue #22）；cron #72 **success**（幂等 tracked） |
| 发布快速通知 | 临时 release 触发 std-text release pipeline #23 **success** → 根 manual #73 **success** 并创建追踪 Issue；测试 release/tag/Issue 已清理 |

## Woodpecker 接入

- 19 个仓库已在 Woodpecker 激活（std ×6、ext ×3、conformance、Design/Preview、tool ×4、ext-web domain、根、CLI）。
- 已写入并合并单仓 `.woodpecker/ci.yml`；std/conformance/ext 的 push/pull_request 镜像 lane 已实际 **success**（例如 conformance 使用 shell 镜像通过）。
- `.woodpecker/reconcile.yml`：仅保护默认分支、manual/cron；`release-reconcile` cron `@every 30m` 已创建；根 repo secret `gitea_token` 已设置（不记录值）。
- 快速通知路径：组件发布成功后由 release 工作流请求根 reconcile；丢失通知由 cron 补偿。两者共用 `scripts/reconcile-releases.py`，不新增接收服务或第二套协议。

## 发布对账

`scripts/reconcile-releases.py`：

- 从既有 catalog/gitmodules 派生受管仓库，扫描 Gitea release（含 prerelease，排除 draft），验证 tag/资产名、大小与下载字节 SHA-256；
- 在根仓库创建带 `release-tracking` 标签与稳定 marker 的追踪 Issue（幂等；历史 release 可用 `--published-after` 基线过滤）；
- 只创建/更新追踪 Issue，不改 gitlink、不发布、不执行 release body/URL；失败资产验证会报告并返回非零；
- 6 个单元测试覆盖 dry-run、资产验证、幂等、draft 过滤、时间过滤、坏资产/路径。

## 未完成 / 限制

- `marqdo-ci-web` 已改为 Ubuntu + Node/Playwright Chromium（npmmirror 镜像源，避免 MCR 拉取超时）并发布；Web domain pipeline 1 success。ghproxy `https://ghproxy.cflmy.top` 对 GitHub 资源可用，但本轮浏览器资源使用 npmmirror，未依赖 ghproxy。
- View 已迁到 rust + Playwright 两步；knowledge/lint/prove 已迁到共享 rust 镜像。
- 组件 release 发布工作流（source archive/tool bundle 上传 + 快速通知）尚未全部落地；当前 CLI rc.5 已发布，reconcile 可追踪新发布。
- registry 镜像 digest/扫描策略、runner 拉取缓存与多架构未验证；CI 账号最小权限仍沿用现有项目 token（M0 记录限制）。
