# `ext/web` — 站点图标、Head 资源与图片装配

| | |
|---|---|
| 状态 | **Accepted · W8 已落地** |
| 日期 | 2026-08-29 |
| 父设计 | [ext-web.md](ext-web.md) · [web-net-capabilities.md](web-net-capabilities.md) |
| 相关 | [web-asgi-servers-and-marqdo.md](web-asgi-servers-and-marqdo.md)（静态仍由嵌入式 `listen` 提供） |
| 硬约束 | `ext/**` 不调 `host_*`；作者面禁止袋胶水；中英库文件分文件 |

---

## 0. 一句话

把 **浏览器默认图标（`/favicon.ico`）**、**`<head>` 里的 link/script**、**正文/品牌图片** 都做成与 `样式装配` 同构的能力：**GFM 表即数据，装配即函数**；由 `plugins/web` ABI 渲染/挂路由，`ext/web` 只做 L1 包装。

---

## 1. 动机与现状缺口

### 1.1 已有

| 能力 | 说明 |
|------|------|
| `app.static` / `应用.静态` | 目录挂到 `/static`（或自定义 mount） |
| `page.meta` / `页面.元数据` | `title` / `description` / `canonical` / `og:*` → `<head>` |
| `app.upload` + `gallery` | **运行时**用户上传媒体，不是站点品牌资源 |

### 1.2 缺口（本设计一次补齐）

| # | 缺口 | 影响 |
|---|------|------|
| G1 | 浏览器默认请求 **`GET /favicon.ico`**，静态挂在 `/static` 时根路径 404 | 标签页无图标 |
| G2 | `page.meta` 未知键一律变成 `<meta name="…">`，**不能**正确写出 `<link rel="icon">` | 作者写 `icon` 无效 |
| G3 | 无 **Head 资源表**（stylesheet / preload / script / manifest / apple-touch-icon） | 只能手写进 `intro` 或改插件 |
| G4 | 无 **图片装配**（表 → `<figure>`/`<img>`），与样式装配不对齐 | logo / 头图 / 插图只能拼 HTML 字符串 |
| G5 | 无 **站点级图标表 → 根路径文件服务 + 默认 head 注入** | 多页站点要在每页重复声明 |

---

## 2. 设计原则

1. **配置即数据、装配即函数** — 与 `web_style` / `应用.装配` 同构。  
2. **应用层管「可达性」**（磁盘文件 → HTTP 路径）；**页面层管「声明」**（`<head>` / 正文 HTML）。  
3. **站点图标可自动注入所有页面**，页面仍可用 `头装配` / `元数据` 覆盖或追加。  
4. **约定优于配置**：若已 `静态` 且目录内存在 `favicon.ico`（或 `favicon.png` / `favicon.svg`），未显式配置时仍挂 `GET /favicon.ico`（及对应静态 URL 由作者自选）。  
5. **中英分文件**：`web.mq.md` ↔ `网页.mq.md`，禁止混排。

---

## 3. 作者面 API（一次交付全集）

### 3.1 模块级装配（返回字符串，可与主题拼装）

| EN | ZH | FFI | 输入 | 输出 |
|----|----|-----|------|------|
| `make_style`（已有） | `样式装配` | `web_style` | 样式表 | CSS 文本 |
| **`make_images`** | **`图片装配`** | **`web_images`** | 图片表 | HTML 片段 |
| **`make_head`** | **`头资源装配`** | **`web_head`** | Head 表 | 供调试/预览的 HTML 片段（可选；页面方法主路径不强制调用） |

### 3.2 `# page` / `# 页面`

| EN | ZH | FFI | 作用 |
|----|----|-----|------|
| `meta`（增强） | `元数据` | `web_page_meta` | 认 `icon` / `favicon` / `apple-touch-icon` → `<link>`，不再误写成 `<meta name>` |
| **`head`** | **`头装配`** | **`web_page_head`** | 挂 Head 资源表到 `page.head`，渲染时写入 `<head>` |
| **`images`** | **`图片装配`**（方法） | **`web_page_images`** | 图片表 → HTML，写入 `page.images_html`，渲染进主区 |

### 3.3 `# app` / `# 应用`

| EN | ZH | FFI | 作用 |
|----|----|-----|------|
| **`icons`** | **`图标`** | **`web_app_icons`** | 站点图标表：根路径文件服务 + 默认 head 链接注入全站 |
| `static`（增强） | `静态`（增强） | `web_app_static` | 约定：目录内 `favicon.ico|png|svg` → 自动注册 `/favicon.ico`（可被 `icons` 显式行覆盖） |

---

## 4. 表形约定

单元格仍为**字面量**；列名中英等价（任选一套，勿混列名语义）。

### 4.1 站点图标表（`app.icons` / `应用.图标`）

| 列（ZH） | 列（EN） | 必填 | 说明 |
|----------|----------|------|------|
| `路径` | `path` | ✅* | 磁盘路径（相对进程 cwd 或入口目录解析规则与 `static` 一致） |
| `关系` | `rel` | | 默认 `icon`；常用 `icon` / `apple-touch-icon` / `mask-icon` |
| `类型` | `type` | | 缺省按扩展名推断 |
| `尺寸` | `sizes` | | 如 `16x16`、`180x180`、`any` |
| `地址` | `url` / `href` | | 对外 URL。缺省：首个 `rel=icon` 且扩展名为 `.ico` → `/favicon.ico`；否则 `/icons/{文件名}` |

\* 若仅提供 `地址` 指向已由 `静态` 提供的 URL、不落盘服务，则 `路径` 可空（只注入 head，不挂新路由）。

**Listen 行为**

1. 规范化为数组 `{path, rel, type, sizes, url}`。  
2. 对每个非空 `path`：注册 **`GET {url}`**，读文件返回，`Content-Type` = `type` 或推断；`Cache-Control` 可用 `public, max-age=86400`。  
3. 若存在至少一个 `rel` 含 `icon` 的行，保证浏览器默认路径：若没有任何 `url=/favicon.ico`，则把**第一行 icon** 额外挂到 `/favicon.ico`（同一文件，可选）。  
4. 将图标行转为默认 head 链接列表，存入 `app.site_head`，listen 时写入 `AppState`，渲染任意页面前 **merge**（页面自有 `head` / meta 图标 **追加且可同 href 去重**）。

**`static` 约定自动图标**

在 `listen` 解析完 `static_dir` 后：若 `app.icons` 为空，且 `{static_dir}/favicon.ico` 或 `.png` / `.svg` 存在，则等价注册：

```text
path = {static_dir}/favicon.{ico|png|svg}
rel  = icon
url  = /favicon.ico
```

（文件仍可通过 `/static/favicon.*` 访问；根路径额外可达。）

### 4.2 Head 资源表（`page.head` / `页面.头装配`）

| 列（ZH） | 列（EN） | 说明 |
|----------|----------|------|
| `关系` | `rel` | `icon` / `apple-touch-icon` / `stylesheet` / `preload` / `modulepreload` / `manifest` / `canonical` / **`script`** / **`module`** |
| `地址` | `href` / `src` / `url` | 资源 URL |
| `类型` | `type` | MIME 或 `module` |
| `尺寸` | `sizes` | icons |
| `媒体` | `media` | CSS media |
| `作为` | `as` | preload `as` |
| `跨域` | `crossorigin` | 空 / `anonymous` / `use-credentials` |

**渲染规则**

| `rel` | 输出 |
|-------|------|
| `script` | `<script src="…"></script>`（`type` 非空则写出） |
| `module` | `<script type="module" src="…"></script>` |
| 其他 | `<link rel="…" href="…" …>`（按列附加 `type`/`sizes`/`media`/`as`/`crossorigin`） |

空 `地址` 的行跳过。

### 4.3 元数据表增强（`page.meta`）

| 键 | 行为 |
|----|------|
| `title` / `description` / `canonical` / `og:*` | **不变** |
| **`icon`** / **`favicon`** | `<link rel="icon" href="{值}"/>`（可选若值含空格后 MIME：本切片仅 href） |
| **`apple-touch-icon`** | `<link rel="apple-touch-icon" href="{值}"/>` |
| 其他 | 仍为 `<meta name="…" content="…"/>` |

### 4.4 图片表（`make_images` / `page.images`）

| 列（ZH） | 列（EN） | 说明 |
|----------|----------|------|
| `源` | `src` | 必填，图片 URL |
| `替代` | `alt` | `alt` 文本，默认空 |
| `标题` | `title` | `title` 属性 |
| `类` | `class` | 加在 `<figure>`（无 figure 时加在 `<img>`） |
| `链接` | `href` / `link` | 非空则用 `<a>` 包裹 `<img>` |
| `宽度` | `width` | 像素，可选 |
| `高度` | `height` | 像素，可选 |
| `加载` | `loading` | `lazy` / `eager`；默认 `lazy`（首行可写 `eager`） |
| `图注` | `caption` | 非空则 `<figcaption>` |
| `槽` | `slot` | 预留：`main`（默认）。本切片只支持主区 |

**输出 HTML（多行）**

```html
<div class="mq-images" data-slot="main">
  <figure class="mq-img …">…</figure>
  …
</div>
```

单行且无图注/类时仍包在 `div.mq-images` 内，保证主题可统一选中。

**`page.images`**：调用 `web_images` 逻辑，结果写入 `page.images_html`。

**渲染位置**：`render_page` 主区中，在 `intro` **之前**插入 `images_html`（品牌图/头图常见需求）；`render_fragment` 的 main 槽同样插入。

路径与 MIME 单元格若含 `/`，须写成引号字面量（T5 单元格表达式会把裸 `/` 当除法）：`"/favicon.ico"`、`"image/png"`。

---

## 5. 插件 / 状态形状

### 5.1 页面对象新增字段

```json
{
  "meta": { "title": "…", "icon": "/favicon.ico" },
  "head": [
    { "rel": "stylesheet", "href": "/static/print.css", "media": "print" }
  ],
  "images_html": "<div class=\"mq-images\">…</div>"
}
```

### 5.2 应用对象新增字段

```json
{
  "static_dir": "public",
  "static_mount": "/static",
  "icons": [
    {
      "path": "public/favicon.ico",
      "rel": "icon",
      "type": "image/x-icon",
      "sizes": "32x32",
      "url": "/favicon.ico"
    }
  ],
  "site_head": [
    { "rel": "icon", "href": "/favicon.ico", "type": "image/x-icon", "sizes": "32x32" }
  ]
}
```

`site_head` 由 `web_app_icons`（及 static 约定）生成，作者不必手写。

### 5.3 `AppState` / `listen` 签名

新增：

- `icon_routes: Vec<IconRoute { url, path, content_type }>`  
- `site_head: Vec<HeadLink>`

在 `home` / 动态与静态 `route` 渲染前：`merge_site_head(&mut page, &state.site_head)`。

---

## 6. MIME 推断

| 扩展名 | Content-Type |
|--------|----------------|
| `.ico` | `image/x-icon` |
| `.png` | `image/png` |
| `.svg` | `image/svg+xml` |
| `.webp` | `image/webp` |
| `.jpg` / `.jpeg` | `image/jpeg` |
| `.gif` | `image/gif` |
| `.webmanifest` / `.json` | `application/manifest+json` |
| 其他 | `application/octet-stream` |

---

## 7. 作者示例（中文）

```markdown
## 站点图标

|路径|关系|类型|尺寸|地址|
|---|---|---|---|---|
|public/favicon.ico|icon|image/x-icon|32x32|/favicon.ico|
|public/apple-touch-icon.png|apple-touch-icon|image/png|180x180|/static/apple-touch-icon.png|

## 头资源

|关系|地址|类型|媒体|
|---|---|---|---|
|icon|/favicon.ico|image/x-icon|
|stylesheet|/static/print.css|text/css|print|
|script|/static/live.js||

## 品牌图

|源|替代|类|链接|宽度|加载|
|---|---|---|---|---|---|
|/static/logo.svg|Marqdo|brand-logo|/|48|eager|

# main

*page = > 网页.页面 标题="博客"*
*page = > page.头装配 表=`头资源`*
*page = > page.图片装配 表=`品牌图`*
*page = > page.元数据 元数据=`seo表`*

*app = > 网页.应用 页面=`page` …*
*app = > app.静态 目录="public" 挂载="/static"*
*app = > app.图标 表=`站点图标`*
> app.监听
```

英文对称：`web.page` / `head` / `images` / `meta`；`web.app` / `static` / `icons`；模块 `web.make_images`。

---

## 8. 实现清单（本切片必须全部完成）

| # | 项 | 归属 |
|---|----|------|
| I1 | `assets.rs`：表解析、MIME、`web_images` HTML、`web_head` HTML、图标规范化 | `plugins/web` |
| I2 | `web_page_head` / `web_page_images` / `web_images` / `web_head` / `web_app_icons` FFI + 注册 | `plugins/web` |
| I3 | `head_html`：meta 图标键 + `page.head` + 合并 `site_head` | `render.rs` |
| I4 | `images_html` 插入主区；fragment main 同理 | `render.rs` |
| I5 | `listen`：图标文件路由 + static 约定 favicon + `AppState` 注入 | `http.rs` / `lib.rs` |
| I6 | `ext/web/web.mq.md` + `网页.mq.md` L1 方法与叙述 | `ext/web` |
| I7 | 金样：离线装配 + live `GET /favicon.ico` + HTML 含 link/img | `tests/ext/` |
| I8 | `examples/marqdo-blog`：真实 favicon + logo 装配 + README | `examples/` |
| I9 | 更新 [web-net-capabilities.md](web-net-capabilities.md)、[ext-web.md](ext-web.md)、[doc/README.md](../README.md)、CHANGELOG Unreleased；Skill 专节一句 | 文档 |

**非目标（本切片不做）**

- 服务端图片缩放 / WebP 转码  
- CDN 签名 URL  
- 把上传相册（`gallery`）与品牌图混为一谈  
- 进程内 TLS（仍见反代文档）

---

## 9. 验收标准

1. `marqdo run tests/ext/web-assets-smoke.mq.md` 离线：icons/head/images 字段与 HTML 片段断言通过。  
2. Live：`GET /favicon.ico` → 200 + 正确 Content-Type；首页 HTML 含 `rel="icon"` 与 `mq-images`。  
3. `examples/marqdo-blog`：`curl /favicon.ico` 与页面源码可见图标与 logo。  
4. 旧金样（`web-static-smoke`、`web-content-smoke` 等）不回归。  
5. `ext/**` 无 `host_*`。

---

## 10. 与能力矩阵的关系

在 [web-net-capabilities.md](web-net-capabilities.md) **类 D** 增补：

| # | 能力 | 状态 |
|---|------|------|
| **D10** | 站点图标 / `/favicon.ico` / apple-touch-icon | ✅ 本设计 |
| **D11** | Head 资源表（link / script / preload / manifest） | ✅ 本设计 |
| **D12** | 图片装配（GFM → figure/img）+ 主区注入 | ✅ 本设计 |

波次标签：**W8（站点资源与图片装配）** — 在 W7+P3 完结之上的内容站体验补齐，不推翻既有边界。
