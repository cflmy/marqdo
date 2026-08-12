# `ext/quantum` Q5 — 最小噪声（教学用）

| | |
|---|---|
| 状态 | **Accepted · 最小实现已落地** |
| 日期 | 2026-08-12 |
| 父设计 | [ext-quantum.md](ext-quantum.md) |
| 路线图 | [roadmap/ext-quantum.md](../roadmap/ext-quantum.md) |

---

## 1. 目标

在 **不** 引入完整开放系统 / 云后端的前提下，让教学文档能写「带噪声的采样」，金样可复现。

**刻意不做（本 Q5 切片）：** 任意 Kraus 组、振幅阻尼完整表、>12 qubits 张量网络、噪声参数学习。

---

## 2. 作者面

电路上可选噪声字段（方法链设置，返回新句柄）：

```markdown
*`qc` = > `qc`.noise kind=bitflip p=0.05 *
# 或
*`qc` = > `qc`.noise kind=depolarizing p=0.02 *
```

| 参数 | 含义 |
|------|------|
| `kind` | `bitflip` \| `depolarizing`（中文：`比特翻转` / `退极化`） |
| `p` | 每比特、每个非 `barrier`/`measure` 门后的错误概率，`0…1` |

`simulate` / `probabilities`：**忽略噪声**（仍为理想酉演化），避免概率图被蒙特卡洛抖动。  
`run shots= seed=`：**启用噪声**（若 `p>0`）：按轨迹采样，结果可复现。

---

## 3. 模拟语义（轨迹）

每个 shot：

1. 从 `|0…0⟩` 施电路门（跳过 `BARRIER`；`MEASURE` 不改变酉演化）。  
2. 每施一个会改态的门后，对**该门涉及的每个比特**独立：  
   - `bitflip`：以概率 `p` 施 `X`；  
   - `depolarizing`：以概率 `p` 均匀随机施 `X`/`Y`/`Z` 之一。  
3. 若电路含 `MEASURE` 标记：只对标记比特做计算基采样；否则测全部比特。

RNG：与现有 `run seed=` 同一 SplitMix64 流（门后噪声与最终采样共用）。

---

## 4. 与作者面补齐的关系

| API | 行为 |
|-----|------|
| `barrier` | 仅可视化；模拟跳过 |
| `measure` | 可视化测量符号；决定 `run` 读出哪些比特 |
| `append` | 追加另一 `circuit` 的 `ops`（或单门） |
| `state` | 等价于当前理想 `simulate` 结果（`quantum_state`） |

---

## 5. 验收

- 金样：理想 Bell `run` 不变；`noise kind=bitflip p=1` 在已知电路上计数偏离理想（`seed=` 固定）。  
- `barrier`/`measure` 出现在 `draw kind=circuit` SVG。  
- 中英 L1 对称；仍无 `host_*` 于 `ext/**`。
