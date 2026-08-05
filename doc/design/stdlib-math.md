# 数学标准库（方案）

| | |
|---|---|
| 状态 | **已实现（M1–M4）** |
| 日期 | 2026-08-05 |
| 相关 | [stdlib-modules.md](stdlib-modules.md) · [stdlib-i18n.md](stdlib-i18n.md) · [markdown-mapping.md](markdown-mapping.md) · [view.md](view.md) |
| 导入 | `lib/math.mq.md`（英）· `lib/数学.mq.md`（中） |
| 范围 | **高中数学**（初等函数、导数、圆锥曲线、简单方程）；非大学 CAS |

---

## 1. 目标

1. **公式类型**：解析 `$$…$$`（及行内 `$…$` 的可选支持），得到可绑定、可传递的 **`formula` 值**——语义上像「公式常量 / 公式变量」。  
2. **数学库对 `formula`（及文本表达式）做操作**：求值、化简、求导、求解、作图等。  
3. **数值 + 随机**：日常算术与可复现随机。  
4. **作图**：view / 静态页内嵌 SVG；CLI 写本地文件（可指定 `path`）。

**难度自觉：** 只覆盖高中知识面，自研小型表达式 AST + 规则即可，**不**追求 Mathematica / SymPy 完备性。

---

## 2. 核心模型：`formula` 类型

### 2.1 运行时

| 类型 | `type` 标签 | 说明 |
|------|-------------|------|
| `Value::Num(f64)` | `num` | 浮点 |
| `Value::Formula(…)` | `formula` | 已解析的表达式树（或等价内部表示） |

`print` / `str` 对 `formula`：输出规范 ASCII（如 `2*x + 1`）或保留源 LaTeX 的一种（实现时写死并文档化；建议 **ASCII 规范式** 便于测试）。

### 2.2 文档中的 `$$` = 公式赋值（与表格同构）

在可执行面用 **空 RHS 赋值** 后跟展示数学块，把公式绑到变量（与 `` `xs` = `` 后跟表格一致）：

````markdown
# main

`f` =
$$
sin(x) + x
$$

*`df` = > diff formula=`f` var=x *

> plot formula=`f` var=x min=-3 max=3 path=f.svg
````

也支持：

- 单行围栏：下一行 `$$x^2 - 2$$`
- 同行 compact：`*`f` = $$x^2 - 2$$ *`

**废除：** `$$:name` / `$$ name=…` 模块级具名块（不再产生绑定）。

**无名 `$$…$$`：** 仅在成段注释（叙述面）里作展示，**不**进入运行时。可执行区若出现未挂在赋值上的裸 `$$` 围栏 → 报错，提示使用 `` `name` = `` + 围栏。

**行内 `$…$`：** v1 不做绑定；需要时用 `> formula text=…` 显式构造。

### 2.3 从文本构造

| 英文 | 中文 | 说明 |
|------|------|------|
| `formula` | `公式` | `text=` 解析为 `formula`（ASCII 或简单 LaTeX 子集） |

数学库函数形参统一接受：**`formula` 值**，或能解析的 **文本**（内部先 `formula` 化）。

### 2.4 与叙述面的关系

- `` `f` = `` + `$$…$$`：代码面赋值，得到 `formula` 值。  
- 成段注释里的 `$$`：仍是注释，不解析为公式。  
- view：公式赋值在 Structure 中显示为绑定，不是 comment。

（见 [markdown-mapping.md](markdown-mapping.md)：空 RHS + `$$` → 公式赋值。）

---

## 3. 知识范围（高中）

### 3.1 表达式与符号（S）

| 纳入 | 例子 |
|------|------|
| 多项式 | `x^2 - 3*x + 2` |
| 初等函数 | `sin` `cos` `tan` `ln` `exp` `sqrt` `abs` |
| 四则、幂 | `+ - * / ^` |
| 化简 / 展开 | 合并同类项、基本三角恒等（小集合） |
| 求导 | 单变元，初等函数求导法则 |
| 代入求值 | `subs` → `num` 或公式 |
| 求解 | 一元一次 / 二次闭式；其它用数值求根并标明 |

**不做（本波）：** 多元方程组、极限 ε-δ、级数、积分保证、矩阵、复数完备、任意证明。

### 3.2 作图（P）

| 纳入 | 说明 |
|------|------|
| 显函数 `y=f(x)` | 采样折线 SVG |
| 导数曲线 | 对 `formula` 先 `diff` 再画，或 `plot` 选项 `derivative=true` |
| 圆锥曲线 | 标准形：圆、椭圆、双曲线、抛物线（参数方程或隐式采样） |
| 点列 | `plot_points` |

**不做：** 三维、隐函数通用求解器、交互缩放（静态 SVG 足够）。

### 3.3 数值与随机（N）

常数、四则、三角、对数、取整、`num` 转换、`random` / `random_int` / `seed`（同前稿）。

---

## 4. API 草图（对 `formula` 操作）

### 4.1 构造与数值

| 英文 | 中文 | 要点 |
|------|------|------|
| `formula` | `公式` | 文本 → formula |
| `pi` `e` | `圆周率` `自然底数` | → num |
| `add`… / `sin`… | `加`… / `正弦`… | 对 num；若传入 formula 则返回 formula（符号）或报错——**v1 建议：算术函数只接 num；符号用 simplify/diff/subs** |
| `eval` | `求值` | `formula` + 变量赋值 → num |
| `random` / `random_int` / `seed` | `随机` / `随机整数` / `设种子` | 见前 |

### 4.2 符号

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `simplify` | `化简` | `formula` | formula |
| `expand` | `展开` | `formula` | formula |
| `diff` | `求导` | `formula`, `var` | formula |
| `subs` | `代入` | `formula`, `var`, `value` | formula 或 num |
| `solve` | `求解` | `formula`, `var`，可选区间 | list（num 或公式根） |

### 4.3 作图

| 英文 | 中文 | 形参 |
|------|------|------|
| `plot` | `绘图` | **`formula`**（或 `expr` 文本）, `var`, `min`, `max`；可选 `steps`, `path`, `derivative`, **`grid`**（默认开；`False`/`假` 关） |
| `plot_conic` | `绘圆锥` | `kind`（circle/ellipse/hyperbola/parabola）+ 系数；可选 `path`, `grid` |
| `plot_points` | `绘点列` | `xs`, `ys`；可选 `path`, `grid` |

返回 **SVG 文本**；坐标轴带箭头与刻度，默认网格；写文件与 view 嵌入规则同前（§5）。

`diff` / `simplify` / `expand` / 未完全数值化的 `subs` 返回 **`formula`**，可继续链式调用。

示例：

```markdown
# main

`f` =
$$
x^2 - 2
$$

*`roots` = > solve formula=`f` var=x *

> print text=`roots`

*`svg` = > plot formula=`f` var=x min=-3 max=3 path=parabola.svg *
```

---

## 5. 作图：CLI / view / 静态（不变精神）

| 场景 | 行为 |
|------|------|
| 带 `path=` | 写 SVG 到该路径（沙箱内） |
| CLI 无 path | 自动 `{stem}-plot-{n}.svg` + 一行 `plot: …` |
| view / 导出无 path | 不强制写盘；加入 **plots 产物列表**，Execution 内嵌 |
| view / 导出有 path | 写盘 + 内嵌 |

不把整段 SVG 默认灌进 CLI stdout。

---

## 6. 分期

| 期 | 交付 |
|----|------|
| **M1** | `Num` + 数值/随机；`Formula` AST + `` `name` = `` + `$$…$$` 赋值绑定 |
| **M2** | `simplify` / `diff` / `subs` / `eval` / 二次 `solve`（高中子集） |
| **M3** | `plot` / `plot_points` / `plot_conic` + view 内嵌 + path 写盘 |
| **M4** | 用户文档 `public/stdlib/math*` · 金样例 · mapping 文档补丁 |

M1–M3 可在同一发版列车上连续落地；验收以「公式赋值 → 求导 → 作图」一条龙为准。

---

## 7. 实现要点

```text
src/formula/          # 词法/AST/化简/求导/求值（高中规则表）
src/host/math.rs      # 数值、随机、对 formula 的 host 入口
src/host/plot.rs      # SVG
lex/parse：空 RHS 赋值 + $$ 围栏 → Expr::Formula
Value::Num / Value::Formula
Interpreter.plots: Vec<String>
lib/math.mq.md · lib/数学.mq.md
```

依赖：纯 Rust；不绑 SymPy。公式复杂度、采样点、SVG 大小设硬上限。

---

## 8. 刻意不做

- `$$:name` 模块级具名绑定（已废除）。  
- 无名 `$$` 自动进运行时。  
- 大学级数 / 完备积分 / 定理证明。  
- 通用隐函数作图引擎。  
- 3D、交互式图表库。

---

## 9. 验收

1. `` `f` = `` + `$$…$$` + `diff` / `solve` / `plot` 金样例（中英库）。  
2. 固定 `seed` 的随机金样例。  
3. `plot` + `path` 在 CLI 落盘；view HTML 含 `<svg`。  
4. `type` 对公式为 `formula`，对浮点为 `num`。  
5. view Structure 将公式赋值显示为绑定，而非 comment。

---

## 10. 一句话

**`` `f` = `` + `$$…$$` 产出 `formula` 值；数学库对它做高中范围的符号运算、求解与作图——数值与随机并行，图在 view 内嵌、在 CLI 写文件。**
