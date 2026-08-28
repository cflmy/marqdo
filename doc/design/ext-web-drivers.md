# `ext/web` 驱动：Postgres · Redis · S3

| | |
|---|---|
| 状态 | **Accepted · 已落地** |
| 日期 | 2026-08-28 |
| 相关 | [ext-web.md](ext-web.md) · [roadmap/ext-web.md](../roadmap/ext-web.md) · [ext-abi.md](ext-abi.md) |
| 目标 | 在**不改作者面 CRUD 心智**的前提下，用 URL 方案切换数据库；另增缓存与对象存储两类驱动。贯彻「表格 + 类方法」：配置即数据，装配即函数。 |

---

## 0. 一句话

**`# db` 的 `url=` 决定后端**：`sqlite:` 默认；`postgres://` / `postgresql://` 走 Postgres，作者面 API 不变。  
**缓存**与**对象存储**是独立类（`# cache` / `# storage`），不塞进 `# db`。

---

## 1. 边界

| 能力 | 归属 | 理由 |
|------|------|------|
| SQLite / Postgres CRUD | `plugins/web` + `ext/web` `# db` | 同一套 `init/select/insert/…`；热路径 ABI |
| Redis 缓存 | `plugins/web` + `ext/web` `# cache` | 键值语义，不是表 CRUD |
| S3 / MinIO 对象 | `plugins/web` + `ext/web` `# storage` | 对象键 + 字节/路径，不是行 |
| 禁止 | 核心 `src/host/` | 驱动依赖重，只进 ABI 插件 |

---

## 2. URL 方案（锁定）

| 方案 | 含义 | 例 |
|------|------|-----|
| `sqlite:` / 裸路径 | SQLite 文件（既有） | `sqlite:data/site.db` |
| `postgres://` / `postgresql://` | Postgres | `postgres://user:pass@127.0.0.1:5432/app` |
| `redis://` / `rediss://` | Redis | `redis://127.0.0.1:6379/0` |
| `memory:` | **进程内** Redis 兼容后端（金样/离线） | `memory:` |
| `s3://bucket` + 查询参数 | S3 兼容（含 MinIO） | `s3://mybucket?endpoint=http://127.0.0.1:9000&region=us-east-1` |
| `file:` | **本地目录**对象存储（金样/离线） | `file:data/blobs` |

凭证：优先 URL 内嵌；S3 亦读环境变量 `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY`（或 `S3_ACCESS_KEY` / `S3_SECRET_KEY`）。

---

## 3. 作者面（代码即文档）

### 3.1 数据库 — 只换 URL

英文：

```markdown
import web:ext/web/web.mq.md

# main

*store = > web.db url="postgres://marqdo:marqdo@127.0.0.1:5432/site"*
*store = > store.init name="posts" fields=`schema`*
```

中文：`网页.数据库` + `url=` 同形。`init` / `select` / `insert` / `paginate` / `事务` **API 不变**。

Postgres 差异（实现层消化，作者尽量无感）：

- `id INTEGER PRIMARY KEY` → `SERIAL PRIMARY KEY`（或 `GENERATED …`）
- 占位符 `?` → `$1,$2,…`
- 标识符双引号规则对齐

### 3.2 缓存 `# cache` / `# 缓存`

```markdown
*c = > web.cache url="memory:"*
*c = > c.set key="k" value="v" ttl=60*
*v = > c.get key="k"*
*c = > c.del key="k"*
```

方法：`get` · `set`（可选 `ttl` 秒）· `del` · `exists` · `ttl`。

配置表可选（装配时合并进 url / 默认 TTL）：

| 键 | 值 |
|----|-----|
| url | redis://127.0.0.1:6379/0 |
| ttl | 3600 |

### 3.3 对象存储 `# storage` / `# 存储`

```markdown
*blob = > web.storage url="file:data/blobs"*
> `blob`.put key="avatar/1.png" path="local.png" content_type="image/png"
*got = > `blob`.get key="avatar/1.png"*
> `blob`.delete key="avatar/1.png"
*keys = > `blob`.list prefix="avatar/"*
```

方法：`put`（`path=` 本地文件或 `body=` 文本）· `get`（返回 `{ok, body|path, content_type}`）· `delete` · `list`。

---

## 4. ABI 名

| ABI | 说明 |
|-----|------|
| 既有 `web_db_*` | 内部按 `url` 方案分发 SQLite / Postgres |
| `web_cache_new/get/set/del/exists/ttl` | 缓存 |
| `web_storage_new/put/get/delete/list` | 对象存储 |

---

## 5. 测试

| 金样 | 内容 |
|------|------|
| `tests/ext/web-drivers-smoke.mq.md` | `memory:` 缓存往返；`file:` 存储 put/get/list/delete；`web_db_new` 接受 `postgres://` 句柄形状（不要求本机有库） |
| 可选 live | `MARQDO_TEST_POSTGRES` / `MARQDO_TEST_REDIS` / `MARQDO_TEST_S3` 非空时跑真实连通（CI 默认关） |

---

## 6. 非本波次

- 连接池大小 / 读写分离配置表  
- Redis 集群 / Sentinel  
- S3 预签名 URL 作者面糖（可后续加方法）  
- 把 session 默认改存 Redis（仍默认 SQLite 表；可选后续）  
