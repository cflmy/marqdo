# Markup v0.3 金样例

| | |
|---|---|
| 设计 | [markdown-mapping-v0.3.md](../../doc/design/markdown-mapping-v0.3.md) |
| 路线 | [markup-v0.3.md](../../doc/roadmap/markup-v0.3.md) |
| 状态 | **现行运行时（无 `--markup` 旗标 · 无 v0.2 兼容）** |

## 约定

- 文件使用 **v0.3 表面语法**（粗体代码、斜体返回、叙述 `` `名` `` 声明、形参体推断）。
- `tests/gold.rs` 中 `markup_v03_*` 为门禁。

## 清单

见各 `.mq.md` frontmatter；汇总期望以 `gold.rs` 为准。

```bash
cargo test --test gold markup_v03
marqdo run tests/markup-v03/inc-prose.mq.md
```
