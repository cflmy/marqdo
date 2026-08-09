# 官方扩展：便捷网络 / 动态网页（未来规划）

| | |
|---|---|
| 状态 | **规划 / 未开工** |
| 日期 | 2026-08-09 |
| 相关 | [ext-cli.md](../design/ext-cli.md) · [ext-abi.md](../design/ext-abi.md) · [ext-llm.md](../design/ext-llm.md) · [stdlib-modules.md](../design/stdlib-modules.md) · [user-site.md](../design/user-site.md) |
| 安装草案 | `marqdo ext add web`（中英面：`web` / `网页`） |

---

## 1. 动机

今日 Marqdo 可写脚本与静态文档站（[`public/`](../../public/)、[user-site.md](../design/user-site.md)），但缺少**一等动态 Web 扩展**：

- 用 `.mq.md` 快速拉起带路由的 HTTP 服务  
- 用 `.env` 配置数据库等连接信息  
- **一键初始化** schema，改动后可 **迁移**  
- 按官方/模板**样式**快速出可用页面（非从零写 CSS）

目标用户：用 Marqdo 做内部工具、演示后台、轻量 CRUD，而不是替换通用 Web 框架的全部生态。

---

## 2. 范围

### 2.1 在范围内

| 能力 | 说明 |
|------|------|
| 动态 HTTP 服务 | `marqdo` 子进程或 host 能力拉起监听；路由与处理器写在 `.mq.md` |
| Env 配置 | `DATABASE_URL` / 驱动名 / 端口等；沿用 dotenv 惯例（不入库） |
| 数据库 | 至少一种官方支持（建议 **SQLite** 默认 + **Postgres** 可选） |
| 一键 init | 按声明的 schema / migration 目录建库建表 |
| 迁移 | 改动 schema 后生成/应用版本化迁移；可重复执行 |
| 样式快速启动 | 内置 1～2 套极简主题（或模板 `.mq.md` + 静态资源），`ext add` 后可 `scaffold` |
| L1 `.mq.md` API | 中英双文件（如 `ext/web/web.mq.md` · `ext/web/网页.mq.md`） |
| **ABI 插件** | 热路径（HTTP server、DB 驱动、迁移引擎）进 `plugins/web`（或拆 `web_http` / `web_db`） |
| **CLI 安装** | `marqdo ext list/add/remove web`，与 llm/agent 同惯例 |

### 2.2 非目标（本规划）

- 完整 ORM / GraphQL / 微服务治理  
- 在核心 `src/host` 固化业务 Web 框架（逻辑进 `ext/` + `plugins/`）  
- 强制用户提交密钥或 `.env`  
- 与 `marqdo view` 调试 UI 合并成同一产品（可日后互链）

---

## 3. 布局与安装（对齐惯例）

与 [ext-cli.md](../design/ext-cli.md) 一致：官方扩展按**域子目录**，不进 `ext/ai/`。

```text
ext/
  web/
    web.mq.md
    网页.mq.md
    templates/           # 可选：快速样式脚手架
      minimal/
    migrations/          # 可选：示例迁移
plugins/
  web/                   # ABI v2 原生插件 crate
    Cargo.toml
    src/lib.rs
```

```text
marqdo ext add web
marqdo ext remove web
```

安装根：`MARQDO_EXT` 或 `~/.marqdo/ext`；复制 `web/*.mq.md` + 平台对应 `native/libweb.so`（或 `.dylib` / `.dll`），与 agent 插件安装同模式。

导入：

```markdown
---
> ext/web/web.mq.md
---
```

---

## 4. 配置（`.env`）

示例键（开工时锁定前缀，避免与 `OPENAI_*` 冲突）：

```env
# HTTP
MARQDO_WEB_HOST=127.0.0.1
MARQDO_WEB_PORT=8080

# Database
DATABASE_URL=sqlite:./data/app.db
# DATABASE_URL=postgres://user:pass@localhost/dbname
```

L1：`## load_env`（复用 llm 侧惯例或 `lib/sys`）后，`# server` / `# 服务` 构造读 env。

---

## 5. API 外形（草案）

```markdown
---
> ext/web/web.mq.md
---

# main

> web.load_env

*`app` = > web.app *
> `app`.route method=GET path=/  handler=首页
> `app`.listen
```

中文面：`网页.应用` / `路由` / `监听`。

数据库：

```markdown
*`db` = > web.db *
> `db`.migrate          # 应用 migrations/
> `db`.init             # 空库一键初始化（或 migrate 的别名策略，开工锁定其一）
```

页面：handler 返回 HTML 字符串、或渲染 `templates/` 下模板；样式包通过 `theme=minimal` 或脚手架复制 CSS。

---

## 6. 数据库 init / 迁移

| 操作 | 行为 |
|------|------|
| `init` | 若不存在库/元表则创建，并应用到最新迁移 |
| `migrate` | 应用未执行的版本；记录于 `_marqdo_migrations`（名可调） |
| 改动后 | 用户新增 `migrations/00x_*.sql`（或 Marqdo 声明式表 → 生成 SQL）；再 `migrate` |

原则：

- 迁移文件进用户项目（可 git），不是藏进插件不可见二进制。  
- 插件只提供执行器与驱动；**真相在仓库文件**。  
- SQLite 为默认零运维路径；Postgres 走同一 `DATABASE_URL`。

---

## 7. ABI 边界

| 放在插件（Rust/C ABI） | 放在 `.mq.md` |
|------------------------|---------------|
| socket listen、连接池、SQL 执行 | 路由表、页面拼装、业务分支 |
| 迁移 runner | 调用 `migrate` / 声明路径 |
| 可选模板引擎热路径 | 主题选择、静态路径 |

遵守：

- ABI v2 + JSON 参数（[ext-abi.md](../design/ext-abi.md)）  
- `ext/web/**` **不**直调 `host_*`；经 `plugin.load` + 注册名（如 `web_listen`、`web_sql`）  
- 不把 Web/DB 域逻辑塞进核心 `HostFn`

---

## 8. 快速样式 / 脚手架

```text
marqdo ext add web
# 可选后续子命令（规划）
marqdo web new myapp --theme=minimal
```

或纯 Marqdo：

```markdown
> web.scaffold dest=./app theme=minimal
```

产出：入口 `.mq.md`、`migrations/001_init.sql`、`static/theme.css`、示例 `.env.example`。

主题要求：少依赖、可离线、与官方文档站黑白极简可区分但不过度设计。

---

## 9. 推荐落地顺序

```text
W0  ext-cli 登记 web；空 L1 + 插件 hello（listen 回固定正文）
W1  env 端口；路由表；静态文件
W2  SQLite + migrate/init；金样（临时目录）
W3  模板/theme minimal + scaffold
W4  Postgres 可选；文档与 public 示例页
```

验收：

1. `marqdo ext add web` 后无手拷即可 `> ext/web/web.mq.md`。  
2. 一键 init → 打开浏览器看到模板页。  
3. 新增迁移文件后再 migrate，数据保留、schema 更新。  
4. 无插件时 L1 给出明确错误（与 agent 一致）。

---

## 10. 开放点

1. 服务模型：阻塞 `listen`（占住 `# main`）vs 后台任务 + `subtask`。  
2. handler 是模块内 `##` 名还是独立 `.mq.md` 路径。  
3. 是否提供 `marqdo web` 子命令，或全部经 `ext` + `.mq.md`。  
4. Windows 下文件锁与 SQLite 路径。

---

## 11. 与其它规划的关系

| 文档 | 关系 |
|------|------|
| [user-site.md](../design/user-site.md) | 静态 gh-pages；本扩展是**动态**服务，互补 |
| [agent-streaming.md](agent-streaming.md) | Web 可作 stream 事件的 EventSource 呈现面（远期） |
| [ext-quantum.md](ext-quantum.md) | 同属官方 ext + ABI + `ext add` 惯例 |
