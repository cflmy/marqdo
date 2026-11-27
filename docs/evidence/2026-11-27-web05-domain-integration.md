# WEB-05 Web 领域根接入与冷安装候选证据（2026-11-27）

父工单：[marqdo#6](https://git.cflmy.cn/cflmy/marqdo/issues/6)。本文件记录根侧候选证据；领域侧见 `marqdo-ext-web` 的 `docs/evidence/WEB-05-domain-activation.md`。主分支组合在合并前保持不变。

## 候选身份

| 对象 | 身份 |
| --- | --- |
| 根候选 | `release/web05-domain-active`（根 `main` 基线 `d4ccb77`） |
| Web 领域 | `cflmy/marqdo-ext-web@f6d0a9067cad9f685a501a7bc3f3c3d302fe4f16`（active，入口 `scripts/domain-acceptance.sh`） |
| Design（active） | `cflmy/marqdo-web-design@1b6607fd3cc09801b4f6eacd2fba547caa0ec637`（entry `scripts/ci.sh`） |
| Preview（active） | `cflmy/marqdo-web-preview@eff273f7f2f3919bab62eacded781c61d5f03f91`（entry `scripts/ci.sh`） |
| web-components | planned（无独立实现，不伪造入口） |
| CLI | 根固定 `cli/deadf6e` 构建的 `marqdo 3.0.0-rc.4`，`sha256:6ede3b9e...` |

## 根侧门禁与结果（本地候选，CI 为最终门禁）

| 门禁 | 命令 | 结果 |
| --- | --- | --- |
| 递归布局 | `python3 scripts/check-layout.py --checkouts` | **PASS**（根 + Web 域 + Design/Preview active 入口；本地候选） |
| 根集成测试 | `python3 -m unittest discover -s tests/integration` | **71 tests OK**（本地候选） |
| 领域组件入口 | `extensions/web/design/scripts/ci.sh`、`extensions/web/preview/scripts/ci.sh` | **PASS**（本地候选；24/24 Design，web Tool install/check/preview/handoff/remove） |
| web Tool 冷安装 | `tool install` → `web check/preview/handoff` → `tool remove`，CLI SHA 前后一致 | **PASS**（CLI `sha256:6ede3b9e...` 前后一致；本地候选） |
| 领域验收 | `extensions/web/scripts/domain-acceptance.sh`（三案例 + 生产 API/浏览器 E2E） | **PASS**（WEB-04 证据：三案例 + API + Chromium E2E，同一领域提交） |
| 既有消费者 | Conformance/View/Knowledge/Prove/Lint 同一 CLI/Bundle | 由 `integration.sh` 6 在 CI candidate 执行（CI 为最终门禁） |

## 边界

- 根与领域 catalog 都登记 `cli` 依赖；根不实现 Design/Preview/生产应用代码。
- 冷安装只安装构建出的 `web` Tool bundle；生产应用不是 Tool 运行依赖。
- `web-components` 未激活、无 entry，不计完成率。
- 本文件不把本地候选当已发布证据；根 CI（layout + candidate）通过后才晋升。
