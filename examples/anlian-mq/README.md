# 暗恋见君 · Marqdo 重写

用 **Marqdo + `ext/web`（Go `libweb`）** 对齐 Django [anlian](../../../anlian/) 的主浏览路径与视觉：原站 CSS/JS、首页四段、列表/详情、发帖门禁、聊天室、专题、搜索入口、API 指南。

默认 **SQLite**（`data/anlian-mq.db`）；生产可换 `postgres://` + Redis。

## 设计约定

| 做 | 不做 |
|----|------|
| GFM 表装配库表/种子/表单/门禁/头资源 | `json.set` 链拼页面 |
| 原站 `public/css|js|img` + DOM class 对齐 | 另起一套模板糊弄验收 |
| `页面.壳HTML` 挂 Django 级导航/页脚 | 业务写进 Go 插件 |
| 缺能力时改 `ext/web` / `libweb` | 在应用层堆不可读胶水 |

## 路径（对齐原站）

| 路径 | 能力 |
|------|------|
| `/` | Hero + 公告 + 新闻时间线 + 论坛板块栏/卡片 |
| `/post`、`/post/{slug}` | 帖子列表 / 详情 |
| `/news`、`/news/{slug}` | 新闻列表 / 详情 |
| `/post/public` | 发帖（需登录） |
| `/post/ten`、`/news/five` | 首页 AJAX 片段 |
| `/chat`、`/chat/lobby`、`/chat/casual` | 房间列表 + 公聊 WS |
| `/topics` | 专题书架 |
| `/search` | 搜索入口 |
| `/guide/automation-api` | API 指南 |
| `/accounts/login` | 粉玻璃登录 |
| `/sitemap.xml`、`/robots.txt` | 站长 |

## 运行

```bash
export PATH="$HOME/.local/go/bin:$PATH"
./scripts/build-web-plugin.sh
export MARQDO_WEB_PLUGIN=$PWD/plugins/web/build/libweb.so
rm -f examples/anlian-mq/data/anlian-mq.db*   # 首次或改 schema 后
./target/debug/marqdo run examples/anlian-mq/index.mq.md
# → http://127.0.0.1:18180/

bash examples/anlian-mq/smoke.sh
```

默认账号：`admin` / `anlian`。
