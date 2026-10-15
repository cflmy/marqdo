# View：文档面与结果引出标注

| | |
|---|---|
| 状态 | **Active** |
| 日期 | 2026-10-15 |
| 相关 | [view.md](view.md) · [view-debug.md](view-debug.md) · [view-markup-surface.md](view-markup-surface.md) · [view-execution-trace.md](view-execution-trace.md) · [view-frame-selection.md](view-frame-selection.md) · [code-as-docs-gaps.md](code-as-docs-gaps.md) |

---

## 1. 动机

`.mq.md` 是叙述与程序的同一载体。`marqdo view` 的 Structure 必须**保真叙述**，并在可执行标记旁展示运行结果。

旧 Browse Structure 用 AST 卡片重建语句，并用 `emit_comments` 只插入**语句行之前**的注释。当叙述与 `**…**` / `*…*` / `` `名` `` 在同一行时，该行被 Stmt 占用，叙述被丢弃——信息密度接近 debug，不适合文档浏览。

## 2. 模式分工

| 命令 | Structure | 叙述 | 结果 |
|------|-----------|------|------|
| `marqdo view`（Browse） | **文档面** | 保留并为主 | 跑后引出标注（callout + CSS L-leader） |
| `marqdo debug` | AST 卡片 + BP gutter | 弱化/可折叠 | Variables / 步进；不做文档 callout |

断点与逐步执行**仅**在 debug。静态 `view output` 与 live view **共用** Browse 文档面渲染。

## 3. 视觉语法

```
文本段…… `变量` …… **语句** …… *返回*
              ━━━━━
                │
                └──→ 结果标注
```

1. 叙述正文正常排版（Markdown）。
2. 可执行片段高亮，并带底部分割线（underline）。
3. 段落下方 **annotation band** 放结果 callout。
4. **CSS L-leader**（绝对定位小盒 + `border-left` + `border-bottom`）连接锚点与 callout。**不使用 SVG。**
5. 拥挤 / 极窄屏 / `prefers-reduced-motion`：取消长横折，改为编号 keyed 列表。

关联强度（递减）：同色 underline + callout 色条 + label → CSS L-leader → keyed 编号。

对齐 [view.md](view.md) HIG：系统字体、无外链、浅色、克制装饰。

## 4. 文档面渲染

实现：`src/view/document.rs`；Browse 入口经 `render_module_structure` 调用。

| 源内容 | 呈现 |
|--------|------|
| 纯叙述段 | Markdown HTML；无 `comment` badge / 厚卡片 |
| 含 `ProseBit` 的行 | 原文保留；Decl / BoldCode / ItalicReturn 包 `mark.mq-anchor`（字节偏移来自 `scan_prose_line`） |
| 整行结构（`#` / `>` / `-` / `1.` / 表） | 表面语法保真（见 [view-markup-surface.md](view-markup-surface.md)）；整句可作锚点 |
| 函数标题 | 轻量 heading + `id="fn-{line}"`（大纲跳转不变） |

Debug 仍走 `render_module_structure_mode(..., Debug)` 的 AST 卡片路径。

## 5. ViewAnnot

跑后（或 `--no-exec` 时为空）组装标注集，嵌入文档面 `data-*` 与 band HTML。

| 字段 | 含义 |
|------|------|
| `id` | 稳定短 id（页内唯一） |
| `kind` | `bind` · `return` · `call` · `expr` · `branch` · `loop` · `writeback` |
| `anchor_line` | 1-based 源行 |
| `anchor_byte_start` / `anchor_byte_end` | 行内字节范围（可选；整行锚点可省略 end） |
| `label` | 短标题（名 / return / arm / loop） |
| `value` | 截断显示 |
| `detail` | 完整文本（inspect） |

| kind | 数据来源 | 锚点 |
|------|----------|------|
| `bind` | **StmtEvent post-stmt**（赋值执行后的值）；禁止用入口终态回填初始化行 | 该赋值行 |
| `return` | 返回事件或入口返回 | ItalicReturn / Return |
| `call` / `expr` / `writeback` | writeback 槽、调用摘要 | 语句行或 BoldCode |
| `branch` | `ViewTrace`：命中 arm、次数 | 条件 / arm |
| `loop` | `ViewTrace`：迭代次数、末次 item | While / ForEach 头 |

标记面、结构块、多调用帧与 AI JSON 见 [view-execution-trace.md](view-execution-trace.md)。

`ViewTrace` 由 tree 解释器在 capture 时静默收集，**不**写入 stderr；与 `trace_eval` 调试轨迹分离。Bytecode 后端可暂无 trace（标注降级为 writeback）。

## 6. 布局与密度

- 每段默认最多 **4** 个常显 callout；其余 hover / 「show more」。
- Leader：按锚点 x 排序 callout，避免交叉；窄屏改为短竖线 + 堆叠。
- 客户端：`ResizeObserver` 重算 leader 盒的 `left/top/width/height`。
- 静态导出：页载 JS 跑一次布局。

右侧 **Variables** 仍为全集绑定；callout 为就地摘要，点击可打开同一 inspect。

## 7. 非目标

- 不在 view 内做断点 / 单步（见 [view-debug.md](view-debug.md)）。
- 不新增作者侧标注 DSL / frontmatter 指令。
- 不引入外链字体、npm 组件或 SVG leader。
- 不把 DB/auth 等塞进 view 壳层。

## 8. 验收

1. `inc-prose.mq.md`：叙述「对于输入变量…」完整可见；`n` / `n=n+1` / 返回高亮。
2. 跑完后：值经 underline + CSS L-leader 连到下方标注（无 SVG）。
3. 分支/循环样例：可见命中 arm 与迭代次数（有 ViewTrace 时）。
4. `marqdo debug` 仍为 AST + 断点；叙述不抢主视觉。
5. 无外链字体/新依赖；`view output` 可离线打开。

## 9. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-10-15 | 初稿：文档面、ViewAnnot、CSS L-leader、与 debug 分工 |
| 2026-10-15 | bind = post-stmt；交叉链 view-execution-trace |
