# Marqdo

**代码即文档。文档即代码。**

Marqdo：Markdown **标记即语法**。源文件 `.mq.md` 既是文稿也是程序。

语法宪法：[doc/design/markdown-mapping.md](doc/design/markdown-mapping.md) · 调用实参：[call-arguments.md](doc/design/call-arguments.md) · 示例布局：[examples-and-tests.md](doc/design/examples-and-tests.md) · 浏览：[view.md](doc/design/view.md) · 示例：[examples/](examples/)

---

## 语法速览（v0.1）

| 标记 | 含义 |
|------|------|
| 无标记行 | 注释 |
| `*…*` | 程序语句 |
| `**…**` | **返回值**（架构） |
| `> print …` 等 | 输出等副作用 = **普通函数**，非架构标记 |
| `#` / `##` | 函数与作用域 |
| `+` / `1.` | 分支（臂头单独 `*` = else） |
| `-` | 循环（头下纯名则为形参） |
| 表格 | 集合 |
| `---` / `***` | 可选：框住分支/循环（不强制） |

```markdown
# main

> print text=Hello World!

## 加一
    - n

**`n` + 1**
```

---

## 理念

1. 叙述默认安全（无标记 = 注释）。  
2. 返回是架构；打印只是函数。  
3. 依赖清单由工具生成（OKF 风格），不是手填配置。  
4. **正经解释器（Rust）**：词法/行分类 → 递归下降语法 → 语义 → 树遍历 → 日后字节码；**不用 Flex/Bison**。见 [路线图](doc/roadmap/interpreter.md) 与 [依赖说明](doc/design/dependencies.md)。

---

## 现状

- 映射 v0.1：[markdown-mapping.md](doc/design/markdown-mapping.md)  
- 解释器路线图：[interpreter.md](doc/roadmap/interpreter.md)  
- **Phase I（树遍历）已通**：`examples/structure/` 与 `examples/keywords/` 金样例均可 `marqdo run`  
- **view**：`marqdo view examples` 按 AST 浏览结构与输出  
- 依赖（Rust，无 Flex/Bison）：[dependencies.md](doc/design/dependencies.md)  
- 选型：[ADR 0001 — Rust](doc/adr/0001-implementation-language.md)  

```bash
cargo run -- run examples/structure/hello.mq.md
cargo run -- run examples/structure/hello.mq.md --backend bytecode
cargo run -- view examples --no-open
```

当前发布：**v0.0.1**（Phase I 树遍历 + `view` + M5 字节码原型）。M5 见 [bytecode.md](doc/design/bytecode.md)；errors 见 `examples/errors/`。


---

## 命名

**Marqdo** = Marq（Markdown）+ do（执行）

## 许可

[Apache License 2.0](LICENSE)
