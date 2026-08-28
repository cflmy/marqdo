# 官方扩展：`ext/web` 实现路线图

| | |
|---|---|
| 状态 | **W0–W6（迁移/FTS）+ 安全硬化 + 内容标配 W4c 已落地** |
| 日期 | 2026-08-28 |
| **锁定设计** | [design/ext-web.md](../design/ext-web.md) |
| 相关 | [ext-cli.md](../design/ext-cli.md) · [ext-abi.md](../design/ext-abi.md) · [ext-llm.md](../design/ext-llm.md) |
| 安装 | `marqdo ext add web`（`web` / `网页`） |

本文只跟踪**实现阶段**。类模型、赋值绑定、后台、`.env`、异步边界、ABI 名等以设计文为准。

---

## 1. 目标回顾（一句）

**表格完成动态站**：页面表拼组件、组件表绑库与样式、样式表定外观；热路径进 ABI 插件；可选 S3 / Redis。人写目标见设计 §0.1 与 `examples/man-write-site/`。

---

## 2. 落地阶段

| 阶段 | 内容 | 状态 |
|------|------|------|
| **W0** | CATALOG + L1 + `web_listen` Hello | **done** |
| **W1** | env 端口；`page` 四栏；nav 表；static | **done** |
| **W2** | SQLite `define`/`migrate`/`all` | **done** |
| **W3** | 三列绑定；变量名即表名；`/admin` CRUD+日志+布局 | **done** |
| **W3s** | 安全硬化：argon2 密码哈希、CSRF、SQLite 会话持久化、CSPRNG、登录限速 | **done** |
| **W3.5** | 人写面首版：`web.assemble` + `` `表`.`字段` `` + `examples/man-write-site` | **done**（编排袋仍可再瘦） |
| **W4** | Postgres；Redis / S3 驱动（见 [ext-web-drivers.md](../design/ext-web-drivers.md)） | **done** |
| **W4c** | 内容标配：SEO / Markdown / RSS / 分页 UI（见 [web-net-capabilities.md](../design/web-net-capabilities.md)） | **done** |
| **W5** | 上传与媒体：multipart 接收 + 落盘 + 下载（见 [web-net-capabilities.md](../design/web-net-capabilities.md) A5） | **done** |
| **W6** | 迁移 + FTS5 搜索；草稿/评论用既有 where/表（WS 广播后续） | **done**（核心） |

验收金样：`tests/ext/web-smoke.mq.md`、`tests/ext/web-security-smoke.mq.md`、`tests/ext/web-drivers-smoke.mq.md`、`tests/ext/web-upload-smoke.mq.md`、`tests/ext/web-db-w6-smoke.mq.md`、`ext_cli_add_web`。

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
