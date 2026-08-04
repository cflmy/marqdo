# Marqdo

**代码即文档。文档即代码。**

Marqdo 是一门编程语言：源文件是可执行的程序，也是可被 Markdown 渲染的文档。语法由 Markdown **标记**本身承担（Markup-as-Syntax），而不是在文档里嵌入另一套关键字。

第一版语法宪法：[doc/design/markdown-mapping.md](doc/design/markdown-mapping.md) · 示例：[examples/](examples/)

---

## 为什么需要一门新语言

传统工作流里，代码与文档是两套工件，文档写完即开始过时。Literate Programming、注释生成文档、Markdown 围栏演示都无法做到「同一份文本既是程序又是说明」。

Marqdo 的路径：**标记即语法**——标题是函数，粗体是输出，斜体是返回值，列表是控制流，表格是集合。

---

## 语法速览（v0）

| 标记 | 含义 |
|------|------|
| `#` / `##` | 函数与作用域 |
| `-` 纯名（函数头下） | 形参 |
| `-`（体内） | 循环 |
| `+` 或 `1.` | 分支（`*` 独行 = else） |
| `**…**` | 输出 |
| `*…*` | 返回值 |
| `` `name` `` | 变量；`=` 绑定，`==` 比较 |
| `>` | 调用；frontmatter 里 `> file.mq.md` 为跨文件导入 |
| 表格 | 集合 |

```markdown
---
title: Hello World
---

# main

> 输出函数 输出内容=Hello World!

## 输出函数
    - 输出内容

**`输出内容`**
```

更多：[examples/index.mq.md](examples/index.mq.md)、[examples/branch.mq.md](examples/branch.mq.md)、[examples/loop.mq.md](examples/loop.mq.md)。

---

## 核心理念

1. **同一份源，两种阅读** — `.mq.md` 对人是 Markdown，对机器是程序。  
2. **手写源，生成清单** — 依赖/模块目录以 OKF 风格由工具生成，不是手填配置（[说明](doc/design/generated-yaml-manifest.md)）。  
3. **行首判别** — 有标记则执行，无标记则注释（注释内标记无语义）。  
4. **设计优先于实现** — 先定映射宪法，再选型实现。

---

## 现状

- 语法映射 v0：**已定论** — [markdown-mapping.md](doc/design/markdown-mapping.md)  
- 调研：[OKF](doc/research/okf-and-marqdo.md)  
- 下一步：实现技术选型、解析器与测试  

文档目录：[doc/](doc/)

---

## 命名

**Marqdo** = **Marq**（mark / Markdown）+ **do**（执行）。

---

## 许可

[Apache License 2.0](LICENSE)
