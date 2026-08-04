# Marqdo

**代码即文档。文档即代码。**

Marqdo：Markdown **标记即语法**。源文件 `.mq.md` 既是文稿也是程序。

欢迎访问[marqdo官方网站](https://www.marqdo.com/)阅读marqdo更多特性

语法宪法：[doc/design/markdown-mapping.md](doc/design/markdown-mapping.md) · 调用实参：[call-arguments.md](doc/design/call-arguments.md) · 测试布局：[examples-and-tests.md](doc/design/examples-and-tests.md) · 用户站：[user-site.md](doc/design/user-site.md) · 浏览：[view.md](doc/design/view.md) · 金样例：[tests/](tests/) · 用户文档：[public/](public/)

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
- **Phase I（树遍历）已通**：`tests/structure/` 与 `tests/keywords/` 金样例均可 `marqdo run`  
- **view**：`marqdo view public` 浏览用户文档；金样例用 `marqdo view tests`  
- **用户静态站**：可执行文稿在 `public/`（无 errors）；`view output` 生成 HTML（ignore），CI 发布到 `gh-pages` — 见 [user-site.md](doc/design/user-site.md)  
- 依赖（Rust，无 Flex/Bison）：[dependencies.md](doc/design/dependencies.md)  
- 选型：[ADR 0001 — Rust](doc/adr/0001-implementation-language.md)  

```bash
cargo run -- run tests/structure/hello.mq.md
cargo run -- run tests/structure/hello.mq.md --backend bytecode
cargo run -- view public --no-open
cargo run -- view output public -o public
powershell -File ./scripts/build-public.ps1
cargo run -- catalog tests -o .marqdo
```

文档：用户站见 [public/](public/) 与 [user-site.md](doc/design/user-site.md)；设计文稿见 [doc/](doc/)；`view` 皮肤见 [view.md](doc/design/view.md)；OKF 清单见 [catalog-cli.md](doc/design/catalog-cli.md)。

---

## 命名

**Marqdo** = Marq（Markdown）+ do（执行）

## 许可

[Apache License 2.0](LICENSE)
