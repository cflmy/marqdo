# Task: contract_milestone_pay (construction milestone payable)

Implement a **contract milestone payment** desk for an EPC / construction settlement.
This is a **business-policy** pipeline (retention, late penalty, change order, tax), **not** an algorithms puzzle.
Use **integer arithmetic only**. Hardcode the demo inputs below and **print one integer**: the final payable.

## Business requirements (authoritative)

1. **Earned value:** `earned = contract_value * progress_pct / 100`.
2. **Retention withhold:** `retention = earned * retain_pct / 100` (demo uses retain_pct = 5).
3. **Net after retention:** `net = earned - retention`.
4. **Late penalty:** if `days_late > grace_days` (grace_days = 7), set `penalty = net * late_rate / 100` (late_rate = 2); otherwise `penalty = 0`. Penalty is always computed from **`net`**, never from `earned`.
5. **After penalty:** `net2 = net - penalty`.
6. **Change order:** `taxed_base = net2 + change_order`. If `taxed_base < 0`, set `taxed_base = 0`.
7. **Tax:** `tax = taxed_base * tax_rate / 100` (tax_rate = 6).
8. **Payable:** `payable = taxed_base + tax`. If `payable < min_pay` (min_pay = 500), set `payable = min_pay`.

## Demo inputs (hardcode)

- `contract_value = 100000`
- `progress_pct = 40`
- `retain_pct = 5`
- `days_late = 10`
- `grace_days = 7`
- `late_rate = 2`
- `change_order = -2000`
- `tax_rate = 6`
- `min_pay = 500`

## Requirements

- Provide named helpers (or clear stages) for earned, retention, penalty, change+tax, and floor — not one opaque expression.
- Print only the final integer (no labels).
- Align to the **business requirements above**; do not invent alternate thresholds or penalty bases.

## Expected output (self-check)

Exactly one line: `37354`

## Deliverable

Write the program as a Marqdo `.mq.md` file. Return the complete source in a single ```markdown fence — no prose outside the fence.
