# 扩展库：index 即代码 · 文档 · SKILL · tools

| | |
|---|---|
| 状态 | **Active**（契约卡；完整开发范式见下行） |
| 日期 | 2026-11-06 |
| 愿景 | [doc/next/012.md](../next/012.md)（手写真相源，本文不修改它） |
| 范式长文 | [ext-skill-tools-paradigm.md](ext-skill-tools-paradigm.md)（布局、陷阱、验收清单、推广顺序） |
| 试点 | `ext/quantum/` |
| 相关 | [ext-abi.md](ext-abi.md) · [ext-quantum.md](ext-quantum.md) · [lib-code-as-docs.md](lib-code-as-docs.md) · [function-as-table-semantics.md](function-as-table-semantics.md) |

---

## 0. 一句话

扩展包里**每一份** `.mq.md` 同时是可执行代码、可读 SKILL、以及可 `marqdo call` 的 tools 面；**函数即工具**，`###` 即子工具；禁止第二套 catalog / OpenAPI 真相源。开发新包或迁包时以 [ext-skill-tools-paradigm.md](ext-skill-tools-paradigm.md) 的验收清单为准。

---

## 1. 整包统一作者契约（无例外）

适用于包内全部 `.mq.md`（含 `index`、子依赖、`_capability`、兼容层、中英镜像）：

| 要求 | 含义 |
|------|------|
| 代码 | 标记可执行；实现可薄委托 ABI / 兄弟模块 |
| SKILL | `#` 下写清领域、何时用、**For AI agents → `marqdo call` 本文件** |
| tools | `##` / `###` 即工具树；无旁路 schema |
| 版式 | **形参契约表紧贴标题下**；**返回契约表在体末**（返回标记前） |
| 叙述 | 每个函数单元叙述占比 **≥ 50%**；公式与约定写进人话 |
| 禁止 | `+` 清单当主表面；ABI 名罗列充文档；「内部文件免检」 |

形参仍由叙述/体中的 `` `名` `` 推断 arity；`| 参数 | 类型 | 说明 |` 为契约表（见 `tests/contracts/`）。

---

## 2. 包布局（quantum 试点）

```text
ext/quantum/
  index.mq.md                 # EN 根 SKILL：无上下文也能照抄 marqdo call
  circuit/index.mq.md         # call 工具 + 作者 # circuit
  gate/ · simulate/ · state/ · density/ · viz/
  _capability/index.mq.md
  量子/                       # ZH 同包（勿再把 量子/ 抬到 ext/ 顶层）
    索引.mq.md
    电路/ · 门/ · 模拟/ · 态/ · 密度/ · 可视化/ · 能力/
```

`index` = 总图与编排，必须写明 FILE/FUNCTION/`--effects plugin`；每个子目录的 `index.mq.md` 单独拿到也要能执行。无单独「兼容层」文件。

---

## 3. `marqdo call` 与会话

```text
marqdo call <FILE> <FUNCTION> [k=v…]
  [--json] [--backend tree|bytecode]
  [--effects none|plugin]     # 默认 none
  [--session <id>]
  [--toget <name>]            # 或尾部 toget <name>
```

- 默认：仅纯函数（今日行为）。
- `--effects plugin`：允许 `plugin.call`（及附属的 `output.write`）；仍拒绝 fs.write / network / process.exec / llm 等。
- `toget` + `--session`：绑定写入 `.marqdo/call-sessions/<id>.json`；参数 `$name` 引用会话值；跨进程 **plugin 句柄经 journal 重放** 重建。
- 嵌套子工具：`parent.child`（`###` 路径）。

值字面量：整数/小数/引号文本/True|False|None；以 `[` / `{` 开头时按 JSON 解析为 List/Map。

---

## 4. 叙述 ≥ 50%

`marqdo check --narrative-ratio`（及 `lib/lint/narrative.mq.md` skill）按函数单元统计叙述字符占比；`ext/quantum/**/*.mq.md` **全量零豁免**。

---

## 5. 与 ADR 0007

作者程序与业务路径仍不写 `ensure_plugin` / `host_*`；插件经 `_capability` / facade 加载。Agent 用 `--effects plugin` 调用 facade，不把 capability 当日常工具（文件本身仍须达标 skill）。

---

## 6. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-11-06 | 初稿：012 试点契约、call effects/toget、quantum 模块树 |
| 2026-11-06 | quantum 中文树收进 `ext/quantum/量子/`；根 skill 增加「选哪条路」；`ext add` 拒绝装进源码树 |
| 2026-11-06 | 链到 [ext-skill-tools-paradigm.md](ext-skill-tools-paradigm.md) 作为开发范式长文 |
