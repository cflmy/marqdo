# 调用实参：具名与位置

| | |
|---|---|
| 状态 | 已定（v0.1 增补） |
| 日期 | 2026-08-04 |
| 相关 | [markdown-mapping.md](markdown-mapping.md) · [keywords.md](keywords.md) |

---

## 1. 目标

`>` 调用同时支持：

| 形式 | 示例 | 说明 |
|------|------|------|
| **具名**（保留） | `> 问候 谁=Marqdo` | `名=值`；值可含空格（至下一 `名=`） |
| **位置**（新增） | `> 问候 Marqdo` | 按形参声明顺序绑定 |

二者可混用：位置实参按「尚未被具名绑定的形参」从左到右填充。具名不可绑定同一形参两次；位置实参不得出现在「已出现具名实参」之后（对齐常见「位置在前、关键字在后」习惯）。

---

## 2. 文法（调用尾部）

```
call_tail  ::= callee arg*
arg        ::= named | positional
named      ::= ident '=' value
positional ::= value_token
```

- `value`（具名右侧）：可含空格与 `` `var` `` 插值，直到下一个 `ident=` 边界。  
- `value_token`（位置）：空白分隔的一个词法单元——数字、`` `名` ``、标识/文本词；**不含空格**。多词字符串请用具名，例如 `> print text=Hello World!`。  
- 内置 `print` 形参名为 `text`：`> print Hi` 与 `> print text=Hi` 等价。

---

## 3. 绑定算法

1. 收集具名 → 映射；冲突则报错。  
2. 按函数形参表顺序：若具名已提供则用之，否则取下一个位置实参。  
3. 位置实参有余、或必填形参仍缺 → 报错（内置 `input` 的 `prompt` 可缺省）。  
4. 用户函数：额外具名键仍写入调用环境（便于扩展）；位置实参不可超过形参数目。

---

## 4. 金样例

见 `examples/structure/positional-call.mq.md` 与更新后的 `examples/structure/import/`。
