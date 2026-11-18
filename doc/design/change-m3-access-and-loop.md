# 变更记录：M3 `access` 与遮蔽循环

| | |
|---|---|
| 状态 | **Implemented** |
| 日期 | 2026-10-22 |
| 里程碑 | M3（[occlusion-syntax.md](occlusion-syntax.md) §4 / §13） |
| 改动文件 | `src/interp.rs` · `src/ast.rs` · `src/parse/mod.rs` · `src/bytecode/compile.rs` · `tests/occlusion.rs` |
| 影响面 | 新增内建 `access`；`Stmt::ForEach` 增可选标签 |

---

## 1. `access(集合, 键)`

旧取元 `[键](集合)` 的语义是**多态**的：Map 按键、List 按 1-based 下标、缺键报错。
纯度路线把取元移进 `()`，因此需要一个与之**完全等价**的调用形式：

```markdown
**a = [取表项](access(`表`, "键"))**
**b = [取列表第二项](access(`列`, 2))**
```

- 别名：`取` / `取值` / `access`
- 缺键 → `missing map key …`（**与旧取元同一错误**，不静默返回 None）
- 用途：M7 迁移把 `[k](c)` 机械改写成 `access(c, k)`，语义零漂移

实现：`interp.rs` 的 special-callee 分派里直接复用 `builtin_footnote_get`，
**不新增平行实现**（避免两套索引语义）。

---

## 2. 遮蔽循环

```markdown
- [篮子里的每个果子](遍历 `列` 为 `果`)
  > print text=`果`
```

- 只在 occlusion 模式识别（classic 模式下 `- [x](c)` 仍是旧 foreach）
- 接受 `遍历`/`for` 与 `为`/`as`/`in`
- 描述存入 `Stmt::ForEach.label`（`Option<String>`），供 M8 渲染/诊断使用

实测：

```
a=值
b=二
果子=一
果子=二
```

---

## 3. 为什么先做 M3 再做 M7

迁移（M7）要把 506 处 `[k](c)` 改写掉，而改写目标必须**先存在且等价**。
先有 `access` 与循环新形，迁移才是"机械替换"，而不是"边迁移边发明语义"。

---

## 4. 测试

| 层 | 内容 | 结果 |
|----|------|------|
| 集成（2） | `access` 表/列表 + 循环遍历输出正确；缺键报 `missing map key`（与旧取元一致） | ✅ |
| 全量 | `cargo test --release` | **595 passed / 0 failed**（M5 后 593 + 2） |

---

## 5. 边界

| 项 | 说明 |
|----|------|
| `access` 不做缺省值 | 缺省语义请用 `get(表, 键, 默认)`；两者并存 |
| 循环描述暂只存不用 | M8 在 `view` 与诊断里使用 |
| 旧 foreach 仍可用 | M7 迁移并加门禁后才变成错误 |

---

## 6. 变更清单

| 文件 | 变更 |
|------|------|
| `src/interp.rs` | `access` / `取` / `取值` 分派（复用 `builtin_footnote_get`） |
| `src/ast.rs` | `Stmt::ForEach.label: Option<String>` |
| `src/parse/mod.rs` | `parse_occluded_foreach`；occlusion 模式下优先识别 |
| `src/bytecode/compile.rs` / `src/ast.rs` / `src/interp.rs` | 穷举模式补 `label: _` |
| `tests/occlusion.rs` + `tests/occlusion/access-and-loop.mq.md` | 2 个集成测试 |
| `doc/design/change-m3-access-and-loop.md` | 本文件 |
