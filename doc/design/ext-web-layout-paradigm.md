# Web 布局工程：先拆布局，直到 Markdown 叶子

| 项目 | 内容 |
|------|------|
| 状态 | Active，2026-11-08 可执行参考工程与浏览器门禁已落地 |
| 包内手册 | [英文根入口](../../ext/web/index.mq.md) · [中文根入口](../../ext/web/网页/索引.mq.md) |
| 布局工具 | [layout](../../ext/web/layout/index.mq.md) · [布局](../../ext/web/布局/索引.mq.md) |
| 实现与边界 | [Web 手册优化](ext-web-handbook.md) · [Artifact](ext-web-artifact.md) · [ADR 0007](../adr/0007-web-document-endpoint.md) |

## 先建立区域树，再写内容

首页先拆成导航、主体、页脚。页脚若有左右两栏，拆成品牌与链接；链接区若还有双栏，再拆成文档与社区。继续按内部独立布局拆分，直到叶子只剩简单 Markdown 可以直接表达的内容。

```text
index.mq.md → components/index.mq.md（纵排）
  ├─ nav/index.mq.md → content.md
  ├─ main/index.mq.md → content.md
  └─ footer/index.mq.md（分栏）
       ├─ brand/index.mq.md → content.md
       └─ links/index.mq.md（分栏）
            ├─ docs/index.mq.md → content.md
            └─ community/index.mq.md → content.md
```

父文件只导入子块、合并兄弟节点并声明一层排列。叶子文件只说明一种内容职责；内容较长时，使用相邻的普通 content.md，避免字符串转义。拆分终点不是行数或文件数量，更不是每句话建一个文件。

## 何时继续拆、何时停止

| 观察 | 决定 |
|------|------|
| 导航、正文、页脚混在一个内容块 | 拆成父级装配和三个职责块 |
| 页脚内部仍同时负责品牌与链接排布 | 再拆左右子块 |
| 链接子块内部仍包含两个独立栏目 | 继续拆到各栏目 Markdown |
| 叶子只包含标题、段落、列表、链接、表格、图片 | 停止；已可用普通 Markdown 表达 |
| 父文件含大量正文、HTML 标签、CSS 字符串 | 职责或表达方式仍未收敛，继续拆解 |
| 结构已经简单，只为减少行数而再拆 | 不必拆；以可读职责为准 |

表单、数据源、交互各有自己的领域职责，应调用对应模块，不以“Markdown 不表达交互”为由恢复整页 HTML 壳。只有确实需要兼容宿主能力时，才在隔离边界使用已有 raw/css 接口并说明原因。

## 同一表结构贯穿所有层

公共列是 `type | slot | value | attrs | style`，中文可用 `类型 | 槽 | 值 | 属性 | 样式`。使用 @ 记录表产生节点列表。

- `component.markdown` / `组件.正文`：叶子值为普通 Markdown。
- `component.wrap` / `组件.包裹`：值中保留子节点列表，可标记 nav/footer/section 等语义区域。
- `layout.stack` / `布局.纵排`：声明上下关系。
- `layout.columns` / `布局.分栏`：声明响应式横向栏目。
- `merge`：只确定兄弟顺序，不会自动产生双栏，也不提前渲染 HTML。

原生运行时提供基本布局几何，窄屏自动折叠。style 列引用类名，不承载 CSS 源码，也不会自动定义主题。父块始终保留结构值；先渲染子块再拼 HTML 会丢失这条统一路径。

## 从文件走到真正页面

[英文参考工程](../../examples/web-layout-handbook/index.mq.md)与[中文参考工程](../../examples/web-layout-handbook-zh/index.mq.md)实现上面的整个拆分树。先运行 preview.mq.md 检查真实渲染，再进入工程目录运行 serve.mq.md。

结构化 Document 显式声明 `view: main` / `视图: main`，main 返回节点；服务启动时装配一次。普通 Markdown Document 保留既有行为，Endpoint 单独处理请求期互动。改动结构或叶子内容后重启。自动数据源绑定继续使用普通 Document，避免与启动装配混用。

验收同时检查源码和产物：叶子只含普通 Markdown，父文件只表达其一层关系；桌面页脚出现真正的两层分栏，手机两层都折叠；关于页、API 仍能工作。规则清单本身不是“已经通过”的证明。

```bash
cargo test --locked --test web_handbook
node tests/web/layout.browser.cjs /absolute/path/to/preview.html
```

详细测试方式和当前能力边界见[优化记录](ext-web-handbook.md)。
