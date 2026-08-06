# 自写回标准库（writeback）

| | |
|---|---|
| 状态 | v1 已落地 |
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

- 注释内仅保留 `marqdo-out` 标识，**不存行号**（避免写回后行号漂移）
- 默认在**调用行下方**插入或替换紧邻块（中间可有空行）
- `at_end=true` / `末尾=true`：在文件末尾保留单块（替换已有末尾块）

## API

| EN | ZH | 说明 |
|----|-----|------|
| `record` | `写回` | `value=`，可选 `at_end=false` |
| `get` | `取` | 可选 `at_end=false` |
| `clear` | `清除` | 可选 `at_end=false` |
| `list` | `列出` | 全文所有输出块（含推断锚点行） |

可选参数语法：`` + `参数名`=默认值 ``（与现有参数列表兼容）。

Host：`host_writeback_*`（`src/host/writeback.rs`）。
