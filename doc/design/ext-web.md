# 官方扩展：`ext/web` 便捷网站（锁定设计）

| | |
|---|---|
| 状态 | **Accepted（W0–W3 已开工落地；W4 Postgres/Redis/S3 驱动待）** |
| 日期 | 2026-08-11 |
| 相关 | [ext-cli.md](ext-cli.md) · [ext-abi.md](ext-abi.md) · [ext-llm.md](ext-llm.md) · [objects.md](objects.md) · [markdown-mapping.md](markdown-mapping.md) §9 · [user-site.md](user-site.md) · [roadmap/ext-web.md](../roadmap/ext-web.md) · [roadmap/tables-maps-footnotes.md](../roadmap/tables-maps-footnotes.md) |
| 安装 | `marqdo ext add web`（中英：`web` / `网页`） |
| 核心目标 | **让用户用官方模板在分钟级拉起带后台的网站**；自定义走表格与赋值绑定，而不是传统路由/REST 堆叠 |

---

## 0. 一句话

`ext/web` 不是「又一个微框架」。它是一组 **Marqdo 对象类**（页面壳、库表、绑定、可选对象存储与 Redis），热路径进 **ABI v2 插件** `plugins/web`，与 llm/agent 一样用 **`marqdo ext`** 安装；用户写的是字典/列表表格与赋值式绑定，插件负责异步 HTTP、SQL、迁移与存储驱动。

---

## 1. 动机与原则

### 1.1 动机

今日有静态站（[`public/`](../../public/)、[user-site.md](user-site.md)）与客户端 HTTPS（[`lib/net`](stdlib-modules.md)），但没有一等**动态网站**扩展。用户需要：

1. 官方模板一键出站（含**后台管理面板**）。  
2. 导航 / 侧栏 / 页脚用 **GFM 表**改，不用手写组件树。  
3. 表结构用 **字典 + 列表表**声明，开发中可迁移。  
4. 前后端数据用 **赋值式绑定**，取消繁复接口层。  
5. 可选对象存储、Redis；连接信息进 `.env`（与大模型同一套 dotenv 习惯）。  
6. 网络层允许异步（性能），但对作者保持简洁表面。

### 1.2 设计原则

| 原则 | 含义 |
|------|------|
| **模板优先** | 默认路径 = scaffold → 改表 → 跑起来；自定义是覆盖槽位，不是从零搭 |
| **类驱动** | 能力以 `#` 对象暴露（`page` / `db` / `bind` / `store` / `cache` / `app`），中英双面 |
| **表即配置** | 导航、路由、schema、后台菜单优先表格几何（见 §5） |
| **赋值即绑定** | 前端槽位 ← 后端查询，用语句赋值；不强制 REST/OpenAPI |
| **插件热路径** | listen / SQL / migrate / S3 / Redis 在 `plugins/web`；`ext/web/**` 不调 `host_*` |
| **dotenv 一致** | `## load_env` → `sys.load_dotenv`；不覆盖已有进程环境；密钥不入库 |
| **可关可选件** | 对象存储、Redis **非必需**；缺配置则相关对象构造失败信息明确，站点仍可只靠 SQLite 跑 |

### 1.3 非目标（本设计波次）

- 完整 ORM / GraphQL / 微服务治理 / 多租户 SaaS 平台  
- 在核心 `src/host` 固化 Web 框架（逻辑只进 `ext/` + `plugins/`）  
- 与 `marqdo view` 调试 UI 合并为同一产品（可日后互链）  
- 第三方扩展市场  
- 强制用户提交 `.env`

---

## 2. 布局、安装与导入

### 2.1 仓库布局

```text
ext/
  web/
    web.mq.md              # 英文 L1
    网页.mq.md              # 中文 L1
    templates/
      starter/             # 官方快速启动模板（含后台）
        app.mq.md
        admin.mq.md
        .env.example
        migrations/
        static/
          theme.css
    README.md              # 短用法（指向本文）
plugins/
  web/
    Cargo.toml             # marqdo_plugin_web，cdylib name = web
    src/
      lib.rs               # ABI init / 注册
      http.rs              # 异步 HTTP
      db.rs                # SQL + migrate
      store.rs             # 对象存储（可选）
      cache.rs             # Redis（可选）
      admin.rs             # 后台元数据 → CRUD 页面
```

### 2.2 CLI（对齐 [ext-cli.md](ext-cli.md)）

```text
marqdo ext list
marqdo ext add web
marqdo ext remove web
```

实现：在 [`src/ext_cli.rs`](../../src/ext_cli.rs) `CATALOG` 增加：

| 字段 | 值 |
|------|-----|
| `id` | `web` |
| `mq_files` | `web/web.mq.md`, `web/网页.mq.md`（及需要一并安装的 `templates/` 树） |
| `native_crate` | `Some("marqdo_plugin_web")` |

安装根：`MARQDO_EXT` 或 `~/.marqdo/ext`。产物：

- `{root}/web/*.mq.md` + `templates/`  
- `{root}/native/libweb.so`（或 `.dylib` / `.dll`）  
- `{root}/web.plugin` 路径提示  

插件解析顺序同 agent：`web.plugin` → `native/libweb.*` → `MARQDO_WEB_PLUGIN` → `CARGO_TARGET_DIR/{debug,release}/` → …

### 2.3 脚手架（快速启动）

```text
marqdo web new myapp --theme=starter
```

或纯 Marqdo：

```markdown
> web.scaffold dest=./myapp theme=starter
```

产出目录（锁定）：

```text
myapp/
  index.mq.md          # # main → load_env → migrate → listen
  admin.mq.md          # 后台入口（可被模板页挂进侧栏）
  .env.example
  migrations/
    001_init.sql       # 由声明式 schema 生成或手写起步
  static/
    theme.css
  data/                # gitignore；SQLite 默认路径
```

验收口径：**复制模板、填 `.env`、`marqdo run index.mq.md`，浏览器打开即见前台 + `/admin` 后台。**

### 2.4 导入

```markdown
---
> ext/web/web.mq.md
---
```

中文：`> ext/web/网页.mq.md`。

L1 启动时：

```markdown
*`p` = > plugin.native_path name=web *
> plugin.load path=`p`
```

无插件时错误文案与 agent 一致（提示 `cargo build -p marqdo_plugin_web` / `marqdo ext add web`）。

---

## 3. 对象模型（类）

全部为 `#` 对象 + `##` 方法；中英成对。英文名稳定进插件注册前缀 `web_*`；中文为 L1 别名面。

| 英文类 | 中文类 | 职责 |
|--------|--------|------|
| `# app` | `# 应用` | 组合页面/库/绑定；`listen`；挂静态与后台 |
| `# page` | `# 页面` | 壳布局：顶栏 / 侧栏 / 主体 / 底栏 |
| `# db` | `# 数据库` | 连接、声明表、migrate、查询/写入 |
| `# bind` | `# 绑定` | 记录「前端槽位 ← 后端源」；服务时求值 |
| `# store` | `# 对象存储` | 可选 S3 兼容上传/下载/URL |
| `# cache` | `# 缓存` | 可选 Redis get/set/expire |

继承：用户可用 `# MyPage = > page` 覆盖 `## render` 等（见 [objects.md](objects.md) §5）；官方模板示范覆盖主体栏。

```mermaid
flowchart LR
  env[".env"] --> app
  env --> db
  env --> store
  env --> cache
  page --> app
  db --> bind
  bind --> page
  store --> app
  cache --> app
  app --> http["plugins/web async HTTP"]
  db --> sql["plugins/web SQL"]
  app --> admin["/admin panel"]
```

---

## 4. `.env` 配置（与 LLM 同惯例）

### 4.1 加载

```markdown
> web.load_env path=.env
```

实现：`## load_env` → `sys.load_dotenv`（不覆盖已有环境变量）。路径相对**源文件目录**（与 `ext/ai/llm` 一致）。

### 4.2 键名（锁定）

```env
# —— HTTP（必填意图：有默认）——
MARQDO_WEB_HOST=127.0.0.1
MARQDO_WEB_PORT=8080
# 可选：对外 URL（对象存储签名、后台链接）
MARQDO_WEB_PUBLIC_URL=http://127.0.0.1:8080

# —— 关系库（默认 SQLite）——
DATABASE_URL=sqlite:./data/app.db
# DATABASE_URL=postgres://user:pass@127.0.0.1:5432/mydb

# —— Redis（可选）——
# REDIS_URL=redis://127.0.0.1:6379/0
# 或
# MARQDO_REDIS_URL=redis://127.0.0.1:6379/0

# —— 对象存储 S3 兼容（可选）——
# MARQDO_S3_ENDPOINT=https://s3.amazonaws.com
# MARQDO_S3_REGION=us-east-1
# MARQDO_S3_BUCKET=my-bucket
# MARQDO_S3_ACCESS_KEY=
# MARQDO_S3_SECRET_KEY=
# MARQDO_S3_PUBLIC_BASE=https://cdn.example.com   # 可选公开读基址
```

规则：

| 类别 | 缺失时 |
|------|--------|
| HTTP | 默认 `127.0.0.1:8080` |
| `DATABASE_URL` | 默认 `sqlite:./data/app.db`（相对项目目录） |
| Redis / S3 | **不构造**对应对象；`# cache` / `# store` 返回明确 `configured=false`；站点其它部分仍可用 |

前缀：`MARQDO_WEB_*` / `MARQDO_S3_*` / `MARQDO_REDIS_*` 避免与 `OPENAI_*` 冲突；`DATABASE_URL` / `REDIS_URL` 为业界通名别名（读取时优先进程已有值）。

---

## 5. 表格即配置（字典 + 列表）

沿用已落地表格几何（[tables-maps-footnotes.md](../roadmap/tables-maps-footnotes.md)）：

| 几何 | 运行时 | 在 web 中的用途 |
|------|--------|-----------------|
| 1 列竖表 | `List` | 简单菜单项、角色名 |
| ≥2 列 + 多行且首列表头为 `@`/`行`/`row` | `List` of `Map` | **导航 / 侧栏 / 底栏 / schema 字段行** |
| ≥2 列字典表 | `Map` | 主题色、功能开关 |

### 5.1 页面壳与主体：先对象，再方法绑定

```markdown
`nav` = | 前端变量 | 后端数据库 | 绑定css样式 | …

`articles` = | 字段 | 类型 | 可空 | …

*`db` = > web.db *
*`articles` = > `db`.init name=articles fields=`articles` *

`主体` = | 前端变量 | 后端数据库 | 绑定css样式 | …

*`page` = > web.page title=Demo *
*`page` = > `page`.nav table=`nav` *
*`page` = > `page`.main table=`articles` bind=`主体` layout=cards
```

**变量名即表名：** `` `articles` `` 先是结构表，再被 `init name=articles fields=`articles`` 收成 `{_type:db_table,name:articles}` 句柄；之后 `` table=`articles` `` 处处可读。框架**无**默认表名。

### 5.2 数据库：表结构 = GFM 表

```markdown
`帖子结构` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| title | text | false |
| body | text | true |

*`库` = > web.db *
> `库`.define table=posts fields=`帖子结构` primary=id
> `库`.migrate
```

列名别名：`字段`/`name`/`列`；`类型`/`type`；`可空`/`null`；可选 `默认`/`唯一`。

行为锁定：

1. `define` 经 `as_fields` 吃表格，再 `CREATE TABLE IF NOT EXISTS`。  
2. `migrate`：对照 `_marqdo_migrations`，应用 `migrations/*.sql`。  
3. 默认 SQLite；Postgres 仍属 W4。

### 5.3 后台菜单

`admin=True` 时自动枚举库中用户表，每张表支持 **增删改查**（列表 / New / Edit / Delete），写入自动记入 `_marqdo_admin_log`（`/admin/log` 可查）。

---

## 6. L1 API 外形（锁定草稿）

### 6.1 `# page` / `# 页面`

| 方法 / 参数 | 含义 |
|------|------|
| `## nav` / `## sidebar` / `## footer` | `table=` 为三列绑定表 |
| `## main` | `table=` 为 `db_table` 句柄（或裸名），`bind=` 为列绑定表 |
| `intro=` / `layout=` | 构造或 `main` 时可设 |
| `## render` | 求值绑定 + 默认主题 HTML |

### 6.2 `# db` / `# 数据库`

| 方法 | 含义 |
|------|------|
| `## init` | `name=` + `fields=`（GFM 结构）；返回 `db_table` 句柄；**变量名应与 name 相同** |
| `## define` | `init` 别名（`table=` 可为句柄） |
| `## follow` / `## all` / `## insert` | `table=` 接受句柄或裸名 |
| `## migrate` / `## query` | 同前 |

### 6.3 绑定 = 变量名即表名 + 三列表

```markdown
`articles` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| title | text | false |

*`db` = > web.db *
*`articles` = > `db`.init name=articles fields=`articles` *

`主体` =

| 前端变量 | 后端数据库 | 绑定css样式 |
|----------|------------|-------------|
| title | title | card-title |

*`页` = > web.page title=站 *
*`页` = > `页`.main table=`articles` bind=`主体` layout=cards *
```

插入仍用 GFM 行表：`` > `库`.insert table=`articles` rows=`种子` ``。

### 6.4 `# store` / `# 对象存储`（可选）

| 方法 | 含义 |
|------|------|
| 构造 | 读 `MARQDO_S3_*`；未配置 → `ok=false` |
| `## put` | `key=` `path=` 或 `bytes=` |
| `## get_url` | 公开基址或预签名 |
| `## delete` | 删对象 |

后台「媒体」页在配置存在时显示上传；否则隐藏。

### 6.5 `# cache` / `# 缓存`（可选）

| 方法 | 含义 |
|------|------|
| 构造 | 读 `REDIS_URL` / `MARQDO_REDIS_URL` |
| `## get` / `## set` | `key=` `value=` `ttl=` |
| `## delete` | |

用于绑定结果缓存、会话票（v1 会话可先 cookie + 服务端 map，Redis 为加速路径）。

### 6.6 `# app` / `# 应用`

```markdown
# main

> web.load_env path=.env
*`p` = > plugin.native_path name=web *
> plugin.load path=`p`

*`db` = > web.db *
> `db`.init
*`page` = > web.page theme=starter nav=`nav` sidebar=`side` footer=`foot` *
*`bind` = > web.bind page=`page` db=`db` *
> `bind`.set slot=main.items source=posts

*`app` = > web.app page=`page` bind=`bind` db=`db` admin=True *
> `app`.static dir=./static
> `app`.listen
```

| 方法 | 含义 |
|------|------|
| `## static` | 挂静态目录 |
| `## mount` | `path=` `page=` 额外页面 |
| `## listen` | 阻塞当前 `# main`：内部跑**异步**运行时直到进程结束 |
| `## admin` | `True` 时挂载 `/admin`（默认 True 于 starter） |

中文：`加载环境` / `应用` / `监听` / `静态` / `绑定` / `数据库` / `页面`。

---

## 7. 后台管理面板

### 7.1 目标

官方模板**默认带后台**：登录后按 schema 对表做列表 / 新建 / 编辑 / 删除；菜单可表覆盖（§5.3）。

### 7.2 路由约定（锁定）

| 路径 | 行为 |
|------|------|
| `/admin` | 仪表盘（表计数、快捷入口） |
| `/admin/login` | 登录 |
| `/admin/{table}` | 列表 |
| `/admin/{table}/new` | 新建 |
| `/admin/{table}/{id}` | 编辑 |

### 7.3 认证（v1）

```env
MARQDO_WEB_ADMIN_USER=admin
MARQDO_WEB_ADMIN_PASSWORD=change-me
```

- 会话：签名 cookie（密钥 `MARQDO_WEB_SECRET`，缺省则由插件首次启动生成并打印警告）。  
- v1 **不做** OAuth；可后续加。  
- 未设密码时 starter 仅绑定 `127.0.0.1` 并警告。

### 7.4 与绑定关系

后台写库 → 前台 `bind` 每请求读库 → 用户**无需**自写同步 API。

---

## 8. 异步网络（实现细节）

### 8.1 为何异步

HTTP + DB + S3/Redis 适合在插件内用异步多路复用；避免一请求一线程撑满。

### 8.2 边界

| 层 | 模型 |
|----|------|
| Marqdo `# main` | `` > `app`.listen `` **同步阻塞**至停机（作者心智简单） |
| `plugins/web` | 内部 **tokio**（或同等）异步：accept、handler、连接池 |
| 绑定 / 页面 render | 在异步 handler 中调用；必要时 `spawn_blocking` 跑短 Marqdo 求值 |

不在本波把「异步」关键词暴露进 Marqdo 语法；性能优化留在插件。

### 8.3 注册名（ABI 草案）

| 注册名 | 参数（CSV） | 作用 |
|--------|-------------|------|
| `web_listen` | `host,port,routes,static_dir` | 启动异步服务器（阻塞至停） |
| `web_db_open` | `url` | 打开池 |
| `web_db_migrate` | `dir` | 执行迁移 |
| `web_db_exec` | `sql,args` | 执行 |
| `web_db_query` | `sql,args` | 查询 → JSON 行数组 |
| `web_store_put` | `key,path` | 可选 |
| `web_store_url` | `key,ttl` | 可选 |
| `web_cache_get` / `web_cache_set` | `key` / `key,value,ttl` | 可选 |
| `web_admin_meta` | `schema` | 后台元数据 |

ABI：**v2** + JSON 参数/返回（[ext-abi.md](ext-abi.md)）。`ext/web/**` 只经 `plugin.load` 调上述名。

### 8.4 路由如何从「表 + 绑定」生成

传统写法（**不作为主推**）：

```text
GET /api/posts → handler → JSON
```

本扩展主推：

1. 顶栏/侧栏 `href` → 服务端渲染的 **页面路由**（HTML）。  
2. 数据来自 `bind`，不是并行维护一套 `/api/*`。  
3. 后台 CRUD 由插件按表自动生成 HTML 表单；若用户需要 JSON，可选用 `## api=True` 挂只读导出（非默认）。

---

## 9. 官方模板（starter）内容

| 文件 | 作用 |
|------|------|
| `index.mq.md` | 前台：nav/side/foot 表 + bind posts + listen |
| `admin` 挂载 | `app admin=True` |
| `migrations/001_init.sql` | `posts` 示例表 |
| `static/theme.css` | 单文件主题；可读、低依赖、非「紫渐变 AI 模板」 |
| `.env.example` | §4.2 全键注释 |

自定义路径：

1. **改表**：导航/字段/菜单。  
2. **改绑定**：`bind.set` 换源表或加 `limit`。  
3. **换主题**：`theme=` 或覆盖 `static/`。  
4. **继承页面类**：只重写 `## body`。  
5. **关后台**：`admin=False`。

---

## 10. 与静态站 / `lib/net` 的关系

| 能力 | 归属 |
|------|------|
| gh-pages 静态文档 | [user-site.md](user-site.md) / `public/` |
| 出站 HTTPS 客户端 | `lib/net`（llm 已用） |
| 入站站点 + DB + 后台 | **`ext/web`（本文）** |

三者互补，不合并命令。

---

## 11. 落地阶段（实现时遵循）

详见 [roadmap/ext-web.md](../roadmap/ext-web.md)。摘要：

| 阶段 | 内容 | 验收 |
|------|------|------|
| **W0** | CATALOG + 空 L1 + 插件 `web_listen` 回固定 HTML | `ext add web`；浏览器见 Hello |
| **W1** | `.env` 端口；`page` 四栏；nav 表 → 路由；static | 改 nav 表即改链接 |
| **W2** | SQLite `define`/`migrate`/`all`；金样临时目录 | init 后表存在 |
| **W3** | `bind.set` + starter 模板 + `/admin` CRUD | 模板一键站 |
| **W4** | Postgres；可选 Redis / S3；文档与示例 | 可选件缺省不炸 |

金样目录建议：`tests/ext/web-*.mq.md`；live 类需 `.env` 的单独门控。

---

## 12. 错误与安全基线

| 项 | 约定 |
|----|------|
| 无插件 | L1 打印可操作错误后 `sys.exit` |
| SQL | 用户 `query` 必须参数化；后台生成语句只用白名单列名（来自 schema） |
| 上传 | 限制大小与扩展名；默认仅管理员 |
| CORS | v1 同站 SSR 为主；不默认放开 `*` |
| 密钥 | `.env` / `.gitignore`；文档强调勿提交 |

---

## 13. 文档与索引更新清单（实现前）

- 本文：`doc/design/ext-web.md`（锁定设计）  
- 路线图：`doc/roadmap/ext-web.md`（阶段与开放点收敛）  
- [ext-cli.md](ext-cli.md) catalog 行指向本文  
- [doc/README.md](../README.md) 增加 design 链接  

实现开始时再动：`src/ext_cli.rs`、`plugins/web`、`ext/web/*`、`Cargo.toml` workspace、`tests/gold.rs`。
