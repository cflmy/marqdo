---
title: table cell expressions
status: Accepted
date: 2026-08-12
related: markdown-mapping.md · roadmap/tables-maps-footnotes.md · call-arguments.md
---

# 表单元格表达式（T5）

| | |
|---|---|
| 状态 | **Accepted** |
| 日期 | 2026-08-12 |
| 相关 | [markdown-mapping.md](markdown-mapping.md) · [tables-maps-footnotes.md](../roadmap/tables-maps-footnotes.md) · [call-arguments.md](call-arguments.md) |

---

## 1. 问题

v1 把 GFM 数据单元格一律收成 **字面量**（`Int` / `Text`）。结果虽是 `Map` / `List`，却不能写：

```markdown
`h` =

| api_key | model |
|---------|-------|
| `api_key` | `model` |
```

作者只好用 `json.set` 链或 JSON 字符串拼接——这是**实现缺口**，不是「字典只能装字面量」的语义必然。

---

## 2. 锁定规则

### 2.1 数据单元格 = 与调用实参相同的值语法

每个**数据**单元格（非表头、非分隔行）按 [`parse_call_arg_value`](../../src/parse/expr.rs) 解析，再在绑定求值时执行：

| 单元格写法 | 含义 |
|------------|------|
| `苹果` / `Alpha` | 裸词 → 文本字面量（与今日竖表兼容） |
| `42` | 整数 |
| `` `api_key` `` | 变量引用 |
| `` `a` + `b` `` | 普通表达式 |
| `True` / `假` / `None` | 布尔 / 空 |
| `"https://…"` / `".marqdo/x"` / `"1/5"` / `"16/9"` | 引号字符串（可含 `/`、插值；**CSS 比率等含 `/` 的值请用引号**，勿依赖空格消歧） |
| `https://api.openai.com/v1` | **路径折叠**：无空格的 `Text/Text/…` 除法链收成整段文本（同调用实参） |
| `gpt-4o-mini` | **连字符散文**：不拆成减法（同调用实参） |
| `` /about ``、`` /chat/completions `` | 以 `/` 开头的路径散文 → 整段文本 |
| `1/5` | **整数除法**（同表达式 / 调用实参；今日整除得 `0`）——要文本请写 `"1/5"` |
| 空单元格 | 空文本 `""` |

### 2.2 表头仍是键名文本

表头单元格**不**求值：键 = trim 后的表头字符串（与今日一致）。不允许 `` | `k` | `` 当动态键（v1 非目标；需要时另文）。

### 2.3 几何语义不变

竖表 / 横表单行 Map / 多行列向 Map / `@`·`行`·`row` 行向记录——**形状规则不变**；只把「单元格 → `Expr::Literal`」换成「单元格 → 完整 `Expr`」。

求值已由解释器对 `Expr::List` / `Expr::Map` 递归完成，字节码后端已发射列表/字典，**无需新 AST 节点**。

### 2.4 求值时机

表出现在 `` `名` = `` 右侧时，在**该绑定语句**求值：单元格内变量读**当时**环境。构造函数体里的表与语句表规则相同。

### 2.5 错误

- 单元格表达式非法 → 与别处表达式相同的 `path:line:col` 诊断（行号取该表行）。  
- 变量未定义 → 求值期报错。  
- 不引入「单元格静默当字面量」的回退（除 §2.1 已列的路径/连字符折叠，与调用实参对齐）。

---

## 3. 作者示例（目标体验）

```markdown
*`api_key` = > sys.env_get name=OPENAI_API_KEY *
*`model` = "gpt-4o-mini" *

`handle` =

| api_key | base_url | model | suffix |
|---------|----------|-------|--------|
| `api_key` | https://api.openai.com/v1 | `model` | /chat/completions |
```

`handle` 为含运行时 `api_key` / `model` 的字典；静态 URL 与 suffix 仍可裸写（路径折叠）。

嵌套：

```markdown
*`prompt` = hello *

`msg` =

| role | content |
|------|---------|
| user | `prompt` |

*`messages` = > json.append list=None item=`msg` *
```

（列表字面量表仍可用竖表；需要「单行记录」时用横表或 `@` 行向表。）

---

## 4. 兼容性

| 旧写法 | 行为 |
|--------|------|
| 纯中文/英文词、整数 | **不变** |
| `gpt-4o-mini`、URL、`/path` | **不变**（折叠规则与调用实参一致） |
| 真正要做除法 / 减法 | 裸写 `1/5` 或空格/括号：`` `n` / 2 ``、`` `a` - 1 ``；或先在表外算再 `` `x` `` 填入 |
| CSS / 含 `/` 的散文值 | **引号优先**：`"1/5"`、`"16/9"`、`"image/png"`（不靠空格把算术「关掉」） |

破坏性：几乎无。唯一需注意的是「想在单元格里写算术且两侧是裸词」——与调用实参同一套消歧；含 `/` 的**文本**一律用引号。

---

## 5. 非目标（本阶段）

- 表头动态键  
- 单元格内 `>` 调用（可用表外算完再引用）  
- 单元格内多语句 / 分支  
- 把叙述区（注释段）里的 GFM 表当成可执行（仍为注释）

---

## 6. 验收

| 项 | 标准 |
|----|------|
| 金样 | 横表单元格 `` `var` `` 读环境；竖表旧金样全绿 |
| 路径 | `https://…`、`/chat/completions`、`gpt-4o-mini` 仍为文本 |
| 含 `/` 文本 | 表单元格用引号：`"1/5"`、`"16/9"`；裸 `1/5` 为除法 |
| 扩展 | `ext/ai/llm` 构造可用表写入运行时字段，去掉无必要的 `json.set` 链 |
| 文档 | 本页 + roadmap T5 + markdown-mapping §9 同步 |

实现跟踪见 [roadmap/tables-maps-footnotes.md](../roadmap/tables-maps-footnotes.md) **T5**。
