# Marqdo 开发文档

本目录存放 Marqdo 各阶段的设计与调研文档，与实现代码分离，便于在语言定型前持续迭代思想。

## 目录约定

| 路径 | 用途 |
|------|------|
| [`research/`](research/) | 外部标准、论文、相邻系统的调研与借鉴分析 |
| [`design/`](design/) | 语言设计草案、语法、语义、模块模型 |
| `adr/` | 架构决策记录（Architecture Decision Records，待建） |
| `roadmap/` | 阶段目标与里程碑（待建） |

## 当前文档

| 文档 | 摘要 |
|------|------|
| [research/okf-and-marqdo.md](research/okf-and-marqdo.md) | Google OKF 调研，以及对 Marqdo 的可借鉴点 |
| [design/generated-yaml-manifest.md](design/generated-yaml-manifest.md) | 机器生成的 OKF 风格 YAML 清单（依赖 / 项目文档，非配置文件） |

## 写作原则

1. **先思想，后实现** — 文档应说清「为什么」与「约束」，避免过早绑定未验证的语法细节。
2. **可演进** — 草案用日期或版本标注；被否决的路径保留在 ADR，不默默删除。
3. **与源文件同构** — 文档本身尽量用清晰 Markdown；示例源文件统一使用 `.mq.md` 后缀。
