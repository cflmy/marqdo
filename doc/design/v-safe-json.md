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

**Host extension (not in Lean `SafeVal`):** JSON non-integer numbers → `Value::Num(f64)`. Non-finite numbers (NaN/Inf) are refused on encode and in Python/Node oracle responses, including nested payloads. Floats are documented host behavior; they are **not** claimed as mechanized \(V_{\mathsf{safe}}\) fidelity.

## Encode (`value_to_json`)

- `Value::Int(n)` with `|n| ≤ MAX_SAFE` → JSON number.
- `Value::Int(n)` with `|n| > MAX_SAFE` → **refuse** (error).
- `Value::Handle(_)` → **refuse** (`json: session handle cannot be serialized`).
- Column-table `Map` whose fields are all `List` with **unequal lengths** →
  **refuse** (`json: table column arity mismatch`).
- Nested `List` / `Map`: any child refuse fails the whole tree.
- GFM table **parse** also refuses unequal row width
  (`table row arity mismatch`) before a value is built.
- Wide integers that must cross a language boundary: author converts to a
  **decimal string** (∈ \(V_{\mathsf{safe}}\)); the foreign program may use
  `BigInt` / Python `int` on that string.

## Foreign oracle (`session_eval` / `session_get`)

Python/Node session bootstraps sanitize results with the same integer gate
before JSON emission. An Ω response containing an OOD integer is a **fault**
(host `Err`), not a soft `ok:false` cell record. Stdout text that authors
`json.parse` themselves is still gated by host `json_to_value`.

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

## Strict request boundary update

The numeric gate applies to the represented value, including integer-valued
floats and exponent notation. `9007199254740992.0` and
`9.007199254740992e15` are refused; fractions such as `1.5` remain in the host
extension. `serde_json` uses `float_roundtrip` to avoid endpoint decoding loss.
The strict text decoder rejects duplicate object keys and unequal column maps
before information is lost. One-shot, call_many and persistent input paths
use recursive domain admission; code, handle and secret values are not copied
implicitly. Auto-detection may leave non-JSON output as text, but valid JSON
outside the domain is a refusal, including in batch results.
