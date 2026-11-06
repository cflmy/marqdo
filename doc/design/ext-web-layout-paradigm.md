# Web 布局工程范式：文件划分 · 直到简单 · 禁止注入

| | |
|---|---|
| 状态 | **Active** |
| 日期 | 2026-11-06 |
| 包内落地 | [`ext/web/index.mq.md`](../../ext/web/index.mq.md) · [`ext/web/layout/index.mq.md`](../../ext/web/layout/index.mq.md) |
| 上位 | [ext-web-artifact.md](ext-web-artifact.md) · ADR [0007](../adr/0007-web-document-endpoint.md) |
| 扩展范式 | [ext-skill-tools-paradigm.md](ext-skill-tools-paradigm.md) |

---

## 0. 一句话

> **页面不是一整块 HTML。页面是目录树上的若干 Document / 组件文件；每个文件只描述一层简单布局，复杂处继续拆文件，直到 View 表与 Markdown 都可读、可绑、可维护。**

值绑定、数据库绑定、样式绑定（`|type|slot|value|attrs|style|`）是默认语言。  
**禁止**把「能跑」建立在 `dom.raw` / 大段 HTML 字符串 / 随意 CSS 注入上——那是逃逸舱，不是工程方法。

---

## 1. 为什么 AI 会走偏

网络扩展提供了绑定与 View 表，但代理常觉得「写 HTML/CSS 更快」，于是：

| 偏航 | 后果 |
|------|------|
| 首页一个文件塞满结构 | 无法复用导航/页脚；改一处炸全站 |
| `dom.raw` / `set_html` 拼页面 | 绕过绑定与主题；不可检视、不可 i18n 对齐 |
| 私有表方言 / `compose_*` | 与 ADR 0007 冲突；回归神对象 |
| 样式写进巨大 `<style>` | 与 `style` 列、主题表脱节 |

本范式把**正确拆法写进扩展 skill 本身**，让冷启动代理先读规则再动手。

---

## 2. 文件划分布局（强制心智）

以站点首页为例，**先划文件，再写内容**：

```text
site/
  index.mq.md              # type: web · route: / · 只编排「壳」与主区意图
  pages/about.mq.md        # 其它 Document
  api/*.mq.md              # Endpoint
  components/
    nav.mq.md              # 顶栏（简单：品牌 + 链接表）
    sidebar.mq.md          # 侧栏（简单：列表或 TOC）
    footer.mq.md           # 底栏入口（若仍复杂 → 再拆）
    footer/
      brand.mq.md          # 底栏左：品牌/版权
      links.mq.md          # 底栏右：链接双栏之一
  styles/
    theme.mq.md            # 具名样式表（供 style 列引用）
  data/                    # 运行时库（gitignore）
```

### 2.1 划分判据

| 若出现… | 则… |
|---------|-----|
| 一个文件里同时有导航 + 主列 + 页脚 | 拆成 `components/*` + 页面正文 |
| 页脚已是「左品牌右双栏链接 + 复杂样式」 | 再拆 `footer/brand` · `footer/links`，各文件只保留一张简单 View 表或短 Markdown |
| View 表超过约一屏且多段语义 | 按槽位/区域拆文件，用 `component.merge` 或页面级 import 组合 |
| 想写 `<div class=…>` 才能对齐 | 先问能否用 `type/slot/style`；不能再考虑逃逸舱并注明原因 |

### 2.2 简单的定义

一个组件文件「够简单」当且仅当：

- 人一眼能说出**单一职责**（例如「仅顶栏链接」）；  
- 主数据是 **一张** View 表或短 Markdown，而不是嵌套 HTML；  
- 样式只通过 **具名 style**（主题表）或 `style` 列引用，不内嵌大段 CSS。

---

## 3. 合法装配面（唯一主路径）

| 允许 | 禁止（默认路径） |
|------|------------------|
| Document Metadata：`type: web` · `route` · `data_source` … | 隐式用户路由；`configure` 垃圾抽屉 |
| 正文 Markdown | 把整页写成 HTML 字符串 |
| View 表 `\|type\|slot\|value\|attrs\|style\|`（ZH 别名可） | `compose_*`；`\|属性\|值\|样式\|` 等废止方言当主装配 |
| `component.nodes` / `component.merge` | `json.set` 袋胶水拼 DOM |
| `web.render` / `page.render` | 作者 `ensure_plugin` / `host_*` |
| `dom.raw` / `page.css` **仅逃逸舱**（须注释「为何不能表驱动」） | 用 raw/css 作为首页主实现 |

数据与鉴权在 `ext/data` · `ext/security`；实时在 `ext/net`——不要塞回 `ext/web` facade。

---

## 4. 智能体操作顺序

冷启动只读 [`ext/web/index.mq.md`](../../ext/web/index.mq.md)：

| 若你需要 | 这样做 |
|----------|--------|
| 理解怎么拆页面 | 打开 `ext/web/layout/index.mq.md`，先 `marqdo call … rules` |
| 预览 Document HTML | `web.render` / `网页.渲染`（`--effects plugin`） |
| 起站 | 站点 `serve.mq.md` → `web.serve`；或 call facade `serve` |
| 检视路由 | `web.inspect` |

写站时循环：

1. 列出区域（导航 / 侧栏 / 主区 / 底栏…）→ 各建文件  
2. 每个文件只做简单 View 或 Markdown  
3. 复杂区域继续拆，直到简单  
4. 页面 Document 只保留路由、数据源与主区意图；壳由组件组合  
5. 绝不先写 HTML 再「补绑定」

---

## 5. 与 skill=tools 迁移

`ext/web` 按 [ext-skill-tools-paradigm.md](ext-skill-tools-paradigm.md) 迁移：根 `index` = SKILL，`layout` = 布局工程工具面，原有 `page`/`route`/… 补齐 For AI agents 与叙述占比。  
**布局范式是 web 包特有硬约束**，与 quantum 试点并列写入扩展内，而不是只写在外部 README。

---

## 6. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-11-06 | 初版：文件划分布局、反 HTML/CSS 注入、与 Artifact / skill=tools 对齐 |
