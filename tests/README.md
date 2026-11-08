# 测试夹具（金样例）

金样例与集成测试同处 `tests/`，用子目录区分类别。设计见 [examples-and-tests.md](../doc/design/examples-and-tests.md)。

| 路径 | 内容 |
|------|------|
| [`structure/`](structure/) | 函数、调用、分支、循环、表、导入、位置实参 |
| [`keywords/`](keywords/) | `print`、布尔与 `and`/`or`/`not` |
| [`markup-v03/`](markup-v03/) | **Markup v0.3 草案金样例**（`#[ignore]` 至旗标落地；见目录 README） |
| [`errors/`](errors/) | 期望失败（诊断文案） |
| [`web_handbook.rs`](web_handbook.rs) | 直接执行双语 Web 手册，验证递归布局参考工程的离线/在线一致性；先构建 Web 插件 |
| [`quantum_handbook.rs`](quantum_handbook.rs) | 直接执行 Quantum 双语手册围栏，校验科学期望、真实契约及双后端；先构建量子插件 |
| [`gold.rs`](gold.rs) | 集成测试：跑上述夹具并比对 stdout / stderr |

```bash
cargo test
marqdo run tests/structure/hello.mq.md
marqdo view tests
marqdo catalog tests -o .marqdo
```

面向访客的可执行介绍在 [`public/`](../public/)（不含 `errors/`），见 [user-site.md](../doc/design/user-site.md)。
