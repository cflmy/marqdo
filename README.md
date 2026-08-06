# Marqdo

**代码即文档。文档即代码。文档即知识库。**

Marqdo 把 Markdown **标记当作编程语法**：同一个 `.mq.md` 文件，既是给人读的文稿，也是给机器跑的程序。写文档的过程就是在写可执行逻辑——大型项目里，说明、示例与实现不再分叉成三份真相。

欢迎访问 [marqdo 官方网站](https://www.marqdo.com/) 阅读更多特性与可执行文档。

语法宪法：[doc/design/markdown-mapping.md](doc/design/markdown-mapping.md) · 对象：[objects.md](doc/design/objects.md) · 调用实参：[call-arguments.md](doc/design/call-arguments.md) · 用户站：[user-site.md](doc/design/user-site.md) · 浏览：[view.md](doc/design/view.md) · 调试：[view-debug.md](doc/design/view-debug.md) · OKF 清单：[catalog-cli.md](doc/design/catalog-cli.md) · VS Code 扩展：[vscode-extension.md](doc/design/vscode-extension.md)（分支 `vscode-extension`） · **AI Skill**：[skills/marqdo/](skills/marqdo/)（说明 [ai-skill.md](doc/design/ai-skill.md)） · 官方扩展：[ext-llm.md](doc/design/ext-llm.md)（`ext/llm`）· [ext-agent.md](doc/design/ext-agent.md)（智能体开发框架）· [ext-cli.md](doc/design/ext-cli.md)（`marqdo ext list/add/remove`） · 原生插件 ABI：[ext-abi.md](doc/design/ext-abi.md)（`lib/plugin`） · 金样例：[tests/](tests/) · 用户文档：[public/](public/)

---

## 为什么重要

| 痛点 | Marqdo 的答案 |
|------|----------------|
| 文档过期、示例跑不通 | **一份源文件**：叙述、结构与执行结果同源 |
| 知识散落在 Wiki / 注释 / 脚本里 | **`.mq.md` 即知识单元**，可浏览、可运行、可检索 |
| 人写给人看，AI / 工具另起一套 | **人机同读同写同构**：标记语言对人类友好，对解释器确定 |
| 大仓里「文档站」与「代码仓」脱节 | `view` 文档站 + `debug` 调试面 + `catalog` 清单，同一棵树长大 |

这不是「再做一个 Markdown 渲染器」，而是：**把文档升格为可解释的程序与可生长的知识库**，让协作在大型项目里依然站得住。

---

## 语法速览（v0.1）

| 标记 | 含义 |
|------|------|
| 无标记行 | 注释（空行分段） |
| `*…*` | 程序语句 |
| `**…**` | **返回值**（架构） |
| `****` / `**` + 空白 + `**` | **空返回**（`None`），并结束本函数体 |
| `> print …` 等 | 输出等副作用 = **普通函数**，非架构标记 |
| `#` | 对象 / 类型（构造体；`# main` = 入口） |
| `##` … | 函数 / 方法（按标题深度嵌套） |
| `` > `obj`.method … `` | 方法调用（`obj` 为带 `_type` 的 map） |
| `+` / `1.` … | 函数头下 `` + `名` `` = 形参；体内 `1.` `2.` … = 分支（`N. *` = else） |
| `-` | 循环（`` - `条件` `` 或 `` - [`项`](`集合`) ``） |
| 表格 | 集合 |
| 函数体内单独一行 `---` / `***` | **结束本函数体**（无返回值收束；见 mapping §11） |
| 文件开头成对 `---` … `---` | Frontmatter（元信息 / 导入），与函数结束符消歧 |

```markdown
# main

> 问候 谁=World

## 问候
    + `谁`

> print text=Hello `谁`!

---

## 加一
    + `n`

**`n` + 1**
```

完整约定：[markdown-mapping.md](doc/design/markdown-mapping.md)。

---

## 理念

1. **叙述默认安全**（无标记 = 注释）。  
2. **返回是架构；打印只是函数。**  
3. **代码即文档即知识库**：文稿可执行，执行可回看结构，结构可导航与调试。  
4. **对齐 [OKF](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)（Open Knowledge Format）方向**：清单由工具从 `.mq.md` **自动生成**，不是手填配置——见下节与 [catalog-cli.md](doc/design/catalog-cli.md)。  
5. **正经解释器（Rust）**：词法/行分类 → 语法 → 语义 → 树遍历（及字节码）；**不用 Flex/Bison**。见 [路线图](doc/roadmap/interpreter.md)。

---

## OKF 风格清单（自动生成）

从源码派生 YAML / Markdown 知识包（勿手改生成物）：

```bash
marqdo catalog [PATH] -o OUT_DIR
marqdo sync [PATH] -o OUT_DIR          # catalog 的别名
```

| 参数 | 默认 | 说明 |
|------|------|------|
| `PATH` | `.` | 工程根或含 `.mq.md` 的目录 |
| `-o` / `--out` | `.marqdo` | 输出目录 |

产物示例：`catalog.yaml`、`index.md`、`modules/*.md`（`type: Marqdo Module` 等）。设计见 [generated-yaml-manifest.md](doc/design/generated-yaml-manifest.md) · 调研 [okf-and-marqdo.md](doc/research/okf-and-marqdo.md)。

```bash
cargo run -- catalog public -o .marqdo
# 或已安装二进制时：
marqdo catalog public -o .marqdo
```

---

## 现状（v0.1.1）

- 映射与解释器：Phase I 树遍历 + 字节码后端可用；金样例在 `tests/`  
- **对象**：`#` = 类型/构造，`##`+ = 函数/方法；见 [objects.md](doc/design/objects.md)  
- **`marqdo view`**：文档浏览器（Structure + 函数大纲/搜索 + Execution + Source）  
- **`marqdo debug`**：独立调试页（断点 / 单步 / locals；默认端口 7430；页面 favicon / 品牌使用官方 Logo）  
- **`marqdo catalog` / `sync`**：OKF 风格 YAML + 模块概念页  
- 标准库：文本、文件、系统（含 `load_dotenv`）、时间、JSON（含 `quote`）、网络（HTTPS + headers）、数学（公式/绘图）、外联（Python 等）、**插件**（`lib/plugin` 加载原生 ABI）  
- **官方扩展 `ext/`**（非 stdlib）：[`ext/llm`](doc/design/ext-llm.md)；[`ext/agent`](doc/design/ext-agent.md) 智能体开发框架。安装：`marqdo ext list` / `add llm|agent` / `remove`（见 [ext-cli.md](doc/design/ext-cli.md)；默认 `~/.marqdo/ext` 或 `MARQDO_EXT`）  
- **原生插件 ABI v1**：[`include/marqdo_abi.h`](include/marqdo_abi.h) · [ext-abi.md](doc/design/ext-abi.md)；demo `plugins/demo`  
- **用户静态站**：`public/` → `view output` → CI 发布 [gh-pages](https://cflmy.github.io/marqdo/)  
- **VS Code 扩展**：分支 `vscode-extension`；Release 附带 `.vsix`（Run / View / Debug + CLI 安装助手）  
- 选型：[ADR 0001 — Rust](doc/adr/0001-implementation-language.md)

```bash
cargo run -- run tests/structure/hello.mq.md
cargo run -- run tests/structure/hello.mq.md --backend bytecode
cargo run -- view public --no-open
cargo run -- debug public --no-open
cargo run -- catalog public -o .marqdo
cargo run -- view output public -o public
powershell -File ./scripts/build-public.ps1
```

**发布包（GitHub Releases）**：单独 `.exe` **不含** `lib/` / `ext/` / 原生插件 `.dll`；请下载带 `lib/`（及可选 `ext/`）的 zip，或另下 `*-stdlib.zip` 解压到可执行文件旁（亦可设 `MARQDO_LIB` / `MARQDO_EXT`）。OpenAI 兼容聊天见 [ext-llm.md](doc/design/ext-llm.md)（项目目录自建 `.env`，勿提交密钥）。智能体布局见 [ext-agent.md](doc/design/ext-agent.md)（`MARQDO_AGENT_PLUGIN` 指向构建出的 `agent` 共享库）。

文档：用户站 [public/](public/) · 设计 [doc/](doc/) · OKF / catalog [catalog-cli.md](doc/design/catalog-cli.md) · 调试 [view-debug.md](doc/design/view-debug.md)

---

## 命名

**Marqdo** = Marq（Markdown）+ do（执行）

## 许可

[Apache License 2.0](LICENSE)
