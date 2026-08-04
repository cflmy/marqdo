# Marqdo

**代码即文档。文档即代码。**

Marqdo：Markdown **标记即语法**。源文件 `.mq.md` 既是文稿也是程序。

语法宪法：[doc/design/markdown-mapping.md](doc/design/markdown-mapping.md) · 示例：[examples/](examples/)

---

## 语法速览（v0.1）

| 标记 | 含义 |
|------|------|
| 无标记行 | 注释 |
| `*…*` | 程序语句 |
| `**…**` | **返回值**（架构） |
| `> 输出 …` 等 | 输出等副作用 = **普通函数**，非架构标记 |
| `#` / `##` | 函数与作用域 |
| `+` / `1.` | 分支（臂头单独 `*` = else） |
| `-` | 循环（头下纯名则为形参） |
| 表格 | 集合 |
| `---` / `***` | 可选：框住分支/循环（不强制） |

```markdown
# main

> 输出 内容=Hello World!

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
- 依赖（Rust，无 Flex/Bison）：[dependencies.md](doc/design/dependencies.md)  
- 选型：[ADR 0001 — Rust](doc/adr/0001-implementation-language.md)  

下一步：按 [dev-setup.md](doc/dev-setup.md) 配置环境；M0 脚手架已就绪，M1 起实现词法。
  

---

## 命名

**Marqdo** = Marq（Markdown）+ do（执行）

## 许可

[Apache License 2.0](LICENSE)
