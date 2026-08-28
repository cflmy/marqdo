# `ext/quantum` Q8 — 可视化美学升级（科技感电路图）

| | |
|---|---|
| 状态 | **Accepted · Q8a/Q8b landed · Q8c pending** |
| 日期 | 2026-08-28 |
| 父设计 | [ext-quantum.md](ext-quantum.md) · [ext-quantum-q7.md](ext-quantum-q7.md) |
| 路线图 | [roadmap/ext-quantum.md](../roadmap/ext-quantum.md) |
| 对照 | [IBM Quantum Composer](https://quantum.cloud.ibm.com/docs/guides/composer) · [Qiskit `iqp` / `iqp-dark`](https://quantum.cloud.ibm.com/docs/api/qiskit/qiskit.visualization.circuit_drawer) · Cirq / Quirk 视觉习惯 |

---

## 0. 一句话

修复 view 内嵌量子 SVG 的**文字–导线粘连**，并把电路 / 概率 / 布洛赫 / 密度类图统一到一套**量子实验室科技感**视觉令牌（深底、门族分色、清晰留白）——仍只输出自包含 SVG，无外链字体/CDN。

---

## 1. 问题诊断（对照用户截图）

| 现象 | 根因（`plugins/quantum/src/draw.rs`） |
|------|----------------------------------------|
| `q0`/`q1` 与导线穿过文字 | 导线从 `x=8` 起画满宽；标签 `x=4` 且基线对齐导线中心 → 线穿字 |
| 「不好看 / 无科技感」 | 单色 `#1a1a1a` + 浅灰盒；无门族色、无背景板、无线宽层次 |
| 门盒贴线、略挤 | `COL_W=56`、盒 28×28、标签栏与第一列无独立 gutter |
| 测量/控点辨识弱 | 与单比特门同色同描边，缺少语义色 |

**不在本切片**：交互拖拽、WebGL、外链字体、第二套 view 皮肤。

---

## 2. 业界对照（联网 2026-08）

| 来源 | 可借鉴点 | 本仓取舍 |
|------|----------|----------|
| **Qiskit `iqp` / `iqp-dark`** | 门族 `displaycolor`（面色+字色）；深色底；线色弱于门 | 移植**色板与分类**，不绑 matplotlib |
| **IBM Composer** | 经典门深蓝、相位门浅蓝、非酉灰；导出 light/dark | 默认 **dark tech**（view 卡片上更醒目）；可选 `theme=light` |
| **Quirk** | 高对比栅格、测量仪表清晰 | 测量盒保留仪表符号，加强描边/填充对比 |
| **教科书 bw** | 印刷友好 | 保留为 `theme=bw` 后备，非默认 |

**刻意避开**：紫白渐变「AI 默认皮肤」；过重 glow 光晕（打印/浅底卡片会糊）。科技感靠：**深 slate 底 + 青/靛门色 + 清晰层级 + 留白**，而非堆霓虹。

---

## 3. 视觉令牌（锁定 `marqdo-iqp`）

### 3.1 主题

| `theme` | 用途 |
|---------|------|
| `dark`（**默认**） | view / 教学演示科技感 |
| `light` | 浅底文档打印 |
| `bw` | 无彩色、高对比 |

作者面：

```markdown
*`svg` = > `qc`.draw kind="circuit" theme="dark"*
*`svg` = > `qc`.draw kind="probs" theme="light"*
```

| 参数 | 默认 | 说明 |
|------|------|------|
| `kind` | `circuit` | 既有枚举不变 |
| `theme` | `dark` | 新增；非法值报错 |
| `path` | — | 既有 |

中文：`主题=dark|light|bw`。ABI：`quantum_draw_circuit` 增可选 `theme`（circuit/probs/bloch 已生效；gate/density 高级图见 Q8c）。

### 3.2 Dark 色板（默认）

| 令牌 | 色 | 用途 |
|------|-----|------|
| `bg` | `#0a0e14` → `#141c28` 渐变 | SVG 圆角底板 |
| `wire` | `#6b7f96` | 比特线（双层描边） |
| `label` / `chip` | `#e8eef6` / `#1e2a3a` | `q0` 芯片标签（线不穿字） |
| `gate.clifford` | 面 `#4c6fff` / 字 `#f2f6ff` | H、X、Y、CX、SWAP… |
| `gate.phase` | 面 `#2de2e6` / 字 `#042024` | S、T、Rz、CZ… |
| `gate.rotation` | 面 `#3dd6c6` / 字 `#041a18` | Rx、Ry |
| `gate.measure` | 面 `#8fa3b8` / 字 `#0a1018` | MEASURE |
| `barrier` | `#9aafc0` 虚线 | BARRIER |
| `ctrl` | `#f0f5ff` | 控制点 / ⊕ |

Light / bw 在实现里各给一套平行表；金样以 dark 主断言（SVG 含 `data-theme="dark"` 与背景色）。

### 3.3 布局（消灭粘连）

```text
| pad | label_col | gutter | col0 | … | colN | pad |
      ^q0 芯片标签    ^导线从此开始（不穿过文字）
```

| 常量 | 实现值 |
|------|--------|
| `LABEL_W` | 52 |
| `GUTTER` | 20 |
| `PAD_X` / `PAD_Y` | 24 / 36 |
| `COL_W` | 80 |
| `ROW_H` | 64 |
| 门盒 | 40×40，`rx=8` |
| 线宽 | 导线双层 3.2+1.8；门描边 ~1.1；控制竖线 2.25 |

标签：圆角芯片 + 居中文字；导线 `x1 = PAD_X + LABEL_W + GUTTER`。

### 3.4 门族映射

| 族 | 门名（归一化后） |
|----|------------------|
| clifford | I X Y Z H CX CNOT SWAP |
| phase | S T RZ CZ |
| rotation | RX RY |
| measure | MEASURE M |
| barrier | BARRIER |
| custom / U | clifford 色或中性 `#546e7a` |

---

## 4. 作者面（增量，向后兼容）

见 §3.1。字符串参数须加引号：`kind="circuit"`、`theme="dark"`（中文 `种类=` / `主题=`）。改插件后须 `marqdo ext add quantum` 再开 view，否则仍加载旧 `.so`。

---

## 5. 分期

| 子阶段 | 内容 | 验收 |
|--------|------|------|
| **Q8a** | 电路轨线：布局 gutter + dark/light/bw 令牌 + 门族分色 | 标签不与线相交；SVG 含 `data-theme`；更新 draw smoke |
| **Q8b** | probs / bloch / gate heatmap 共用令牌 | 柱/球/热力与电路同族色 |
| **Q8c** | hinton / city / qsphere / paulivec / multibloch 换肤 | 高级图不「脱节」；viz-advanced smoke 仍绿 |

---

## 6. 实现约束

1. 仅改 `plugins/quantum` 绘图 + L1 透传 `theme`；**不**改 Marqdo 核心。  
2. SVG **内联**颜色与字体栈（`ui-sans-serif, system-ui, …`）；无 `@import` / CDN。  
3. view 仍走 `record_plot`；底板自带圆角矩形，避免白底卡片上「漂浮黑线」。  
4. 金样：断言结构标记（`data-theme`、`data-gate-family`）+ 既有 kind；不像素比对。

---

## 7. 非目标

- 复制 Qiskit matplotlib 像素级一致  
- 动画 / 3D 电路  
- 用户自定义完整 JSON 主题文件（可 Q9）

---

## 8. 一句话

**先把字从线上挪开，再给门族涂上实验室色——默认 dark tech，打印用 light，论文用 bw。**
