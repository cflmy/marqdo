# 候选 Webhook 可信入口与漏通知对账证据（#3 / #10 GAP-05）

日期：2026-11-27。责任仓：根 `marqdo`。工单：[marqdo#3](https://git.cflmy.cn/cflmy/marqdo/issues/3)、[#10](https://git.cflmy.cn/cflmy/marqdo/issues/10)（发布传播/漏通知恢复 GAP-05/GAP-06 的仓库侧基础）。

## 实现

`scripts/webhook-adapter.py`（无长驻服务、无合并/推送权）：

1. **来源认证**：`X-Gitea-Signature` HMAC-SHA256 校验原始 body；secret 仅来自环境变量，缺失/不匹配即拒绝，拒绝前不解析为可信输入。
2. **事件白名单**：`push`（拒绝删除分支）与 `pull_request`（仅 opened/synchronized/reopened）；其它事件拒绝。
3. **仓库身份**：从 accepted 根图的远端允许列表解析；未知仓库拒绝；payload 的仓库名只用于匹配该表。
4. **base 语义**：组件候选 base 使用 accepted 根图当前固定 SHA（新基线需重验），payload base/branch 只作 hint；根候选 base 为 accepted revision。
5. **CI 聚合**：至少一个 status 且全部 `success` 才算成功；URL 必须是允许 CI 主机上的 https；pending/failure/空状态拒绝。
6. **复用同一验证器**：交给 `scripts/candidate-notify.py` 做可达性、去重/重放、基线 revalidate 与 `scripts/impact.py` 影响范围；输出计划 `auto_promote=false`/`auto_merge=false`。
7. **漏通知对账**：`--reconcile` 扫描允许列表仓库开放 PR 的 head statuses，对未入账的成功候选执行同一验证并写入幂等状态；重复运行报 duplicate；API transient 错误有限退避重试，401/403 立即失败，报告不写成功。

## 验证（真实临时 Git 仓库，无网络 fixture）

```sh
python3 -m unittest tests.integration.test_webhook_adapter
# 8 tests OK
python3 -m unittest discover -s tests/integration
# 79 tests OK
```

覆盖：签名正确/篡改/缺失 secret、未知事件/删除分支、未知仓库、failure/pending/空 CI、PR 事件、基线固定而非 payload base、重放拒绝、对账恢复丢失通知且第二次幂等、API 失败不写状态。

## 未验证 / 外部依赖

- 未在本环境配置真实 Gitea 仓库 webhook endpoint/secret 与 Woodpecker 触发器；本证据只证明仓库侧可信入口与对账逻辑，不代表平台已部署（不伪造部署 PASS）。
- 长驻接收服务、bot 账号最小权限、cron 工作流与平台并发/限流按 #10 M0 实测后部署；仓库侧不新增第二套协议。
- 受保护发布工作流/制品资产（GAP-05 发布侧）仍属 #10。
