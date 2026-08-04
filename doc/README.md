# Marqdo 开发文档

本目录存放各阶段设计与调研文档。

## 目录约定

| 路径 | 用途 |
|------|------|
| [`research/`](research/) | 外部标准与调研 |
| [`design/`](design/) | 语言设计（语法映射、清单模型等） |
| [`adr/`](adr/) | 架构决策记录 |
| `roadmap/` | 里程碑（待建） |

## 当前文档

| 文档 | 摘要 |
|------|------|
| [design/markdown-mapping.md](design/markdown-mapping.md) | **v0.1 语法宪法** |
| [design/code-vs-comment.md](design/code-vs-comment.md) | 叙述 / 可执行 / 外联 |
| [design/return-hr-and-code-surface.md](design/return-hr-and-code-surface.md) | 语句/返回/输出/分割线决议记录 |
| [design/generated-yaml-manifest.md](design/generated-yaml-manifest.md) | 生成式 OKF 清单 |
| [design/tech-stack.md](design/tech-stack.md) | 实现栈对比分析 |
| [adr/0001-implementation-language.md](adr/0001-implementation-language.md) | 选型 ADR（Python Spike **已通过**） |
| [research/okf-and-marqdo.md](research/okf-and-marqdo.md) | OKF 调研 |

示例程序见仓库 [`examples/`](../examples/)。

## 写作原则

1. 先思想，后实现。  
2. 草案可演进；否决项进 ADR。  
3. 示例统一 `.mq.md`。
