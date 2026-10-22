# 变更记录：M7 迁移与版本门（per-file 标记 + `migrate labels` + 全仓迁移）

| | |
|---|---|
| 状态 | **Implemented** |
| 日期 | 2026-10-22 |
| 里程碑 | M7（[occlusion-syntax.md](occlusion-syntax.md) §13） |
| 改动文件 | **新增 `src/migrate.rs`** · `src/load.rs` · `src/parse/mod.rs`(expr) · `src/host/dispatch.rs` · **80 个语料文件** · 测试 |
| 影响面 | 语法模式按文件切换；全仓 256 处索引 + 23 处循环已迁移 |

---

## 1. 原计划的问题：同形换义

原设计想"翻转默认 + 对旧形硬报错"。但 `[键](集合)` 与 `[描述](实现)` **外形完全相同**：

- 翻转默认 → 506 处取元**静默变成**"遮蔽整个集合"，正是文档点名的最坏变更
- 硬报错 → 无法实现：解析器拿不到"这是旧写法"的证据（两种读法都合法）

**结论：模式不能全局翻转，必须按文件声明。**

## 2. 新方案：per-file `markup:`

```markdown
---
markup: occlusion
---
```

| 文件 | 解析模式 | `[a](b)` 的含义 |
|------|---------|----------------|
| 无 `markup:` | classic（进程默认） | 取元 |
| `markup: occlusion` | 纯度 | 遮蔽 |

实现：`load.rs` 先读 frontmatter 取出模式，用 `parse::expr::with_markup(m, …)`
（线程局部覆盖，压栈/出栈）包住解析。**同一进程里可以同时加载两种模式的文件**
（imports！），且 `MARQDO_MARKUP=occlusion` 仍可作为进程默认。

好处：迁移是**逐文件、可增量、可回滚**的；不存在"半迁移的静默错义"。

## 3. 迁移工具 `marqdo migrate labels`

```bash
marqdo migrate labels .            # 干跑：列出将改什么
marqdo migrate labels . --write    # 实际写入
marqdo migrate labels . --json     # 机器可读
```

### 3.1 改写规则

| 旧 | 新 |
|----|----|
| `[键](集合)` | `[键](access(集合, "键"))` |
| `` [`名`](集合) `` | `` [名](access(集合, `名`)) `` |
| `- [项](集合)` | `- [项](遍历 集合 为 项)` |

`access` 是新增宿主函数（`host_access`，id 212），语义与旧取元**逐字一致**
（Map 按键 / List 1-based / 缺键 `missing map key`）。它进 `HostFn` 是有意的：
**两个后端共用同一实现**，不再有特例分支。

### 3.2 安全模型：改写 → 重解析 → 证明等价 → 才写

1. 原文件按 classic 解析，得到 AST-A
2. 文本改写后，按 occlusion 解析，得到 AST-B
3. **反归一化** B：把"标签与键同名 + payload 为 `access(x, k)`"的遮蔽还原成 `Index`
4. 比较 `Debug(A)` 与 `Debug(反归一化(B))`（抹掉行号，因为插入了 marker 行）
5. 只有相等才写入；否则**原文件一字不动**并报告 `equivalence check failed`

这条校验是真干活的：它先后抓出 4 类改写 bug（表格散文链接、payload 继承文本模式、
链式索引、`[问候] (…)` 与遮蔽混淆），每一类都是**先被拦下、再修工具**，没有污染文件。

### 3.3 只改代码行

用词法层的 `LineKind::Code` 限定范围；表格行只改 `**…**` 单元格
（散文里的 markdown 链接 `[docs](https://…)` 必须原样保留）。

### 3.4 由内向外

链式索引 `[content]([1]([messages](req)))` 必须**最内层先改**，
否则外层改写会让内层的位置失效；已迁移的 `[k](access(…))` 有守卫，不会二次改写。

## 4. 全仓迁移结果

```
migrate labels: 256 index, 23 loop, 80 file written
```

- 覆盖 `lib/ ext/ examples/ public/ cli/ tests/`
- **0 文件被等价性检查拒绝**
- 迁移后 `cargo test --release` → **604 passed / 0 failed**
- 再跑一次迁移 → **幂等**（只剩刻意保留的 classic 测试样例）

## 5. 顺带修的两处真实语义缺陷

| 缺陷 | 现象 | 修法 |
|------|------|------|
| payload 继承外层"文本模式" | `f text=[k](access(c,k))` 里 `access(...)` 不被当调用 → 退化成插值文本 | `()` 内**永远**允许 `callee(args)`（`parse_occluded` 临时置位） |
| 遮蔽形状判定过宽 | `[问候] ([格式化] "Bo")` 被误判为遮蔽 | 要求 `]` 后**紧跟** `(`；否则落回方括号调用 |

## 6. 版本门

- 旧写法**不再**被硬报错——因为无法与遮蔽区分；真正的门是 **per-file 声明**：
  没写 `markup: occlusion` 的文件按 classic 解析，行为不变。
- 需要"禁止新写旧形"的仓库可用 `migrate labels` 干跑做 CI 门禁（有改动即失败）。

## 7. 测试

| 层 | 内容 | 结果 |
|----|------|------|
| 单元（6） | 静态键 / 动态键 / 嵌套三层 / 已迁移不重写 / 循环 / 非循环 | ✅ |
| 集成（3） | 干跑不写盘；写入后**运行输出不变**且幂等；等价失败则文件不动 | ✅ |
| 全量 | `cargo test --release` | **604 passed / 0 failed**（M3 后 595 + 9） |
| 双后端 | gold 套件含 bytecode 变体 | ✅ |

## 8. 变更清单

| 文件 | 变更 |
|------|------|
| `src/migrate.rs` | **新增**：改写器、等价性证明、`migrate_file` / `migrate_path` |
| `src/load.rs` | `frontmatter_markup` + `with_markup` 包裹两处 `parse_source` |
| `src/parse/expr.rs` | `with_markup` 线程局部覆盖；`()` 内强制允许调用；`occlusion_shape_ahead` 严格判定 |
| `src/host/dispatch.rs` | `HostFn::Access`（enum / from_u16 212 / from_name / name / required_params / call_host） |
| `src/main.rs` | `migrate labels` 子命令 |
| `src/interp.rs` | 移除 access 特例（改用 HostFn，单一实现） |
| 80 个语料文件 | 索引与循环迁移 + `markup: occlusion` 标记 |
| `tests/migrate.rs` + `src/migrate.rs` 单测 | 9 个测试 |
| `doc/design/change-m7-migration.md` | 本文件 |
