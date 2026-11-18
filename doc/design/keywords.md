# Marqdo 关键字与内置名（v0 最小集）

| | |
|---|---|
| 状态 | 已定（中英双名） |
| 日期 | 2026-08-05 |
| 原则 | **少而精**；控制流靠 Markdown；内核中英**别名等价**（见 [keywords-i18n.md](keywords-i18n.md)） |
| 相关 | [markdown-mapping.md](markdown-mapping.md) · [keywords-i18n.md](keywords-i18n.md) · [stdlib-i18n.md](stdlib-i18n.md) |

---

## 1. 设计原则

1. **标记承担架构**：函数=`#`，**形参=`+`**，**分支=`1.`…**，循环=`-`，返回=`**…**`，语句=`*…*`。因此 **不**把 `def`/`if`/`else`/`while`/`for`/`return` 做成关键字。  
2. **内核双名**：字面量 / 逻辑词 / 五个核心内置函数均有中英别名，**无需导入**，可混用。  
3. **学 Python 的边界**：区分 **关键字** vs **内置函数名**。  
4. **宁少勿多**：扩展能力进可导入标准库（中英分文件，见 [stdlib-i18n.md](stdlib-i18n.md)）。

---

## 2. 关键字（词法保留，不可作变量名）

| 英文 | 中文 | 含义 |
|------|------|------|
| `True` | `真` | 布尔真 |
| `False` | `假` | 布尔假 |
| `None` | `空` | 空值 |
| `and` | `且` | 逻辑与 |
| `or` | `或` | 逻辑或 |
| `not` | `非` | 逻辑非 |

**不引入：** `if` `else` `while` `for` `def` `return` …（分支臂 `*` 不是单词 `else`。）

---

## 3. 内置函数（可调用，无需导入）

| 英文 | 中文 | 作用 |
|------|------|------|
| `print` | `打印` | 写 stdout |
| `input` | `输入` | 读一行；capture / view 用 frontmatter `stdin:` / `输入:`、表单或 `--stdin-file` 预置 |
| `len` | `长度` | 文本/表长度 |
| `str` | `文本` | 转为显示文本 |
| `int` | `整数` | 转为整数 |

**文本**：裸 token 按字面量；需 `\n` 等转义时用 `"..."`（见 [markdown-mapping.md](markdown-mapping.md) §7）。

形参别名：

| 英文 | 中文 | 用于 |
|------|------|------|
| `text` | `内容` | `print` / `打印` |
| `prompt` | `提示` | `input` / `输入` |
| `value` | `值` | `len`/`str`/`int` 及其中文名 |

```markdown
> print text=Hello
> 打印 内容=你好

*`n` = > 长度 值=`s`*
```

宿主原语 `type` / `trim` / `split` / `join` / `at` **无**内核中文别名；文本/表扩展走标准库文件（见 [stdlib-i18n.md](stdlib-i18n.md)）。转换错误见 [stdlib.md](stdlib.md)。

---

## 4. 与 Markup 调用的关系

```markdown
# main

> 打印 内容=Hello World!

*`n` = 1*

1. `n` > 0
  > print text=positive
2. *
  > 打印 内容=other
```

- `>` 是调用标记；`print` / `打印` 是同一内置的别名。  
- 用户 `# print` 或 `# 打印` 会遮蔽对应名字。

---

## 5. 标识符规则（摘要）

- 变量：`` `name` ``；不可为上表任一中英文关键字。  
- 函数名：`#` 标题；中英文均可。  
- 比较：`==` `>` `<` `>=` `<=`；逻辑：`and`/`且`、`or`/`或`、`not`/`非`。

---

## 6. 后续可增（未批准）

`float`、`range`；以及 `break`/`continue`。增加前开短 ADR，并同步考虑是否值得占用双名配额。
