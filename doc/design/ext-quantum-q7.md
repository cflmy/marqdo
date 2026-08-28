# `ext/quantum` Q7 — 高阶线性代数 + 高级可视化

| | |
|---|---|
| 状态 | **Accepted · Q7a/Q7b/Q7c 已落地** |
| 日期 | 2026-08-28 |
| 父设计 | [ext-quantum.md](ext-quantum.md) |
| 路线图 | [roadmap/ext-quantum.md](../roadmap/ext-quantum.md) |
| 对照 | [IBM Quantum state plots](https://quantum.cloud.ibm.com/docs/en/guides/plot-quantum-states) · [QuTiP visualization](https://qutip.readthedocs.io/en/latest/guide/guide-visualization.html) · Cirq `plot_density_matrix` |

---

## 0. 一句话

在 **Q0–Q6 态向量教学模拟**之上，补齐两块「教科书级」能力：

1. **高阶线性代数**（密度矩阵、Kronecker、部分迹、Hermitian 谱分解、Schmidt / SVD、Pauli 期望值）  
2. **高级静态 SVG 展示**（Hinton / City / Pauli 向量 / QSphere 风格 / 多比特布洛赫）

全部经 **`plugins/quantum` ABI**；`ext/quantum/**` 只做 L1 表包装，**禁止** `host_*`。

---

## 1. 动机与业界对照（联网调研 2026-08）

### 1.1 为何现在做

| 已有（Q0–Q6） | 缺口 |
|---------------|------|
| 纯态态向量 + 施门 + 采样 | 混态 / 约化子系统无法一等公民表达 |
| `bloch` 单比特约化 | 纠缠与多比特关联只能猜 |
| `gate.draw kind=matrix` 热力图 | 无密度矩阵 / Pauli 分解 / 球面振幅布局 |
| 内部 `apply_unitary` / Kronecker | **未**暴露为作者面 API |

教学文档写「贝尔态纠缠」时，读者需要：**约化密度矩阵 → 纯度 < 1 → Schmidt 系数 → Hinton/QSphere 图**。这些正是 Qiskit / QuTiP / Cirq 的标配。

### 1.2 可视化对标（只取可静态 SVG 落地的子集）

| 业界 | 含义 | Q7 决策 |
|------|------|---------|
| Qiskit `plot_state_hinton` | 元大小表示幅值，色表示正负 | ✅ `kind=hinton` |
| Qiskit `plot_state_city` | Re/Im 三维条 | ✅ **2.5D 等轴测 SVG**（非 Matplotlib） |
| Qiskit `plot_state_paulivec` | Pauli 串期望值柱 | ✅ `kind=paulivec` |
| Qiskit `plot_state_qsphere` | 振幅点在球面上，色=相位 | ✅ `kind=qsphere`（教学投影） |
| Qiskit `plot_bloch_multivector` | 每比特一球 | ✅ `kind=multibloch` |
| Cirq density Argand 格 | 每元幅值圆 + 相位针 | ✅ 并入 `kind=density`（默认） |
| QuTiP Wigner / 过程层析 | 连续相空间 / χ 矩阵 | ❌ 本切片不做 |

### 1.3 线性代数对标（经典模拟侧，非量子算法）

| 能力 | 教学用途 | Q7 |
|------|----------|-----|
| ρ = \|ψ⟩⟨ψ\| | 纯→密度 | ✅ |
| Kronecker / 张量积 | 复合系统 | ✅ |
| 部分迹 | 约化、纠缠证据 | ✅ |
| Hermitian 本征分解 | 谱 / 测量可观测量 | ✅（小维） |
| Schmidt（纯态二分）= 整形 SVD | 纠缠熵入口 | ✅ |
| Pauli 期望 ⟨P⟩ | 布洛赫 / paulivec 数据源 | ✅ |
| 纯度 Tr(ρ²)、保真度 | 混态度量 | ✅ 最小 |
| 量子奇异值变换 / VQSVD / 云后端 | 算法研究 | ❌ |

---

## 2. 硬约束（沿用 + 本切片追加）

1. **仅 ABI**：新 FFI 全部注册在 `marqdo_plugin_init`；L1 经 `plugin.load`。  
2. **`ext/**` 无 `host_*`**。  
3. **中英分文件**，禁止混 API。  
4. **维数上限**：  
   - 态向量模拟：仍 ≤ 12 qubits（既有）。  
   - **密度矩阵稠密运算**（部分迹结果展示、谱分解、Hinton/City）：默认 **≤ 6 qubits**（dim ≤ 64）；超限显式报错。  
5. **无新核心依赖**：插件内手写复数矩阵 + Jacobi / 薄 SVD；**不**捆绑 BLAS / nalgebra（可后续再议）。  
6. **view**：仍走 `host_query("record_plot")` + `quantum_svg`；无外链字体/CDN。

---

## 3. 新运行时类型

| `_type`（英） | 中文 | 字段（锁定） |
|---------------|------|----------------|
| `quantum_state` | `量子态` | 既有：`qubits`, `amplitudes:[{re,im}]` |
| `quantum_density` | `密度矩阵` | `qubits`, `dim`, `matrix`（dim×dim 嵌套 `{re,im}`） |
| `quantum_spectrum` | `谱` | `eigenvalues:[f64]`, `eigenvectors`（列向量嵌套 list） |
| `quantum_schmidt` | `施密特` | `coeffs:[f64]`, `ua`, `ub`（局域基），`entropy`（可选） |
| `quantum_svg` | — | 既有；`kind` 扩展见 §5 |

---

## 4. 作者面 API

### 4.1 英文（`ext/quantum/quantum.mq.md`）

在既有 `# circuit` / `# gate` 之外增加（或扩展方法）：

```markdown
# density
    + `state`=None          # quantum_state 或 circuit（先 simulate）
    + `matrix`=None         # 直接给定 ρ

## matrix                   # → 嵌套 list
## purity                   # Tr(ρ²)
## expect
    + `obs`                 # Pauli 串 "ZI" / "XX" 或门矩阵
## partial_trace
    + `keep`                # 保留的比特列表（LSB=0）
## eig                      # Hermitian 谱 → quantum_spectrum
## draw
    + `kind`=hinton         # hinton|city|density|paulivec
    + `path`=None

# linop                     # 自由线性代数（也可挂在 quantum.* 模块函数）
## kron
    + `a`
    + `b`                   # 矩阵或态 → 张量积
## schmidt
    + `state`
    + `cut`=1               # 前 cut 个比特为 A，其余为 B
## fidelity
    + `a`
    + `b`                   # 两态或两密度（纯态公式优先）
```

**电路侧便捷方法**（返回新句柄，不改旧对象）：

```markdown
## density                  # simulate 后 ρ=|ψ⟩⟨ψ|
## expect
    + `obs`
## schmidt
    + `cut`=1
## draw
    + `kind`=…              # 扩展：hinton|city|density|paulivec|qsphere|multibloch
```

### 4.2 中文对称

| 英文 | 中文 |
|------|------|
| `density` | `密度` |
| `partial_trace` | `部分迹` |
| `purity` | `纯度` |
| `expect` | `期望` |
| `eig` | `本征` |
| `kron` | `张量积` |
| `schmidt` | `施密特` |
| `fidelity` | `保真度` |
| `hinton` / `city` / `paulivec` / `qsphere` / `multibloch` | 同音译或 `欣顿`/`城市`/`泡利向量`/`球`/`多布洛赫`（L1 锁定中英别名表） |

### 4.3 ABI 函数名（稳定英文）

| FFI | 作用 |
|-----|------|
| `quantum_density_from_state` | 态 / 电路 → `quantum_density` |
| `quantum_density_from_matrix` | 校验 Hermitian / 维数 → 句柄 |
| `quantum_density_matrix` | 取出嵌套矩阵 |
| `quantum_density_purity` | Tr(ρ²) |
| `quantum_density_partial_trace` | keep 列表 |
| `quantum_density_eig` | Jacobi Hermitian |
| `quantum_density_expect` | Pauli 串或矩阵 |
| `quantum_density_draw` | hinton/city/density/paulivec |
| `quantum_kron` | 两矩阵或两态振幅 |
| `quantum_schmidt` | 纯态二分 |
| `quantum_fidelity` | 两态 |
| `quantum_draw_circuit` | **扩展** kind 枚举（向后兼容） |

---

## 5. 高级绘图语义

| `kind` | 输入 | SVG 要点 |
|--------|------|----------|
| `hinton` | ρ | 网格方块；边长 ∝ √\|ρᵢⱼ\|；正 Re 浅 / 负深；小 dim 可标数值 |
| `city` | ρ | 等轴测「楼宇」：左 Re、右 Im 两城，或单城双色柱 |
| `density` | ρ | Cirq 风格：每格幅值圆 + 相位线段；对角概率矩形 |
| `paulivec` | 态或 ρ | 全 Pauli 基（n≤3）或用户 `ops=` 列表；柱高 = ⟨P⟩ |
| `qsphere` | 纯态 | 基矢点映射到球/圆；半径∝振幅，色相∝arg；n≤4 推荐 |
| `multibloch` | 纯态 | 每比特约化布洛赫并排 |

既有 `circuit|probs|bloch|gate|matrix` **不变**。

---

## 6. 数值语义（锁定）

| 项 | 规则 |
|----|------|
| 比特序 | **qubit 0 = LSB**（与既有概率标签一致） |
| `partial_trace keep=` | 保留比特升序组成新系统的 LSB…MSB |
| Pauli 串 | 左→右 = **高位→低位** 或文档写死「左=qubit n-1」——**锁定：字符串左端 = 最高比特（qubit n−1）**，与 Qiskit 习惯一致；文档 + 金样双锁 |
| Hermitian 检查 | ‖ρ−ρ†‖_F < 1e-8，否则报错 |
| 谱 | 实特征值降序；特征向量列归一 |
| Schmidt | 将 \|ψ⟩ 整形为 `2^cut × 2^(n-cut)` 复矩阵做薄 SVD；奇异值 = Schmidt 系数 |
| 纯度 | 纯态 ≈ 1；最大混态 = 1/dim |

---

## 7. 分期交付

| 子阶段 | 内容 | 验收 |
|--------|------|------|
| **Q7a** | density / kron / partial_trace / eig / schmidt / expect / purity / fidelity + L1 中英 | `tests/ext/quantum-linalg-smoke.mq.md`（+ zh） |
| **Q7b** | draw kinds：hinton, city, density, paulivec, qsphere, multibloch | `tests/ext/quantum-viz-advanced-smoke.mq.md`（SVG 含关键标记） |
| **Q7c** | 示例页 + 用户站短页 + skill/reference 摘要 | `examples/quantum-entanglement/` 或扩展 bell |

---

## 8. 刻意不做（本 Q7）

- 任意 Kraus / 完整 CPTP 通道库（仍用 Q5 轨迹噪声）  
- >6 qubit 稠密 ρ 的谱 / Hinton（请用态向量 API）  
- Matplotlib / WebGL / 交互拖拽  
- 量子线性系统算法（HHL）、QSVT 电路构造  
- 把线性代数并入 `lib/math`

---

## 9. 评审清单

1. 密度矩阵上限 6 qubits 是否可接受？  
2. Pauli 串左右端比特约定是否锁定为「左=高比特」？  
3. City 用 2.5D SVG 而非真 3D 是否足够？  
4. `# density` 独立类 vs 仅 `circuit.density` —— **两者都提供**（类用于给定矩阵；电路方法便捷）？

评审默认：**全部锁定为上文**；实现按 Q7a → Q7b → Q7c。

---

## 10. 一句话

**Q7 让 Marqdo 量子文档能「算清纠缠、画清态」：线性代数在 ABI 里跑，高级图用自绘 SVG 进 view——仍是代码即文档。**
