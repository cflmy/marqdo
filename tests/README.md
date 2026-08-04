# 测试夹具（金样例）

金样例与集成测试同处 `tests/`，用子目录区分类别。设计见 [examples-and-tests.md](../doc/design/examples-and-tests.md)。

| 路径 | 内容 |
|------|------|
| [`structure/`](structure/) | 函数、调用、分支、循环、表、导入、位置实参 |
| [`keywords/`](keywords/) | `print`、布尔与 `and`/`or`/`not` |
| [`errors/`](errors/) | 期望失败（诊断文案） |
| [`gold.rs`](gold.rs) | 集成测试：跑上述夹具并比对 stdout / stderr |

```bash
cargo test
marqdo run tests/structure/hello.mq.md
marqdo view tests
marqdo catalog tests -o .marqdo
```

面向访客的可执行介绍在 [`public/`](../public/)（不含 `errors/`），见 [user-site.md](../doc/design/user-site.md)。
