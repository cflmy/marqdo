# 变更记录：HostFn 五张表合一（`HOST_FN_TABLE` 单一来源）

| | |
|---|---|
| 状态 | **Implemented** |
| 日期 | 2026-10-22 |
| 来源 | [code-volume-audit.md](code-volume-audit.md) §4 第 4 项 |
| 改动 | `src/host/dispatch.rs`：**−404 行**（639 删 / 235 增） |
| 行为 | **零变化**（203 项逐字节一致） |

---

## 1. 原来有五张并行表

202 个宿主函数的元数据分散在五个手写 match 里：

| 表 | 行数 | 作用 |
|----|------|------|
| `from_u16` | ~215 | id → 变体 |
| `from_name` | ~215 | 名字/别名 → 变体（**保留**，见 §4） |
| `name` | ~210 | 变体 → 规范名 |
| `required_params` | ~145 | 变体 → 必填参数 |
| `optional_params` | ~90 | 变体 → 可选参数 |

五张表必须同步维护，而 [code-volume-audit.md](code-volume-audit.md) 记录过
`from_name` 接受 `http_get` 而 `name()` 只输出 `host_http_get` 的一类漂移。

## 2. 现在只有一张

```rust
pub struct HostFnDef {
    pub id: u16,
    pub f: HostFn,
    pub name: &'static str,
    pub required: &'static [&'static str],
    pub optional: &'static [&'static str],
}

pub const HOST_FN_TABLE: &[HostFnDef] = &[
    HostFnDef { id: 1, f: HostFn::ReadText, name: "host_read_text",
                required: &["path"], optional: &[] },
    …
];
```

四个函数改为查表（`from_u16` 按 id 查，其余按判别值查）；
`call_host` 保留它那 202 个一行臂——**分派逻辑不动**，只把元数据集中。

## 3. 怎么保证没改坏

迁移采用**快照比对**：

1. 先在旧实现上导出全部 203 项的 `(id, name, required, optional)` 到 TSV
2. 合并成表后，用同一段代码再导出一份
3. `diff` 两份文件 → **完全一致**

```
$ diff /tmp/host_fns_baseline.tsv /tmp/host_fns_new.tsv && echo IDENTICAL
IDENTICAL
```

快照测试是一次性的，比对通过后删除；长期守护交给四个不变量测试：

| 测试 | 内容 |
|------|------|
| `every_assigned_id_round_trips` | 每个已分配 id → `as_u16()` 往返（id 空间稀疏是**退役 id**，不复用） |
| `every_id_has_a_name_that_round_trips` | `name()` → `from_name()` 回到同一变体 |
| `names_are_unique` | 规范名唯一 |
| `required_and_optional_params_are_disjoint` | 必填/可选不重叠、`all_params` 无重复 |

## 4. 为什么 `from_name` 没有并进表

它是**别名注册表**（`http_get`、`get`、中文别名…），语义与"规范名"不同；
并进表会让 `name()` 与 `from_name` 又变成一对映射。
现在它保留原样，由上面的往返测试保证不与表漂移。

## 5. 收益

| 指标 | 前 | 后 |
|------|----|----|
| `dispatch.rs` | 2,052 行 | **1,648 行** |
| `src/` 合计 | 53,040 | **52,668** |
| 需同步维护的表 | 5 | **1** |

## 6. 验证

| 项 | 结果 |
|----|------|
| `cargo build --release` | 零警告 |
| `cargo test --release` | **609 passed / 0 failed** |
| 快照 diff | 203 项逐字节一致 |
| 四个不变量测试 | 全绿 |

## 7. 变更清单

| 文件 | 变更 |
|------|------|
| `src/host/dispatch.rs` | 新增 `HostFnDef` / `HOST_FN_TABLE`（203 项）；`from_u16` / `name` / `required_params` / `optional_params` 改为查表；删除对应四个 match |
| `doc/design/change-hostfn-table.md` | 本文件 |
