# 知识分层示例：本体 → 知识个体 → 调用面

这个目录演示 [知识分层设计](../../doc/design/knowledge-layers.md)：

| 层 | 文件 | 内容 |
|----|------|------|
| **L0 原始语料** | `corpus/libai.md` | 李白原文；不执行，只作出处 |
| **L1 本体 / 模式** | `poet.mq.md` | `# 诗人` + 字段/类型表 |
| **L2 可执行知识** | `libai.mq.md` | `# 李白 = > 诗人` + `##` 事实 + 来源 |
| **L3 调用面** | `main.mq.md` / `main.compat.mq.md` | `[]()` 类调用 |

> **关键术语**：`# 李白 = > 诗人` 是**知识个体/单体知识类**（语法上是继承/特化），
> 不是运行时实例；`> 李白` 才是运行时 map 实例。

## 今天可跑（v2.0.0 兼容）

```bash
marqdo run examples/knowledge-layers/main.compat.mq.md
# 701
```

`main.compat.mq.md` 用当前能力完成同一件事：

```markdown
---
import libai:libai.mq.md
---

# main

**li = > libai.李白**
**year = > `li`.出生年份**
> print text=`year`
```

- `import` 绑定知识文档；
- `> libai.李白` 构造实例；
- `` `li`.出生年份 `` 调用实例方法；
- 当前 `libai.mq.md` 还保留 `import poet:poet.mq.md`，因为 v2.0.0 的跨文档继承需要 import；
  v2.1 的项目类索引会去掉这个 import。

## 目标（v2.1）

`main.mq.md`：

```markdown
# main

诗人李白的出生年份是[出生年份](李白.出生年份)。
```

- `李白` 是项目类索引里的知识个体；
- `出生年份` 是它的类函数；
- 正文链接由 `view`/`check` 解析；`run` 不执行；
- 要终端输出，把同一段代码放进代码链接并加 `print`。

期望：

```bash
$ marqdo run examples/knowledge-layers/main.mq.md
# 无输出（正文链接不执行）

$ marqdo check examples/knowledge-layers/main.mq.md --json
{"code":"link.ok", ...}
```

`view` 期望显示：

```text
诗人李白的出生年份是 701。
                        ↑
      悬浮：出生年份 → 李白.出生年份
            class 李白 · () -> number · value 701
            来源：corpus/libai.md#出生年份
```

## 为什么 `# 李白 = > 诗人` 不是「实例化」

- `# 诗人` 是类/本体；
- `# 李白 = > 诗人` 是**继承/特化**，表达「李白是诗人」；
- 但它带有一个命名个体的全部事实，所以是**单体知识类**；
- `李白.出生年份` 是类调用，不需要 `> 李白` 构造实例；
- 如果要运行时实例，仍可 `**li = > 李白**`，它是 map + `_type`，和知识个体分离。

详见 [doc/design/knowledge-layers.md](../../doc/design/knowledge-layers.md)。

## 相关

- 链接语法：[doc/design/link-syntax.md](../../doc/design/link-syntax.md)
- 证据复核：[doc/research/occlusion-vs-executable-knowledge-2026-10.md](../../doc/research/occlusion-vs-executable-knowledge-2026-10.md)
- 旧遮蔽设计：[doc/design/occlusion-syntax.md](../../doc/design/occlusion-syntax.md)
