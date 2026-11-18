# 调研：线性代数 × 公式类 × 展示

| | |
|---|---|
| 状态 | 调研纪要（为 [ext-linalg](../design/ext-linalg.md) 供输入） |
| 日期 | 2026-09-07 |
| 相关 | [stdlib-math.md](../design/stdlib-math.md) · [ext-quantum-q7.md](../design/ext-quantum-q7.md) · SymPy MatrixExpr · Symbolics.jl · LAFigureSpecs |

---

## 1. 问题陈述

Marqdo 已有：

- `Value::Formula` + `` `f` = `` + `$$…$$`（高中标量符号）
- `formula::Expr::Matrix`：**数值**稠密矩阵字面量（`[[…]]` / `\begin{pmatrix}`），供量子门核对与 view KaTeX
- `ext/quantum` Q7：**量子语境**下的密度矩阵 / Kronecker / 谱 / SVD 风格图（ABI）

缺口：一门**经典线性代数**扩展——作者用**公式级矩阵表达式**写 \(A^\top A\)、\((X^\top X)^{-1}X^\top y\)、分块、Kronecker，在**不立刻展开成巨大稠密矩阵**的前提下做重写与化简，并在文档 / view 中**好看地呈现**分解与结构。

---

## 2. 业界对照

### 2.1 符号矩阵表达式（大规模优化的关键）

| 系统 | 模型 | 对 Marqdo 的启示 |
|------|------|------------------|
| **SymPy `MatrixExpr`** | `MatrixSymbol(name, n, m)` + `MatMul` / `MatAdd` / `Inverse` / `Transpose` / `Trace` / `Determinant` / `BlockMatrix` / `KroneckerProduct`；`as_explicit()` 才展开 | **默认保持抽象**；形状元数据传播；显式求值是可选动作 |
| SymPy `block_collapse` | 分块表达式在块级化简，避免整块稠密展开 | 分块是教学与优化的第一公民 |
| SymPy Kronecker | `KroneckerProduct` 惰性 + `combine_kronecker` 规则 | Kronecker **绝不能**默认 `expand`（维数爆炸） |
| SymPy 矩阵求导 | 经 array expr 再收回 MatrixExpr；闭式导数 | v1 **不做**；记入远期 |
| **Symbolics.jl** | 「数组的符号」vs「符号数组」：后者 O(1) 表示整阵，保留 `A*B` 不展开 | 与 MatrixExpr 同构；强调 **shape / eltype 元数据** |
| NumPy / Eigen / nalgebra | 数值热路径，无公式树 | 对应 Marqdo **插件数值层**，不是文档作者主路径 |

**结论：** 「公式类线性代数」= **抽象表达式 DAG + 形状检查 + 重写规则**；稠密数值是 `eval` / `as_explicit` 的终点，不是默认形态。

### 2.2 教学与展示（效果最好的做法）

| 做法 | 来源 | 效果 | Marqdo 映射 |
|------|------|------|-------------|
| KaTeX / MathJax 渲染 `bmatrix` / `pmatrix` | SymPy `latex()` + Jupyter `init_printing` | 公式即排版；多式用 `display` 并列 | 既有 view `formula-card` + KaTeX |
| 标签化等式 `A = UΣV^\top` | 教材 / Qiita display+Math | 步骤可读 | `show` / writeback 多卡片 |
| **结构图**（SVD / QR / 消元轨） | [LAFigureSpecs](https://github.com/ea42gh/LAFigureSpecs) + matrixlayout；TeX `drawmatrix` | 比数字表更能讲「秩 / 零空间 / 对角结构」 | SVG `kind=svd|qr|ge|eig` |
| 热力图 / Hinton | Qiskit / QuTiP / 既有 quantum | 稠密数值矩阵的幅度/符号 | `kind=heatmap|hinton`（经典实/复） |
| 分块着色矩形 | CTAN `drawmatrix` | 零块 / 三角 / 带状一目了然 | `kind=structure`（形状示意图，可无元素值） |
| 仅打印 ASCII | CLI 默认 | 金样可断言 | `str` / `print` → 规范 ASCII / LaTeX 源 |

**展示优先级（锁定建议）：**

1. **符号式**：始终可 `latex` → view KaTeX（不依赖数值）  
2. **结构式 SVG**：分解 / 分块 / 秩示意（教学主图）  
3. **数值热力图**：仅在 `eval` 后、维数可控时  
4. CLI：短 ASCII；大矩阵默认摘要 `m×n …` + 可选 `preview=`

### 2.3 一门「够用」的线性代数库需要什么

按大学一学期 + 数据科学入门，能力桶：

| 桶 | 内容 | 符号层 | 数值层 |
|----|------|--------|--------|
| A 构造 | 符号阵、常数阵、单位/零/对角、从 `$$` / 表 / list | ✅ | ✅ |
| B 代数 | 加减乘、转置、逆、伴随、幂、Kronecker、分块 | ✅ 重写 | ✅ |
| C 分解 | LU / QR / 特征 / SVD / Cholesky（对称） | 结果对象 + 结构图 | ✅ |
| D 系统 | `Ax=b`、最小二乘、伪逆 | 公式写法 + 数值解 | ✅ |
| E 度量 | 行列式、迹、秩、范数、条件数 | 部分可符号 | ✅ |
| F 展示 | latex / ASCII / SVG 结构 / heatmap | ✅ | ✅ |
| G 微积分 | 矩阵导数、Jacobian 积木 | 远期 | — |

**不做（首版）：** 稀疏专用格式、GPU、分布式、完整 CAS 证明、通用张量网络、与 `lib/math` 高中标量混成一个巨模块。

---

## 3. 与现有 Marqdo 的衔接点

| 已有 | 用法 | 扩展时注意 |
|------|------|------------|
| `$$` + Matrix 数值字面量 | 小门矩阵、手算核对 | 保留；符号阵用 `MatrixSymbol` 或 `matrix.symbol`，勿与数值 Matrix 混为一谈 |
| `lib/math` formula | 标量 simplify/diff/plot | **不**把大学线性代数塞进 `lib/math`（与 Q7 结论一致） |
| view KaTeX | formula-card | 符号 MatExpr 的 `latex` 字符串直接喂 KaTeX |
| `ext/quantum` Q7 linop | 量子密度 / 纠缠 | **经典 API 不复用 quantum 命名空间**；共享算法可在插件内私有复用，作者面分离 |
| `plugins/*` ABI | 重数值 / SVG | 稠密分解与大图走 `plugins/linalg` |
| GFM 表 | 数据即代码 | 矩阵输入：表或 `$$`；分解步骤：`@` 记录表 |

---

## 4. 「大规模优化」具体指什么

在公式类语境下，优化 ≠ BLAS 微内核，而是：

1. **惰性**：`A*B*C` 保持乘积树，直到 `eval` / `explicit`  
2. **形状短路**：不相容维数在符号阶段报错，不跑数值  
3. **代数恒等**：`(A^{-1})^{-1}→A`，`(AB)^\top→B^\top A^\top`，`I*A→A`，分块 `block_collapse`  
4. **禁止爆炸展开**：Kronecker / 高维符号积默认不 `expand`  
5. **数值批处理**：同形状多 RHS、分解一次多用（`factorize` 对象）  
6. **展示成本**：大矩阵不默认内嵌完整 KaTeX 数字阵；结构图 + 摘要

---

## 5. 推荐产品叙事（一句）

**线性代数文档用公式写变换，用规则做化简，用结构图讲分解，用可控数值做验算——公式是一等公民，稠密矩阵是可选终点。**

---

## 6. 参考链接

- SymPy Matrix Expressions: https://docs.sympy.org/latest/modules/matrices/expressions.html  
- SymPy Matrix derivatives (远期): https://docs.sympy.org/dev/explanation/modules/matrices/matrixderivatives.html  
- Symbolics.jl Symbolic Arrays: https://docs.sciml.ai/Symbolics/v6.54/manual/arrays/  
- LAFigureSpecs（教学结构图）: https://github.com/ea42gh/LAFigureSpecs  
- CTAN drawmatrix: 矩阵形状示意  
- 既有：`doc/design/stdlib-math.md` · `doc/design/ext-quantum-q7.md`
