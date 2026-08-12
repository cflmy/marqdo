# 官方扩展：`ext/quantum` 实现路线图

| | |
|---|---|
| 状态 | **Q0–Q4 已落地**（设计 Accepted；Q5 可选噪声/更大 n 另文） |
| 日期 | 2026-08-12 |
| **锁定设计** | [design/ext-quantum.md](../design/ext-quantum.md) |
| 相关 | [ext-cli.md](../design/ext-cli.md) · [ext-abi.md](../design/ext-abi.md) · [stdlib-math.md](../design/stdlib-math.md) · [view.md](../design/view.md) |
| 安装 | `marqdo ext add quantum`（`quantum` / `量子`） |

本文只跟踪**实现阶段**。作者面、模拟语义、view 可视化、分期以设计文为准。

---

## 1. 目标回顾（一句）

**可运行的量子电路文档**：表声明门序列 → ABI 模拟概率/采样 → view 内嵌电路/概率 SVG；文档跑通即公式约定成立。

---

## 2. 落地阶段

| 阶段 | 内容 | 状态 |
|------|------|------|
| **Q0** | CATALOG + `plugins/quantum` ping + L1 `ensure_plugin` | **done** |
| **Q1** | 态向量；I/X/H/CX；`simulate` / `probabilities`；贝尔态 gold | **done** |
| **Q2** | Y/Z/S/T/R\* /CZ/SWAP；`run seed=`；中英 API | **done** |
| **Q3** | `steps=` 电路表；`draw` 轨线 SVG；view `record_plot` 或值识别 | **done** |
| **Q4** | 概率条 / 布洛赫；`matches_matrix`；examples + 用户文档 | **done** |
| **Q5** | （可选）噪声 / 更大 n —— 另文 | pending |

验收金样：`tests/ext/quantum-bell-smoke.mq.md`、`quantum-draw-smoke.mq.md`、`quantum-gate-matrix-smoke.mq.md`、中文对称样例。

---

## 3. 已收敛的原开放点

| 原疑问 | 锁定结论（见设计文） |
|--------|----------------------|
| 并进 math？ | **否**，独立 `ext/quantum` |
| 电路表 | **要做**（Q3），与方法链等价 |
| view 图 | 自绘 SVG；优先 `host_query("record_plot")`；不依赖 web |
| 自由函数面 | **不**与对象面并行 |

---

## 4. 与其它文档

| 文档 | 关系 |
|------|------|
| [design/ext-quantum.md](../design/ext-quantum.md) | **真理** |
| [stdlib-math.md](../design/stdlib-math.md) | 公式/作图精神对照；实现分离 |
| [ext-web.md](../design/ext-web.md) | 同属官方 ext + ABI + `ext add` 模板 |
| [view.md](../design/view.md) | 结构卡 + Execution 内嵌 SVG |
