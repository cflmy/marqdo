# Foreign Strong: Lean ↔ host correspondence

2026-11-02 补充：`ForeignSource.lean` 对显式局部环境、返回、函数调用/递归、
分支和有界 while 的源解释器与独立 effect 编译器证明一般精化，并推出安全
观察和旧句柄不可复活。`CheckForeignSource.lean` 消费真实加载后 AST 的显式
适配格式，用独立 counter oracle 对照 tree/bytecode 的返回、journal、fault
与故障后 n。它已用于发现并修复 tree 的父词法作用域查找缺陷。
这是源控制流层的证明与实现 translation validation；Rust parser/adapter、
全部 AST 构造、机器数值运算和物理调度的完整 refinement 仍须完成。
模型中的 fuel 是分析界限，不能称为真实程序必然终止或有界取消。

Pinned with PLDI Strong / paper-v5 (sessions as values; three gates).
Marqdo **2.3.0** host implements the same objects the Lean development decides.

| Lean / paper rule | Host surface | Evidence |
|---|---|---|
| `SessionReg` + `sessionOpen` / `Close` / `sessionEval` | `HostContext.foreign_sessions` + `Value::Handle` | `DocCalc.lean` S1–S6, `requires_live`; `foreign_session.rs` |
| `Ω(s, τ, c)` | Persistent kernel + process order | Oracle journal `(cell, source_hash, stdout, result)` |
| `SafeVal` / Encode / Decode | `src/host/json.rs` + bootstrap sanitize | `ForeignJSON.lean`; gold `foreign-*-vsafe-*` |
| Handle ∉ \(V_{\mathsf{safe}}\) | `value_to_json(Handle) → Err` | `foreign-session-handle-no-serialize.mq.md` |
| Closed / Closed-Close | use-after-close / double-close `Err` | gold misuse; `sessionClose_then_eval_none_w` |
| Oracle-Domain | Ω OOD integer → host `Err` | `foreign-session-result-vsafe-refuse.mq.md` |
| Table arity stuck | Parse refuse unequal GFM; JSON unequal columns | `Surface.lean`; `tests/errors/table-arity.mq.md` |
| Snapshot hash match/mismatch | `view/output.rs` `snapshot_page` | unit snapshot hash tests |
| **Thm. corr** (`corr_fresh_functional`) | Cold `evaldoc`; step values = Ω | `corr_step_eq_omega`; `corr_doc_fresh_foreign`; foreign-session gold |
| `OracleJournal` / `replayCheck` | `oracle_journal` + `DocumentSession::replay_check` | `replayCheck_*`; fate R1–R3 |
| `corr_fail_warm_store` / `corr_fail_reordered_plan` | cold vs warm / schedule order | DocCalc witnesses; Table corr-fail |
| Dual-backend differential | `tests/gold.rs` tree + bytecode | 308 × 2; foreign session/V_safe subset |

## Scope (not distance)

- SafeVal mechanization covers the **integer** fragment; finite floats are admitted by the host under the same refuse-not-coerce bridge policy for non-integers already in SafeVal.
- Session discipline is a **dynamic registry** (affine close + borrow-use). Static linear types over OS-killable processes are out of scope by design, not an unfinished host feature.
- Correspondence is mechanized for the **closed / fresh-open / functional-Ω** fragment (`corr_fresh_functional`): per-step interactive value = Ω answer, interactive runner = normative runner, `replayCheck` on the journal. Broken hypotheses are `corr_fail_*`. Host discharge is foreign-session gold plus R1–R3, not R1–R3 alone.

## Reviewer-driven implementation update (2026-11-01)

The independent live cursor and multisession action-plan simulation are in
`DocCalc.runForeignLive_simulation` and `MarqdoProof.ForeignTrace`. The language
now exposes oracle observations and pure journal comparison through
`foreign.oracle_journal` / `foreign.replay_check`. Strict interactive snapshot
replay uses the full oracle history with source identity and per-step input.
See `oracle-journal-values.md` for the API and the research experiment manifest
under `PLDI/experiments/reviewer-v5/` for exact source hashes and actual runs.
