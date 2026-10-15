# View：静态页交互与公式/代码卡视觉

| | |
|---|---|
| 状态 | **Active** |
| 日期 | 2026-10-15 |
| 相关 | [view.md](view.md) · [user-site.md](user-site.md) · [view-document-annotations.md](view-document-annotations.md) · [view-frame-selection.md](view-frame-selection.md) · [ADR 0002 browser WASM](../adr/0002-browser-marqdo-wasm.md) |

**已确认（2026-10-15）**

| 决策 | 选择 |
|------|------|
| 引擎 | **B**（快照 + S1 帧切换 + 可选懒加载 WASM S2） |
| 体验原则 | 静态页与 live view **尽量同构**；控件形态对齐；无服务端时用 WASM/JS 补齐 |
| 视觉原则 | **一层主视觉**；一眼能懂；公式/代码不与 Bind chip / 源残留叠床架屋 |
| 演示参数条 | **要**（历史实参 select + 可编辑；S1 切帧 / S2 重跑） |
| code-card Run | 有 WASM 时可用；否则隐藏（不对齐「假按钮」） |
| 公式 Bind chip | **卡内不显示**；Variables 可留终值 |

---

## 1. 动机（对照 `public/00-欢迎.mq.md` / gh-pages）

静态导出（`marqdo view output` → `gh-pages`）与 live `marqdo view` 已共用**文档面 Structure** 渲染路径，但用户站实测仍有三类落差：

1. **交互控件缺失 / 偏弱**  
   - 函数「问候」仅有参数 chip「谁」，没有可切换的调用帧下拉（即便源里 `> 问候` 写了两次）。  
   - 参数行 `+ 谁` 在文档面呈声明高亮，读者无法在静态页改「谁」并看到新输出。  
   - 外联 `code-card` 的 Run 在静态模式被禁用（`Run is available in live marqdo view`）。  
   - 静态 stdin 仅在有 preset 时显示只读提示，**不能**再跑。

2. **无服务端时仍可交互的预期**  
   用户站是纯静态托管：没有 `/api/run`。合理路径是 **页内 JS** 和/或 **嵌入 Marqdo WASM**，在浏览器里重算 Structure 标注、Execution 区，甚至模拟 `input` / 外联（沙箱约束另定）。

3. **公式 / 代码块视觉凌乱**  
   同一赋值同时出现多层表象，信息重复、线框错位：

   | 层 | 公式 `f = $$…$$` | 外联 `hi = ```python…``` |
   |----|------------------|---------------------------|
   | A 卡片 | `formula` badge + KaTeX | `code` badge + 高亮源码 + cmd/Run |
   | B 标注 chip | `f` → `x^2 - 2` | `hi` → 整段 fence 摘要 |
   | C 源残留 | 卡片外/旁再露 `$$` 或 ASCII | 卡片下再露 `python print…` 明文 |

   再叠加 CSS L-leader 与卡内边框，首屏显得碎、重复、「坏掉的线」。

**本文件只定问题与方案；确认前不改代码。**

---

## 2. 现状解剖

### 2.1 静态 vs live（同一 `page_file`）

| 能力 | Live | Static（`LinkMode::Static`） |
|------|------|------------------------------|
| 文档面 Structure + annot | ✅（跑后） | ✅（导出时跑后写入） |
| call / loop picker JS | ✅（读 `#mq-view-trace`） | ✅（同源 JS；若轨迹未写入则无控件） |
| `/api/run` · Stream | ✅ | ❌ |
| stdin 表单 | ✅ 可再跑 | 只读 hint（或无） |
| code-card Run | ✅ `POST` 外联 | ❌ 禁用按钮 |
| WASM 重算 | ❌（未接） | ❌（未接） |

结论：**不是「静态另做一套皮肤」**，而是「导出瞬间的快照 + 部分 JS 帧切换」；**没有**浏览器内重执行闭环。

### 2.2 欢迎页结构（问题复现源）

```text
> 问候 谁=marqdo
> 问候 谁=marqdo user
## 问候
    + `谁`
> 打印 内容=欢迎，`谁`！
`f` = $$ x^2 - 2 $$
`hi` = ```python … ```
```

- 期望：`## 问候` 旁 **调用** 下拉（`1: 谁=marqdo` / `2: 谁=marqdo user`），切换时打印行的绑定/输出随帧变。  
- 现状风险：若导出时 CallFrame &lt; 2、或 picker 未挂上、或读者期望的是「编辑谁」而非「选历史调用」，都会表现为「没有下拉框」。  
- 公式/代码：`SpecialAssign` → formula/code **card**，同时又对同名绑定建 **Bind annot**，文档面还可能保留块源视觉 → 三重展示。

### 2.3 已有红线（方案不得破坏）

- 代码即文档：`.mq.md` 源可读；不发明第二套嵌套 DSL（见 web-artifact / marqdo-dev）。  
- Browse ≠ Debug：静态站不做断点。  
- `public/**/*.html` 不入库；gh-pages 由 CI/`view output` 生成（[user-site.md](user-site.md)）。  
- 外链字体禁令仍在 [view.md](view.md)；WASM/重型资源须 **可选加载**，默认站可降级为纯快照。

---

## 3. 目标（验收口径）

### G1 — 帧与参数交互（静态可读、可拨）

1. 同函数 ≥2 次调用时，静态页 **必有**「调用」下拉；默认末次；切换只改 chip/臂/相关输出，**源码常显**（同 [view-frame-selection.md](view-frame-selection.md)）。  
2. 导出保证写入完整 `#mq-view-trace`（含 `calls` / `events` / 可选 `loops`）。  
3. （增强）演示向参数可提供 **受控输入**：默认值来自末次调用或 FM；改参后可触发页内重算（见 §5 引擎选项）。

### G2 — 无服务端可重算（可选引擎）

| 档位 | 能力 | 依赖 |
|------|------|------|
| **S0 快照**（现状加强） | 导出时跑完；帧 picker；只读 Execution | 无 WASM |
| **S1 页内重放轨迹** | 不重跑程序，只切换已序列化帧/标注 | JSON only |
| **S2 WASM 重跑** | `marqdo_wasm` + bridge：改 stdin/实参后重跑，刷新 Structure/Execution | WASM 包（可 CDN/`public/wasm/`） |
| **S3 外联沙箱** | code-card Run：浏览器内 pyodide **或** 明确标「仅 live」 | 可选；默认可关 |

默认用户站：**S0 + S1 必达**；**S2 为推荐主路径**（与「用 wasm 模拟输出」对齐）；S3 单独开关。

### G3 — 公式 / 代码视觉单层真相

对「特殊赋值」块约定 **一层主视觉 + 至多一层结果**：

| 块 | 主视觉 | 允许的结果层 | 禁止 |
|----|--------|--------------|------|
| 公式 | KaTeX（formula-card 精简） | 无 Bind chip，或仅在 Variables | 再渲一份 `$$` 明文；annot 与卡重复同一 TeX |
| 外联代码 | 高亮源码（code-card） | 运行输出区 `code-out`；静态可藏 Run 或接 S2/S3 | annot 再贴整段 fence；卡下重复裸源 |
| 普通赋值 | 标记面语句 | Bind chip + L-leader | — |

Leader：仅连接「主视觉锚点 → 唯一结果 chip」；卡内不用第二套 L 线。

---

## 4. 问题清单（成文）

| ID | 问题 | 严重度 | 根因假设 |
|----|------|--------|----------|
| P1 | 静态页看不到调用/循环下拉 | 高 | 轨迹未导出完整 / picker 条件（≥2）未满足 / 读者要的是编辑参而非选帧 |
| P2 | 不能改参/stdin 再看输出 | 高 | Static 禁用 `/api/run`；无 WASM 闭环 |
| P3 | code-card Run 静态无效 | 中 | `initForeignCards` 显式禁用 |
| P4 | 公式卡 + chip + `$$` 三重 | 高 | card 与 Bind annot 叠加；块源未完全被 SpecialAssign 吞掉 |
| P5 | 代码卡 + chip + 明文三重 | 高 | 同 P4 |
| P6 | 卡内/旁碎线、错位 | 中 | leader 锚到错误节点；卡 CSS 与 annot-band 叠层 |
| P7 | 静态 stdin 体验差 | 中 | 仅有只读 hint |

---

## 5. 解决方案（详案，待选）

### 5.1 引擎策略（三选一，推荐 **B**）

| 方案 | 做法 | 优点 | 代价 |
|------|------|------|------|
| **A. 纯快照 + S1** | 加强导出轨迹与 picker；不引入 WASM | 体积小、离线稳 | 不能改参真重算 |
| **B. 快照默认 + 可选 WASM（S0/S1/S2）** | `view output` 仍嵌入跑后 HTML；页脚/侧栏「在浏览器中重跑」懒加载 WASM；改参后 patch Structure/Execution | 对齐用户预期；无 WASM 时仍可读 | 打包/加载体积；宿主能力子集 |
| **C. 静态页全面 WASM 首屏** | 打开即 WASM 跑 | 交互最强 | 冷启动慢；gh-pages 带宽；与「文档先可读」冲突 |

**推荐 B**：文档先可读（S0），帧切换不依赖 WASM（S1），需要改参/演示时再拉 WASM（S2）。

### 5.2 交互控件（与文档面一致）

1. **调用 / 循环 picker**（已有组件）  
   - 导出验收：欢迎页「问候」必须出现调用下拉。  
   - 单测：对 `00-欢迎.mq.md` / `document-annot-demo` 断言 HTML 含 `call-picker` 且 options≥2。

2. **演示参数条（新，窄范围）**  
   - 仅当函数被标记为「可演示」**或**静态页检测到 ≥1 形参且存在 CallFrame 时，在函数标题旁增加：  
     `谁 [ text input | select(历史实参) ]  [应用]`  
   - **不**改 `.mq.md` 语法；数据来自 trace 的 `args_brief` / 末次绑定。  
   - 「应用」：S1 仅切换已有帧；S2 调 WASM `run` 后替换 `#structure` / Execution。

3. **stdin**  
   - 静态：若源含 `input`，渲染与 live 同形的 textarea +「重跑」（S2）或说明「需 WASM」。  
   - 无 input 则不显示。

### 5.3 公式 / 代码去重（视觉）

**规则：SpecialAssign 行只走 card，不再为同名同值挂 Bind chip。**

```text
公式赋值行
  └─ formula-card（badge + 名 + KaTeX）
  └─ ✗ 不再 render_annot_band(Bind f)
  └─ ✗ 不再在卡外输出 $$ 源

外联赋值行
  └─ code-card（badge + 名 + 高亮体 + 可选 Run）
  └─ ✗ 不再 Bind chip 贴 fence 全文
  └─ 运行结果只进 .code-out
```

静态 Run 策略（与 §5.1 对齐）：

- S0：隐藏 Run，或保留禁用并 tooltip「打开 live view / 启用 WASM」。  
- S2：Run → WASM 宿主 `foreign`（若 WASM 支持）或提示不支持。  
- S3：可选 Pyodide（默认关）。

CSS：formula-card / code-card 与 `.annot-band` 垂直节奏统一；禁止对 KaTeX 节点拉 L-leader。

### 5.4 导出管线

```text
marqdo view output
  → build_file_view（跑 + annot + trace_json）  // 已有
  → page_file(Static)
       + 嵌入 #mq-view-trace
       + （可选）data-wasm-url / 懒加载脚本
       + 公式/代码去重后的 structure_html
  → public/ + gh-pages
```

`--no-exec`：文档面无标注；可提示「本页为结构-only」；S2 仍可尝试 WASM 首跑（可选，默认关以免惊喜执行）。

### 5.5 明确不做（本提案）

- 静态页断点调试（仍属 `marqdo debug`）。  
- 在 `public/*.mq.md` 发明新的「交互表」DSL（除非另开设计并获批）。  
- 默认全站加载 Pyodide。  
- 提交生成 HTML 进 `main`。

---

## 6. 分阶段开发计划（确认后执行）

| 阶段 | 内容 | 交付 | 依赖确认 |
|------|------|------|----------|
| **D0** | 本文定稿；选定引擎 **A/B/C**；是否要「演示参数条」 | 状态 → Active | **本次决策** |
| **D1** | P4/P5/P6：SpecialAssign 与 Bind 互斥；卡 CSS；去掉重复源 | 欢迎页公式/代码干净；单测 | D0 |
| **D2** | P1：导出轨迹验收 + 欢迎页 call-picker 金样；修任何「≥2 调用无下拉」bug | HTML 断言 | D0 |
| **D3** | P2/P7 S1 夯实：静态 stdin UI 形态；无 WASM 时文案清晰 | UX | D0 |
| **D4** | （若选 B）S2：静态页懒加载 WASM；「应用参数 / 重跑」刷新 Structure+Execution | `public/wasm` 或 CDN 策略；体积预算 | D0=B |
| **D5** | P3：code-card 与 S2/S3 策略落地；默认隐藏无效 Run | — | D1+D4 |
| **D6** | `build-public` / pages CI 回归；更新 [user-site.md](user-site.md) · [view.md](view.md) 验收条 | gh-pages 抽检欢迎页 | D1–D5 |

建议默认路径：**D0 选 B → D1 → D2 → D3 → D4 → D5 → D6**。

---

## 7. 决策记录

见文首「已确认」。开发按 §6：D1 → D2 → D3 → D4 → D5 → D6。

---

## 8. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-10-15 | 初稿：静态交互缺口、公式/代码凌乱、S0–S3 与阶段 D0–D6 |
| 2026-10-15 | Active：引擎 B；对齐 live；干净视觉；演示参数条；Run 接 WASM 否则隐藏 |
