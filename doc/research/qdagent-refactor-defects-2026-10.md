# 缺陷实录：qdagent 重构期间发现的 Marqdo 运行时问题

| | |
|---|---|
| 状态 | **Open**（D1 待决策 · D2/D3 待修复 · D4–D6 已记录） |
| 日期 | 2026-10-22 |
| 报告者 | qdagent 重构（`/workspace/qdagent`） |
| 运行时 | `marqdo 1.5.2`（本机自建 release，zig cc / glibc 2.36）· Ext pack 1.5.2 |
| 来源 | `execution/docgate.mq.md` 文档门禁开发 · `agent/qdagent.mq.md` 输出契约收紧 |
| 关联 | [markdown-mapping-v0.3.md](../design/markdown-mapping-v0.3.md) · [ext-agent.md](../design/ext-agent.md) · [ext-ai-sandbox.md](../design/ext-ai-sandbox.md) |

---

## 0. 摘要

qdagent 重构把「智能体输出与沉淀笔记必须是可运行 `.mq.md`」设为硬契约，并为此实现了
机械门禁（提取 → 结构检查 → 沙箱试跑）。该过程把 Marqdo 的一批**行为与文档不一致**、
**诊断信息不可用**、**静默降级**问题暴露出来。逐条如下。

| ID | 主题 | 严重度 | 状态 |
|----|------|--------|------|
| D1 | `sandbox.profile` 仅回显，不改变实际能力 | **高** | Open（安全语义） |
| D2 | `CALL：`/`调用：` 全角冒号无法解析，但提示文案推荐它 | **高** | Open（协议契约） |
| D3 | 行首数字 / `+` 使叙述行被当语句，中文文档解析失败 | 中 | Open（可用性） |
| D4 | `sandbox.run source=` 的报错丢失文件上下文 | 中 | Open（诊断） |
| D5 | 成员式取元 `[key](map)` 在键缺失时硬崩 | 中 | Open（健壮性） |
| D6 | 未绑定 GFM 表不可迭代，但报错不指向根因 | 低 | Open（诊断） |
| D7 | 沙箱内 `subtask spawn path=` 静默不可用（设计如此，需文档明示） | 低 | Documented |
| D8 | 函数名含空格：定义成功、调用必崩 | **高** | Open（不对称） |
| D9 | 未闭合引号静默吞行，退出码 0 | **高** | Open（静默） |
| D10 | `table.put` 非原地修改，累积状态静默失效 | 中 | Open（文档） |

---

## D1 — `sandbox.profile` 仅回显，不改变实际能力

### 现象

`profile="readonly"` 打开沙箱后，`sandbox.inspect` 返回的能力仍是 default 行：

```json
{"caps_default":{"exec":false,"fs_write":true,"net":false,"plugin":false},
 "profile":{"exec":"false","fs_write":"true","name":"default","net":"false","plugin":"false"}}
```

即 `fs_write` 仍为 `true`，`profile.name` 回显为 `default` 而非 `readonly`。
沙箱内的写操作（CoW upper）照常可用。

### 根因

- `ext/ai/sandbox.mq.md` 的 `## profile` 是一张**文档用**的能力表，`open root= profile=`
  把 `profile` 透传但原生层不接收（`plugins/agent/src/sandbox.rs` 只转 `root`）。
- 原生 `sandbox_run` 的 `RunOptions` 中四项能力**硬编码**：

```rust
// src/host/sandbox_fs.rs
allow_fs_write: true,
allow_exec: false,
allow_net: false,
allow_plugin: false,
```

### 影响

调用方看到 `profile: readonly` 会合理地认为「沙箱内不能写」，从而把
「用户数据落在 CoW upper 也不会污染宿主」当成安全保证；
但 `readonly` 与 `default` 实际等价，安全语义**静默失效**。
qcagent 的门禁依赖「失败绝不落盘」，一旦有人把门禁写成「先写进沙箱再看 profile」，
就会得到错误结论。

### 建议

二选一，但必须明确：

1. **实现**：`RunOptions` 接受 profile，`readonly` 置 `allow_fs_write=false`；
   `sandbox.write` 在 readonly 下报 `fs_write denied (profile=readonly)`。
2. **降级为文档事实并改名**：把 `## profile` 标注为「仅回显 · 不执行」，
   并从 `open root= profile=` 的公开签名里移除该参数，避免误用。

倾向方案 1：`readonly` 的价值正是「验证型试跑不产生任何 CoW upper」，
这与「门禁只读校验」的语义完全吻合。

---

## D2 — 全角冒号 `调用：` 无法解析，而提示文案推荐它

### 现象

`ext/ai/agent.mq.md` 的 `## extract_tool_name` 用半角 `:` 切分：

```marqdo
**parts = > split value=`t` sep=":"**
```

因此只有 `CALL:x` / `调用:x`（半角）能被识别。
但同一文件给模型的提示文案写的是全角：

```
When calling an allowed tool, reply with exactly one line: CALL:<name>
(Chinese ok: 调用：<name>)
```

模型按提示输出 `调用：tool_find`（全角）时，`split sep=":"` 不切分，
`head` 变成整行 `调用：tool_find`，匹配失败 → **静默忽略该 CALL**，
模型以为调用已发出，实际没有工具结果，最终答案缺少证据。

### 影响

这是**协议层**的静默失败：不报错、不提示，只表现为「工具没被调用」。
对中文模型（尤其国内模型）全角标点是默认习惯，命中率不低。

### 建议

解析器同时接受全角与半角：

```marqdo
**t2 = > replace text=`t` old="：" new=":" count=None**
**parts = > split value=`t2` sep=":"**
```

同类问题应在 `READ:` / `DECISION:` / `SUMMARY:` / `SLUG:` 一并检查——
调研显示这些解析器已经支持全半角，唯独 `extract_tool_name` 与 `extract_call_args` 不一致
（`READ` §900-942 支持全角，`CALL` §130-159 只支持半角）。

---

## D3 — 行首数字 / `+` 导致中文文档解析失败

### 现象

按 v0.3 语法，非标记行是叙述（注释）。但某些行首字符会让该行被当成语句。

**实测修正（1.5.2，逐条跑过）**——不是所有标记都出问题：

| 行首 | 叙述区实测 | 说明 |
|------|-----------|------|
| `9 号是数字。` | ❌ `unrecognized statement` | 数字字面量是真语句 |
| `42` / `2026 年发布。` | ❌ `unrecognized statement` | 同上 |
| `+ 加号` | ❌ `unrecognized statement` | `+` 是前导符语法 |
| `- 列表项` | ✅ 安全 | 不构成有效 foreach → 跳过 |
| `> 引用` | ✅ 安全 | 无对应函数时不当调用 |
| `\| a \| b \|` | ✅ 安全 | 未绑定表不算语句 |
| `[{"a":1}]` | ✅ 安全 | 无归属的链接索引 → 跳过 |

判据是**该行能否被解析成一条完整语句**：能则执行，不能则当叙述跳过。
所以在函数体内上下文成立时，`- [`x`](`xs`)` 仍是真循环。

实测（qdagent 门禁）：

```
gate.门禁 text="...\n# 结论\n\n9 号是数字。\n\n# main\n..."
→ {"ok":false,"stage":"structure",
   "violations":[{"line":6,"rule":"narration_marker","text":"9 号是数字。"}]}
```

### 影响

对**人类作者**这是设计取舍（执行标记优先于叙述，符合「标记即语法」）。
但对**LLM 生成的中文文档**是高频陷阱：中文写「2026 年」「42 条」「3 个要点」
都是默认表达习惯，一旦出现在叙述区，整篇文档解析失败。

这是「文档即可运行」与「文档可读」之间的真实张力。
qdagent 的应对是**在提示词里逐条禁止**并**门禁机械校验**（见
`doc/design/09-document-contract-and-docgate.md`），但根源在语言层面。

补充（实测发现的更隐蔽问题）：**未闭合引号会静默吞行**，不报错，
只在别处表现为 `undefined variable`。见使用手册
[`tutorial/15-常见陷阱.md`](../tutorial/15-常见陷阱.md) §1.4。

### 建议

不改语义（执行标记优先是 v0.3 的基石），但增强可诊断性：

1. `unrecognized statement` 报错附一句提示：
   「若此行为叙述，请以文字开头，或用行内代码/围栏包裹」。
2. `marqdo check` 增加 `narration-marker` 规则，一次性列出全部风险行。
3. 文档（`markdown-mapping-v0.3.md`）显著位置增加「中文叙述常见陷阱」小节。

---

## D4 — `sandbox.run source=` 的报错丢失文件上下文

### 现象

```marqdo
**r = > sb.run id=`id` source="> print text=hello"**
→ {"error":"<memory>: 1:1: expected heading (`#` object or `##` function), got: > print text=hello",
   "ok":false,"stdout":"","value":null}
```

报错用 `<memory>` 作文件名，且**没有指出**「源文本必须含 `# main`」。
对 `path=` 运行报错带真实路径，对 `source=` 只有 `<memory>`，两者诊断质量不一致。

### 影响

门禁类工具需要用 `source=` 直接跑生成的文本（避免先写盘）——
这正是 qdagent 的用法。诊断不可用时会显著拉长修复轮次。

### 建议

`source=` 运行使用伪文件名 `sandbox-source.mq.md`（或调用方可传 `name=`），
并在缺少 `# main` 时返回明确错误：
`no # main object in source; add "# main" before statements`。

---

## D5 — 成员式取元 `[key](map)` 在键缺失时硬崩

### 现象

```marqdo
**err = [error](r)**        # r 来自 sandbox.run
→ error: missing map key 'error'
```

`sandbox.run` **成功**时返回 `{ok, stdout, value}`，**没有** `error` 键；
失败时才有 `error`。于是「成功分支里读 error」会直接崩掉整个文档。

### 影响

同一函数的返回形状随成败变化（成功缺 `error`、失败缺 `value` 之外的键），
按「防御式读取」写的代码极易在成功路径崩溃——而成功路径往往测试覆盖更少。

### 建议

二选一：

1. **统一形状**：`sandbox.run` 始终返回 `{ok, value, stdout, error}`（成功时 `error: ""`），
   与 `llm` 的软失败风格一致。
2. 或在 `markdown-mapping-v0.3.md` 明确 `[key](map)` 是**断言式**取值（缺键即错），
   并提供 `get`/`has` 作为防御式写法（已有 `table.get` / `table.has`）。

qdagent 的修法是改用 `table.get map= key=` 取可选键，但这是每个调用方都要踩一次的坑。

---

## D6 — 未绑定 GFM 表不可迭代，但报错不指向根因

### 现象

```marqdo
`pres` =              # 注意：没有 name =，未绑定

| 前缀 |
|------|
| "9"  |

- [p](pres)           # 迭代未绑定表
→ error: foreach needs a list or map
```

未绑定表不会进入绑定图，因此不可迭代；但报错只说「需要一个 list 或 map」，
不提示「这张表没有被绑定到变量」。

### 建议

报错附上下文：「`foreach` 目标未绑定；请写作 `` `name` = `` 绑定 GFM 表」。

---

## D7 — 沙箱内 `subtask spawn path=` 静默不可用（设计如此）

`src/host/subtask.rs` 在沙箱内拒绝 `path=` / `code=` / `lang=` 的 spawn
（`subtask spawn disabled (exec capability off)`），只保留 `fn=` 进程内调用。
这是**能力硬顶的合理结果**，但：

- 该限制不出现在 `ext/ai/sandbox.mq.md` 的 `## profile` 说明里；
- 工具框架（`run_tool` → `subtask.spawn fn=`）恰好只用 `fn=`，因此**表面上一切正常**，
  直到有人写了基于 `path=` 的工具才发现。

建议在 `ext/ai/sandbox.mq.md` 顶部「能力硬顶」清单中补一条，与 exec/net/plugin 并列。

---

## D8 — 函数名含空格：定义成功，调用必崩

### 现象

```markdown
## 加一 号
    + `n`

*`n` + 1*

# main

**r = > 加一 号 n=1**
```

```
error: t8.mq.md:13:1: undefined variable `号`
```

英文同理：`## my func` 调用时报 `undefined variable func`。

### 根因

定义侧（`##` 标题）接受空格；调用侧按空格切词，
第一个词当函数名，其余当位置参数或**变量插值**。两侧不对称。

### 影响

这是**静默不对称**：`marqdo check` 通过（`无契约问题`），
只有真跑才崩，且报错指向「未定义变量」而非「函数名非法」，
排查方向被引偏。中文标识符天然爱加空格（`断言无 main`、`加一 号`），
LLM 生成代码时尤其容易命中。

### 建议

二选一：

1. **定义侧拒绝空格**——`##` 名字含空格时报
   `function name may not contain spaces`，定义期即失败。
2. **调用侧贪婪匹配**——按已知函数名最长前缀匹配，允许空格名（改动大，不推荐）。

倾向方案 1：与「标识符不含空格」的通行约定一致，成本最低。

---

## D9 — 未闭合双引号静默吞行，无任何报错

### 现象

```markdown
**s = "abc
> print text=hi
*""
```

输出 `hi`，退出码 0，**没有报错**，`s` 那一行凭空消失。

### 根因

未闭合引号使该行无法解析为语句，于是被当作**叙述**跳过。
叙述与「解析失败的语句」没有区分。

### 影响

- 变量「莫名未定义」，后续在别处报 `undefined variable`，根因被掩盖。
- 生成的文档会少一节内容却不报错——对「文档即程序」而言这是**最危险**的一类：
  文档看起来完整、能跑、退出码 0，但少了一段。

### 建议

未闭合引号应在解析期报错：
`unterminated string literal at line N`。
若担心破坏「叙述自由」，至少让 `marqdo check` 检出
「以 `**` 开头但不以 `**` 结尾」的行并 warn。

---

## D10 — 取元 `[key](map)` 与非原地 `table.put` 的认知负担

已在 [D5](#d5--成员式取元-keymap-在键缺失时硬崩) 记录取元硬崩。
补充**协同陷阱**：`table.put` **不原地修改**，返回新 map：

```markdown
**m = > json.parse text={"n":0}**
**r = > table.put in=`m` at="n" value=1**
```

`m` 仍是 `{"n":0}`，`r` 才是 `{"n":1}`。
于是「靠 `> table.put in=`state` ...` 累积计数」**不生效且无报错**——
计数器永远是 0，自检看起来「全部通过」。

建议：在 `lib/table` 的 `put` 文档里用一句话强调「返回新表」，
并考虑提供 `put!` 或 `set!` 的就地版本以匹配直觉。

---

## 附录：本次开发中确认正常的项

避免误报，以下行为经实测**符合文档**，不在缺陷之列：

| 项 | 实测 |
|----|------|
| 沙箱 CoW 隔离 | 宿主目录在 `commit` 前不变，`diff` 正确列出 write/remove |
| 沙箱能力默认值 | `exec:false` / `net:false` / `plugin:false` / `fs_write:true (CoW only)` |
| `sandbox.run` 成功形状 | `{ok:true, value, stdout}` |
| `agent.run efi=` | EKC preflight 拦截 CREATE，返回 `{status:"error", error:"efi_create_denied"}` |
| `agent.run sandbox=` | CREATE 阶段走 CoW trial，ok 才 commit |
| LLM 软失败 | transport 失败返回 `status:"error"` 而非中断 VM |
| `marqdo check` 契约检查 | 对 qdagent 全部改动文件返回「无契约问题」 |

---

## 复现清单

| ID | 复现 | 观察点 |
|----|------|--------|
| D1 | `sb.open root="." profile="readonly"` + `sb.inspect` | `profile.name` 回显 `default`，`fs_write:true` |
| D2 | 模型输出 `调用：tool_find`（全角） | 无工具结果、无报错 |
| D3 | 文档叙述区写 `9 号是数字。` 或 `+ 加号` | `unrecognized statement` / 门禁 `narration_marker` |
| D4 | `sb.run id= source="> print text=x"` | `error` 以 `<memory>` 开头，未提示缺 `# main` |
| D5 | `[error](r)`，`r` 为成功返回 | `missing map key 'error'` |
| D6 | 未绑定 GFM 表后 `- [x](表)` | `foreach needs a list or map` |
| D8 | `## my func` + `> my func n=1` | `undefined variable func`（check 却通过） |
| D9 | `**s = "abc` 不闭合 | 静默吞行，退出码 0 |
| D10 | `> table.put in=`m` at=…` 后读 `m` | 原表未变；计数永远是 0 |

---

## 变更记录

| 日期 | 变更 |
|------|------|
| 2026-10-22 | 首次记录 D1–D7（qdagent 文档门禁开发期） |
| 2026-10-22 | 补 D8–D10；D3 按逐条实测收敛（`-` / `>` / `\|` / `[` 实为安全，只数字与 `+` 报错）；配套使用手册 `tutorial/15-常见陷阱.md` |
