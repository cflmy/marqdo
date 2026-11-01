# Foreign Strong: Lean ↔ host correspondence

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
