# `ext/linalg` — 公式优先的线性代数扩展

| | |
|---|---|
| 状态 | **Draft · L0–L6 + 中缀 v1 · 公式文档面见专文** |
| 日期 | 2026-09-07 |
| 调研 | [research/ext-linalg-formula.md](../research/ext-linalg-formula.md) |
| **公式文档面** | [ext-linalg-formula-doc.md](ext-linalg-formula-doc.md)（跑通即正确 · 代数面/计算面） |
| 路线图 | [roadmap/ext-linalg.md](../roadmap/ext-linalg.md) |
| 相关 | [stdlib-math.md](stdlib-math.md) · [ext-abi.md](ext-abi.md) · [ext-cli.md](ext-cli.md) · [view.md](view.md) · [ext-quantum-q7.md](ext-quantum-q7.md) |
| 安装（规划） | `marqdo ext add linalg`（`linalg` / `线性代数`） |

---

## 0. 一句话

**经典线性代数以「矩阵公式」为一等公民**：抽象表达式可化简、可展示、可选求值；稠密数值与分解图走 ABI 插件——**不**并入 `lib/math`，**不**挂在 `ext/quantum` 作者面。

作者面北极星见 **[ext-linalg-formula-doc.md](ext-linalg-formula-doc.md)**：跑通即正确；代数面像讲义，计算面是显式步骤。

---

## 1. 目标与非目标

### 1.1 目标

1. **公式类扩展**：在现有标量 `formula` 之上，引入 **矩阵表达式（MatExpr）**——`MatrixSymbol`、乘加、转置、逆、迹、行列式、分块、Kronecker——默认**不展开**。  
2. **大规模优化**：形状检查 + 重写规则 + 惰性求值 + 分解对象复用（见 §5）。  
3. **易于展示**：KaTeX 公式卡、结构 SVG（SVD/QR/消元/特征）、数值热力图；CLI 可断言的 ASCII。  
4. **文档即实验**：GFM 表声明矩阵 / 步骤；`.mq.md` 可读可跑。

### 1.2 非目标（首版）

| 不做 | 原因 |
|------|------|
| 塞进 `lib/math` | 高中标量范围已锁定；大学 LA 属官方 ext |
| 复用 `quantum.*` 作者 API | 领域混淆；量子 Q7 保持量子语义 |
| 完整 SymPy / Mathematica | 规则表可控子集即可 |
| 稀疏格式 / GPU / 分布式 | 超出文档语言定位 |
| 矩阵微积分闭式（Jacobian 积木） | 记入路线图远期 |
| 在 `ext/**` 调 `host_*` | 硬约束；数值走插件名 |

---

## 2. 分层架构

```text
作者 .mq.md
    │  import la:ext/linalg/linalg.mq.md
    ▼
L1 官方 ext（GFM + ## 包装，中英分文件）
    │  plugin.load name=linalg
    ▼
plugins/linalg（ABI v2）
    ├── matexpr：符号树构造 / 重写 / latex
    ├── dense：稠密 f64（可选复数）运算与分解
    └── draw：结构 SVG / heatmap
    │
    ▼（可选）
核心 Value::Formula 仅保留「数值矩阵字面量」现状；
符号 MatExpr 以 **map `_type=matrix`（中文构造为 `矩阵`）** 承载，避免撑爆高中 formula AST；遗留标签 `linalg_expr` / `linalg_dense` 仍可解析。
```

### 2.1 为何 MatExpr 不直接塞进 `formula::Expr`

| 方案 | 利 | 弊 | 决策 |
|------|----|----|------|
| 扩展 `formula::Expr` | 与 `$$` 一体 | 核心膨胀；高中/大学边界糊 | ❌ |
| 插件侧表达式 + L1 对象 | 核心干净；可独立发版 | 需稳定 `_type` 与 latex 桥 | ✅ **采用** |
| 仅 list-of-list 数值 | 简单 | 无法公式优化与符号展示 | ❌ 仅作 `explicit` 结果 |

**既有** `formula::Expr::Matrix`（数值 `$$`）继续服务量子门 / 小例子；`ext/linalg` 通过 `la.matrix` / `la.from_formula` 吸收为稠密叶子。

### 2.2 与量子的边界

| | `ext/linalg` | `ext/quantum` Q7 |
|--|--------------|------------------|
| 域 | 经典实/复矩阵 | 态 / 密度 / Pauli |
| Kronecker | 经典 ⊗ | 复合量子系统 |
| 图 | SVD/QR/GE/heatmap | hinton/city/qsphere/… |
| 复用 | 插件**私有**可共享 Jacobi/SVD 代码 | 作者面 API **分离** |

---

## 3. 运行时对象（锁定草案）

全部经插件构造，L1 返回 **map**（`_type` 稳定，金样可断言）：

| `_type`（英） | 中文标签 | 字段（最小） | 含义 |
|---------------|----------|--------------|------|
| `linalg_expr` | `线性式` | `repr`（S-expr 或内部 id）、`shape`=`[m,n]` 或符号维、`latex`、`ascii` | 抽象矩阵公式 |
| `linalg_dense` | `稠密阵` | `rows`, `cols`, `data`（行主 list[list]）、`dtype`=`real`\|`complex` | 显式数值 |
| `linalg_factor` | `分解` | `kind`=`lu`\|`qr`\|`svd`\|`eig`\|`chol`，因子字段，`latex`，可选 `svg` | 分解结果 |
| `linalg_svg` | — | `kind`, `svg` 文本 | 与 quantum_svg 同构展示约定 |
| `linalg_solve` | `求解` | `x`, `residual`, `method` | 线性系 / 最小二乘 |

形状：`shape` 允许整数或符号维名（字符串 `"n"`）；不相容 → 硬错误。

---

## 4. 作者面 API（英文 `ext/linalg/linalg.mq.md`）

中文镜像：`ext/linalg/线性代数.mq.md`（`矩阵` / `化简` / `求值` / `绘图`…）。

### 4.1 构造

```markdown
# matrix

## symbol
    + `name`
    + `rows`
    + `cols`
# → linalg_expr  MatrixSymbol

## eye / zeros / ones / diag
    + …

## from_list
    + `data`          # list[list] 或 table

## from_formula
    + `formula`       # 吸收 Value::Formula Matrix 字面量

## block
    + `blocks`        # 表或嵌套 list of expr
```

文档写法偏好：

````markdown
`A` = > la.symbol name=A rows=n cols=n

`b` =
$$
\begin{pmatrix}1\\0\\0\end{pmatrix}
$$

*`b` = > la.from_formula formula=`b` *
````

作者面优先顶层 `la.symbol` / `la.mul`（`la.matrix.symbol` 无接收者会硬错误，见 [module-namespace.md](module-namespace.md) §4.3）。
### 4.2 公式代数（返回 `linalg_expr`，惰性）

| 英文 | 中文 | 语义 |
|------|------|------|
| `mul` / `add` / `sub` | `乘` / `加` / `减` | 形状检查后建树 |
| `transpose` / `T` | `转置` | |
| `inv` | `逆` | 符号逆；不立即求数值 |
| `pow` | `幂` | 整数幂 |
| `trace` / `det` | `迹` / `行列式` | 可得到标量 expr 或数值 |
| `kron` | `克罗内克` | **禁止**默认 expand |
| `simplify` | `化简` | 应用 §5 规则集 |
| `collapse` | `分块化简` | 对标 `block_collapse` |
| `latex` / `ascii` | `乳胶` / `文本` | 展示字符串 |
| `shape` | `形状` | |

链式 / 中缀（v1 已落地）：

```markdown
*`C` = `A` + `B`*
*`P` = `A` * `B`*
*`Pt` = > `P`.T*
*`s` = > `Pt`.simplify*
```

运行时对双方为 `matrix` / `矩阵` / 数值 `$$` 矩阵 / 遗留 `linalg_expr` 的 `+` `-` `*` 分派到插件；结果保留左操作数 `_type` 以便中英方法链。
### 4.3 求值与分解（热路径 → 稠密）

| 英文 | 说明 |
|------|------|
| `eval` / `explicit` | 符号树 + 绑定表 → `linalg_dense`；超维硬错误 |
| `factorize` | `kind=lu\|qr\|svd\|eig\|chol` → `linalg_factor` |
| `solve` | `A`,`b`；或 `factor` + `b` |
| `lstsq` | 最小二乘 |
| `rank` / `norm` / `cond` | 数值度量 |
| `matmul` | 稠密乘（已 explicit 时） |

维数上限（草案，可配置）：

| 操作 | 默认上限 |
|------|----------|
| `explicit` 稠密元素总数 | \(n\cdot m \le 10^4\)（如 100×100） |
| SVD / eig 方阵边长 | \(\le 64\)（与 quantum 密度同级） |
| 结构 SVG（无元素） | 边长示意可达数百（只画框） |
| KaTeX 完整数字阵 | \(\le 8\times 8\)；更大只出摘要 + 结构图 |

### 4.4 展示

| 英文 | `kind` | 说明 |
|------|--------|------|
| `show` | — | 写入 view：优先 `latex` 公式卡 |
| `draw` | `structure` | 形状/分块矩形（drawmatrix 风格） |
| `draw` | `heatmap` / `hinton` | 数值幅度 |
| `draw` | `svd` / `qr` / `ge` / `eig` | 分解结构图（对标 LAFigureSpecs） |
| `draw` | `path`= | 可选落盘 SVG |

`show` 应对符号式**始终可用**（无需 `eval`）。

### 4.5 最小可运行示例（设计验收叙事）

````markdown
---
import la:ext/linalg/linalg.mq.md
import table:lib/table.mq.md
---

# main

*`A` = > la.symbol name=A rows=3 cols=3 *
*`x` = > la.symbol name=x rows=3 cols=1 *
*`b` = > la.mul a=`A` b=`x` *

*`s` = > la.simplify expr=`b` *
> la.show expr=`s`

`M` =
$$
\begin{bmatrix}2&0\\0&3\end{bmatrix}
$$

*`M` = > la.from_list data=[[2,0],[0,3]] *
*`f` = > la.factorize matrix=`M` kind=eig *
> la.draw factor=`f` kind=eig path=eig.svg
````
---

## 5. 公式重写与「大规模优化」（核心）

### 5.1 原则

1. **构造便宜，展开昂贵**——乘加只分配节点。  
2. **形状是类型系统**——`(m×k)·(k×n)` 合法；否则错误指向调用点。  
3. **化简是规则不动点**——有限规则表，可测、可文档化。  
4. **expand 显式开启**——`expand=True` 才对 Kronecker / 多项式矩阵做爆炸性展开。  
5. **数值路径旁路符号**——已 `explicit` 的叶子用稠密核，不再走规则。

### 5.2 v1 规则集（最小够用）

| ID | 规则 | 例 |
|----|------|-----|
| R1 | 单位元 | `I*A → A`，`A*I → A`，`A+0 → A` |
| R2 | 双逆 | `(A^{-1})^{-1} → A` |
| R3 | 转置反序 | `(AB)^\top → B^\top A^\top`，`(A^\top)^\top → A` |
| R4 | 逆转置 | `(A^{-1})^\top → (A^\top)^{-1}` |
| R5 | 结合扁平 | `MatMul` 链扁平化 |
| R6 | 分块塌缩 | 对标 `block_collapse`（乘/加在块级分配） |
| R7 | Kronecker 结合 | `(A⊗B)⊗C → A⊗B⊗C`；**不**展开成元素 |
| R8 | 标量提出 | `c*(A+B) → c*A+c*B`（可选；默认关以免树变宽） |

### 5.3 数值侧优化

| 策略 | 说明 |
|------|------|
| `factorize` 一次 | 多 RHS `solve` 复用 |
| 小阵专用核 | 2×2 / 3×3 闭式逆 / det（教学友好、可对照公式） |
| 复杂度护栏 | 超限错误信息建议「保持符号 / 降维 / 分块」 |
| 禁止静默降精度 | 病态 `cond` 可警告（stderr 或结果字段） |

---

## 6. 展示设计（效果）

### 6.1 通道

| 通道 | 符号 MatExpr | 稠密 | 分解 |
|------|--------------|------|------|
| view Structure | KaTeX `latex` 卡 | 小阵 KaTeX；大阵摘要 | 公式 + 可选 SVG |
| view Execution plots | — | heatmap SVG | `kind=svd|…` SVG |
| CLI `print` | `ascii` | `[[…],…]` 或摘要 | `svd(U,S,V)` 摘要 |
| writeback | 可选记录 latex | 可选 | 可选 |

### 6.2 LaTeX 约定

- 符号：`A`，`A^{\top}`，`A^{-1}`，`AB`，`A \otimes B`  
- 数值小阵：`\begin{bmatrix}…\end{bmatrix}`  
- 分块：`\begin{bmatrix} A & B \\ C & D \end{bmatrix}`  
- 喂给既有 view：与 `formula-card` / KaTeX 同源路径（L1 `show` → `host_query("record_plot")` 或扩展「record_math」若已有；无则复用 writeback + plot 列表）

### 6.3 结构 SVG（教学主视觉）

对标 LAFigureSpecs / drawmatrix，**不**依赖 Matplotlib：

| `kind` | 画什么 |
|--------|--------|
| `structure` | 矩形块 + 标签（零块灰色） |
| `svd` | \(U\Sigma V^{\top}\) 块布局 + 秩分隔 |
| `qr` | 瘦/胖 QR 示意 |
| `ge` | 消元后上三角高亮 |
| `eig` | \(A=PDP^{-1}\) 或谱列表 + 对角 |
| `heatmap` | 单元填色（实数蓝红；复数幅值） |

主题：`theme=light|dark|bw`（与 quantum Q8 令牌对齐；默认 `light`）。

---

## 7. L1 / 插件 / CLI 约定

1. **`ext/linalg/**` 禁止 `host_*`**；只调 `plugin.*` 注册名（如 `linalg_simplify`）。  
2. **中英分文件**，CATALOG 双 id：`linalg` / `线性代数`。  
3. **`marqdo ext add linalg`**：与 web/agent/quantum 相同——优先 Release 预编译 native zip。  
4. **金样**：离线、确定性；SVG 断言含关键子串或稳定 hash 前缀。  
5. **依赖**：插件内自研稠密核（或慎重引入单一 crate，如 `nalgebra`——**实现期再议**；设计默认「可自研小维 + 可选 crate」）。

---

## 8. 分期（摘要；细节见路线图）

| 阶段 | 交付 |
|------|------|
| **L0** | CATALOG + ping + L1 `ensure_plugin` |
| **L1** | `matrix.symbol` / dense 构造 / `mul`·`T`·`inv` 树 + `simplify` R1–R5 + `latex`/`show` |
| **L2** | `explicit` / 小阵运算 / `solve` / `det`·`trace` 数值 |
| **L3** | 分块 + `collapse`；Kronecker 惰性 |
| **L4** | `factorize` LU/QR/SVD/eig + 结构 SVG |
| **L5** | heatmap/hinton；examples + public 文档 + skill 摘要 |
| **L6+** | 矩阵微积分 / 更大 BLAS（仅当用户明确要求）；复数全路径算子 |

---

## 9. 验收标准（设计级）

1. 符号式 `(AB)^\top` 经 `simplify` 得 `B^\top A^\top`，**不**产生稠密数据。  
2. 不相容形状在符号 `mul` 即失败。  
3. `$$` 数值阵 → `from_formula` → `factorize kind=eig` → view 含 SVG 或 latex。  
4. Kronecker 默认不 expand；显式 `expand` 受维数护栏。  
5. 中英 L1 金样各至少一条；`ext add linalg` 路径文档化。  
6. 与 `ext/quantum` 无作者面符号冲突。

---

## 10. 风险与开放问题

| 项 | 备注 |
|----|------|
| MatExpr 序列化 | 金样 / write：倾向可打印 S-expr 或稳定 latex+shape，避免不透明指针跨进程 |
| 与标量 `formula` 混运算 | v1：标量只作 `c*A` 系数；`det` 结果若符号则另型或降为 `formula` 文本 |
| nalgebra 等依赖 | 实现 L2 时评估二进制体积与 ABI 体积 |
| record_math vs plot | 若仅 SVG/KaTeX 字符串，可先复用 plot/writeback |

---

## 11. 一句话（收束）

**`ext/linalg` 让线性代数公式保持可优化的表达式，用规则与结构图服务文档阅读，用受控稠密核服务验算——公式优先，数值可选，展示分层。**
