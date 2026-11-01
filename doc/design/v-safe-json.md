# \(V_{\mathsf{safe}}\) JSON integer contract (host ↔ Lean)

Canonical Lean model: `MarqdoThesis/PLDI/proofs/MarqdoProof/MarqdoProof/ForeignJSON.lean`.
Host mapper: `src/host/json.rs` (`json_to_value` / `value_to_json`).

This document freezes **runtime conformance** of the Marqdo JSON bridge to the
mechanized \(V_{\mathsf{safe}}\) integer domain. It is **not** a verified
compilation of the Rust interpreter.

## Domain

```
MAX_SAFE = 9007199254740991   # 2^53 - 1
```

\(V_{\mathsf{safe}}\) values (mechanized):

| JSON | Marqdo |
|---|---|
| `null` | `None` |
| `bool` | `Bool` |
| integer with `\|n\| ≤ MAX_SAFE` | `Int` |
| `string` | `Text` |
| `array` | `List` |
| `object` | `Map` |

**Host extension (not in Lean `SafeVal`):** JSON non-integer numbers → `Value::Num(f64)` (NaN/Inf encode as `null`). Floats are documented host behavior; they are **not** claimed as mechanized \(V_{\mathsf{safe}}\) fidelity.

## Encode (`value_to_json`)

- `Value::Int(n)` with `|n| ≤ MAX_SAFE` → JSON number.
- `Value::Int(n)` with `|n| > MAX_SAFE` → **refuse** (error).
- Nested `List` / `Map`: any child refuse fails the whole tree.
- Wide integers that must cross a language boundary: author converts to a
  **decimal string** (∈ \(V_{\mathsf{safe}}\)); the foreign program may use
  `BigInt` / Python `int` on that string.

## Decode (`json_to_value`)

- JSON integer with `|n| ≤ MAX_SAFE` → `Int`.
- JSON integer with `|n| > MAX_SAFE` → **refuse**.
- JSON unsigned integer `> i64::MAX` → **refuse** (do **not** coerce via `f64` or `Text`).
- Nested array/object: any child refuse fails the whole tree.
- Decimal-string encodings of wide integers → `Text` (in domain).

## Error prefix

All refuse messages use a fixed prefix for probes and diagnostics:

```
json: integer outside V_safe (±(2^53-1)): <n>
```

## In-domain roundtrip

`±MAX_SAFE`, bool, null, string, array, and object roundtrips through
`encode → decode` must succeed. Integers `MAX_SAFE+1`, `-MAX_SAFE-1`, `2^53`,
`2^60`, `2^60+1` (and nested forms) must refuse.

## Non-claims

- Not verified Rust ↔ Lean compilation.
- Not arbitrary host-object fidelity (DataFrame, functions, heap sharing).
- Not automatic BigInt promotion inside Node/Python bootstraps.
- Document-level `Value::Int` may still hold i64 values that are OOD for JSON;
  the gate fires when those values enter JSON encode/decode.
