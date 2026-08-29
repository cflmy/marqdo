# examples/marqdo-blog

一个用 Marqdo 的 `ext/web` 网络扩展库搭建的博客系统示例：美观大气的现代化主题、
SQLite 驱动的文章/标签、动态路由、WebSocket 实时终端、以及受会话门禁保护的后台 CRUD。

## 运行

```bash
cargo build -p marqdo_plugin_web
cargo run -- examples/marqdo-blog/index.mq.md
```

打开 http://127.0.0.1:18085/：

- `/` — 首页：文章卡片网格 + 实时 WebSocket 终端
- `/post/{slug}` — 动态路由渲染单篇文章（标题/日期/标签/正文）
- `/tags` — 标签归档；`/tag/{slug}` 查看某标签下的文章
- `/about` — 关于页
- `/admin` — 会话门禁后台，登录 `admin` / `marqdo`，可增删改文章与标签
- `/live` — WebSocket 回显端点
- `/static/*` — 静态资源（`public/` 下的 `live.js`、favicon、logo）
- `/favicon.ico` — 站点图标（`应用.图标` 表装配）

## 设计亮点（文档即代码）

- **样式即数据表格，装配即函数**（`styles/theme.mq.md`）：
  主题不再是一大段手写 CSS 字符串，而是由一张张 GFM 样式表组成 ——
  `|选择器|属性|值|` 规则行（同一选择器多行自动合并成一条规则），
  响应式用 `|媒体|选择器|属性|值|` 分组进 `@media` 块。
  每个 `## 段`（基础/顶栏/侧栏/主体/卡片/页脚/表单/响应式）导出一张样式表，
  `## 全局` 用 `网页.样式装配` 函数把每张表装配成 CSS 文本，再 `text.str_join` 拼成完整样式表，
  最后经 `页面.样式` 注入页面。样式保持可读的数据表格，不写胶水代码。
- **图标 / 头资源 / 图片装配**（W8）：`应用.图标` 表挂 `/favicon.ico` 与 SVG；
  `页面.头装配` 写 apple-touch-icon；`页面.图片装配` 把 logo 表注入主区（`mq-images`）。
  含 `/` 的路径与 MIME 单元格需加引号（如 `"/static/logo.svg"`、`"image/png"`）。
- **页面 = 组件装配 + 主体绑定**（`index.mq.md`）：
  导航/侧栏/页脚用 `|组件|样式|` 表装配，文章列表用 `|属性|值|` 绑定 SQLite 字段。
- **动态路由**：`/post/{slug}` 的 `slug` 注入查询条件，`页面.详情` 把首行渲染成文章。
- **数据模型**（`db/`）：`schema.mq.md` 定义 `posts`/`tags`/`post_tags` 三张表，
  `seed.mq.md` 提供种子数据，`index.mq.md` 幂等建表 + 灌数据。

## 冒烟测试

服务器启动后（端口 18085），运行：

```bash
bash examples/marqdo-blog/smoke.sh
```

覆盖：首页/卡片/主题 CSS/媒体查询、静态资源、动态路由详情、标签归档、鉴权门禁、
登录会话、后台表单、WebSocket 回显。全部通过时输出 `通过 N / N`。

可从仓库根目录或本目录运行：相对 `static_dir`（`public`）与相对数据库路径
（`data/marqdo-blog.db`）都按**入口脚本所在目录**解析，不依赖终端工作目录。
