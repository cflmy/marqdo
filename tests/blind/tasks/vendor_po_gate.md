# Task: vendor_po_gate (complex procurement business policy)

Implement a **vendor purchase-order payable gate** for an enterprise procurement desk.
This is a **business-policy** task (multi-rule approval economics), **not** an algorithms puzzle.
Use **integer arithmetic only**. Hardcode the demo inputs below and **print one integer**: the final payable.

## Business requirements (authoritative)

1. **Base line:** `base = qty * unit_cost`.
2. **Preferred-vendor discount:** if `vendor_tier >= 2` (tier-2+ preferred suppliers), set `amount = base * 97 / 100`; otherwise `amount = base`.
3. **Rush surcharge:** if `urgency == 1`, set `amount = amount + amount * 5 / 100`; otherwise keep `amount`.
4. **Budget clamp:** if `amount > budget_cap`, set `amount = budget_cap`.
5. **Compliance fee:** if `risk_flag == 1`, `fee = 80`; else `fee = 0`.
6. **Payable floor:** `payable = amount + fee`; if `payable < 200` then `payable = 200`.

## Demo inputs (hardcode)

- `qty = 20`
- `unit_cost = 40`
- `vendor_tier = 2`
- `urgency = 1`
- `budget_cap = 900`
- `risk_flag = 0`

## Requirements

- Provide named helpers (or clear stages) for discount, rush, budget clamp, fee, and assemble — not one opaque expression.
- Print only the final integer (no labels).
- Align the program to the **business requirements above**; do not invent alternate thresholds.

## Expected output (self-check)

Exactly one line: `814`

## Deliverable

Write the program as a Marqdo `.mq.md` file. Return the complete source in a single ```markdown fence — no prose outside the fence.
