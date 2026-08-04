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
| [design/code-vs-comment.md](design/code-vs-comment.md) | 叙述 / 可执行 / 外联 |
| [design/return-hr-and-code-surface.md](design/return-hr-and-code-surface.md) | 语句/返回/输出决议 |
| [design/generated-yaml-manifest.md](design/generated-yaml-manifest.md) | 生成式 OKF 清单 |
| [design/tech-stack.md](design/tech-stack.md) | 实现栈对比 |
| [adr/0001-implementation-language.md](adr/0001-implementation-language.md) | **Accepted：Rust 参考解释器；不用 Flex/Bison** |
| [design/dependencies.md](design/dependencies.md) | 依赖详单（无 Flex/Bison） |
| [design/keywords.md](design/keywords.md) | **英文最小关键字 / 内置 `print` `input`** |
| [design/pipeline-debug.md](design/pipeline-debug.md) | **流水线 `--dump-*` 可视调试** |
| [dev-setup.md](dev-setup.md) | 开发环境配置（Rust M0） |
| [roadmap/interpreter.md](roadmap/interpreter.md) | 正经解释器路线图 |
| [research/okf-and-marqdo.md](research/okf-and-marqdo.md) | OKF 调研 |

示例与验收集：[`examples/`](../examples/)。Spike 存档：[`spike/`](../spike/)。

## 写作原则

1. 先宪法与路线图，再实现。  
2. 金样例不过 = 未完成。  
3. 示例统一 `.mq.md`。
