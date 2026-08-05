# Marqdo 开发文档

本目录存放各阶段设计、选型与路线图。

## 目录约定

| 路径 | 用途 |
|------|------|
| [`research/`](research/) | 外部标准与调研 |
| [`design/`](design/) | 语言设计（语法映射等） |
| [`adr/`](adr/) | 架构决策记录 |
| [`roadmap/`](roadmap/) | 实现路线与里程碑 |

## 当前文档

| 文档 | 摘要 |
|------|------|
| [design/markdown-mapping.md](design/markdown-mapping.md) | **v0.1 语法宪法** |
| [design/ai-skill.md](design/ai-skill.md) | **AI 接入 Skill**（[`skills/marqdo/`](../skills/marqdo/)） |
| [design/code-vs-comment.md](design/code-vs-comment.md) | 叙述 / 可执行 / 外联 |
| [design/return-hr-and-code-surface.md](design/return-hr-and-code-surface.md) | 语句/返回/输出决议 |
| [design/generated-yaml-manifest.md](design/generated-yaml-manifest.md) | 生成式 OKF 清单 |
| [design/tech-stack.md](design/tech-stack.md) | 实现栈对比 |
| [adr/0001-implementation-language.md](adr/0001-implementation-language.md) | **Accepted：Rust 参考解释器；不用 Flex/Bison** |
| [design/dependencies.md](design/dependencies.md) | 依赖详单（无 Flex/Bison） |
| [design/keywords.md](design/keywords.md) | **英文最小关键字 / 内置函数** |
| [design/keywords-i18n.md](design/keywords-i18n.md) | **内核中英双名（无需导入；少而精）** |
| [design/stdlib.md](design/stdlib.md) | **P2 标准库：内置 + `lib/` + 错误约定** |
| [design/stdlib-i18n.md](design/stdlib-i18n.md) | **标准库中英命名：靠库文件名区分，无 lang 开关** |
| [design/stdlib-modules.md](design/stdlib-modules.md) | **本波 L1：文件/系统/时间/JSON/网络（JSON 中英同库）** |
| [design/stdlib-math.md](design/stdlib-math.md) | **方案：数学库（数值/随机/作图/轻量求解）** |
| [design/stdlib-foreign.md](design/stdlib-foreign.md) | **暂缓：外联胶水（\`\`\`lang 具名块）** |
| [design/call-arguments.md](design/call-arguments.md) | **调用：具名 + 位置实参** |
| [design/examples-and-tests.md](design/examples-and-tests.md) | **tests/ 金样例目录布局** |
| [design/view.md](design/view.md) | **`marqdo view` / `view output` 黑白极简 + 静态文档** |
| [design/user-site.md](design/user-site.md) | **用户站：`public/` → `gh-pages`** |
| [design/catalog-cli.md](design/catalog-cli.md) | **`marqdo catalog` / `sync` OKF 清单命令** |
| [design/view-debug.md](design/view-debug.md) | **`marqdo debug` 独立调试页** |
| [design/vscode-extension.md](design/vscode-extension.md) | **VS Code 扩展：分支 `vscode-extension`（main 不跟踪源码）** |
| [design/bytecode.md](design/bytecode.md) | **M5 字节码 / 双后端** |
| [design/pipeline-debug.md](design/pipeline-debug.md) | **流水线 `--dump-*` 可视调试** |
| [dev-setup.md](dev-setup.md) | 开发环境配置（Rust M0） |
| [roadmap/interpreter.md](roadmap/interpreter.md) | 正经解释器路线图 |
| [roadmap/next-phase.md](roadmap/next-phase.md) | **下一阶段：input / 诊断 / 标准库 / view 调试** |
| [research/okf-and-marqdo.md](research/okf-and-marqdo.md) | OKF 调研 |

金样例与集成测试：[`tests/`](../tests/)。面向访客的可执行文档：[`public/`](../public/)。Spike 存档：[`spike/`](../spike/)。

## 写作原则

1. 先宪法与路线图，再实现。  
2. 金样例不过 = 未完成。  
3. 示例统一 `.mq.md`。
