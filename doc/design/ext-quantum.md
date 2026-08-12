# 官方扩展：`ext/quantum` 量子计算模拟

| | |
|---|---|
| 状态 | **Accepted · Q0–Q4 已落地**（概率/布洛赫/`matches_matrix`/用户文档；Q5+ 见路线图） |
| 日期 | 2026-08-12 |
| 相关 | [stdlib-math.md](stdlib-math.md) · [ext-abi.md](ext-abi.md) · [ext-cli.md](ext-cli.md) · [ext-web.md](ext-web.md) · [view.md](view.md) · [stdlib-i18n.md](stdlib-i18n.md) · [roadmap/ext-quantum.md](../roadmap/ext-quantum.md) |
| 安装（目标） | `marqdo ext add quantum`（中英：`quantum` / `量子`） |
| 本文目的 | 锁定**作者面**、**模拟语义**、**view 可视化**与**分期**；指导 ABI 插件实现 |

---

## 0. 一句话

`ext/quantum` 用 **GFM 表格 + 类方法 +（可选）`$$` 狄拉克/矩阵公式** 描述量子门与电路；**经典状态向量模拟**给出概率与采样。  
文档能跑通，公式与电路约定就成立——这是 Marqdo「代码即文档」在量子教学上的最强展示位。

**不改 Marqdo 核心语法**；热路径只在 ABI 插件；**禁止**把量子核链进 `marqdo` 主二进制或在 `ext/**` 里调用 `host_*`。

---

## 1. 为何做量子扩展（与 math 的关系）

### 1.1 与 `lib/math` 的相似处

| math | quantum |
|------|---------|
| `` `f` = `` + `$$…$$` → `formula` | 门矩阵 / 态矢量可用 `$$` **展示与核对** |
| 库函数对公式求值、求导、作图 | 类方法对电路施门、模拟、画图 |
| view 内嵌 SVG；CLI 可 `path=` | view 内嵌**电路图 / 概率条 / 布洛赫**；CLI 可落盘 |
| 金样例失败 = 文档/公式错 | 金样例失败 = 门定义或电路叙述错 |

math 是**高中符号与曲线**；quantum 是**离散希尔伯特空间上的可执行电路文档**。两者都是「写出公式 → 跑通 → 证明文档正确」。

### 1.2 与 math 的刻意分界

| 做 | 不做 |
|----|------|
| 量子在 **`ext/quantum`**，经 `plugin.load` | **不**并进 `lib/math`（避免 stdlib 膨胀） |
| 复数振幅用 `{re, im}` map（与现有 `Value` 兼容） | **不**为量子在核心新增 `Value::Complex`（可二期） |
| 角度可用 `math.pi` 等常量 | **不**在 quantum 插件内重写 CAS |

### 1.3 「代码即文档」在这里的作用

典型教学页：

```markdown
# 贝尔态 |Φ+⟩

对 qubit 0 施 H，再 CNOT(0→1)。理想测量：`00` 与 `11` 各约 1/2。

# main

`steps` =

| 步 | 门 | 比特 |
|----|----|------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

*`qc` = > quantum.circuit qubits=2 steps=`steps` *
*`p` = > `qc`.probabilities *
```

- 叙述与表格是**给人看的电路说明**；  
- 同一文件是**可执行程序**；  
- gold / CI 断言概率 → **文档与物理约定同步**，无需另维护「幻灯片公式」。

---

## 2. 硬约束（评审锁定）

1. **仅 ABI 插件接入**：`plugins/quantum` → `libquantum.so`；L1 在 `ext/quantum/*.mq.md`。  
2. **`ext/quantum/**` 禁止 `host_*`**；经 `plugin.native_path` + `plugin.load`（同 web/agent）。  
3. **中英分文件**：`quantum.mq.md` / `量子.mq.md`，**禁止**混排 API（[stdlib-i18n.md](stdlib-i18n.md)）。  
4. **不改核心语法**；电路表单元格保持字面量；门名/比特下标由类方法解析。  
5. **模拟器有硬上限**（默认 ≤ 12 qubits）；超限报错，不静默截断。  
6. **view 必须有专项展示**（结构卡 + 执行区 SVG），禁止只 dump JSON。

---

## 3. 布局与安装

```text
ext/quantum/
  quantum.mq.md          # 英文类 API
  量子.mq.md             # 中文类 API
plugins/quantum/
  Cargo.toml
  src/lib.rs             # ABI v2：线性代数 / 施门 / 画 SVG
examples/quantum-bell/   # 规范示例（Q3）
tests/ext/quantum-*.mq.md
```

```bash
cargo build -p marqdo_plugin_quantum
marqdo ext add quantum
```

导入：

```markdown
---
> ext/quantum/quantum.mq.md
---
```

或中文：`> ext/quantum/量子.mq.md`。

---

## 4. 核心模型

### 4.1 运行时句柄（JSON map + `_type`）

| `_type`（英） | 中文类名 | 含义 |
|---------------|----------|------|
| `quantum_gate` | `量子门` | 具名门或自定义酉矩阵 |
| `quantum_circuit` | `量子电路` | n qubits + 有序门列表 |
| `quantum_state` | `量子态` | 状态向量（振幅列表） |
| `quantum_result` | `量子结果` | 概率 / 采样 / 可选坍缩态 |
| `quantum_svg` | （产物） | 带 `svg` 字段的可视化结果 |

插件返回 map；Marqdo 对象系统挂 `_type` 后可用方法链。

### 4.2 状态表示

- **纯态**状态向量：长度 `2^n`，每元 `{re, im}`（f64）。  
- 默认初态：`|0…0⟩`（仅 index 0 振幅为 1）。  
- 门作用后可选重归一化（阈值误差阈值，默认开）。

### 4.3 公式面（对齐 math 精神，不抢 math 实现）

允许用 `` `G` = `` + `$$…$$` 书写门矩阵或态（**展示 + 教学核对**）：

````markdown
`H_matrix` =
$$
\frac{1}{\sqrt{2}}\begin{pmatrix}1&1\\1&-1\end{pmatrix}
$$

*`H` = > quantum.gate name=H *
*`ok` = > `H`.matches_matrix matrix=`H_matrix` *
````

- v1：**具名门**以插件内置酉矩阵为准；`matches_matrix` 为可选数值核对（容差）。  
- 叙述区无名 `$$` 仍只是注释（同 [stdlib-math.md](stdlib-math.md) §2.4）。  
- **不**要求核心 `Value::Formula` 理解量子；矩阵核对可走「公式 ASCII/LaTeX → 插件解析子集」或作者显式 `matrix=` 表。

**v1 优先路径：** 电路用**表 + 方法**；`$$` 作文档展示；矩阵核对可放 **Q3**。

---

## 5. 作者面 API（锁定草图）

### 5.1 英文 `ext/quantum/quantum.mq.md`

```markdown
# gate
    + `name`=None          # H X Y Z S T I CX CZ SWAP …
    + `theta`=None         # Rx/Ry/Rz

## matrix                 # → 振幅表 / 嵌套 list（文档与调试）
## draw                   # → quantum_svg（单门符号）
## matches_matrix
    + `matrix`
    + `tol`=1e-9

# circuit
    + `qubits`
    + `steps`=None         # 可选：GFM 门序列表

## h / x / y / z / s / t
    + `qubit`
## rx / ry / rz
    + `qubit`
    + `theta`
## cx / cz
    + `control`
    + `target`
## swap
    + `a`
    + `b`
## barrier                 # 仅可视化分隔，不改变态
## measure                 # 标记测量（模拟时按计算基）
    + `qubits`=None        # 默认全部
## append                  # 追加另一电路或门
    + `op`
## simulate                # → quantum_state（不坍缩）
## probabilities           # → map 基矢标签 → 概率
## run
    + `shots`=1024
    + `seed`=None          # → quantum_result（计数直方图）
## draw
    + `path`=None          # SVG；无 path 时仍可进 view 产物
## state                   # 当前缓存态（若已 simulate）
```

门方法返回**更新后的 circuit 句柄**（与 web `compose_*` 相同：`` `qc` = > `qc`.h qubit=0 *``）。

### 5.2 中文 `ext/quantum/量子.mq.md`

| 英文 | 中文 |
|------|------|
| `gate` | `门` |
| `circuit` | `电路` |
| `h` | `哈达玛` |
| `x` / `y` / `z` | `泡利X` / `泡利Y` / `泡利Z` |
| `cx` | `控非` |
| `cz` | `控相` |
| `rx`/`ry`/`rz` | `绕X`/`绕Y`/`绕Z` |
| `simulate` | `模拟` |
| `probabilities` | `概率` |
| `run` | `运行` |
| `draw` | `绘图` |
| `measure` | `测量` |
| `qubits` | `比特数` |
| `steps` | `步骤` |
| `shots` / `seed` | `次数` / `种子` |

插件 FFI 名稳定英文：`quantum_circuit_new`、`quantum_h`、`quantum_cx`、`quantum_simulate`、`quantum_draw_circuit`…

### 5.3 电路表（文档优先写法）

```markdown
`steps` =

| 步 | 门 | 比特 | 参数 |
|----|----|------|------|
| 1 | H | 0 | |
| 2 | CX | 0,1 | |
| 3 | RZ | 1 | pi/4 |
```

列名中英兼容：`步|step`、`门|gate`、`比特|qubits|qubit`、`参数|theta|params`。

*`qc` = > quantum.circuit qubits=2 steps=`steps` *` 与方法链**等价**；表更适合「整页即电路说明书」。

### 5.4 返回值形态

| 方法 | 返回 |
|------|------|
| `simulate` | `{_type, qubits, dim, amplitudes:[{re,im},…]}` |
| `probabilities` | `{ "00": 0.5, "11": 0.5 }`（标签为小端比特串，文档写死） |
| `run` | `{ counts: {…}, shots, seed }` |
| `draw` | `{_type: quantum_svg, kind: circuit\|gate\|probs\|bloch, svg: "…" }` |

---

## 6. 模拟器语义

| 项 | 锁定 |
|----|------|
| 模型 | 封闭系统纯态；计算基测量 |
| 默认上限 | **12 qubits**；`MARQDO_QUANTUM_MAX_QUBITS` 可下调（不可暗中上调超过编译期硬顶，硬顶建议 16） |
| 超限 | 错误信息含当前 n 与上限 |
| `simulate` / `probabilities` | **无 RNG**、确定性 |
| `run` | 多项式采样；`seed=` 可复现（金样必用） |
| 数值 | f64；门后 ‖ψ‖ 偏离阈值则重归一化并可选 warning 字段 |

实现：插件内 Kronecker / 按比特作用稀疏更新；可用 `nalgebra` 或手写，**不**捆绑外部量子云。

---

## 7. view 模式支持（专项）

对齐 [view.md](view.md) 与 math 作图：**Structure / Execution / Source 共用渲染**；禁止第二套皮肤。

### 7.1 Structure（结构区）

| 源形态 | 展示 |
|--------|------|
| `` `qc` = `` + 电路表 | **circuit-card**：比特数、门序列时间线（文本/紧凑 SVG） |
| `` `G` = `` + `$$` 门矩阵 | 沿用 **formula-card**（KaTeX）；徽章可标 `gate`（若绑定来自 quantum.gate） |
| 方法链语句 | 普通调用行；不强制每步展开 |

### 7.2 Execution（执行区）

| 产物 | 展示 |
|------|------|
| `quantum_svg` | 与 math `plots` 同级：**内嵌 SVG**（电路轨线图 / 概率直方图 / 单比特布洛赫） |
| `probabilities` / `counts` | 表格 + 可选自动条形 SVG |
| `quantum_state` | 默认折叠振幅表；小 n（≤3）可并排显示基矢 |

### 7.3 产物如何进入 view（实现锁定）

插件**不能**直接碰核心 `HostContext.plots`。采用其一（实现时二选一，优先 A）：

| 方案 | 做法 |
|------|------|
| **A（推荐）** | ABI `host_query("record_plot")`：参数 JSON `{svg}`；写入与 math 相同的 plots 列表（[ext-abi.md](ext-abi.md) allowlist 增补一条） |
| **B** | 仅返回 `quantum_svg`；view 在渲染执行值时识别 `_type`/`svg` 字段并内嵌（不扩 ABI） |

CLI：`draw path=bell.svg` 写沙箱文件；无 path 时不把整段 SVG 灌 stdout（同 math §5）。

### 7.4 可视化内容（v1 必须 / 可后）

| 图 | v1 | 说明 |
|----|----|------|
| 电路轨线（qubit 线 + 门盒 + 控点） | **必须** | `circuit.draw` |
| 测量概率条形图 | **必须** | `probabilities` / `run` 后可 `draw kind=probs` |
| 单比特布洛赫球（SVG） | 应有 | 仅 n=1 或指定 qubit 约化（纯态投影） |
| 门矩阵热力图 | 可后 | Q4 |
| 交互拖拽编辑电路 | **不做** | 静态文档足够 |

视觉：内联 CSS；**无外链**字体/CDN（同 view 硬规则）。

---

## 8. 分期（开发计划）

```text
Q0  脚手架：CATALOG + plugins/quantum Hello + L1 ensure_plugin
Q1  态向量 + I/X/H/CX + simulate/probabilities；贝尔态 gold
Q2  Y/Z/S/T/Rx/Ry/Rz/CZ/SWAP + run(seed) + 中英 API 对齐
Q3  电路表 steps= + draw 电路 SVG + view record_plot/识别；examples/
Q4  概率/布洛赫图；matches_matrix；文档站 public/features   ← **done**
Q5  （可选，另文）噪声/密度矩阵；>12 qubits 张量网络——不挡 Accepted
```

| 阶段 | 交付物 | 验收 |
|------|--------|------|
| **Q0** | `marqdo_plugin_quantum` 注册 `quantum_ping`；L1 加载失败可读 | `ext add quantum` 路径通 |
| **Q1** | 贝尔态 `probabilities` ≈ 0.5/0.5 | `tests/ext/quantum-bell-smoke.mq.md` |
| **Q2** | 旋转门 + `run seed=` 计数稳定 | 中英双金样 |
| **Q3** | `steps=` 表；`draw` SVG 在 `view` HTML 含 `<svg` | 与 math-plot 同级断言 |
| **Q4** | 概率图 + 布洛赫；`matches_matrix`；用户文档页 | `quantum-draw-smoke` / `quantum-gate-matrix-smoke`；`view output` 可浏览示例 |

实现路线图跟踪见 [roadmap/ext-quantum.md](../roadmap/ext-quantum.md)。

---

## 9. 规范示例（目标作者体验）

```markdown
---
title: Bell state
> ext/quantum/quantum.mq.md
---

# Bell |Φ+⟩

H on qubit 0, then CNOT. Expect |00⟩ and |11⟩ each with probability 1/2.

# main

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

*`qc` = > quantum.circuit qubits=2 steps=`steps` *
*`p` = > `qc`.probabilities *
*`svg` = > `qc`.draw *

> print text=`p`
```

中文面对称：`量子.电路`、`哈达玛`、`控非`、`概率`、`绘图`。

---

## 10. 刻意不做（本 Accepted 范围）

- 真机 / 云后端（IBM、Braket…）  
- 开放系统、完整噪声模型（可 Q5+）  
- 符号电路编译优化器  
- 交互式电路编辑器  
- 把量子并入 `lib/math` 或核心 opcode  
- 无上限比特的「生产级」模拟  

---

## 11. 开放点（已收敛 / 剩余）

| 议题 | 结论 |
|------|------|
| 对象面 vs 自由函数 | **主推对象/方法链**；自由函数不平行提供 |
| 电路表 | **v1 纳入**（Q3），与方法链等价 |
| 振幅过大 | v1 全进 Value；n≤12 可接受；超限已挡 |
| view 产物 | **优先** `host_query("record_plot")` |
| 与 web | **不依赖** web 做可视化；SVG 自绘 |

---

## 12. 评审清单

1. ABI-only + 中英分文件是否认可？  
2. 默认 12 qubits 上限是否合适？  
3. 电路表列设计（步/门/比特/参数）是否够用？  
4. view：电路轨线 + 概率条为 v1 必须，是否同意？  
5. `$$` 矩阵核对放 Q4，是否同意先表驱动？  

评审通过后：本文冻结为 Accepted → 按 §8 从 Q0 实现。

---

## 13. 一句话

**表格写出电路，类方法模拟与绘图；跑通金样即证明文档中的量子公式与步骤成立——math 管曲线，quantum 管希尔伯特空间上的可执行说明书。**
