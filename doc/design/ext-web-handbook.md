# Web：可执行手册与递归布局工程

| 项目 | 内容 |
|------|------|
| 日期 | 2026-11-08 |
| 使用与开发入口 | [英文 index](../../ext/web/index.mq.md) · [中文索引](../../ext/web/网页/索引.mq.md) |
| 工程规范 | [布局拆分](ext-web-layout-paradigm.md) |
| 参考工程 | [英文](../../examples/web-layout-handbook/index.mq.md) · [中文](../../examples/web-layout-handbook-zh/index.mq.md) |
| 自动验收 | [源手册与活站点](../../tests/web_handbook.rs) · [浏览器布局](../../tests/web/layout.browser.cjs) |
| 思想来源 | [next/010](../next/010.md) · [next/011](../next/011.md) · [next/012](../next/012.md) · [Quantum 优化](ext-quantum-handbook.md) |

## 原有规范为什么没有落实

原有 Web skill 已写「拆到简单」，但缺少可沿着文件树复现的工程。原生 `RenderNodes` 仅渲染平面 title/text/link/html；拼接两张表只产生兄弟节点，不会产生双栏。`serve` 又只把 Document 正文当 Markdown，没有执行文档里的组件装配。只加强口头禁止 HTML，无法补上这些实际缺口。

本轮以一个完整首页检验规范：导航、主体、页脚分开；页脚拆品牌与链接；链接再拆文档与社区。所有末端 content.md 都是普通 Markdown。每个父 index.mq.md 只负责其下一层排列关系。

## 使用同一份 View 表表示父与叶

沿用 `type | slot | value | attrs | style`，不新增核心语法、不建立第二套布局方言：

| 类型 | 值列 | 含义 |
|------|------|------|
| markdown / 正文 | Markdown 文本 | 内容叶子 |
| row / 横排 | 子节点列表 | 响应式分栏 |
| column / 纵排 | 子节点列表 | 上下排列 |
| nav/header/footer/section/aside | 子节点列表 | 语义区域 |
| 既有 title/text/link/html | 兼容原值 | 保留旧节点行为；html 为显式兼容边界 |

纯 `component.markdown` / `正文` 与 `component.wrap` / `包裹` 构造节点，`layout.stack` / `纵排` 和 `layout.columns` / `分栏` 声明几何。`merge` 接受 @ 记录列表，确定兄弟顺序并保留嵌套值；不再声称它会自动归一化所有列式表或把平面表变成双栏。

递归渲染保留节点类型，不把子表转成 `map[...]` 字符串。原生渲染器诊断无效子节点、列长不等和过深嵌套。运行时固定的布局样式提供基础响应式几何，样式列继续是类名引用，并非 CSS 编译器。叶子正文经过既有 Markdown 渲染器。

## 显式 Document 装配

普通 `type: web` 保留既有 Markdown/数据源路径。需要结构化页面时显式增加 `view: main`（中文 `视图: main`），普通 `# main` 返回 View 节点列表。`serve` 在启动时通过已有 `run_artifact` 回调执行装配，用与离线预览相同的渲染器生成文档，不再套入旧站点的另一份导航/页脚壳。

这是启动时装配，修改内容后重启；不声称具备自动请求期模板绑定。装配函数不再次调用渲染器或启动服务器。请求期业务仍用 Endpoint。自动 `data_source` 绑定使用普通 Document；与 `view: main` 混用会明确报错，避免静默忽略数据源。

`web.inspect root=...` / `网页.检视 根=...` 新增离线扫描入口，不启动进程也不执行组件。空应用的新进程无法读取另一个活进程的私有路由；手册不再将此作为活站点检视方法。检视结果加入 `_type: web_inspection`，使不同长度的 declared/system 列表保持对象语义，而不被 JSON 边界当作列式表。

## 手册与工具一致

另修复既有元数据型 Endpoint 在 Bytecode 没有函数时误报编译错误的问题：与 Tree 共用声明摘要路径，普通无入口程序仍然拒绝执行。

全部 Web 模块清理说明性伪函数，参数和返回契约均放在执行前；对象 API 保留。纯 DOM 构造与客户端描述不再无故加载插件。命令示例保护文本引号与会话美元引用。布尔/数字默认值与相邻标点分开，保证真正省略参数时仍符合契约。

原 `accept_component` 仅根据名称和职责就返回 ok=True，无法代表实际验收。现在返回明确的 checklist 状态，实际验收依据源码职责、渲染结构和运行结果。`site_playbook` 讲流程，`preview_markdown` / `preview_nodes` 给真实预览，不再把增加叙述字数当作完成指标。

## 验证

```bash
bash scripts/build-web-plugin.sh
cargo test --locked --test web_handbook
cargo test --locked --test call_cli web_
cargo test --locked --test gold ext_web -- --test-threads=1
```

手册门禁直接执行 18 份包内入口的双语 sh 围栏，在 Tree/Bytecode 共执行 80 次命令；同样执行根手册的作者程序。参考工程的双语双后端检查实际预览、启动服务，核对首页与预览相同、关于页仍走普通 Document、API 正常返回 JSON；不把路由声明误当作已经监听。

浏览器检查可对参考预览或活站点执行，按环境指定现有 Playwright 模块和 Chromium：

```bash
node tests/web/layout.browser.cjs /absolute/path/to/preview.html
```

它测量 1440px 下的外层与内层双栏、390px 下的两层纵向折叠，并检查无横向溢出。无需外部资源或业务 JavaScript。Go 单元测试覆盖递归 Markdown、顺序、带类型列式表、错误形状与 Document 显式执行边界。

通用数据文档渲染测试已从硬编码个人数据库改为临时目录建表并注入已知数据，使完整 Go 套件无需个人站点状态。

Web 提交时，全库 Rust 为 436 通过、25 项既有策略/回放失败，基线失败集合相同。后续已单独修复 Rust→Lean 命令缺字段的问题：所有操作发送完整的 op、steps、value，进程错误保留退出码与 stderr。原 25 项失败全部消除，新增诊断回归后库级测试为 462 通过、0 失败。策略判定仍由 Lean 执行。
