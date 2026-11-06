# 扩展库开发范式：code = document = skill = tools

| | |
|---|---|
| 状态 | **Active**（quantum 试点已落地；其它官方扩展按本范式推广） |
| 日期 | 2026-11-06 |
| 愿景 | [doc/next/012.md](../next/012.md)（手写真相源，本文不修改它） |
| 契约摘要 | [ext-index-skill-tools.md](ext-index-skill-tools.md) |
| 试点 | [`ext/quantum/`](../../ext/quantum/)（完成）· [`ext/web/`](../../ext/web/)（迁移中，含[布局范式](ext-web-layout-paradigm.md)） |
| 相关 | [ext-cli.md](ext-cli.md) · [ext-abi.md](ext-abi.md) · [ext-quantum.md](ext-quantum.md) · [lib-code-as-docs.md](lib-code-as-docs.md) · [stdlib-i18n.md](stdlib-i18n.md) |

---

## 0. 一句话

一份 `.mq.md` **同时是**可执行代码、给人读的文档、给智能体的 SKILL、以及可 `marqdo call` 的 tools 面。  
**函数即工具**（`##`），**子函数即子工具**（`###`）。禁止旁路 catalog / OpenAPI / MCP 包装成为第二真相源。

---

## 1. 为什么这样设计

| 传统拆法 | 本范式 |
|----------|--------|
| README + SDK + tool schema 三套漂移 | 同一文件既可跑又可讲 |
| Agent 调 MCP，作者调库 API | 智能体与作者共读同一路径；智能体默认 `marqdo call` |
| 扩展 = 功能清单 | 扩展 = 可执行知识包：根 skill 先教会「怎么调」 |

智能体冷启动时只应：打开根 `index.mq.md` → 按「选哪条路」→ 照抄 `marqdo call`。不必发明第二套 API。

---

## 2. 包布局（强制形态）

一个官方扩展 **一个目录**（与 `web` / `ai` 一致）。中英 **同包分文件**，不要把中文树抬到 `ext/` 顶层变成「第二个包」。

```text
ext/<id>/
  index.mq.md                 # EN 根 SKILL（先读）
  <module>/index.mq.md        # 模块：## call 工具 + 可选 # 作者对象
  _capability/index.mq.md     # 仅插件加载（日常勿当业务工具）
  <中文名>/                   # ZH 镜像树（同等标准）
    索引.mq.md
    <模块>/索引.mq.md
```

试点现状：

```text
ext/quantum/
  index.mq.md · circuit/ · gate/ · simulate/ · state/ · density/ · viz/
  _capability/
  量子/索引.mq.md · 电路/ · 门/ · 模拟/ · 态/ · 密度/ · 可视化/ · 能力/
```

安装后落在 `~/.marqdo/ext`（或 `MARQDO_EXT`），**不是**仓库源码 `ext/`：

```text
~/.marqdo/ext/
  quantum/...
  native/libquantum.so
  quantum.plugin          # 路径提示；勿提交源码树
```

---

## 3. 根 skill 怎么写

根 `index.mq.md` / `索引.mq.md` 是冷启动入口，须按顺序包含：

1. **选哪条路**（表格，勿用 `1.` 列表——在 Marqdo 里 `1.` 是分支语法）  
   - 只要演示结果 → 根上的编排工具（如 `bell_phi_plus` / `贝尔态`）  
   - 要句柄继续 → 贴出分步 `call`（同一 `--session` + `--toget`）  
   - 换语言 → 指向另一语言树，禁止混用 EN/ZH 工具名  
2. **如何执行任意工具**：固定 CLI 形状、`--effects plugin`、`--json`、`$name`  
3. **包布局**：每个模块路径 + 一句用途  
4. **可选一键编排工具**（如贝尔态）：无参或极少参，便于验收  

智能体默认路径是 `##` + `marqdo call`。`# circuit` / `# 电路` 等对象是 **作者程序** import 用的，须在根 skill 写明，避免代理误走方法链。

---

## 4. 模块文件怎么写

每个 `index.mq.md` 单独拿到也要能用：

| 块 | 要求 |
|----|------|
| 开篇叙述 | 领域是什么、何时打开本文件 |
| For AI agents / 给智能体 | 可照抄的 `marqdo call`；缺能力时 **写明下一步打开哪个兄弟模块**（例如 circuit 造线后概率在 simulate） |
| `##` 工具 | 形参契约表紧贴标题下；返回契约表在返回标记前；叙述 ≥ 50% |
| `#` 对象（可选） | 作者链式 API；加载插件走 facade（如 `cap.load` / `能力.加载`），禁止作者面 `host_*` |

薄实现：工具体调用 ABI（`*> quantum_…*`）或转发兄弟模块即可，真相在叙述与契约表。

---

## 5. `marqdo call` 与会话

```text
marqdo call <FILE> <FUNCTION> [k=v…]
  [--json]
  [--effects none|plugin]     # 默认 none；ABI 扩展必须 plugin
  [--session <id>]
  [--toget <name>]            # 或尾部 toget <name>
```

| 机制 | 用途 |
|------|------|
| `--effects plugin` | 允许 `plugin.call`；仍拒 fs/network/process/llm 等 |
| `--session` + `--toget` | 跨进程保存返回值；`$name` 引用；plugin 句柄经 journal 重放 |
| `parent.child` | 嵌套 `###` 子工具 |

参数字面量：数字、引号文本、True/False/None、以 `[`/`{` 开头的 JSON、或 `$name`。

---

## 6. 叙述 ≥ 50%

```bash
marqdo check --narrative-ratio ext/<id>/path/index.mq.md
```

- 按函数单元计；契约表计为叙述；语言代码围栏计为代码。  
- 试点包 `ext/quantum/**` **零豁免**（含 `_capability` 与中文树）。  
- 狗粮 skill：`lib/lint/narrative.mq.md`。

---

## 7. ABI 与 capability

- 原生能力只经 `plugin.native_path` + `plugin.load`（集中在 `_capability` / `能力`）。  
- 领域模块在工具开头调用 `cap.load` / `能力.加载`，**不**把 capability 当日常业务工具。  
- 遵守 ADR 0007：作者程序与 `ext/**` 业务面不写 `host_*` / `ensure_plugin` 垃圾抽屉。

---

## 8. 中英与导入

| 规则 | 做法 |
|------|------|
| 分文件 | EN 与 ZH 两套树；禁止同一文件混排 API 名 |
| 同包 | `ext/<id>/` 内放中文子树，勿 `ext/<中文包名>/` 顶层并列 |
| 作者导入 | `import: circuit: ext/quantum/circuit/index.mq.md` → `circuit.circuit` |
| 中文导入 | `导入 量子:ext/quantum/量子/电路/索引.mq.md` → `量子.电路` |
| 智能体 | 打开对应语言的根 skill，工具名与路径保持同语言 |

---

## 9. 安装与源码卫生

```bash
cargo build -p marqdo_plugin_<id>          # 若有原生插件
MARQDO_EXT_SOURCE=/path/to/repo/ext \
  marqdo ext add <id>                      # 装到 ~/.marqdo/ext
```

| 禁止 | 原因 |
|------|------|
| `MARQDO_EXT` 指向仓库 `ext/` 再 `ext add` | 同文件 `fs::copy` 会清空源码；并写入 `*.plugin` / `native/` |
| 提交 `ext/*.plugin`、`ext/native/` | 安装产物；已 gitignore |

`marqdo ext add` **拒绝**把安装根设成源码 `ext/`（防再踩坑）。

---

## 10. 常见陷阱

| 陷阱 | 正确做法 |
|------|----------|
| 正文写 `1. xxx` 当条目 | 用表格或项目符号；`1.` 是分支 |
| 漏 `--effects plugin` | 根 skill 与模块 For AI 段反复写明 |
| 只发 `circuit` 文件却找 `probabilities` | 在 circuit 的智能体段贴出 simulate 的完整 call |
| 智能体去调 `#` 对象方法 | 根 skill「选哪条路」声明默认走 `##` call |
| 中英混用 `新建` + `ext/quantum/circuit` | 选定一棵树用到底 |
| 用「兼容层」大文件双挂 API | 删除；模块树即唯一面 |

---

## 11. 开发验收清单

新扩展或把旧包迁到本范式时，逐项打勾：

- [ ] `ext/<id>/index.mq.md`：选哪条路 + call 形状 + 布局 +（可选）一键编排  
- [ ] 每个模块 `index.mq.md`：For AI agents 可单独照抄；缺能力时指向兄弟模块  
- [ ] 全部 `.mq.md`：`marqdo check --narrative-ratio` 通过  
- [ ] 需要 ABI：`_capability` 唯一加载；领域只调 facade  
- [ ] 中文：同包子树，路径与 ## 名全中文  
- [ ] `ext_cli` catalog 的 `mq_files` 列全 EN+ZH 源文件  
- [ ] 金样：`marqdo call` 一键工具 +（若有句柄）分步 `--session`/`--toget`  
- [ ] 作者 smoke：import 模块后的 `#` 对象链仍绿  
- [ ] 未把安装产物写进仓库 `ext/`

---

## 12. 智能体视角（试点自检）

任务「制备 Bell 态」时，合格根 skill 应使代理：

1. 打开 `ext/quantum/index.mq.md`（或中文 `量子/索引.mq.md`）  
2. 只要结果 → `bell_phi_plus` / `贝尔态`  
3. 要句柄 → 照抄 circuit→simulate 四条 call，同一 session  

不必阅读 gate/density，不必发明 Python/MCP。

---

## 13. 推广顺序（建议）

1. **已完成试点**：`ext/quantum/`（含 `量子/`）  
2. **进行中**：`ext/web/` — 根 `index` + `layout` 布局铁律（文件划分、反 HTML 注入）已入库；facade 路径保持兼容；模块叙述与 call 金样继续补齐  
3. **下一优先**：`linalg` 公式面、`ai` agent 面  
4. **`ext/net` / `ext/data` / `ext/security`**：随 web 站点场景补根 skill，勿把 DB/鉴权 API 塞回 `web` facade  

每迁一包：更新本文件「试点/推广」表与该包 `doc/design/ext-*.md` 布局节，并加 call 金样。  
**Web 特有**：布局工程必须写进包内 skill（见 [ext-web-layout-paradigm.md](ext-web-layout-paradigm.md)），不能只放在外部 README。

---

## 14. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-11-06 | 初版：由 quantum 试点归纳的整包开发范式与验收清单 |
