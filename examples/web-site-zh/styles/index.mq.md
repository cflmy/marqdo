---
markup: occlusion
title: 递归 View 的文学主题入口
import:
  document: lib/document.mq.md
  网页: ext/web/网页.mq.md
  table: lib/table.mq.md
  text: lib/text.mq.md
---

先读本入口再修改呈现。shell.tables.md 保留每条原始声明，shell.mq.md 仍能编译原来的 CSS。本入口只把选择器投影到明确的 View 树，恢复旧装配附着的命名属性，再加入 view.tables.md 中有界的包装层几何。颜色、属性值、媒体条件、粒子与关键帧正文都来自原始数据，不用简化主题掩盖结构迁移。

## _selector

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| selector | text | 必填 | 原主题中由作者控制的单个选择器 |
| mappings | list | 必填 | 从 view.tables.md 读取的明确 old/new 地址行 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | 映射到当前递归框架的等效选择器 |

使用`selector`和`mappings`。旧 body 的侧栏与后台状态已移入 site-layout，主体改为 section，避免在既有文档 main 中再嵌套 main。原来的分类、主标题、徽章容器和阅读步骤分别对应终端 Markdown 的引用、标题、无序与有序列表。

文档背景、根令牌、选中文字、重置与关键帧保持全局，局部选择器增加本站作用域，避免基础 View 默认值覆盖明确主题。这是已知组件树的固定迁移，不是通用 CSS 解析器，也不接收访客的重写规则。调用者每个区域只读取一次映射，再显式传给本纯函数，避免逐条声明重复文件读取。

- [按本工程明确的结构地址映射处理](for mappings as mapping)
  我们 **selector = [只替换已知的选择器地址](text.replace(text=`selector`, old=access(mapping, "old"), new=access(mapping, "new")))**.
1. `selector` == "*" or `selector` == "::selection" or text.starts_with(text=`selector`, prefix="body") or text.starts_with(text=`selector`, prefix="html") or text.starts_with(text=`selector`, prefix=":root") or text.starts_with(text=`selector`, prefix="@")
  最后返回 *selector* ，使文档规则与关键帧保持原有作用域。
1. text.starts_with(text=`selector`, prefix=".site-layout")
  最后返回 *[只为布局状态增加一次作用域](".mq-document " + selector)*.
最后返回 *[将区域选择器限定在本站框架内](".mq-document .site-layout " + selector)*.

## _table

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | 必填 | 原始 vars、motion、layout、responsive、chrome 或 widgets 章节 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | 仅选择器字段变化的原始声明行 |

使用`name`，从站点根授权 fs.read。原生 Markdown 读取保留声明和字面类型，列转行后保持顺序。每条记录的属性、值、可选媒体与原位置都不变，只有选择器经过上面的固定映射；不可变更新不会污染原来的主题表。

本函数不编译或写 CSS。保留可调用的投影行，让验收能逐项核对属性、值和媒体条件，再观察浏览器结果。缺失章节、列长不齐或不支持的字面量都应保留真实诊断，不得静默丢弃样式。令牌和关键帧地址自身也保持不变，不能只比较颜色却删掉动画。

我们 **mappings = [读取明确的结构迁移表](document.read_table(path="styles/view.tables.md", title="selectors"))**.
我们 **columns = [读取未改变的原始声明叶子](document.read_table(path="styles/shell.tables.md", title=`name`))** 和 **rows = [逐行保留原有声明](table.from_columns(columns=`columns`))** 和 **projected = [开始有序的选择器投影](table.empty_list())**.
- [在原位置保留每条原始声明](for rows as row)
  我们 **selector = [只映射这一行的结构地址](_selector(selector=access(row, "选择器"), mappings=`mappings`))** 和 **record = [保持其余声明字段不变](table.put(in=`row`, at="选择器", value=`selector`))** 和 **projected = [保留原始属性与值](table.append(list=`projected`, item=`record`))**.
最后返回 *projected* ，供实际编译或独立比较。

## _property

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | 必填 | 原来的命名属性/值章节 |
| selector | text | 必填 | 新树中明确语义目标；名称不含开头的点 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | 原始属性在明确新目标上的实际编译结果 |

使用`name`与`selector`，授予 fs.read 与 plugin。旧装配曾把这些属性附到生成的字段，现在同一声明直接选择导航、侧栏/页脚、原生卡片和终端 Markdown 元素，值不改变，也不由用户内容推断。

目标由主题入口控制，以 mq-document 开头，原生编译器补上类选择器的点。明确目标让旧属性表仍有用，同时不恢复已移除的页面绑定方言。返回只是文本，保存路径和原子写入由服务入口负责；编译不等于文件已经挂载或浏览器已经加载。

我们 **properties = [读取原来的命名属性声明](document.read_table(path="styles/shell.tables.md", title=`name`))**.
最后返回 *[编译明确的语义目标](网页.编译样式(名称=`selector`, 属性=`properties`))*.

## css

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | 保留原始值、明确映射与包装层几何的完整 View 主题 |

从站点文件根授予 fs.read 与 plugin。令牌和关键帧保持原顺序，随后编译布局、响应断点、引言与控件；命名属性恢复旧装配单独附着的字体和区域外观。不会以替代色板或删减主题提高通过率。

最后的几何表只处理运行时包装：移除基础预览的固定宽度和字体默认，让导航资源的原生语义子块参与已有命名网格，并保留公共侧栏与后台状态。Markdown 项目符号成为原来的徽章组，首个解释段落使用原有导语属性，正文无需隐藏的 HTML 模板。

编译不会写样式表，也不发放浏览器验收结果。服务入口在发现页面之前原生保存确切文件，再测试实际计算颜色、字体尺寸、宽屏侧栏、窄屏堆叠、表单与后台。选择器失效时修正映射，不能删除视觉能力。各区域顺序和命名属性目标都在相邻数据叶子中可直接阅读；缺少任何章节应停止，而不是退回旧生成文件。生成产物可以重建，原声明与迁移表才是编辑入口；不要手改生成 CSS 后误认为源码已经修复。样式变化仍需宽屏和窄屏两个方向的实际观察。

我们 **areas = [读取保留的主题区域顺序](document.read_table(path="styles/view.tables.md", title="areas"))**.

我们 **parts = [开始实际有序的主题编译](table.empty_list())**.
- [编译每个原始主题区域](for areas as name)
  我们 **rows = [保留全部原始值与媒体条件](_table(name=`name`))** 和 **compiled = [编译实际投影后的声明](网页.编译样式(名称=`name`, 属性=`rows`))** 和 **parts = [保留原来的级联顺序](table.append(list=`parts`, item=`compiled`))**.

我们 **properties = [读取实际命名属性目标](document.read_table(path="styles/view.tables.md", title="properties"))**.

- [恢复每组原来附着的命名属性](for properties as property)
  我们 **compiled = [编译明确的语义选择器](_property(name=access(property, "name"), selector=access(property, "selector")))** 和 **parts = [保留真实的命名属性组](table.append(list=`parts`, item=`compiled`))**.
我们 **geometry = [读取有界的 View 包装层适配](document.read_table(path="styles/view.tables.md", title="geometry"))** 和 **compiled = [编译原生几何适配](网页.编译样式(名称="view", 属性=`geometry`))** 和 **parts = [以明确的包装规则结束](table.append(list=`parts`, item=`compiled`))**.
最后返回 *[按顺序保留实际编译的各段](text.str_join(xs=`parts`, sep=""))*.
