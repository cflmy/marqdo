# 变更记录：M4 L1 引用一致性（`label.ref_dangling`）

| | |
|---|---|
| 状态 | **Implemented** |
| 日期 | 2026-10-22 |
| 里程碑 | M4（[label-freshness-and-repair.md](label-freshness-and-repair.md) §3.1 / [occlusion-syntax.md](occlusion-syntax.md) §11.1） |
| 改动文件 | `src/label.rs` · `tests/occlusion.rs` · `tests/occlusion/ref-*.mq.md` |
| 影响面 | `marqdo check`（新增一条 error） |

---

## 1. 规则

> 描述里用反引号写出的名字 = **断言**：它必须能解析
> —— 要么 `()` 的实现用到它，要么文档里定义了它，要么它是内置函数。

```markdown
**a = [按 `税率` 计算](100 * 0.1)**     # ❌ label.ref_dangling（税率为空悬）
**b = [按 `税率` 算](100 * `税率`)**     # ✅ 实现里用到
**c = [取长度](len("abc"))**            # ✅ 内置函数
**d = [用 `加` 求和](> 加 1 2))**        # ✅ 模块内函数
```

实测：

```
dangling.mq.md:3:1: 描述引用了不存在的实体：`税率` —— 本段实现没有用到它，模块里也没有定义
contract check: 1 条诊断（1 error）   # exit 1
```

---

## 2. 为什么只认反引号（可证伪原则）

| 写法 | 是否检查 | 原因 |
|------|---------|------|
| `` `税率` `` | ✅ | 显式引用，可证伪 |
| 普通汉字"税率" | ❌ | 自然语言无法机械判定，检查必然假阳性 |

代价：作者不写反引号就得不到保护。这是刻意的——**机器只校验它看得懂的断言**，
同时反向抑制"为凑数写废话"。

---

## 3. 实现

| 环节 | 做法 |
|------|------|
| 抽取 | `refs(label)`：扫描成对反引号，取内部去空白；未闭合/空串忽略 |
| 作用域 | `payload_idents(payload)`（Var / callee / path / receiver / `[k]` 的变量键 / 插值变量）∪ `module_symbols(module)`（函数与子函数、参数、import/use 绑定、frontmatter 键）∪ `HostFn::from_name` |
| 诊断 | `label.ref_dangling`，**error**，带 `doc_quote`（描述原文）与可执行建议 |

`OcclusionSite` 增加 `payload_idents` 字段，在 `collect` 阶段一次算好，`check` 只做集合查找。

---

## 4. 测试

| 层 | 内容 | 结果 |
|----|------|------|
| 单元（3） | 单引用 / 多引用与散文混排 / 无引用与未闭合反引号不 panic | ✅ |
| 集成（2） | 悬空引用 exit 1 + 文案；三类可解析引用（payload 变量 / 内置 / 模块函数）零诊断 | ✅ |
| 全量 | `cargo test --release` | **583 passed / 0 failed**（M2.5 后 578 + 5） |

---

## 5. 边界与后续

| 项 | 说明 |
|----|------|
| 只查"能否解析" | 不查"语义是否仍成立"——那是 T1（M6）与人工/LLM 复核的活 |
| 不查"能解析但实现没用到" | 该情形在模块级符号时无法区分误报，留待 T1 变更驱动 |
| 描述里的反引号不是变量绑定 | 只是断言，不进入任何作用域 |

---

## 6. 变更清单

| 文件 | 变更 |
|------|------|
| `src/label.rs` | `refs()` · `payload_idents()` · `module_symbols()` · `OcclusionSite.payload_idents` · `check` 增 `label.ref_dangling` · 3 个单测 |
| `tests/occlusion.rs` + `tests/occlusion/{ref-dangling,ref-ok}.mq.md` | 2 个集成测试 |
| `doc/design/change-m4-label-refs.md` | 本文件 |
