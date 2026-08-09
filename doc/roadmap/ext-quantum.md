# 官方扩展：量子计算模拟（未来规划）

| | |
|---|---|
| 状态 | **规划 / 未开工** |
| 日期 | 2026-08-09 |
| 相关 | [ext-cli.md](../design/ext-cli.md) · [ext-abi.md](../design/ext-abi.md) · [stdlib-math.md](../design/stdlib-math.md) · [ext-web.md](ext-web.md) |
| 安装草案 | `marqdo ext add quantum`（中英面：`quantum` / `量子`） |

---

## 1. 动机

为教学、算法演示与智能体「可运行的量子电路文档」提供官方扩展：

- 在 `.mq.md` 里**声明量子比特与电路**（可读即文档）  
- 支持**基础量子门**与常见电路写法  
- **经典模拟**量子态在电路下的演化，输出状态向量 / 概率 /（可选）测量采样  
- 重计算进 **ABI 插件**；安装走现有 `marqdo ext` 惯例  

不宣称替代专业量子 SDK（Qiskit / Cirq 等），定位：**Marqdo 文档内可执行的最小量子模拟层**。

---

## 2. 范围

### 2.1 在范围内

| 能力 | 说明 |
|------|------|
| 状态表示 | 纯态状态向量（复数振幅）；qubit 数有明确上限（见 §6） |
| 基础单比特门 | 至少：`I` `X` `Y` `Z` `H` `S` `T` 及 `Rx/Ry/Rz(θ)` |
| 基础多比特门 | 至少：`CNOT`（CX）、`CZ`；可选 `SWAP`、Toffoli（可用分解） |
| 电路构建 | 按序追加门；支持指定目标/控制比特下标 |
| 模拟 | 初态（默认 \|0…0⟩）→ 应用电路 → 返回状态或测量概率 |
| 测量 | 计算基概率；可选按概率采样坍缩（经典 RNG） |
| L1 API | `ext/quantum/quantum.mq.md` · `ext/quantum/量子.mq.md` |
| **ABI 插件** | `plugins/quantum`：线性代数与门作用热路径 |
| **CLI 安装** | `marqdo ext add quantum` / `remove` / `list` |

### 2.2 非目标（本规划）

- 真机 / 云量子后端调度  
- 开放系统密度矩阵、噪声模型（可列二期）  
- 符号电路优化编译器  
- 把量子核链进 `marqdo` 主二进制  
- 无上限比特数的「生产级」模拟器

---

## 3. 布局与安装（对齐惯例）

```text
ext/
  quantum/
    quantum.mq.md
    量子.mq.md
    examples/              # 可选：贝尔态、量子传送草稿
plugins/
  quantum/
    Cargo.toml
    src/lib.rs             # ABI v2
```

```text
marqdo ext add quantum
marqdo ext remove quantum
```

安装：复制 L1 源 + `native/libquantum.so`（等）到 `MARQDO_EXT`，与 [ext-cli.md](../design/ext-cli.md) / agent 插件一致。

导入：

```markdown
---
> ext/quantum/quantum.mq.md
---
```

---

## 4. API 外形（草案）

```markdown
---
> ext/quantum/quantum.mq.md
---

# main

*`qc` = > quantum.circuit qubits=2 *
> `qc`.h qubit=0
> `qc`.cx control=0 target=1
*`state` = > `qc`.simulate *
*`probs` = > `qc`.probabilities *
> print text=`probs`
```

中文面示例：

```markdown
*`电路` = > 量子.电路 比特数=2 *
> `电路`.哈达玛 比特=0
> `电路`.控非 控制=0 目标=1
*`概率` = > `电路`.概率 *
```

门命名中英对照表在落地时写入 L1 文档头；插件注册名建议稳定英文（`quantum_h`、`quantum_cx`、`quantum_simulate`）。

### 4.1 返回值形态

| 方法 | 建议返回 |
|------|----------|
| `simulate` | map：`qubits`、`amplitudes`（复数列表或 `{re,im}` 对）、可选 `dim` |
| `probabilities` | 列表或 map：基态标签 → 概率（如 `"00"` → 0.5） |
| `measure` | 采样得到的比特串 +（可选）坍缩后状态 |

复数如何进 Marqdo `Value`：优先 `{re, im}` map 列表，避免过早引入原生复数类型（若语言日后有复数，再收紧）。

---

## 5. 电路即文档

鼓励把电路写在可读段落旁：

```markdown
# 贝尔态

制备 `|Φ+⟩`：H 后 CNOT。

# main

*`qc` = > quantum.circuit qubits=2 *
…
```

可选：GFM 表描述门序列（与 [tables-maps-footnotes.md](tables-maps-footnotes.md) 横表字典对齐，**不阻塞**本扩展 v1——v1 以方法链为准）。

---

## 6. 模拟器约束

| 项 | 提案 |
|----|------|
| 默认上限 | **≤ 12 qubits**（状态 2^12；可 env `MARQDO_QUANTUM_MAX_QUBITS` 下调） |
| 超限 | 明确错误，不静默截断 |
| 数值 | `f64` 振幅；门后可选重归一化阈值 |
| 确定性 | `simulate` / `probabilities` 无 RNG；`measure` 可传 `seed=` |

实现放在插件内（`nalgebra` / 手写 Kronecker + 稀疏门作用均可）；L1 只做参数校验与结果包装。

---

## 7. ABI 边界

| 插件 | `.mq.md` |
|------|----------|
| 分配/更新状态向量、施门、算概率 | 电路编排、打印、教学叙述 |
| 参数化旋转门 | `theta=` 从 Marqdo 传入 |

遵守：

- ABI v2 + JSON（[ext-abi.md](../design/ext-abi.md)）  
- `ext/quantum/**` **禁止** `host_*`；经 `plugin.load`  
- 与 [module-namespace.md](../design/module-namespace.md) / agent 硬规则一致：域逻辑不进核心 `HostFn`

---

## 8. 推荐落地顺序

```text
Q0  ext-cli 登记 quantum；插件注册 quantum_sim_version；L1 加载失败可读
Q1  n qubits 全零态 + H/X/CNOT + simulate/probabilities；金样贝尔态
Q2  Y/Z/S/T/Rz + measure seed=；中英 API
Q3  examples/ + catalog/OKF 可选 Module 页；文档
Q4  （可选）电路表语法糖；密度矩阵二期另文
```

验收：

1. `marqdo ext add quantum` 后可跑贝尔态金样，概率约 `00/11 ≈ 0.5`。  
2. 无原生插件时错误指引 `ext add` / 构建 `marqdo_plugin_quantum`。  
3. 超比特上限失败信息含上限值。  
4. 双端（若金样走 tree）稳定；不强制 bytecode 同步若无量子 opcode（电路走插件即可）。

---

## 9. 开放点

1. 是否提供无对象的自由函数面（`## h circuit=…`）与对象面并行。  
2. 振幅列表过大时是否写临时文件而非全进 Value。  
3. 与 [stdlib-math.md](../design/stdlib-math.md) 复用：仅角度常数，不把量子并进 math 库。  
4. 教学是否依赖 [ext-web.md](ext-web.md) 做布洛赫球可视化（可选、非阻塞）。

---

## 10. 与其它规划的关系

| 文档 | 关系 |
|------|------|
| [ext-web.md](ext-web.md) | 同属「新域官方 ext + ABI + CLI」模板；可互为示例 |
| [object-inheritance.md](object-inheritance.md) | 可选：`# 变分电路 = > quantum.circuit`（不阻塞 Q1） |
| [agent-streaming.md](agent-streaming.md) | 无直接依赖；智能体可 `plan` 生成电路文件再 simulate |
