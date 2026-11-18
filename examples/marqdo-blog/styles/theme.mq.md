---
markup: occlusion
title: 博客主题字面样式调用手册
import:
  document: lib/document.mq.md
  web: ext/web/web.mq.md
  text: lib/text.mq.md
  table: lib/table.mq.md
---

本文件负责可调用的说明与编排，[theme.tables.md](theme.tables.md)负责可独立阅读的 Markdown 常量。调用时把 --fs-root 指向 examples/marqdo-blog，读取需要 fs.read；连接数据库或编译样式时还需要 plugin。

## 基础

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

基础段声明纸张、文字和强调色变量，以及字体与页面骨架。返回值仍是原生样式表数据，编译与浏览器加载由各自的明确步骤完成。

从站点根目录授予 fs.read 后调用。实际常量位于 [theme.tables.md](theme.tables.md#基础)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取基础的实际 Markdown 常量](document.read_table(path="styles/theme.tables.md", title="基础"))*.

## 顶栏

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

顶栏段保留吸顶导航、磨砂背景、品牌渐变与链接状态。选择器与值都位于数据叶子，调用函数说明用途，避免把整套声明堆进执行流程。

从站点根目录授予 fs.read 后调用。实际常量位于 [theme.tables.md](theme.tables.md#顶栏)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取顶栏的实际 Markdown 常量](document.read_table(path="styles/theme.tables.md", title="顶栏"))*.

## 侧栏

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

侧栏段保留归档面板的布局、链接间距和悬停反馈。它只提供样式数据，具体侧栏内容仍由独立组件或 Document 负责。

从站点根目录授予 fs.read 后调用。实际常量位于 [theme.tables.md](theme.tables.md#侧栏)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取侧栏的实际 Markdown 常量](document.read_table(path="styles/theme.tables.md", title="侧栏"))*.

## 主体

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

主体段定义内容区留白、标题和导语的排版节奏。数据读取保持原有字段和值；主题编译不会替页面构造布局节点。

从站点根目录授予 fs.read 后调用。实际常量位于 [theme.tables.md](theme.tables.md#主体)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取主体的实际 Markdown 常量](document.read_table(path="styles/theme.tables.md", title="主体"))*.

## 卡片

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

卡片段保留文章列表的网格、阴影、封面和悬停状态。样式类名应与页面输出保持一致；此函数只读取声明，不查询文章数据。

从站点根目录授予 fs.read 后调用。实际常量位于 [theme.tables.md](theme.tables.md#卡片)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取卡片的实际 Markdown 常量](document.read_table(path="styles/theme.tables.md", title="卡片"))*.

## 页脚

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

页脚段保留页面末尾的留白、分隔线和链接外观。页脚若分为多个布局区域，仍应沿组件目录拆分，样式声明不会代替工程结构。

从站点根目录授予 fs.read 后调用。实际常量位于 [theme.tables.md](theme.tables.md#页脚)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取页脚的实际 Markdown 常量](document.read_table(path="styles/theme.tables.md", title="页脚"))*.

## 表单

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

表单段保留输入控件、按钮和管理操作的状态样式。字段校验、保存和鉴权属于数据与安全资源；样式只改变已存在控件的外观。

从站点根目录授予 fs.read 后调用。实际常量位于 [theme.tables.md](theme.tables.md#表单)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取表单的实际 Markdown 常量](document.read_table(path="styles/theme.tables.md", title="表单"))*.

## 响应式

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

响应式段保留窄屏断点、侧栏流向、内容间距和列表列数。媒体条件仍由原生样式编译器读取，浏览器结果需要实际窄屏验证。

从站点根目录授予 fs.read 后调用。实际常量位于 [theme.tables.md](theme.tables.md#响应式)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取响应式的实际 Markdown 常量](document.read_table(path="styles/theme.tables.md", title="响应式"))*.

## _编译段

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| 名称 | text | 必填 | theme.tables.md 中精确且唯一的样式章节名 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | 由所选样式声明编译的 CSS |

使用`名称`选择一个具有独立职责的声明段，授权 fs.read 和 plugin。字面表读取保留原始选择器、属性和值，原生样式编译器负责媒体条件及 CSS 单元格处理。标题错误会保留文档诊断；此步骤不写文件，也不会把样式附加到浏览器。

我们 **数据 = [读取这一段的实际样式常量](document.read_table(path="styles/theme.tables.md", title=`名称`))**。
完成后返回*[编译所选声明段](web.make_style(name=`名称`, table=`数据`))*.


## 全局

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | 全部声明段按稳定顺序编译的 CSS 文本 |

从站点根授权 fs.read 与 plugin 后调用。名称表保留基础、导航、侧栏、主体、卡片、页脚、表单和断点的原有顺序；每段通过同一编译步骤读取自己的 Markdown 常量。变量声明因此先于使用它们的组件规则，响应式覆盖放在最后。返回值仍是 CSS 文本，保存与资源引用由明确的后续步骤完成。各命名数据函数也继续保留，方便独立检查某个区域。

`names` =

| 名称 |
|---|
| 基础 |
| 顶栏 |
| 侧栏 |
| 主体 |
| 卡片 |
| 页脚 |
| 表单 |
| 响应式 |

我们 **parts = [开始有序的 CSS 段列表](table.empty_list())**。
- [依次编译每个独立声明段](for names as name)
  我们 **css = [编译当前区域](_编译段(名称=`name`))**，并 **parts = [保留覆盖顺序](table.append(list=`parts`, item=`css`))**。
完成后返回*[按区域顺序连接 CSS](text.str_join(xs=`parts`, sep="\n"))*.
