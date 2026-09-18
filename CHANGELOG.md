# Changelog

## Unreleased

### Added

### Fixed

### Changed

## Ext pack v1.0.6 — 2026-09-18

### Highlights

**ext/web OIDC、表单删除与安静管理台（CLI 仍为 v1.0.3）**：OIDC 登录/登出回首页、表单 `删除` 动作、默认关闭内置 admin/RBAC desk、独立 Redis 会话 URL、Postgres CRUD。安装：

```bash
marqdo ext add web   # CDN latest → 1.0.6
```

### Added
- **OIDC**：`auth.oidc` / 网页鉴权 OIDC；登录入口与 `next` 回跳。
- **表单删除**：`form` 动作 `删除`/`delete`（通用 CRUD，无站点特判）。
- **列表/路由**：请求上下文、列表装配延续；注销默认回 `/` 避免 IdP 再登录环。

### Fixed
- 内置 admin / RBAC desk 默认安静，直至显式 `admin` / `enable_rbac`。
- 静态资源缓存与 WASM 启动延迟；会话 CSRF 与 Postgres 方言。

### Changed
- 扩展包 SemVer **1.0.6**（`ext/VERSION`）；与 CLI **v1.0.3** 可独立安装。

## Ext pack v1.0.5 — 2026-09-18

### Highlights

**ext/web 请求上下文与表单来源（CLI 仍为 v1.0.3）**：字段 `来源`/`source`（session / route / now）、server-only 盖戳、页面 `列表装配`/`compose_list`；禁止按业务表特判。安装：

```bash
marqdo ext add web   # CDN latest → 1.0.5
```

### Added
- **Request context**：表单提交携带 `_mq_params` / `_mq_return`；`ApplySources` 从 session/route/now 写入。
- **字段来源列**：`client`（默认）/ `session.username` / `route.NAME` / `now`；非 client 不渲染且覆盖客户端伪造。
- **列表装配**：`compose_list` / `列表装配`（副绑定 + 条件/排序/插槽），详情页可同页渲染子列表。
- 设计文：[ext-web-request-context.md](doc/design/ext-web-request-context.md)；金样 `ext_web_form_source_live`。

### Changed
- 扩展包 SemVer **1.0.5**（`ext/VERSION`）；与 CLI **v1.0.3** 可独立安装。
- 移除评论表名特判（`enrichCommentSubmit`）。

## Ext pack v1.0.4 — 2026-09-17

### Highlights

**ext/web 能力面扩展（CLI 仍为 v1.0.3）**：可配置 RBAC、租户作用域、WS `on_message`/presence、原生 SPA 内存路由、协同记事本 MVP。安装：

```bash
marqdo ext add web   # CDN latest → 1.0.4
```

### Added
- **RBAC**：五表（`web_permissions` / `web_roles` / `web_role_permissions` / `web_users` / `web_user_roles`）；`app.enable_rbac`、`app.gate permissions=`、`rbac.can`、注册页、`/_rbac/*` 管理 API/desk。
- **Tenant**：`app.tenant`（path/header/subdomain）+ `tenant_scope` 自动过滤/盖戳。
- **WebSocket G-WS2**：`route_ws` 支持 `on_message` 钩子、动态 room、`presence` 通道。
- **Native SPA**：内存 `store_set` / `spa_goto` 与全页 WASM SPA 样站。
- **Collab MVP**：LWW 文档协议（`DOC\t`）双端协同记事本样站。

### Changed
- 扩展包 SemVer **1.0.4**（`ext/VERSION`）；与 CLI **v1.0.3** 可独立安装。

## v1.0.3 — 2026-09-17

### Highlights

**分发渠道齐备**：Ubuntu PPA 装 CLI；扩展包走 CDN（`ext.marqdo.com`）与 GitHub；CLI 与扩展可分开发版。本版 CLI 与扩展包 **同步为 1.0.3**。

**语法**：未绑定的 GFM 表格是可丢弃的集合值，不再报 `unrecognized statement`（文档页可直接写安装表）。

```bash
# Ubuntu
sudo add-apt-repository ppa:cflmy/marqdo
sudo apt update && sudo apt install marqdo
marqdo ext add web

# 或源码 / Releases
git checkout v1.0.3 && cargo build --release
```

### Added
- Ubuntu PPA packaging: `debian/` + `scripts/ppa-ship.sh` (一键装工具/打包/dput) + `ppa-build-source.sh` + [ubuntu-ppa.md](doc/design/ubuntu-ppa.md).
- **Extension CDN** (`https://ext.marqdo.com` / Cloudflare R2)：`ext add` 下载顺序改为 CDN → GitHub → proxy；独立包版本见 `ext/VERSION` 与 CDN `latest/VERSION`；上传脚本 `scripts/upload-ext-r2.py`（凭据 `~/.marqdo/r2.env`）。设计见 [ext-cdn.md](doc/design/ext-cdn.md)。
- `ext add` 下载：GitHub 失败后自动回退 `https://proxy.cflmy.top/github.com/…`；可用 `MARQDO_EXT_DOWNLOAD_BASE` 指定首选镜像；加长读写超时。
- 用户文档（README / `public/00-欢迎` / `features/05-扩展`）标明安装渠道：Ubuntu PPA、GitHub Releases、源码；扩展走 CDN。

### Fixed
- `ext/` 导入默认搜索 `~/.marqdo/ext`（与 `marqdo ext add` 安装根一致），无需再手动 `export MARQDO_EXT`。
- PPA/noble：捆绑 `third_party/rust`（现代 cargo）以构建 Cargo.lock v4；修复首次 Launchpad 构建失败（distro cargo 1.75）。
- 未绑定的 GFM 表格不再报 `unrecognized statement`：作为集合值求值后丢弃（文档中可写表）。

### Changed
- 允许 **CLI 与扩展包分开发版**：扩展包 SemVer 可与 `Cargo.toml` 不同（`MARQDO_EXT_VERSION` / `ext/VERSION` / CDN latest）。
- 发版 skill：支持 `full` / `cli` / `ext`；扩展必传 R2+GitHub；CLI 走 PPA；需用户操作时打印确切命令。

## v1.0.2 — 2026-09-16

### Highlights

**表驱动页面引言（`compose_intro` / `引言装配`）**：英雄区用 GFM「属性 / 值 / 样式」表声明，不再手写 HTML 字符串；WWW Demo 样站与后台门禁一并加固。

```bash
git checkout v1.0.2
cargo build --release
bash ./scripts/build-web-plugin.sh
# 或：解压 Release 的 marqdo-1.0.2-*-native-*.zip / Linux CLI zip
marqdo run examples/web-site-zh/index.mq.md
```

### Added
- Go `libweb`：`web_compose_intro` — 将引言表装配为 `.main-intro` HTML；中英 API `页面.引言装配` / `page.compose_intro`（`ext/web`）。
- 验收样例：`tests/ext/web-intro-compose.mq.md`；`examples/web-site` / `web-site-zh` 升级为表驱动引言 + 文学风壳层 + 卡片→详情路由。

### Fixed
- 后台样站：导航/侧栏/页脚槽位与门禁登录挂载对齐；表单字段加宽便于创建。

### Changed
- Demo 叙述与种子数据面向 WWW Industry / 可复现演示；作者仍零业务 JS、零手写 `.css`。

## v1.0.1 — 2026-09-15

### Highlights

**ext/web（Go libweb）回归修复**：动态路由查询条件恢复 `{param}` 代入；详情 SSR + Markdown；样式表数字不再误报。

```bash
git checkout v1.0.1
cargo build --release
bash ./scripts/build-web-plugin.sh
# 或：解压 Release 的 marqdo-1.0.1-*-native-*.zip / Linux CLI zip
marqdo run examples/marqdo-blog/index.mq.md
```

### Added
- Go web plugin：详情页正文 Markdown（goldmark GFM），对齐 Rust `pulldown-cmark` 路径。

### Fixed
- **动态路由 `{param}` 未代入查询条件**（`ext/web` Go）：`selectPageData` 递归展开 `query` 全部字符串叶子；`data-slot-src` / `_route` 同步替换。`examples/marqdo-blog` `/post/{slug}` 详情 SSR 恢复。
- **样式表数字单元格误报「可疑 `/` 除法」**：`cssValueSuspicious` 不再把合法 `z-index`/`opacity` 等 number 当警告。
- Go 1.22+ ServeMux：`/static/` 与 `/static/favicon.svg` 的 GET/HEAD 注册冲突导致博客启动 panic。
- Rust archive 同步：`resolve_placeholders` 递归 + CSS number 误报收窄（非默认构建路径）。

## v1.0.0 — 2026-09-14

### Highlights

**Markdown 语法映射 v0.3 成为 1.0 宪法**：`**粗体**` = 代码，`*斜体*` = 返回，叙述里的 `` `名` `` 声明并推断形参。这是相对 v0.4 / 旧 `*语句*`/`**返回**` 的破坏性对调。Linux 预编译拓展包与 CLI bundle 为 Release 必出资产。

```bash
git checkout v1.0.0
cargo build --release
# Linux：也可直接解压 Release 的 marqdo-1.0.0-x86_64-unknown-linux-gnu.zip
marqdo run tests/markup-v03/inc-prose.mq.md
```

### Added
- **Markup v0.3 运行时闭环**：粗体=`代码`、斜体=`返回`、叙述 `` `名` `` 声明与形参推断为唯一映射；金样 `tests/markup-v03/`。
- **方括号标记调用**与前置修饰语：`礼貌 [问候] x`；取元主语法 `[键](集合)`（脚注取元过渡保留）。
- **代码即文档叙述面 G1–G4**：模块级 `---` 跳过、装饰性强调、浮点面量、动态键。
- **粗体表达式语句** `**n + 1**`；`--dump-lines` 标注 `[decl]/[bold]/[italic]`；`--dump-ast` 显示 inferred 形参与死绑定。
- **Release Linux 包必出**：`linux` job 上传 `*-native-x86_64-unknown-linux-gnu.zip` 与 Linux CLI bundle。

### Fixed
- 行首 `` `名`… `` 叙述不再当语句（修复 pages CI：`public/features/16-fs-mid2.mq.md`）。
- 句中斜体强调（`*very important*` / `*foo.bar*`）不再误收束函数；Decl 名须为标识符；残缺粗体保持 GAP 恢复。
- 胶合 `` `名`=default …prose `` 为 Decl 而非赋值。

### Changed
- 教程 / Skill / `lib` / `ext` / `public` 对齐 v0.3 标记；view 推断形参 `chip inferred`。

## v0.4.0 — 2026-09-13

### Highlights

**Go `libweb` 默认落地（W-G0–W-G14）**：官方网页原生插件改为 Go C ABI；CLI / gold / Release 走 `scripts/build-web-plugin.sh`；Rust 源码归档。另附 **anlian-mq** 验收站与表单插槽 / `MARQDO_EXT` 提示修复。

```bash
git checkout v0.4.0
# 构建 web 原生插件（需 Go + cgo）：
bash ./scripts/build-web-plugin.sh
marqdo ext add web
marqdo run examples/anlian-mq/index.mq.md
```

### Added
- **`plugins/web` Go 全量重写（W-G0–W-G13）**：C ABI 对等装配/DB/表单/listen/鉴权/上传/SEO/WS/驱动/proxy·invoke/定制壳；`scripts/build-web-plugin.sh` → `libweb.so` / `web.dll`；ADR [0004](doc/adr/0004-web-plugin-go.md) · 施工 [ext-web-go-rewrite.md](doc/design/ext-web-go-rewrite.md)。
- **W-G14 验收**：`examples/anlian-mq/`（暗恋见君风格论坛：路由/登录/粉玻璃主题/片段 API；SQLite 本地可跑）。
- **GAP-11**：`compose_form` / `表单装配` 支持 `target` / `表单插槽`（如 `#id`），把表单注入引言 HTML 内的挂载点。

### Fixed
- **GAP-12**：`ext/…` import 解析失败且未设置 `MARQDO_EXT` 时，错误附带 `hint: set MARQDO_EXT to the directory that contains web/`。
- **listen 会话 / CSRF**：登录后 `IssueCookie` 不再误设 `Max-Age=0`；页面 ABIs（`query`/`order`/`link_prefix`/`css`/`detail`）与 `web_page_chrome`、`invoke return=html` 对齐 anlian 需求。

### Changed
- **默认 `libweb` 为 Go**：Release / gold / `ext add web` 不再构建 Rust `marqdo_plugin_web`；旧 Rust 树移至 `plugins/web-rust-archive/`（非 workspace 成员）。作者面 `ext/web` ABI 名保持兼容。

## v0.3.9 — 2026-09-09

### Highlights

**业务联调缺口 GAP-06–10**：解析器不再吞掉嵌套反引号/`text=[…]` 后的函数体；`invoke` 的 `Null` 不再伪装成功；`sys.exec capture=` 可取 stdout；流式 proxy 反缓冲头。

```bash
git checkout v0.3.9
```

### Added
- **`sys.exec` / `系统.执行`**：可选 `capture=` / `捕获=`；为真时返回 `{code, stdout, stderr}`（GAP-09）。

### Fixed
- **GAP-06/07**：`*…*` 分类对反引号/字符串内的 `*` 不再误判；错误收尾的赋值行不再吞掉后续 `##` 体（`>` / 合法 `*…*` 可脱出注释段）；命名参数边界尊重 `[…]`/`{…}`。
- **GAP-08**：`app.invoke` 对 `Null`/`None` 返回不再伪装成 `{"ok":true}`，改为 500 + `{"ok":false,"error":"null result"}`。
- **GAP-10**：`app.proxy` `stream=true` 响应补齐 `Cache-Control: no-cache, no-transform` 与 `X-Accel-Buffering: no`。

### Changed

## v0.3.8 — 2026-09-09

### Highlights

**中层标准库 Mid2（M7–M10）收口**：日历/时区、URL、TOML、HTML 转义、base32、实用 fs、薄统计与日志字段——日常脚本与工具配置不再依赖外联或插件。另修复嵌套插件调用后 `GLOBAL_HOST` 被清空导致 `invoke` 失败。

```bash
git checkout v0.3.8
# 或下载 Release 的 exe / zip
marqdo run public/features/14-datetime-url.mq.md
marqdo run public/features/17-stats-log.mq.md
```

### Added
- **中层标准库 Mid2 M10**：`lib/stats` / `统计`（mean/median/stdev）与 `log`/`日志` 可选 `fields=`；金样与 public `17-stats-log`。**Mid2 M7–M10 收口**（zip 跳过）。设计 [stdlib-stats-log.md](doc/design/stdlib-stats-log.md)。
- **中层标准库 Mid2 M9**：`fs.make_dirs` / `remove_tree` / `stat` / `walk`（中英）；金样与 public `16-fs-mid2`。设计 [stdlib-fs-mid2.md](doc/design/stdlib-fs-mid2.md)。
- **中层标准库 Mid2 M8**：`lib/toml` / `配置表`（只读子集）、`lib/html` / `超文本`（escape/unescape）、`encoding` base32；金样与 public `15-toml-html-encoding`。设计 [stdlib-toml.md](doc/design/stdlib-toml.md) · [stdlib-html.md](doc/design/stdlib-html.md)。
- **中层标准库 Mid2 M7**：`lib/datetime` / `日期时间`（`{unix,zone,iso}` · parse/format/add/in_zone）与 `lib/url` / `地址`（parse · query_parse/stringify）；金样与 public `14-datetime-url`。设计 [stdlib-datetime.md](doc/design/stdlib-datetime.md) · [stdlib-url.md](doc/design/stdlib-url.md)。
- **文档**：中层标准库 Mid 之后缺口盘点与 Mid2（M7–M10）设计/路线——[stdlib-gaps-after-mid.md](doc/research/stdlib-gaps-after-mid.md) · [stdlib-mid2.md](doc/design/stdlib-mid2.md) · [stdlib-mid2.md](doc/roadmap/stdlib-mid2.md)。

### Fixed
- **`host_query` / GLOBAL_HOST**：嵌套 `call_registered` 结束后恢复外层宿主指针，不再清空；`web_listen` 多轮 `invoke`（中间再调插件）不再报 `no active host context`。

### Changed


## v0.3.7 — 2026-09-07

### Highlights

**宿主集成 H1–H4**：浏览器同域 SSE/HTTP 流式中继（`app.proxy`）、HTTP→用户 `##`（`app.invoke`）、跨模块 db 方法分发、`corpus_search` 可发现、MCP Server stdio——下游可去掉 Python 旁路。

```bash
git checkout v0.3.7
# 或下载 Release 的 exe / zip
marqdo ext add web && marqdo ext add agent
# *app*.proxy / *app*.invoke · agent.mcp_server
```

### Added
- **宿主集成 H1–H4**：`ext/web` `app.proxy`（同域 SSE/HTTP 流式中继）与 `app.invoke`（HTTP→`lib.member`）；跨模块 `#` 对象方法查找；`agent.corpus_search` 自动加载插件；`agent.mcp_server` stdio。金样 `web-proxy-invoke-smoke` · `web-hosting-live` · `web-db-cross-module-smoke` · `agent-mcp-server-smoke`。设计 [ext-hosting-fill.md](doc/design/ext-hosting-fill.md) · [ext-hosting.md](doc/roadmap/ext-hosting.md)。
- **文档**：宿主集成缺口盘点——[ext-hosting-gaps.md](doc/research/ext-hosting-gaps.md)。

### Fixed
- **方法分发**：`find_object_type` 沿导入树传递查找，跨模块拿到的 `web.db` 等句柄可再调方法。

### Changed

## v0.3.6 — 2026-09-07

### Highlights

**中层标准库 Mid M1–M6 收口**：通用原语沉入 `lib/`（正则、编解码、路径、哈希、机密、CSV、加厚文本/集合、CLI、日志、UUID），写日常脚本与多数扩展库不再需要原生插件。另含 `ext/linalg` 公式文档面与 agent/llm 稳健性修复。

```bash
git checkout v0.3.6
# 或下载 Release 的 exe / zip
marqdo run public/features/13-log-uuid.mq.md
marqdo ext add linalg   # 可选：公式文档 / 线性代数
```


### Added
- **中层标准库 M6**：`lib/log` / `日志`（级别过滤的一行日志）与 `lib/uuid` / `标识`（v4）；金样与 public `13-log-uuid`。设计 [stdlib-log-uuid.md](doc/design/stdlib-log-uuid.md)。**Mid M1–M6 收口。**
- **中层标准库 M5**：`table`/`表` 加厚（sort/sort_by/unique/chunk/zip/flatten）与 `lib/cli`/`命令行`（parse）；金样与 public `12-list-cli`。设计 [stdlib-list-cli.md](doc/design/stdlib-list-cli.md)。
- **中层标准库 M4**：`lib/csv` / `逗号表`（parse/stringify）与 `lib/text` / `文本` 加厚（contains/replace/case/pad/…）；金样与 public `11-csv-text`。设计 [stdlib-csv.md](doc/design/stdlib-csv.md) · [stdlib-text-mid.md](doc/design/stdlib-text-mid.md)。
- **中层标准库 M3**：`lib/hash` / `哈希`（sha256/sha1/md5/hmac_sha256）与 `lib/secrets` / `机密`（token_hex/token_urlsafe）；金样与 public `10-hash-secrets`。设计 [stdlib-hash.md](doc/design/stdlib-hash.md) · [stdlib-secrets.md](doc/design/stdlib-secrets.md)。
- **中层标准库 M2**：`lib/path` / `lib/路径`（join/split/normalize/…）与 `fs`/`文件` 加厚（`copy_file`/`move`/`make_temp`）；金样与 public `09-path-fs`。设计 [stdlib-path.md](doc/design/stdlib-path.md) · [stdlib-fs-mid.md](doc/design/stdlib-fs-mid.md)。
- **中层标准库 M1**：`lib/re` / `lib/正则`（`is_match`/`find`/`find_all`/`replace`/`split`）与 `lib/encoding` / `lib/编码`（base64/hex）；host `host_re_*` + `host_encoding_*`；金样与 public `08-re-encoding`。设计 [stdlib-mid.md](doc/design/stdlib-mid.md) · [stdlib-re.md](doc/design/stdlib-re.md) · [stdlib-encoding.md](doc/design/stdlib-encoding.md)。
- **`ext/linalg`**：L0–L6 + 公式文档面 **F1–F3**（中缀、`declare`、展示/`explicit` 默认化简、`T_ascii`、public「如何写公式文档」）；示例 `linalg-transpose` · `linalg-svd` · `linalg-least-squares`。设计：[ext-linalg-formula-doc.md](doc/design/ext-linalg-formula-doc.md)。

### Fixed
- **`ext/agent`**：写回改走 `wb.record`（避免 `writeback` 形参遮蔽模块名），不再直调 `host_*`。
- **`ext/llm`**：`stream_result` 忽略未知 SSE 类型（如 `reasoning`），不再因此退出。

### Changed

## v0.3.5 — 2026-09-04

### Highlights

**`marqdo ext add` 无需本机 Rust**：Release 发布预编译 native 插件；CLI 可自动下载 L1（`*-ext.zip`）与 `*-native-*.zip`。Windows 便携包内含 `ext/native/*.dll`。

```bash
git checkout v0.3.5
# 或下载 Release 的 exe / zip
marqdo ext add web
marqdo ext add agent
marqdo ext add quantum
```

### Added
- **预编译原生插件资产**：`marqdo-*-native-x86_64-pc-windows-msvc.zip`、`marqdo-*-native-x86_64-unknown-linux-gnu.zip`；Windows zip 捆绑 `ext/` + native DLL。
- **`ext add` 下载回退**：本地无 `target/` 时从 GitHub Release 拉取；`MARQDO_EXT_NO_DOWNLOAD=1` / `MARQDO_EXT_VERSION` 可配置。

### Changed
- 文档：[ext-cli.md](doc/design/ext-cli.md)、[08-ext-deploy-coupling](doc/design/ext-web-customization/08-ext-deploy-coupling.md)、README 安装步骤以预编译为默认路径。
- Release workflow：Windows 编插件并上传 native zip；新增 Linux native job。

## v0.3.4 — 2026-09-04

### Highlights

**`ext/web` 定制波次 C0–C4 收口**：可配置 admin 前缀与登录回跳、段边界 gates、shell/layout、样式单元格 strict、脚本 defer/version/`asset_version`、条件导航 when/media。附智能体 Wave B0 示例与写回遮蔽修复。

```bash
git checkout v0.3.4
cargo build --release
cargo build --release -p marqdo_plugin_web
marqdo ext add web
# golds: tests/ext/web-c0-admin-prefix … web-c4-nav-when
```

### Added
- **智能体 Wave B0**：宪法优先示例 [agent-pong](examples/agent-pong/)（call_site/源码注入）· [agent-okf-flywheel](examples/agent-okf-flywheel/)（OKF `cache=hit`）；调研 [agent-framework-2026-09.md](doc/research/agent-framework-2026-09.md)；离线 harness [`scripts/agent-harness.sh`](scripts/agent-harness.sh)；金样 `agent-constitution-b0`。
- **`.env.example`**：开发期 OpenAI 兼容网关变量模板（密钥仅本地 `.env`）。

### Fixed
- **`agent.step` / `plan` 写回**：参数名 `writeback` 遮蔽 `lib/writeback` import，导致 `writeback=True` 时 `method receiver must be an object map`；改为 `host_writeback_record`。金样 `agent-writeback-shadow`。
- **`ext/web` C0**：`admin=False` 不再挂载/保留 `/admin*`；可配 `admin_prefix` / `login_redirect` / `logout_redirect`；金样 `web-c0-admin-prefix`。
- **`ext/web` C1**：门禁段边界匹配（不再误伤 `/admin-publish`）；`gate` 支持 `match` / `on_deny` / `exclude`；`auth` 默认 `on_deny=redirect` + `login_path`；金样 `web-c1-gates`。
- **`ext/web` C2**：`shell_css=full|minimal|off`；`layout=sidebar|stacked|bare|rail`；full 壳含小屏单列 media；金样 `web-c2-shell-layout`。
- **`ext/web` C3**：样式装配对数字/布尔值单元格告警，`strict`/`strict_css_cells` 硬错误；`page.css` 作 raw CSS；head 表支持 `defer`/`async`/`version`，`asset_version` 统一 `?v=`；金样 `web-c3-style-head`。
- **`ext/web` C4**：导航表 `当`/`when`（`auth`/`guest`/`hide`）与 `媒体`/`media`（响应式显隐）；listen 注入 `_logged_in`；金样 `web-c4-nav-when`。

### Changed
- `build_step_context`：call site 提前到 task 之后；开场白强调 code-as-documentation。
- 中文 `构建单步上下文` 委托英文版（含预算 / READ）；新增 `dump_step_context`；live 用例改用仓库根 `.env`。
- 文档：[ext-web 定制限制](doc/research/ext-web-customization-limits.md) + [解决设计 C0–C4](doc/design/ext-web-customization/)。

## v0.3.3 — 2026-09-04

### Highlights

**浏览器作者零业务 JS（路线 D/E/F）**：官方 bridge 自启 + 宿主效应补齐（DOM/路由/storage/ws/文件/Canvas/音频/Observer/拖放）；`lib/browser` + GFM 表格写法；新增 [marqdo-dev](.cursor/skills/marqdo-dev/SKILL.md) 开发约定。

```bash
git checkout v0.3.3
cargo build --release
marqdo wasm build
# examples: browser-hello / browser-app / browser-media / web-client-site
```

### Added
- **路线 D（作者零 JS）**：[roadmap](doc/roadmap/browser-wasm-d.md) · 设计 §16。官方 bridge `mount` / `data-mq-*` 自启；`client_embed` 带 `wasm`/`source`/`boot`；DOM 效应；`examples/web-client-site/`。
- **路线 E（前端补齐）**：[browser-wasm-e.md](doc/roadmap/browser-wasm-e.md)。多节点 DOM、`add_class`/`remove_class`、cookie、clipboard/download、`fetch_all`/`ws`/`interval`；示例 [browser-app](examples/browser-app/)。
- **路线 F（媒体/文件，F1–F5）**：[browser-wasm-f.md](doc/roadmap/browser-wasm-f.md)。`read_file`、Canvas 2D、Audio beep、Intersection/ResizeObserver、drag/drop；示例 [browser-media](examples/browser-media/)。
- **`lib/browser` / `lib/浏览器`**：WASM 客户端效应助手（GFM 表格 + named helpers；勿用 `json.set` 链）。
- **Skill `marqdo-dev`**：代码即文档 / GFM-first / 库开发约定。

### Changed
- 浏览器示例（hello / app / media / web-client）改为表格 + `lib/browser`；`ext/web` 效应助手改用 `table.put`。
- 标准库模块表收录 `browser`；路线 F 收口 F1–F5（F6 Service Worker 仍可选）。

## v0.3.2 — 2026-09-01

### Highlights

**浏览器 Marqdo（路线 C）首版完备**：`wasm-core`、`marqdo wasm build`、会话 ABI、DOM/`fetch`/`after` 效应、官方 bridge、体积与冒烟；并修 Admin 长内容布局、`ext add` 原生插件预检、`@keyframes` 与样式表 `/` 比率约定。

```bash
git checkout v0.3.2
cargo build --release
marqdo wasm build
marqdo ext add web   # 改插件后请重新 add
```

### Added
- **浏览器 Marqdo（路线 C）**： [ADR 0002](doc/adr/0002-browser-marqdo-wasm.md) · [design](doc/design/browser-marqdo-wasm.md) · [roadmap C0–C5](doc/roadmap/browser-wasm.md)。
- **C0/C1**：`wasm-core` feature 门控；`run_source`；crate `crates/marqdo-wasm`（`mq_run` ABI）；示例 [examples/browser-hello](examples/browser-hello/)。
- **C2/C3**：`marqdo wasm build`；会话 `mq_boot`/`mq_call`；GFM wire 表 + `set_text` DOM 回写；[interact.html](examples/browser-hello/interact.html) 计数器示例。
- **C4**：[ADR 0003](doc/adr/0003-browser-async-effects.md) 效应表（`fetch` / `after`）+ bridge 续体；[fetch.html](examples/browser-hello/fetch.html)。
- **C5**：`release-wasm` 体积配置；`marqdo wasm build` 报告 KiB 并可选 `wasm-opt`；`run_source` bytecode 单测；会话仍为 tree。
- **WASM 收口**：规范 `crates/marqdo-wasm/js/marqdo-bridge.js`；`wasm build` 同时拷贝 bridge；Node 冒烟 `tests/wasm`；`web.client_embed` / `网页.客户端挂载`；路线图标 Completed。
- **发版 Skill**：`.cursor/skills/marqdo-release/`（探测版本 → 用户确认 → 文档/资产/扩展/Release 说明）。

### Fixed
- **Admin UI**：主区取消 `max-width:56rem` 限制，表格外包可横向滚动的 `.table-wrap`，单元格完整展示并自动换行；表单加宽且 textarea 可随内容增高。
- **`marqdo ext add`**：安装含原生插件的扩展时**先**定位/自动 `cargo build` 再拷贝 `.mq.md`，避免只装上文稿、运行时报 `native plugin not found`；扩大 `.so` 搜索路径。
- **`web_style` / 样式装配**：正确输出 `@keyframes` 块（停点为嵌套规则，不再写成 `0%: opacity: 0`）。
- **样式表含 `/` 的值**：文档明确**引号优先**（`"16/9"`、`"1/5"`）；不引入靠空格消歧的表单元格比率折叠。

## v0.3.1 — 2026-08-29

### Highlights

**W8 站点资源**：favicon / Head 资源表 / 图片装配一次补齐；并修复相对入口路径下 `entry_dir` 双重拼接导致的嵌套目录问题。

```bash
git checkout v0.3.1
cargo build --release
marqdo ext add web   # 改插件后请重新 add
```

### Added
- **`ext/web` W8（站点图标 / Head / 图片装配）** ([web-assets-and-images.md](doc/design/web-assets-and-images.md))：`app.icons` / `应用.图标` 挂 `GET /favicon.ico`；`page.head` / `头装配`；`make_images` / `图片装配`；`meta` 认 `icon`/`favicon`/`apple-touch-icon`；`static` 约定 `favicon.*`；金样 `web-assets-smoke` / `web-assets-live`；[marqdo-blog](examples/marqdo-blog/) 接入。

### Fixed
- **`host_query("entry_dir")`**：相对入口脚本（如 `tests/ext/….mq.md`）时对进程 cwd 绝对化，避免与 `for_run` 已设的脚本目录 cwd 再次拼接成 `tests/ext/tests/ext/…`。

## v0.3.0 — 2026-08-29

### Highlights

本版把 **`ext/web` 网络能力（W0–W7 + P3）**、**`ext/quantum` Q7/Q8**、**`ext/agent` A1–A4** 一并收口到正式发布；并修了网页扩展叙述行误解析、补充 ASGI/生产部署说明与 agent 下一波（B0–B5）缺口调研。

**如何拿到本版**

```bash
# 从源码（推荐跟 main / 本 tag）
git clone https://github.com/cflmy/marqdo.git && cd marqdo
git checkout v0.3.0
cargo build --release
./target/release/marqdo version

# 或从 GitHub Releases 下载 Windows 包 / 源码 zip（见 Release 资产）
marqdo version --check
```

**扩展（非 stdlib）**

```bash
cargo build --release -p marqdo_plugin_web -p marqdo_plugin_agent -p marqdo_plugin_quantum
marqdo ext add web && marqdo ext add agent && marqdo ext add quantum
# 中文 id：网页 / 智能体 / 量子 / 大模型
```

### Added
- **调研：`ext/agent` A0–A4 之后缺口** ([agent-framework-gaps-after-a4.md](doc/research/agent-framework-gaps-after-a4.md))：产品化示例、评测 harness、真 MCP、plan resume/HITL、飞轮指标与建议分期 B0–B5；挂到 [ext-agent-optimize.md](doc/roadmap/ext-agent-optimize.md) 与 [doc/README.md](doc/README.md)。
- **生产部署调研（ASGI）** ([web-asgi-servers-and-marqdo.md](doc/design/web-asgi-servers-and-marqdo.md))：不能挂 Daphne/Uvicorn；路径为反向代理 → 嵌入式 `listen`（axum）。
- **`ext/agent` A4（RAG/MCP 证据工具）**: `corpus_search` 本地语料关键词检索；`mcp_list_tools` / `mcp_call` 读 JSON fixture；返回 `authority=workbook`。金样 `agent-tools-rag-a4`。
- **`ext/agent` A3（上下文预算）**: `source_brief` / 加深版 `skill_brief`；`build_step_context` 默认截断并提示 `READ:source|skill`；`step` 支持最多 `max_reads` 次加深；父 `READ:skill`。金样 `agent-context-budget-a3`。
- **`ext/agent` A2（过程可见）**: `plan` 过程事件默认写入返回 map 的 `events`（SSE 仍仅 `stream=True`）；OKF 命中记 `REUSE` decision；view plan 卡渲染过程时间线。对齐 [agent-streaming.md](doc/roadmap/agent-streaming.md)。
- **`ext/agent` A1（OKF 复用飞轮）**: `agent_kb_list_tasks` 返回 description/aliases/status/llm_free/hits；`plan` 命中路径暴露 `match`/`score`（summary 含 match kind）；soft_match 策展提示带 status/llm_free/description；view plan 卡显示 match。金样 `agent-kb-plan-hit`。路线 [ext-agent-optimize.md](doc/roadmap/ext-agent-optimize.md)。
- **`ext/web` 样式装配（`web_style` / `网页.样式装配`）**: 样式即数据表格 —— GFM 样式表（`|选择器|属性|值|`，可用 `|媒体|` 列分组进 `@media` 块）经 `样式装配` 函数转成 CSS 文本，再拼装成完整主题。替代"整段手写 CSS 字符串"的写法，贯彻 文档即代码。见 [marqdo-blog](examples/marqdo-blog/styles/theme.mq.md)。
- **网络能力调研（`doc/design/web-net-capabilities.md`）**: 盘点 `ext/web` + `plugins/web` + `lib/net` 现状，对照主流语言网络栈（FastAPI / Express / Flask / axum），给出「开发一个完整 Web 项目」所需能力的差距清单与分波次补强路线（W1–W7 + P3，**2026-08-28 已全部落地**）。
- **AI Skill + 文档（ext/web 完结复核）**: `skills/marqdo/` 与 `.cursor/skills/marqdo/` 增加 **ext/web 动态站** 专节（硬规则、API 摘要、§13 最小站点样例）；`doc/design/web-net-capabilities.md` 结论/对照表与 W7+P3 实现对齐；`doc/design/ai-skill.md` 更新用途说明。
- **`ext/quantum` Q7（高阶线性代数 + 高级可视化）**: 密度矩阵 / 部分迹 / Hermitian 谱 / Schmidt / Pauli 期望 / 纯度；SVG：hinton、city、density、paulivec、qsphere、multibloch。全部经 `plugins/quantum` ABI。设计 [ext-quantum-q7.md](doc/design/ext-quantum-q7.md)；金样 `quantum-linalg-smoke`、`quantum-viz-advanced-smoke`；示例 [quantum-entanglement](examples/quantum-entanglement/)。
- **`ext/quantum` Q8a/Q8b（可视化美学）**: `draw theme=dark|light|bw`（默认 dark）；标签芯片 + gutter 消除线穿字；门族分色；probs/bloch 共用令牌；view 对 `data-theme` 换图框。设计 [ext-quantum-viz-style.md](doc/design/ext-quantum-viz-style.md)。改插件后须 `marqdo ext add quantum` 再开 view。

### Fixed
- **`ext/web` / `网页.mq.md`**：叙述行勿以反引号开头，避免被解析为语句（`examples/marqdo-blog` 可正常 `listen`）。

### Changed
- **Frontmatter import syntax**: `import bind:target` / `导入 bind:target` (file `.mq.md` or short name `lib.member`). Removed legacy `> path.mq.md` / `> use` imports. See [module-namespace.md](doc/design/module-namespace.md).

## v0.2.0 — 2026-08-12

### Highlights — official extension libraries

This release centers on **`ext/`**: optional packages that are **not** part of the embedded stdlib. Install them with the CLI; native plugins (`.so` / `.dll`) ship beside L1 Markdown APIs.

**How to get extensions**

```bash
# List what the installer knows about
marqdo ext list

# Install from the repo's ./ext (or MARQDO_EXT_SOURCE), into ~/.marqdo/ext (or MARQDO_EXT)
marqdo ext add llm
marqdo ext add agent
marqdo ext add web
marqdo ext add quantum

# Chinese ids work the same (大模型 / 智能体 / 网页 / 量子)
marqdo ext add 量子

marqdo ext remove quantum
```

Native plugins (agent / web / quantum) must be built once, then `ext add` copies them into `MARQDO_EXT/native/`:

```bash
cargo build --release -p marqdo_plugin_agent
cargo build --release -p marqdo_plugin_web
cargo build --release -p marqdo_plugin_quantum
marqdo ext add agent
marqdo ext add web
marqdo ext add quantum
```

Runtime also resolves plugins from `CARGO_TARGET_DIR`, `target/`, next to the `marqdo` binary, or `MARQDO_*_PLUGIN` env vars. Design: [ext-cli.md](doc/design/ext-cli.md) · [ext-abi.md](doc/design/ext-abi.md).

| Package | What you get |
|---------|----------------|
| **llm** | OpenAI-compatible chat — [ext-llm.md](doc/design/ext-llm.md) |
| **agent** | Agent layout / orchestration helpers + native plugin — [ext-agent.md](doc/design/ext-agent.md) |
| **web** | HTTP + SQLite site helpers + native plugin — [ext-web.md](doc/design/ext-web.md) |
| **quantum** | State-vector circuits, draw, noise, formula `matrix=` custom gates — [ext-quantum.md](doc/design/ext-quantum.md) |

User-facing intro: [`public/features/05-extensions.mq.md`](public/features/05-extensions.mq.md) / [`05-扩展.mq.md`](public/features/05-扩展.mq.md).

### Added
- Full **ext CLI** catalog: `llm`, `agent`, `web`, `quantum` (EN/ZH ids).
- **Quantum (Q0–Q6)**: gates, `run`/`steps`, draw (circuit/probs/bloch), heatmaps, teaching noise incl. amplitude damping, **custom gates from `$$` / `matrix=`**.
- **Web** extension (listen / render / SQLite) with L1 EN/ZH.
- **Agent** native plugin path + installer copy into `native/`.
- View **Variables** panel: previews + click-to-open rich modal (matrices / KaTeX).
- Formula matrix parse (`pmatrix` / `[[…]]`) for executable gate matrices.

### Changed
- Release notes and docs emphasize extensions vs embedded stdlib.
- Highlight.js CDN path for view pages; Variables panel script escaping fixed.

## v0.1.2 — 2026-08-07

### Added
- **Embedded standard library**: official `lib/*.mq.md` ships inside the `marqdo` binary; disk `lib/` and `MARQDO_LIB` still override.
- **`lib/writeback`** / **`lib/subtask`**: Jupyter-style writeback; concurrent subtasks (file / function / foreign).
- **Surface syntax v0.2**: `` + `param` `` parameters, `1.` ordered branches, backtick identifiers, quoted strings.
- **`marqdo version --check`**: compare installed CLI with latest GitHub release.
- **VS Code extension v0.0.6** (branch **`vscode-extension` only**): v0.2 grammar, update check — see [doc/design/vscode-extension-commit.md](doc/design/vscode-extension-commit.md)

### Changed
- Subtask `spawn` accepts `path=`, `fn=`, `code=`, or `lang=`+`source=` (not file-only).
- Release notes: standalone `.exe` includes stdlib; bundle/stdlib zips remain optional for overrides.

## v0.1.1

- v0.2 syntax migration, writeback/subtask v1 (file subprocess only), view input deferral, optional parameters.

## v0.1.0

- Initial public releases: tree + bytecode backends, `view` / `debug` / `catalog`, core stdlib, `ext/` installer.
