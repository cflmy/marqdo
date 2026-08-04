# 解释器流水线可视调试

| | |
|---|---|
| 状态 | 已定（开发期强制友好） |
| 日期 | 2026-08-04 |
| 参考 | [Crafting Interpreters — Scanning](https://www.craftinginterpreters.com/scanning.html)（先打 token）· [AstPrinter](https://craftinginterpreters.com/representing-code.html) · CLI `--dump-ast` 类实践 |

---

## 1. 目标

开发 `marqdo` 时，对任意 `.mq.md` 应能**分阶段看见**中间结果，避免黑盒；正式 `run` 默认安静，仅输出程序 `print` 的内容。

---

## 2. CLI 开关

```bash
marqdo run FILE [dumps...]
marqdo dump FILE --stage lines|tokens|ast|sema|all
```

| 标志 | 阶段 | 输出内容 |
|------|------|----------|
| `--dump-lines` | 行分类 | 每行：`LINE kind=Comment\|Code\|Blank` + 原文预览 |
| `--dump-tokens` | 词法 | Token 流：`type lexeme line:col` |
| `--dump-ast` | 语法 | S 表达式或缩进树（模块/函数/语句） |
| `--dump-sema` | 语义 | 作用域、导入图、提升后的符号表 |
| `--trace-eval` | 求值 | 进入/离开函数、绑定、分支臂选择、`print` 调用 |
| `--dump-all` | 以上全部 | 按管道顺序打印，段之间用横幅分隔 |

格式约定：

- 人类可读优先；段首统一：`=== marqdo: <stage> ===`  
- 可选 `--dump-format=text|json`（json 后置，text 先做）  
- 有 dump 时：若求值未实现，仍打印已实现阶段再以非 0 退出（便于 M1 只测 lines）

---

## 3. 横幅示例

```
=== marqdo: lines (examples/hello.mq.md) ===
   1  Blank
   2  Comment  | # Hello World
   3  Blank
   4  Comment  | 本程序调用...
   5  Blank
   6  Code     | > print text=Hello World!
=== marqdo: end lines ===
```

---

## 4. 实现要点

- dump 逻辑放在 `src/debug/`（或各阶段 `fmt`），**不**与安静求值路径搅在一起。  
- 每个阶段纯函数：`input → (artifacts, diagnostics)`，CLI 负责打印。  
- 单测可断言 dump 字符串包含关键行（稳定子集），避免绑死全文格式。

---

## 5. 与里程碑

| 里程碑 | 必须可用的 dump |
|--------|-----------------|
| M1 | `--dump-lines`（及可选 tokens 雏形） |
| M2 | `--dump-ast` |
| M3–M4 | `--dump-sema`、`--trace-eval` |
| 全程 | `--dump-all` 随阶段增长自动变完整 |
