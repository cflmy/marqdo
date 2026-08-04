# Marqdo examples（金样例）

本目录即测试夹具，见 [doc/design/examples-and-tests.md](../doc/design/examples-and-tests.md)。

| 目录 | 内容 |
|------|------|
| [`structure/`](structure/) | 函数、调用、分支、循环、表、导入、位置实参 |
| [`keywords/`](keywords/) | `print`、布尔与 `and`/`or`/`not` |
| [`errors/`](errors/) | 期望失败（未定义名 / 未知函数 / 实参 / 语法） |

```bash
cargo test
marqdo run examples/structure/hello.mq.md
marqdo view examples
```
