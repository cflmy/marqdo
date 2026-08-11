# 官方扩展：`ext/web` 实现路线图

| | |
|---|---|
| 状态 | **W0–W3 骨架已落地；W4 待** |
| 日期 | 2026-08-11 |
| **锁定设计** | [design/ext-web.md](../design/ext-web.md) |
| 相关 | [ext-cli.md](../design/ext-cli.md) · [ext-abi.md](../design/ext-abi.md) · [ext-llm.md](../design/ext-llm.md) |
| 安装 | `marqdo ext add web`（`web` / `网页`） |

本文只跟踪**实现阶段**。类模型、赋值绑定、后台、`.env`、异步边界、ABI 名等以设计文为准。

---

## 1. 目标回顾（一句）

官方模板分钟级拉起带 **后台** 的网站；导航/库表用 **GFM 表**；前后端用 **赋值式绑定**；热路径进 **ABI 插件**；可选 S3 / Redis。

---

## 2. 落地阶段

| 阶段 | 内容 | 状态 |
|------|------|------|
| **W0** | CATALOG + L1 + `web_listen` Hello | **done** |
| **W1** | env 端口；`page` 四栏；nav 表；static | **done** |
| **W2** | SQLite `define`/`migrate`/`all` | **done** |
| **W3** | 三列绑定；变量名即表名；`/admin` CRUD+日志+布局 | **done**（登录鉴权仍简） |
| **W4** | Postgres；Redis / S3 驱动 | pending |

验收金样：`tests/ext/web-smoke.mq.md`、`ext_cli_add_web`。

---

## 3. 已收敛的原开放点

| 原疑问 | 锁定结论（见设计文） |
|--------|----------------------|
| listen 阻塞 vs 后台任务 | `# main` 内 `listen` **同步阻塞**；插件内部异步 |
| handler 形态 | **不主推**手写 REST；表 href + bind + 自动后台 |
| `marqdo web` 子命令 | scaffold 经 `web.scaffold`；`marqdo web new` 可后续补 |
| Windows + SQLite | 路径用 `DATABASE_URL` |

---

## 4. 与其它文档

| 文档 | 关系 |
|------|------|
| [design/ext-web.md](../design/ext-web.md) | **真理** |
| [user-site.md](../design/user-site.md) | 静态站互补 |
| [ext-quantum.md](ext-quantum.md) | 同属官方 ext + ABI + `ext add` |
