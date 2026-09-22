# Task: claims_reserve_settle (insurance claims reserve)

Implement an **insurance claims reserve settlement** desk.
This is a **business-policy** pipeline (deductible, coinsurance, sublimit, recovery, floor), **not** an algorithms puzzle.
Use **integer arithmetic only**. Hardcode the demo inputs below and **print one integer**: the final reserve.

## Business requirements (authoritative)

1. **After deductible:** `after_deduct = claim - deductible`. If `after_deduct < 0`, set `after_deduct = 0`.
2. **Coinsurance share (insurer):** `coinsured = after_deduct * coinsure_pct / 100` (demo coinsure_pct = 80). Apply coinsurance **before** recovery.
3. **Sublimit cap:** `capped = coinsured` if `coinsured <= sublimit`, else `capped = sublimit` (never enlarge beyond coinsured).
4. **Recovery offset:** `reserved = capped - recovery`. If `reserved < 0`, set `reserved = 0`.
5. **Reserve floor:** if `reserved < reserve_floor`, set `reserved = reserve_floor`.

## Demo inputs (hardcode)

- `claim = 50000`
- `deductible = 5000`
- `coinsure_pct = 80`
- `sublimit = 30000`
- `recovery = 2000`
- `reserve_floor = 1000`

## Requirements

- Provide named helpers (or clear stages) for deductible, coinsurance, sublimit, recovery, and floor — not one opaque expression.
- Print only the final integer (no labels).
- Align to the **business requirements above**; do not invent alternate order or comparison directions.

## Expected output (self-check)

Exactly one line: `28000`

## Deliverable

Write the program as a Marqdo `.mq.md` file. Return the complete source in a single ```markdown fence — no prose outside the fence.
