# 数学标准库（候选）

| | |
|---|---|
| 状态 | **草案 / 暂缓**（本波不实现；五库之后再议） |
| 日期 | 2026-08-05 |
| 相关 | [stdlib-modules.md](stdlib-modules.md) · [stdlib-i18n.md](stdlib-i18n.md) · [view.md](view.md) · [markdown-mapping.md](markdown-mapping.md) |
| 灵感 | Markdown `$$…$$` / `$…$` 公式面；文档即计算笔记本 |

---

## 1. 为什么值得做

Markdown 读者已习惯用 `$$` 写公式。若 Marqdo 能：

1. **认出**文档中的公式（或显式传入 LaTeX/ASCII-math 字符串）；  
2. **符号整理 / 求值 / 求解**；  
3. 在 **`marqdo view` 里画出**简单图像；  

则同一份 `.mq.md` 既是讲义又是可运行实验，差异化远大于「再包一层 `sin`」。

风险同样大：CAS、数值稳定、渲染栈都会显著增加依赖与维护面。故标为**候选**：先定分层，允许只实现「数值 + 显式字符串」，再渐进接 `$$`。

---

## 2. 产品分层（建议）

| 层 | 能力 | 依赖量 | 建议阶段 |
|----|------|--------|----------|
| **Math-N（数值）** | 四则、幂、根、三角函数、常数；列表上的 map 式计算 | 小（可纯 Rust） | 若纳入官方库，**先做这层** |
| **Math-S（符号）** | 化简、展开、求导、代入、解方程（单变元优先） | 中–大（自研子集或嵌 CAS） | 第二期 |
| **Math-P（作图）** | 把表达式/点列渲染为 SVG（或 PNG）嵌入 view | 中（SVG 即可） | 与 view 同期 |
| **Math-D（文档公式）** | 解析正文 `$$…$$`，命名后供调用 | 中（词法/源映射） | 第三期 |

**默认发版建议：** 官方 `lib/math` 先只承诺 **Math-N**；S/P/D 放 `experimental` 或 feature `math-cas` / `math-plot`，避免「标准库」名不副实。

---

## 3. 导入与命名

| 导入 | API 语言 |
|------|----------|
| `> lib/math.mq.md` | 英文 |
| `> lib/数学.mq.md` | 中文 |

宿主原语（L0.5）示例：`math_sin`、`math_eval`、`math_solve` —— 仅供库包装。

---

## 4. 用户 API 草图

### 4.1 数值（Math-N）

| 英文 | 中文 | 说明 |
|------|------|------|
| `pi` / `e` | `圆周率` / `自然底数` | 无参，返回文本或「十进制文本」——**本波仍无 float 类型时用文本十进制**，或引入 `num` 宿主类型 |
| `add` `sub` `mul` `div` `pow` `sqrt` | `加` `减` `乘` `除` `幂` `根` | |
| `sin` `cos` `tan` `ln` `exp` | `正弦` `余弦` `正切` `对数` `指数` | 弧度 |
| `round` `floor` `ceil` | `四舍` `向下` `向上` | |
| `eval_num` | `求值` | `expr` 文本 → 数值文本 |

**类型抉择（必须先 ADR）：**

| 选项 | 优点 | 缺点 |
|------|------|------|
| A. 继续只用 `int` + 十进制 `text` | 不碰 L0 | 慢、易错 |
| B. 新增宿主 `num`（十进制或 f64） | API 干净 | 类型与字节码要跟进 |
| C. 数学库内部不透明 `handle` | 隔离好 | 与 `print`/`json` 互通差 |

倾向：**B（`num`）**，与 JSON 的非整数数字策略一并设计。

### 4.2 符号（Math-S）

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `simplify` | `化简` | `expr` | 表达式文本（或 handle） |
| `expand` | `展开` | `expr` | 同上 |
| `diff` | `求导` | `expr`, `var` | 同上 |
| `subs` | `代入` | `expr`, `var`, `value` | 同上 |
| `solve` | `求解` | `expr`, `var` | 解列表（文本） |

表达式互换格式（草案）：**ASCII 优先**（`2*x^2 + 1`），LaTeX 作为可选输入 `format=latex`。

单变元多项式 / 简单超越方程为 v1 目标；多变元方程组、积分：**不做承诺**。

### 4.3 作图（Math-P）

| 英文 | 中文 | 说明 |
|------|------|------|
| `plot_fn` | `绘函数` | `expr`, `var`, `min`, `max`, 可选 `steps` → **SVG 文本** |
| `plot_points` | `绘点列` | `xs`, `ys` → SVG 文本 |

view 行为：

- 若 stdout / 返回值为「带标记的 SVG」或约定 MIME，Execution 区**内嵌渲染**（非仅 `<pre>`）。  
- 静态 `view output`：把 SVG 写入 HTML。  
- CLI：默认打印 SVG 文本；`--plot-file out.svg` 可选。

不引入完整浏览器 Chart 框架；**SVG 路径足够**教微积分与拟合入门。

### 4.4 文档公式面（Math-D）

与 [markdown-mapping.md](markdown-mapping.md) 的衔接：

| 现状 | 目标 |
|------|------|
| `$$` 多半落入叙述/未定义 | 可选：**命名公式块**可被引用 |

建议语法（草案，需进 mapping 修订）：

```markdown
$$:energy
E = mc^2
$$
```

或 frontmatter / 标题锚定：

```markdown
## energy
$$
E = mc^2
$$
```

调用：

```markdown
---`
> lib/math.mq.md
---`

*`e2` = > simplify expr=> formula name=energy *
```

更简单的 v1：**不解析正文 `$$`**，只接受：

```markdown
*`e2` = > simplify expr=E=mc^2 format=latex *
```

正文 `$$` 仍由 view 的 Markdown 渲染（KaTeX/MathJax）负责「好看」，计算走显式字符串——**降低词法耦合**，推荐作为第一刀。

---

## 5. 实现策略选项

| 策略 | 说明 | 建议 |
|------|------|------|
| **纯 Rust 数值** | `libm` / 自写 | Math-N 必选 |
| **嵌入轻量 CAS** | 如符号子集自研，或绑 `meval`/`fasteval` 仅数值 | 求值用 |
| **外挂 SymPy（经外联库）** | 符号交给 Python | 与 [stdlib-foreign.md](stdlib-foreign.md) 合流；标准库变薄 |
| **WASM CAS** | 可嵌入 view | 后期 |

务实路径：**Math-N 自研 → Math-P SVG → Math-S 先外联 SymPy（可选）→ 再考虑内嵌**。这样「数学标准库」在官方包里可以很瘦，重能力走外联 feature。

---

## 6. 安全与性能

- 表达式长度、求解迭代、绘图采样点设硬上限。  
- 禁止公式里跑任意宿主代码（与外联分离）。  
- view 中绘图同步、限时。

---

## 7. 是否定为「标准库」的决策标准

纳入默认 `lib/math` 当且仅当：

1. Math-N API 稳定且有中英金样例；  
2. 不强制下载数百 MB 依赖；  
3. 无外联时文档站示例仍可跑（纯数值 + 可选 SVG）。

若符号/作图依赖 SymPy 或大型引擎 → 标 **`lib/math` = 数值**，符号示例放到 `examples/math-cas` 并要求 `--allow-foreign`。

---

## 8. 一句话

**用 Markdown 的公式习惯服务「可计算文档」；官方数学库先数值与 SVG，符号与 `$$` 引用分期，必要时借外联而非做大内核。**
