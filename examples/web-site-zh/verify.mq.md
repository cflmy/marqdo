---
markup: occlusion
title: 主题迁移的原生核对证据
import:
  document: lib/document.mq.md
  table: lib/table.mq.md
  hash: lib/hash.mq.md
  result: lib/result.mq.md
  theme: styles/index.mq.md
  original: styles/shell.mq.md
---

本工具读取实际声明并编译原主题和 View 投影，不发放整个站点的通过证书，也不能替代 HTTP 与浏览器验收。它不初始化或写入业务数据库。路径和数据读取都通过固有能力完成，没有另一种语言搬运源码或计算期望值。

```sh
marqdo call examples/web-site-zh/verify.mq.md theme_values --effects plugin,fs.read --fs-root examples/web-site-zh --require-ok --json
```

## _row

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| before | map | 必填 | 原始字面声明的单条记录 |
| after | map | 必填 | 相同位置的实际投影记录 |
| keep_selector | bool | False | 令牌和关键帧还须保持地址自身 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | 允许的地址变化之外，整条声明是否相同 |

使用`before`、`after`、`keep_selector`=False。先只排除双方同名的选择器字段，再比较完整记录，因此属性、值、可选媒体和其他已有字段都不能消失。原生字典比较不依赖键的显示顺序；值类型与内容仍必须相同，不用文本格式化掩盖差异。

全局令牌与动画关键帧连地址自身也必须保持，这时再比较原始记录中的选择器。本纯函数不读取文件、不编译 CSS，也不宣称相同记录已经呈现在浏览器中。不可变删键返回新记录，输入仍供地址比较和后续诊断使用。

我们 **left = [只排除原始结构地址](table.delete(map=`before`, key="选择器"))** 和 **right = [只排除投影结构地址](table.delete(map=`after`, key="选择器"))**。
1. `left` != `right`
  返回 *False*，因为声明数据本身已经变化。
1. `keep_selector`
  返回 *[全局声明同时保持地址](access(before, "选择器") == access(after, "选择器"))*。
返回 *True*，表示只发生允许的局部地址投影。

## _area

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | 必填 | 精确的原始主题章节名称 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result，成功值为该区段实际逐行核对数 |

输入`name`，从站点根授予 fs.read。原表继续使用选择器、属性、值和媒体的中文列名，投影保留这些键，不能为复用代码而重写原始数据。先核对实际行数，再按相同位置逐项比较，因此删除、额外添加和交换不同声明都能保留失败。

数据来自原生字面表读取，投影来自当前样式入口。缺失章节或非法列形态保留原生诊断；有变化则返回具体区段的失败 Result，不将空输出当作成功。累计值只来自已完成比较的每一行。令牌和关键帧采用更严格的全局地址规则，其余区段仅允许明确的选择器映射。

列表下标从零开始，每次只读取相同位置的投影。不能把记录先排序再比较，因为相同选择器重复声明时，原顺序本身决定 CSS 级联结果。只有一行完整通过后才递增位置，失败不会把已经比较的部分数目包装成整个区段的成功值；上层据此决定是否继续统计。

我们 **columns = [读取原始声明章节](document.read_table(path="styles/shell.tables.md", title=`name`))**、**rows = [保留原始行序](table.from_columns(columns=`columns`))** 和 **projected = [运行真实选择器投影](theme._table(name=`name`))**。
1. len(`rows`) != len(`projected`)
  返回 *[保留声明数量变化](result.error(code="site.theme_rows", message=`name`, operation="theme_values"))*。
我们 **position = 0**。
- [按相同位置核对完整声明](for rows as row)
  我们 **actual = [读取对应投影行](table.at(xs=`projected`, i=`position`))** 和 **same = [核对允许地址之外的全部字段](_row(before=`row`, after=`actual`, keep_selector=(`name` == "vars" or `name` == "motion")))**。
  1. not `same`
    返回 *[保留实际声明差异](result.error(code="site.theme_value", message=`name`, operation="theme_values"))*。
  我们 **position = position + 1**，只累计真正通过的声明。
返回 *[交出该区段的实际核对数](result.ok(value=`position`))*。

## theme_values

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result，包含已核对行数、原主题及实际 View CSS 的摘要 |

从站点根授予 plugin 与 fs.read。逐区段运行实际原始表和投影的核对工具，失败时原样返回失败，不继续累加其他结果。六个区段必须全部完成，媒体、值、属性和顺序均由各自完整记录核对，令牌和关键帧还要求地址保持不变。

成功值含 preserved_rows、original_sha256、view_sha256，摘要来自原生编译器真实输出。新 View 地址和包装层几何会让两份 CSS 不同，不能把摘要相同当成唯一通过条件。它们只是可追踪产物身份，浏览器仍需验证色板、字体、几何、表单和当前记录。

命令中的 --require-ok 让失败 Result 影响进程退出，自动化不能只看外层调用完成就读取成功字段。读取与编译不写文件、不初始化数据库，也不清理用户目录。实际站点行为使用独占副本测试，服务退出后再清理。这个工具不发放整个项目的完成证书，也不会用删除失败规则的方式修复差异。

`areas` =

| name |
|---|
| vars |
| motion |
| layout |
| responsive |
| chrome |
| widgets |

我们 **count = 0**。
- [核对每个实际原始主题区段](for areas as name)
  我们 **checked = [运行有界区段比较](_area(name=`name`))**。
  1. not access(`checked`, "ok")
    返回 *checked*，保留真实失败。
  我们 **count = [累计实际核对数](count + access(checked, "value"))**。
我们 **old_css = [编译保留的原主题](original.css())**、**view_css = [编译实际 View 主题](theme.css())**、**original_sha256 = [识别真实原始 CSS](hash.sha256(text=`old_css`))** 和 **view_sha256 = [识别真实 View CSS](hash.sha256(text=`view_css`))**。

`evidence` =

| preserved_rows | original_sha256 | view_sha256 |
|---|---|---|
| `count` | `original_sha256` | `view_sha256` |

返回 *[交出真实核对和编译证据](result.ok(value=`evidence`))*。
