# 链接语法示例：两文档，一次类调用

这个目录是 [链接语法设计声明](../../doc/design/link-syntax.md) 的**目标示例**：

> `[李白的出生年份](李白.李白的出生年份)`
> 不是新的语法：`[]` 是 Markdown 链接的描述，`()` 是已有调用；
> `# 李白` 是类，`## 李白的出生年份` 是函数，`李白.李白的出生年份` 是类调用。

## 文件

| 文件 | 作用 | v2.0.0 可跑 | v2.1 目标 |
|------|------|-------------|-----------|
| `libai.mq.md` | 知识文档：`# 李白` → `## 李白的出生年份` → `*701*` | ✅ | ✅ |
| `main.mq.md` | 用户原始示例：正文链接 + 类调用 | ❌（安静无输出） | ✅ view 显示；`run` 加 print 输出 |
| `main.compat.mq.md` | 当前等价写法：`import` + 实例 + 本地链接 | ✅（输出 `701`） | ✅ |
| `expected-link-info.json` | 目标 `LinkInfo` 形状 | — | — |
| `links.lock.yaml` | 可选锁文件形状 | — | — |

> **v2.0.0 的 `main.mq.md` 不会报错，也不会有输出**——因为：
> 1. 正文链接默认不执行（叙述默认安全）；
> 2. `李白.李白的出生年份` 目前被当成变量/实例方法路径，类调用还没实现。
>
> v2.1 的目标是：`view` 显示这个链接的目标、类、函数、签名、值；`check` 返回 LinkInfo；
> 要终端输出，就在代码链接里加 `print`。

## 今天可跑

```bash
marqdo run examples/link-syntax/main.compat.mq.md
# 701
```

`main.compat.mq.md` 用当前能力完成了同一件事：

```markdown
---
markup: occlusion
import libai:libai.mq.md
---

# main

**li = > libai.李白**
**year = [李白的出生年份](li.李白的出生年份())**
> print text=`year`
```

- `import` 绑定知识文档；
- `> libai.李白` 构造对象（当前 `#` 是对象/类型）；
- `li.李白的出生年份()` 调用实例方法；
- `[]()` 把调用包成一句人话。

这正是 v2.0.0 的「实例方法」路径；v2.1 要把 `##` 恢复为**类函数**，
让 `李白.李白的出生年份` 直接调用，去掉 `import`/实例两步。

## 目标形式

`main.mq.md`：

```markdown
# main
用户想要知道李白的出生年份，我们要告诉用户[李白的出生年份](李白.李白的出生年份)
```

v2.1 期望：

```bash
$ marqdo run examples/link-syntax/main.mq.md
# 无输出（正文链接不执行；加 print 才有输出）

$ marqdo check examples/link-syntax/main.mq.md --json
{"code":"link.ok", ...}
```

`view` 期望：

```text
用户想要知道李白的出生年份，我们要告诉用户 701。
                                      ↑
        悬浮：李白的出生年份 → 李白.李白的出生年份
              class 李白 · function · () -> number · value 701
```

要终端输出：

```markdown
# main

**year = [李白的出生年份](李白.李白的出生年份)**
> print text=`year`
```

```bash
$ marqdo run examples/link-syntax/main.mq.md
701
```

## 为什么是 `李白.李白的出生年份`

- `# 李白` 是类；
- `## 李白的出生年份` 是类下的函数；
- `李白.李白的出生年份` 是**类调用**；
- `[]` 是描述，`()` 是已有调用；没有新语法，没有协议前缀，没有别名开关。

## 相关

- 设计声明：[doc/design/link-syntax.md](../../doc/design/link-syntax.md)
- 证据复核：[doc/research/occlusion-vs-executable-knowledge-2026-10.md](../../doc/research/occlusion-vs-executable-knowledge-2026-10.md)
- 旧遮蔽设计：[doc/design/occlusion-syntax.md](../../doc/design/occlusion-syntax.md)
