# Foreign Strong: Lean ↔ host correspondence

Pinned with PLDI Strong narrative (`FOREIGN-NARRATIVE-ASSESSMENT-2026-11-01.md`).
**Not** verified compilation — conformance evidence only.

| Lean / paper rule | Host surface | Evidence |
|---|---|---|
| `SessionReg` + `sessionOpen` / `Close` / `sessionEval` | `HostContext.foreign_sessions` + `Value::Handle` | `DocCalc.lean`; `foreign_session.rs` |
| `Ω(s, τ, c)` | Persistent kernel + process order | Oracle journal records `(cell, source_hash, stdout, result)` |
| `SafeVal` / Encode / Decode | `src/host/json.rs` + session bootstrap sanitize | `ForeignJSON.lean`; `v-safe-json.md`; gold `foreign-*-vsafe-*` |
| Handle ∉ \(V_{\mathsf{safe}}\) | `value_to_json(Handle) → Err` | `foreign-session-handle-no-serialize.mq.md` |
| Closed / Closed-Close | use-after-close / double-close `Err` | gold misuse suite |
| Oracle-Domain | Ω OOD integer → host `Err` | `foreign-session-result-vsafe-refuse.mq.md` |
| Table arity stuck | Parse refuse unequal GFM row width; JSON refuse unequal column lists | `Surface.lean` `table_arity_*`; `tests/errors/table-arity.mq.md` |
| Snapshot hash match/mismatch | `view/output.rs` `snapshot_page` | unit `snapshot_page_rejects_source_hash_mismatch` |
| `replay_check` vs journal \(J\) | `DocumentSession::replay_check` | unit + `defense/fate_matrix/run_replay_pack.py` |
| Dual-backend differential | `tests/gold.rs` tree + bytecode | 308 tests × 2; foreign subset includes session/V_safe |

## Intentional non-claims

- No static affine/linear session types (host death unsound).
- Floats are host extension outside Lean `SafeVal`.
- `replay_check` is host/document-session API, not a second interpreter backend.
