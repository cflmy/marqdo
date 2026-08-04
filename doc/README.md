# Marqdo 开发文档

本目录存放各阶段设计与调研文档。

## 目录约定

| 路径 | 用途 |
|------|------|
| [`research/`](research/) | 外部标准与调研 |
| [`design/`](design/) | 语言设计（语法映射、清单模型等） |
| `adr/` | 架构决策记录（待建） |
| `roadmap/` | 里程碑（待建） |

## 当前文档

| 文档 | 摘要 |
|------|------|
| [design/markdown-mapping.md](design/markdown-mapping.md) | **v0 语法宪法**：Markup-as-Syntax 全映射 |
| [design/generated-yaml-manifest.md](design/generated-yaml-manifest.md) | 机器生成的 OKF 风格依赖/文档清单 |
| [design/tech-stack.md](design/tech-stack.md) | 参考实现技术选型（TypeScript + remark + Vitest） |
| [research/okf-and-marqdo.md](research/okf-and-marqdo.md) | Google OKF 调研与借鉴 |

示例程序见仓库 [`examples/`](../examples/)。

## 写作原则

1. 先思想，后实现。  
2. 草案可演进；否决项进 ADR。  
3. 示例统一 `.mq.md`。
