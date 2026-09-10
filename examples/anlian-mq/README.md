# 暗恋见君 · Marqdo 重写（验收切片）

用 **Marqdo + `ext/web`（Go `libweb`）** 重写暗恋见君的主路径，验证「代码即文档」能否撑起真实社区站点。

外部依赖（完整生产）：PostgreSQL + Redis。本示例默认 **SQLite**，便于本机一键跑通；URL 换 `postgres://…` 即可外接。

## 设计约定

| 做 | 不做 |
|----|------|
| GFM 表装配导航、样式、库表、种子、表单、门禁 | `json.set` 链拼页面 |
| `网页.*` / `table.put` 命名方法 | 手写业务 JS 袋 |
| 分文件：`db/`、`components/`、`styles/` | 单文件堆全部 HTML |
| 对照原站迁粉玻璃、Hero 动效、卡片悬停 | 另起一套「暖纸模板」糊弄验收 |

对照原站 [anlian](../../../anlian/)（Django）与施工图 [ext-web-go-rewrite.md](../../doc/design/ext-web-go-rewrite.md) §8。

视觉资源在 `public/`（`bg-body-960.webp`、`bg-hero-640.webp`、`logo.svg`），样式表在 `styles/shell.mq.md`（变量 / `@keyframes` / 顶栏 / Hero / 卡片 / 响应式均为 GFM 表）。

## 已覆盖主路径

| 路径 | 能力 |
|------|------|
| `/` | 首页 + 最新帖子卡片 |
| `/posts`、`/post/{slug}` | 帖子列表 / 详情 |
| `/news`、`/news/{slug}` | 新闻列表 / 详情 |
| `/write` | 发帖表单（需登录） |
| `/search` | FTS 关键词（表单 → 查询页） |
| `/chat` | 公聊页 + `/chat/ws` broadcast |
| `/admin` | 鉴权门禁后台 |
| `/sitemap.xml`、`/robots.txt` | 站长 |

## 运行

```bash
# 仓库根目录
export PATH="$HOME/.local/go/bin:$PATH"
./scripts/build-web-plugin.sh
./target/debug/marqdo run examples/anlian-mq/index.mq.md
# → http://127.0.0.1:18180/

# 另开终端
bash examples/anlian-mq/smoke.sh
```

离线装配（不 listen）：

```bash
./target/debug/marqdo run examples/anlian-mq/offline-smoke.mq.md
```

默认账号：`admin` / `anlian`（种子用户表）。

## 发现缺陷时

若某段 `.mq.md` 必须堆 `json` 才能完成，优先改 **`ext/web` / 插件 ABI`**，而不是在应用层打补丁。在本目录 `NOTES.md`（如有）或 PR 描述里记下可读性缺口。
