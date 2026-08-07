# 自写回标准库（writeback）

| | |
|---|---|
| 状态 | v1 已落地 |
| 日期 | 2026-08-07 |
| 库 | `lib/writeback.mq.md` · `lib/自写回.mq.md` |

## 定位

**Jupyter 输出单元的 Marqdo 等价物**：执行得到结果 → `record` / `写回` 写入入口 `.mq.md` 的 `<!-- marqdo-out … -->` → `marqdo view` 在对应语句下显示 **output-card**。普通 Markdown 预览不显示该区域。

## 存储格式

```markdown
*`n` = > len value=`xs` *
<!-- marqdo-out
42
-->
```

命名槽（智能体成功/失败互不覆盖）：

```markdown
<!-- marqdo-out ok
status=ok result=…
-->
<!-- marqdo-out error
pending
-->
```

- 注释内为 `marqdo-out`，可选后跟槽名（`ok` / `error` 等），**不存行号**
- 默认在**调用行下方**插入或替换紧邻**无键**块
- `at_end=true`：替换文件末尾的**无键**块
- `key=` / `键=`：在**当前调用锚点下方**创建或替换该命名槽（智能体单步用父调用行，即 `step` 语句下方）；`ensure` 在缺失时写入占位且不覆盖已有正文。命名槽互不覆盖，也不使用文件末尾单块。
- `line=` / `行=`：显式锚点行号。子任务 / 异步写回必须带上，否则会锚到子任务内部调用行。主机侧有全局互斥锁，写前从磁盘重读入口源，避免并发写互相覆盖。

### 替换跨度与空行

替换 `<!-- marqdo-out … -->` 时，被替换跨度**包含**注释块结束 `-->` 后的**单个**换行符（不把该换行留在文件里）。这样多次 `record` **不会**在块下方累积多余空行。

命名槽仍锚在调用行（如 `step` / `写回`）下方；整簇结束后保留**一条**有意空行即可，且应稳定。

## API

| EN | ZH | 说明 |
|----|-----|------|
| `record` | `写回` | `value=`；可选 `at_end`、`key`、`line` |
| `get` | `取` | 可选 `at_end`、`key`、`line` |
| `clear` | `清除` | 可选 `at_end`、`key`、`line` |
| `ensure` | `确保` | `key=` + 可选 `placeholder=`、`line=`；槽已存在则不动 |
| `list` | `列出` | 全文所有输出块（命名槽含 `key` 字段） |

可选参数语法：`` + `参数名`=默认值 ``（与现有参数列表兼容）。

Host：`host_writeback_*`（`src/host/writeback.rs`）；智能体侧经 `lib/subtask` 异步调用写回，避免阻塞推理主路径。
