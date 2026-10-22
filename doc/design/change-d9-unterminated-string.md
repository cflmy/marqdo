# 变更记录：D9 未闭合引号不再静默吞行

| | |
|---|---|
| 状态 | **Implemented**（待合入） |
| 日期 | 2026-10-22 |
| 缺陷 | [D9](../research/qdagent-refactor-defects-2026-10.md) |
| 设计 | [parser-diagnostics-d8-d9.md](parser-diagnostics-d8-d9.md) §1 |
| 改动文件 | `src/lex/mod.rs` · `src/parse/mod.rs` |
| 影响面 | 行分类 + 语句解析（仅影响**畸形**输入） |

---

## 1. 修了什么

**修复前**：`**s = "abc`（引号未闭合）被静默当成叙述丢弃，退出码 0。

```bash
$ marqdo run u.mq.md
$ echo $?
0                     # 没有输出、没有报错，s 这一行凭空消失
```

**修复后**：解析期报错，退出码非 0。

```bash
$ marqdo run u.mq.md
error: u.mq.md: 7:1: unterminated string: this line starts bold code but
contains an odd number of `"` -- close the string and the trailing `**`
(line: **s = "abc)
$ echo $?
1
```

（实测记录：报错回显中 `unterminated` 前的空格与行号格式由
`Diagnostic` 的 span 渲染决定，与其它语法错一致。）

---

## 2. 根因

问题**不在**解析器——`parse_quoted_string` 里的
`bail!("unterminated {quote} string")` 一直都在，措辞也对。
问题是这一行**根本到不了解析器**：

```
"**s = \"abc"
  ↓ classify_line
  ↓ 首字符 '*' → is_marqdo_star_line?
      matching_bold_close() 扫描时遇到 '"' → in_str = true
      之后再也没有 '"' 来关闭 → 后续所有 '*' 都被当作字符串内容跳过
      → 找不到收尾 '**' → 返回 None
  ↓ is_marqdo_star_line = false
  ↓ 落入 `Some('*') if !is_marqdo_star_line` → LineKind::Comment
  ↓ 成为「叙述」，解析器看不到
```

**这是分类层的静默降级，不是解析层的缺陷。**

它比一般的解析错误更危险，因为：

| 场景 | 后果 |
|------|------|
| 手写漏一个引号 | 该行消失 → 别处报 `undefined variable`，排查方向被引偏 |
| LLM 生成文档 | 少一段内容，但 `exit 0`、`check` 通过，**文档看起来完整却缺节** |
| CI / 文档门禁 | 门禁试跑通过 → 残缺文档被沉淀入库 |

---

## 3. 怎么改的

两层，缺一不可。

### 3.1 分类层：把畸形行留在 Code 类

`src/lex/mod.rs`

```rust
if trimmed.starts_with("**") {
    if trimmed.trim() == "** **" {
        return true;
    }
    if matching_bold_close(trimmed).is_some() {
        return true;
    }
    // D9: 粗体起始但引号永不闭合 → 找不到收尾 `**`
    // 判为 Comment 会静默丢行；判为 Code 让解析器报错
    return has_unterminated_string(trimmed);
}
```

新增辅助函数：

```rust
pub fn has_unterminated_string(line: &str) -> bool {
    // 统计未被 \ 转义的双引号个数，奇数即「开了没收」
}
```

### 3.2 解析层：给出精确报错

`src/parse/mod.rs`，紧跟既有粗体分支之后：

```rust
// D9: 粗体起始但引号未闭合 —— 走到这里说明没有收尾 `**`，
// 通用 "unrecognized statement" 会掩盖真正原因。
if trimmed.starts_with("**") && has_unterminated_string(trimmed) {
    bail!("{span}: unterminated string: ... (line: {trimmed})");
}
```

没有这一层，用户会看到 `unrecognized statement: **s = "abc`——
能定位到行，但不知道是引号没闭合。

---

## 4. 影响面（改前量化过）

对仓库内全部 `.mq.md`（`target/` 除外）做静态扫描，正确跳过 `\` 转义：

| 指标 | 数值 |
|------|------|
| `.mq.md` 文件总数 | 845 |
| 含 `**` 起始代码行的文件 | 620 |
| **引号计数为奇数的代码行** | **0** |

**结论：不改变任何现有文件的解析结果。** 这条修复只作用于畸形输入。

---

## 5. 测试

### 5.1 新增单元测试（`src/lex/mod.rs`，4 个）

| 测试 | 断言 |
|------|------|
| `unterminated_quote_inside_bold_is_code_not_comment` | 未闭合引号的粗体行 → `Code`（修复前失败，修复后通过） |
| `balanced_quote_inside_bold_still_code` | 闭合引号仍是 `Code` |
| `prose_with_quotes_stays_comment` | 叙述里提引号 → 仍是 `Comment` |
| `prose_bold_emphasis_with_quote_stays_comment` | 叙述里含孤立引号 → 仍是 `Comment` |

**先写测试确认失败**（TDD）：

```
---- lex::tests::unterminated_quote_inside_bold_is_code_not_comment
panicked: expected Code (so parser reports the error),
          got [ClassifiedLine { kind: Comment, text: "**s = \"abc" }]
```

修复后 14/14 通过。

### 5.2 回归

| 范围 | 结果 |
|------|------|
| `cargo test --release --lib` | **237 passed, 0 failed**（基线 233 + 新增 4） |
| `cargo test --release --test gold` | 276 passed, 8 failed |
| ↑ 上述 8 个失败是否本次引入 | **否**。`git stash` 后基线同样 8 failed |
| 8 个失败的原因 | 全部 `ext_web_*`，依赖 Go 插件 `libweb` 构建，本机环境不具备 |

---

## 6. 已知边界

| 情形 | 行为 | 说明 |
|------|------|------|
| `**s = "a` + 下一行 `b"**` | 报错 | 字符串不跨行，符合现状 |
| 单引号 `'...'` 未闭合 | **仍是旧行为**（静默） | 本次只处理双引号；单引号在 Marqdo 里使用频率低，留作后续 |
| 叙述行含孤立 `"` | 不受影响 | 判定入口是 `starts_with("**")`，叙述行不进入 |

> 单引号的情况建议后续按同样思路处理，但需要单独评估
> （单引号也用于英文撇号，误判风险高于双引号）。

---

## 7. 变更清单

| 文件 | 变更 |
|------|------|
| `src/lex/mod.rs` | `is_marqdo_star_line` 增加未闭合引号分支；新增 `pub fn has_unterminated_string`；新增 4 个测试 |
| `src/parse/mod.rs` | import 辅助函数；`parse_simple_stmt` 增加精确报错分支 |
| `doc/design/parser-diagnostics-d8-d9.md` | 新增（设计依据 + 影响面 + 方案评判） |
| `doc/design/spaced-function-names-d8.md` | 新增（D8 方案评估，待审阅） |
| `doc/design/change-d9-unterminated-string.md` | 本文件 |
