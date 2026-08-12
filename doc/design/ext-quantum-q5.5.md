# `ext/quantum` Q5.5 — 振幅阻尼（教学轨迹）

| | |
|---|---|
| 状态 | **Accepted · 已落地** |
| 日期 | 2026-08-12 |
| 父设计 | [ext-quantum.md](ext-quantum.md) · [ext-quantum-q5.md](ext-quantum-q5.md) |
| 路线图 | [roadmap/ext-quantum.md](../roadmap/ext-quantum.md) |

---

## 1. 目标

在 Q5 的 `bitflip` / `depolarizing` 之外，补一种**耗散**教学噪声：`amplitude_damping`（中文 `振幅阻尼`），仍只影响 `run` 轨迹；`simulate` / `probabilities` 忽略。

**刻意不做：** 完整 Kraus 表 API、任意算符噪声、密度矩阵引擎、相位阻尼完整 Lindblad。

---

## 2. 作者面

```markdown
*`qc` = > `qc`.noise kind=amplitude_damping p=0.1 *
# 中文
*`qc` = > `qc`.噪声 种类=振幅阻尼 概率=0.1 *
```

别名：`amp_damp` / `amplitude-damping`。

---

## 3. 模拟语义

与 Q5 相同：每个改态门后，对涉及比特独立掷骰。

对每个比特，以概率 `p` 施加**跳跃**算符 \(E_1 = |0\rangle\langle 1|\)（只作用该比特），再归一化：

- 若态在该比特上几乎全为 `|0⟩`（范数 ≈ 0），跳跃无效，态不变。  
- 否则振幅从 `|…1…⟩` 搬到对应 `|…0…⟩` 并归一化。

金样：`X` 后 `amplitude_damping p=1` → 全部 shot 落在 `|0⟩`。

---

## 4. 验收

- `tests/ext/quantum-amp-damp-smoke.mq.md`（+ 中文对称）  
- L1 EN/ZH 文档列出第三种 `kind`
