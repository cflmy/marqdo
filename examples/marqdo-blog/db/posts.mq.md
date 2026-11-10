---
markup: occlusion
title: 当前 Artifact 博客数据调用手册
import:
  document: lib/document.mq.md
  data: ext/data/db.mq.md
  fs: lib/fs.mq.md
  text: lib/text.mq.md
---

本文件负责可调用的说明与编排，[posts.tables.md](posts.tables.md)负责可独立阅读的 Markdown 常量。调用时把 --fs-root 指向 examples/marqdo-blog，读取需要 fs.read；连接数据库或编译样式时还需要 plugin。

## schema

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

当前站点的文章字段包括标题、slug、摘要、正文、标签和创建日期。原生数据库初始化器负责 id；本结构保留当前自动列表与详情页所需字段。

从站点根目录授予 fs.read 后调用。实际常量位于 [posts.tables.md](posts.tables.md#schema)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取schema的实际 Markdown 常量](document.read_table(path="db/posts.tables.md", title="schema"))*.

## seed

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

两篇当前站点的起始文章保留稳定 slug、内容与日期。只有调用 open 且文章表为空时才插入；读取种子本身不会改变现有文章。

从站点根目录授予 fs.read 后调用。实际常量位于 [posts.tables.md](posts.tables.md#seed)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取seed的实际 Markdown 常量](document.read_table(path="db/posts.tables.md", title="seed"))*.

## open

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | any | None | None 使用授权站点根内 data/blog.db；验收可传独占绝对 URI |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 已初始化且保留原有文章的实际数据库资源 |

使用`url`=None，以 examples/marqdo-blog 为 --fs-root 授予 fs.write 与 plugin。默认路径先通过固有能力创建 data 目录，再解析真实绝对文件地址，避免被插件模块位置或终端工作目录影响。显式 SQLite 路径同样经过根目录和符号链接检查；其他受支持后端按其连接契约执行，文件授权不限制外部数据库服务。

字段与种子由原生 Markdown 读取器保留字面类型。初始化只确保文章表存在，再查一行判断是否已有业务数据；只为空表插入原来的两条记录。已有或部分填充的表都原样保留，这不是列迁移器，也不会删除或重排用户文章。连接、结构或插入失败应停止启动，不能继续监听并声称数据库已经准备好。

隔离验收使用调用者拥有的临时站点副本，保留同一资源继续查询与写入。进程退出、连接关闭后再清理该目录，不对示例的现有 data 目录执行重置。读取结构或种子本身不写库，写入只发生在这份明确初始化流程里。

1. `url` == None
  我们 **[创建本站默认数据目录](fs.make_dirs(path="data"))**、**file = [按原生授权根解析数据库文件](fs.resolve(path="data/blog.db"))** 和 **url = "sqlite:" + file**。
1. [明确选择 SQLite 后端](text.starts_with(text=`url`, prefix="sqlite:"))
  我们 **file = [取出实际 SQLite 地址](text.replace(text=`url`, old="sqlite:", new="", count=1))**、**file = [检查根目录和符号链接边界](fs.resolve(path=`file`))** 和 **url = "sqlite:" + file**。
我们 **store = [连接实际选择的数据库](data.db(url=`url`))** 和 **fields = [读取当前文章结构](schema())**。
接着 **[确保文章结构但不清空数据](> `store`.init name="posts" fields=`fields`)** 和 **existing = [检查是否已有业务文章](> `store`.select table="posts" limit=1)**。
1. not `existing`
  我们 **rows = [读取空表使用的原始记录](seed())** 和 **[只初始化空表](> `store`.insert table="posts" rows=`rows`)**。
准备完成后返回 *store*，供当前服务流程继续使用。
