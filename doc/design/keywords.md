# Marqdo 关键字与内置名（v0 最小集）

| | |
|---|---|
| 状态 | 已定（前期够用） |
| 日期 | 2026-08-04 |
| 原则 | **英文**；参考 [Python](https://realpython.com/python-keywords/) 但远更少；控制流仍靠 Markdown 标记，不引入 `if`/`while`/`def` |
| 相关 | [markdown-mapping.md](markdown-mapping.md) |

---

## 1. 设计原则

1. **标记承担架构**：函数=`#`，分支=`+`，循环=`-`，返回=`**…**`，语句=`*…*`。因此 **不**把 `def`/`if`/`else`/`while`/`for`/`return` 做成关键字。  
2. **英文标识**：内置与字面量用英文，便于工具高亮与国际协作。  
3. **学 Python 的边界**：Python 3 里 [`print` / `input` 是内置函数，不是关键字](https://realpython.com/python-keywords/)；布尔与空值是字面量关键字。Marqdo 同样区分 **关键字** vs **内置函数名**。  
4. **宁少勿多**：前期只列执行 examples 所需；要加再进 ADR。

---

## 2. 关键字（词法保留，不可作变量名）

| 关键字 | 含义 | 备注 |
|--------|------|------|
| `True` | 布尔真 | 对齐 Python 大小写 |
| `False` | 布尔假 | |
| `None` | 空值 | |
| `and` | 逻辑与 | 表达式内 |
| `or` | 逻辑或 | |
| `not` | 逻辑非 | |

**本期不引入：** `if` `else` `elif` `while` `for` `def` `class` `return` `import` `pass` `break` `continue` `in` `is` `lambda` …  
（`else` 分支臂仍用臂头单独的 `*` 标记，不是单词 `else`。）

---

## 3. 内置函数（可调用名，建议勿遮蔽）

| 名称 | 作用 | 调用示例 |
|------|------|----------|
| `print` | 写到 stdout（副作用） | `> print text=Hello World!` |
| `input` | 从 stdin 读一行（返回文本） | `*`name` = > input prompt=Name: *` |

参数约定（v0）：

- `print`：具名实参 `text`（必填）；日后可扩展 `end` 等。  
- `input`：可选 `prompt`；返回用户输入字符串（不含换行）。

旧文档中的中文 `print` **废弃**，统一为 `print`。

---

## 4. 与 Markup 调用的关系

```markdown
# main

> print text=Hello World!

*`n` = 1*

+ `n` > 0
  > print text=positive
+ *
  > print text=other
```

- `>` 是**调用标记**（架构）。  
- `print` 是**被调名字**（内置库）。  
- 用户自定义 `# print` 会遮蔽内置（实现应警告）。

---

## 5. 标识符规则（摘要）

- 变量：行内 `` `name` ``；`name` 不可为上表关键字。  
- 函数名：`#` 标题文本；建议 ASCII/英文，中文标题仍允许但内置名保持英文。  
- 比较：`==` `>` `<` `>=` `<=`；逻辑：`and` `or` `not`。

---

## 6. 后续可增（未批准）

`len`、`str`、`int`、`float`、`range`；以及 `break`/`continue`（若循环需要）。增加前开短 ADR。
