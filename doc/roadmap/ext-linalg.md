# 官方扩展：`ext/linalg` 实现路线图

| | |
|---|---|
| 状态 | **L0–L3 已落地 · L4+ 待做** |
| 日期 | 2026-09-07 |
| **锁定设计** | [design/ext-linalg.md](../design/ext-linalg.md) |
| 调研 | [research/ext-linalg-formula.md](../research/ext-linalg-formula.md) |
| 相关 | [ext-cli.md](../design/ext-cli.md) · [ext-abi.md](../design/ext-abi.md) · [stdlib-math.md](../design/stdlib-math.md) · [view.md](../design/view.md) · [ext-quantum-q7.md](../design/ext-quantum-q7.md) |
| 安装（规划） | `marqdo ext add linalg`（`linalg` / `线性代数`） |

本文只跟踪**实现阶段**。作者面、公式规则、展示以设计文为准。

---

## 1. 目标回顾（一句）

**可运行的线性代数文档**：矩阵公式可化简、可 KaTeX 展示；需要时再稠密求值与分解，并内嵌结构 SVG。

---

## 2. 落地阶段

| 阶段 | 内容 | 状态 |
|------|------|------|
| **L0** | CATALOG + `plugins/linalg` ping + L1 `ensure_plugin`；`ext add` 条目 | **done** |
| **L1** | MatExpr：`symbol` / 稠密叶子 / `mul`·`add`·`T`·`inv`；`simplify` R1–R5；`latex`·`ascii`·`show` | **done** |
| **L2** | `explicit` / 小阵 gemm·解系·`det`·`trace`；维数护栏；金样 `linalg-basic-smoke` | **done** |
| **L3** | `block` + `collapse`；`kron` 惰性 + R7；金样 `linalg-block-kron` | **done** |
| **L4** | `factorize` LU/QR/SVD/eig（+ 可选 chol）；结构 SVG `svd|qr|ge|eig`；金样 `linalg-factor-smoke` | **pending** |
| **L5** | `heatmap`/`hinton`；examples（最小二乘 / SVD 示意）；public + skill 摘要 | **pending** |
| **L6** | 复数 dtype；`lstsq`；`norm`/`cond`；主题令牌对齐 | **pending** |
| **L7+** | 矩阵微积分 / 更大 BLAS 依赖（仅当用户明确要求） | **deferred** |

验收金样（规划路径）：`tests/ext/linalg-*-smoke.mq.md`；示例：`examples/linalg-least-squares/`、`examples/linalg-svd/`。

---

## 3. 每阶段出口检查

### L0

- [x] `ext/linalg` CATALOG 含 `linalg`  
- [x] 插件 `ping` → ok  
- [x] 文档：`marqdo ext add linalg`

### L1

- [x] `(AB)^\top` → `simplify` → `B^T*A^T`（无 dense）  
- [x] 形状错误含行列信息  
- [x] CLI `ascii` / `show` SVG 通道  
- [x] 中英 L1 各一 smoke

### L2

- [x] `from_formula` 吸收 `$$` 数值阵  
- [x] `solve` 与手算 2×2 一致  
- [x] 超维 `explicit` 硬错误（单元测护栏）

### L3

- [x] 分块乘 `collapse` 后块级正确  
- [x] Kronecker 默认不膨胀元素个数（惰性 `A⊗B`）

### L4

- [ ] SVD/特征结果对象字段稳定  
- [ ] SVG 含可断言标记（如 `linalg-svd`）  
- [ ] `factorize` + 二次 `solve` 复用（若 LU）

### L5

- [ ] 至少 1 个 examples 可 `marqdo run`  
- [ ] public/features 或 tutorial 短页  
- [ ] skill / CHANGELOG Unreleased 条目

---

## 4. 依赖与并行

| 依赖 | 说明 |
|------|------|
| ABI v2 + ext CLI | 已有 |
| view KaTeX / plot | 已有；L1 `show` 接现成通道 |
| Release native zip | 随首次含 `plugins/linalg` 的发版接入 |
| quantum 数值核 | **可选**私有复用；不阻塞 L1–L2 |

不阻塞项：浏览器 WASM 上的 linalg（默认 host/native；WASM 另议）。

---

## 5. 建议开发顺序（下一手）

1. 实现 **L0 + L1**（公式树与展示）——兑现「公式类」承诺，无需完整数值栈。  
2. 再 **L2** 打通验算闭环。  
3. **L3–L4** 教学冲击力（分块 / SVD 图）。  
4. **L5** 文档与示例收口。

---

## 6. 一句话

**先公式与展示，后稠密与分解图；每阶段都有可跑金样，避免一次性巨插件。**
