# `ext/quantum` — 自定义酉门（公式矩阵 → 可施加）

| | |
|---|---|
| 状态 | **Accepted · 已落地** |
| 日期 | 2026-08-12 |
| 父设计 | [ext-quantum.md](ext-quantum.md) |

---

## 1. 目标

作者用 `$$…$$`（或嵌套 list）写出酉矩阵 → `quantum.gate matrix=…` 得到可复用门句柄 → `` `qc`.apply gate=… qubits=… `` 施加到电路。

---

## 2. 公式矩阵

`$$` 围栏若含 `[[…]]` 或 `\begin{pmatrix}` / `\begin{bmatrix}`，在**解析期**求值为嵌套数值 list（非整棵 `Formula` 标量树）。

支持：

- ASCII：`[[1,0],[0,-1]]`、`(1/sqrt(2))*[[1,1],[1,-1]]`
- LaTeX 子集：`\frac{…}{…}`、`\sqrt{…}`、`pmatrix` / `bmatrix`（`&` / `\\`）

复数：list 路径仍可用 `{re,im}` 或 `[re,im]`；公式路径本切片以实矩阵为主。

---

## 3. API

```markdown
*`U` = > quantum.gate matrix=`H_matrix` name=U *
# 或构造参数 matrix=；name 可选（默认 U）

*`qc` = > `qc`.apply gate=`U` qubits=0 *
# qubits= 标量或 list；矩阵维数须为 2^k
```

中文：`量子.门 矩阵=` / `` `qc`.施加 门= 比特= ``。

具名门仍可用 `name=H`；`matrix` / `matches_matrix` / `draw` 对自定义门同样有效。

---

## 4. 验收

- 公式（或 list）定义的 H 与内置 `H` 概率一致  
- EN/ZH 金样
