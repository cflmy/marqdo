# 变更记录：M6 T1 会话内陈旧检测 + 会话末汇总

| | |
|---|---|
| 状态 | **Implemented** |
| 日期 | 2026-10-22 |
| 里程碑 | M6（[t1-session-staleness.md](t1-session-staleness.md) · [staleness-providers.md](staleness-providers.md) T1） |
| 改动文件 | `src/label.rs` · `src/mlsp.rs` · `src/view/mod.rs` · `tests/occlusion.rs` |
| 影响面 | MLSP `repair_apply` 新增可选字段；新增 `session.summary` 方法 |

---

## 1. 机制

`repair_apply` 内部同一作用域里同时有 `source`（改前）与 `new_source`（改后），
所以「描述是否还成立」可以**纯内存**判定：

```
stale ⟺ 标签文本未变 ∧ payload 指纹变了
```

配对按 `(标签文本, 出现序号)`，因此**移动代码不会误报**。

请求：

```json
{"method":"repair_apply","params":{
  "source":"…","ranges":[[3,3]],
  "edits":[{"op":"replace","line":3,"text":"**a = [按税率算总价](100 * 0.2)**"}],
  "check_labels":true}}
```

响应：

```json
{"id":1,"ok":true,"result":{
  "source":"…","edits_applied":1,
  "labels":{"newly_seen":1,"stale":[
    {"label":"按税率算总价","line":3,
     "code_past":"100 * 0.1","code_now":"100 * 0.2"}]}}}
```

**同一份 `source` / `new_source` 就是"前一状态"**——不需要 git、不需要台账、
不需要沙箱里的 exec 权限。这是"agent 回路不需要 VCS"的落地。

---

## 2. 会话末汇总（你要求做）

`session.summary` 返回并**重置**计数：

```json
{"ok":true,"labels_stale_seen":1,"labels_fixed":0,
 "labels_outstanding":1,"items":["按税率算总价"]}
```

| 字段 | 含义 |
|------|------|
| `labels_stale_seen` | 本次会话累计发现过多少条（去重按标签） |
| `labels_fixed` | 曾经 outstanding、现在不再 stale 的条数 |
| `labels_outstanding` | 仍未处理 |
| `items` | 未处理的标签文本（只给文本，不重复诊断细节） |

设计：客户端在收尾时调用一次即可闭合；**不主动打屏**，不污染协议。

---

## 3. 关键取舍

| 取舍 | 理由 |
|------|------|
| `check_labels` 默认 **false** | 零开销、零噪音；agent 框架在修复回路里显式打开 |
| 标签变了 → **不报** | 作者改描述本身就是"已复核"的信号 |
| 新标签 → 不报 | 无历史可比 |
| 指纹用 `Debug` | **只需进程内一致**；跨版本稳定指纹是 T2/T3 的事 |
| payload 渲染用 `view::expr_display` | 让 `code_past/code_now` 是人能读的表面语法，而非 AST 调试串 |

---

## 4. 测试

| 层 | 内容 | 结果 |
|----|------|------|
| 集成（3） | 只改 payload → `stale` 含 `code_past`/`code_now`；同时改标签 → `stale: []`；`session.summary` 计数正确 | ✅ |
| 全量 | `cargo test --release` | **590 passed / 0 failed**（M10 后 587 + 3） |

**反向验证**：`t1_silent_when_label_also_changes` 证明"改描述即复核"不会误报。

---

## 5. 边界

| 项 | 说明 |
|----|------|
| 只看本次编辑 | 跨会话漂移是 T2/T3 的职责（可选 provider） |
| 解析失败则跳过 | 修复后的源码可能语法错误，此时不评测标签，交给 `validate` |
| 会话状态是进程级 | 一条 MLSP 连接（一个进程）即一个会话；`session.summary` 后清零 |

---

## 6. 变更清单

| 文件 | 变更 |
|------|------|
| `src/label.rs` | `StaleItem` · `payload_fingerprint` · `compare` · `SessionSummary`（含 `record`）· `OcclusionSite.payload_fp/payload_src` |
| `src/mlsp.rs` | `repair_apply` 的 `check_labels`；`session.summary`；进程级 `SESSION` |
| `src/view/mod.rs` | `pub use render::expr_display`（供诊断渲染 payload） |
| `tests/occlusion.rs` | 3 个 T1 集成测试 |
| `doc/design/change-m6-t1-session.md` | 本文件 |
