# Procurement skill · reproducible demo

This is a **local simulation** in one callable `.mq.md` source. It has no external purchase side effects and does not require an LLM API key.

## What it demonstrates

- A `¥600` spending limit with a caller-supplied balance.
- An active supplier `SUP_JN`, quoting `¥36` for a sample bearing.
- Eight units total `¥288` with an estimated seven-day delivery.
- Supplier approval, quote comparison, budget check and local work-order generation as direct Marqdo function calls.
- Unconnected capabilities (live quotes, real work orders and receipts, negotiation and supplier switching) explicitly labeled as stubs.

## Run from the repository root

```bash
marqdo check examples/procurement/index.mq.md --narrative-ratio
marqdo call examples/procurement/index.mq.md 查询采购商 "采购商状态='正常'" --json
marqdo call examples/procurement/index.mq.md 查询采购商价格 "采购商标识符='SUP_JN'" 数量=8 有效秒数=300 --json
marqdo call examples/procurement/index.mq.md 查询余额 已花费=120 --json
marqdo call examples/procurement/index.mq.md 向采购商发工单 "采购商标识符='SUP_JN'" 数量=8 单价=36 已花费=120 交付要求天数=7 --effects time.read,process.control --json
marqdo call examples/procurement/index.mq.md 向采购商发工单 "采购商标识符='SUP_JN'" 数量=8 单价=36 已花费=360 交付要求天数=7 --effects time.read,process.control --json
marqdo call examples/procurement/index.mq.md 查询采购商价格 "采购商标识符='SUP_YY'" 数量=8 --json
```

## Interpret the result

The return from `向采购商发工单` is a **business-level** `ok/value/error` map. In the low-spend case it should contain `ok=True`, `工单金额=288` and `状态="本地模拟，未发送"`. When already spent reaches `360`, the function should return `ok=False` and a budget rejection reason. The CLI's outer JSON call envelope is separate from this nested domain result.

These are **expected outcomes**, not independently verified terminal captures. The workflow [procurement-example.yml](../../.github/workflows/procurement-example.yml) exercises the commands using the released Linux CLI.

## Inspect in View

```bash
marqdo view examples/procurement --no-open
```

Open `向采购商发工单` and expand the annotations for `查询采购商价格`, `查询余额`, and `剩余额度`. Merely reading and expanding details does not execute side effects.

## Production integration boundaries

The real supplier quote API, persistent authoritative ledger, concurrent budget reservation, ERP transaction, order receipt, negotiation, and supplier switching are **not connected**. Before any real purchase, integrate server-side budget reservations, quote signatures/evidence, idempotent transactions, durable logs and an explicit human authorization policy.

The GIF-like animated [SVG storyboard](../../assets/marketing/procurement-flow.svg) and the [View-inspired concept](../../assets/marketing/procurement-view-concept.svg) are illustrations, not instrumented evidence.