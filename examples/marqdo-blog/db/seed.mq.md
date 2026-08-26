## posts

种子文章。首页与详情页的初始数据。created_at 用于按时间倒序展示；tag 是主标签 slug。

`posts` =

| 行 | title | slug | summary | content | tag | created_at | updated_at |
|----|-------|------|---------|---------|-----|------------|------------|
| 1 | 你好，Marqdo 博客 | hello-marqdo | 用 Marqdo 的 web 扩展库搭建的第一个博客。表格即组件、文档即代码。 | 这是一篇介绍博客系统的文章。它由 Marqdo 脚本在启动时写入 SQLite，通过 ext/web 的组件装配渲染成页面。 | marqdo | 2026-08-01 | 2026-08-02 |
| 2 | 动态路由与文章详情 | dynamic-routing | 如何用 /post/{slug} 动态路由渲染单篇文章。 | 本文说明动态路由机制：作者面注册带 {slug} 占位符的路由，请求时把路径参数注入页面查询条件。 | web | 2026-08-05 | 2026-08-05 |
| 3 | 用 GFM 表格表达数据 | gfm-tables | Marqdo 的字典操作不依赖 json 库，GFM 表格即可完成。 | 表格即数据、表格即组件。这篇文章展示如何在 .mq.md 里用横向表写种子数据，用脚注索引读取字段。 | tutorial | 2026-08-10 | 2026-08-11 |
| 4 | WebSocket 实时小部件 | websocket-live | 给博客加一个实时回显的 WebSocket 端点。 | 通过 app.路由实时 注册 /live 端点，配合 public/live.js 在首页显示实时连接状态。 | realtime | 2026-08-15 | 2026-08-15 |
| 5 | 会话与登录鉴权 | session-auth | 后台用 session 门禁保护，只有登录用户能发布文章。 | 用 app.鉴权 配置管理员用户表，/admin 下的 CRUD 页面都需要有效会话。 | web | 2026-08-18 | 2026-08-20 |
| 6 | 数据库与 CRUD | database-crud | 博客的文章、标签、关联全部由 SQLite 支撑。 | db.初始化 建表、db.插入 写种子、后台表单增删改。文章与标签通过 post_tags 关联。 | database | 2026-08-20 | 2026-08-21 |

**`posts`**

## tags

种子标签。

`tags` =

| 行 | name | slug |
|----|------|------|
| 1 | Marqdo | marqdo |
| 2 | Web | web |
| 3 | 教程 | tutorial |
| 4 | 数据库 | database |
| 5 | 实时 | realtime |

**`tags`**

## post_tags

种子文章-标签关联。

`post_tags` =

| 行 | post_id | tag_id |
|----|---------|--------|
| 1 | 1 | 1 |
| 2 | 1 | 2 |
| 3 | 2 | 2 |
| 4 | 3 | 1 |
| 5 | 3 | 3 |
| 6 | 4 | 5 |
| 7 | 4 | 2 |
| 8 | 5 | 2 |
| 9 | 6 | 4 |
| 10 | 6 | 1 |

**`post_tags`**