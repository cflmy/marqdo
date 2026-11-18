---
markup: occlusion
title: Python 经营洞察 · View Cells
description: 多 cell 共享 Python 内核，做订单生成、区域/产品指标、异常检测、报告输出与独立复核。
import:
  foreign: lib/foreign.mq.md
---

# main

`内核表` =

| 名称 | 语言 | 命令 | 说明 |
|------|------|------|------|
| 分析 | python | | 共享数据清洗、指标与异常检测 |
| 复核 | python | | 独立内核，只通过显式 JSON 文件复核 |

`会话表` =

| 单元 | 内核 | 说明 |
|------|------|------|
| 准备数据 | 分析 | 生成可复现的订单数据 |
| 计算指标 | 分析 | 区域、产品、周度趋势 |
| 检测异常 | 分析 | 按区域金额做 z-score |
| 生成报告 | 分析 | ASCII 图 + JSON 报告 |
| 独立复核 | 复核 | 另一个内核读取报告做校验 |

`会话` = > foreign.session kernels=`内核表` cells=`会话表`

`准备数据` =
```python
import math
import random
from collections import defaultdict
from datetime import date, timedelta

SEED = 20261017
rng = random.Random(SEED)
START = date(2026, 1, 5)
REGIONS = ["华东", "华南", "华北", "西部"]
PRODUCTS = {"A": "智能传感器", "B": "边缘网关", "C": "数据订阅", "D": "运维服务"}
BASE_PRICE = {"A": 420.0, "B": 980.0, "C": 260.0, "D": 640.0}

def make_orders(n=180):
    rows = []
    for i in range(n):
        week = i // 15 + 1
        region = REGIONS[i % len(REGIONS)]
        product = "ABCD"[(i * 7 + week) % 4]
        seasonal = 1.0 + 0.12 * math.sin(week / 2.2)
        qty = max(1, int(rng.gauss(3.0, 1.2)))
        amount = round(BASE_PRICE[product] * qty * seasonal * rng.gauss(1.0, 0.08), 2)
        rows.append({
            "id": f"O{i + 1:04d}",
            "week": week,
            "region": region,
            "product": product,
            "qty": qty,
            "amount": amount,
            "day": (START + timedelta(days=i * 4)).isoformat(),
        })
    return rows

orders = make_orders()
assert len(orders) == 180
print(f"生成订单: {len(orders)} 条")
print(f"时间范围: {orders[0]['day']} → {orders[-1]['day']}")
print(f"种子: {SEED}")
print(f"样例: {orders[0]}")
```

`计算指标` =
```python
import statistics

by_region = defaultdict(list)
by_product = defaultdict(float)
by_week = defaultdict(float)
for o in orders:
    by_region[o["region"]].append(o["amount"])
    by_product[o["product"]] += o["amount"]
    by_week[o["week"]] += o["amount"]

region_stats = []
for region, amounts in sorted(by_region.items()):
    region_stats.append({
        "region": region,
        "orders": len(amounts),
        "total": round(sum(amounts), 2),
        "mean": round(statistics.mean(amounts), 2),
        "median": round(statistics.median(amounts), 2),
    })
region_stats.sort(key=lambda r: r["total"], reverse=True)

weekly_avg = [round(by_week[w] / 15, 2) for w in sorted(by_week)]
first3 = statistics.mean(weekly_avg[:3])
last3 = statistics.mean(weekly_avg[-3:])
trend_pct = round((last3 / first3 - 1) * 100, 2)
top_products = sorted(by_product.items(), key=lambda kv: kv[1], reverse=True)

total_revenue = round(sum(o["amount"] for o in orders), 2)
metrics = {
    "order_count": len(orders),
    "total_revenue": total_revenue,
    "avg_order_value": round(total_revenue / len(orders), 2),
    "trend_pct": trend_pct,
    "region_stats": region_stats,
    "top_products": [
        {"code": code, "name": PRODUCTS[code], "revenue": round(value, 2)}
        for code, value in top_products
    ],
}

print(f"总营收: ¥{total_revenue:,.2f}")
print(f"客单价: ¥{metrics['avg_order_value']:,.2f}")
print(f"周度趋势: {trend_pct:+.2f}%（首 3 周 vs 末 3 周）")
print("\n区域表现（按营收降序）:")
for r in region_stats:
    print(f"  {r['region']}  订单 {r['orders']:>2}  营收 ¥{r['total']:>10,.2f}  中位数 ¥{r['median']:>8,.2f}")
print("\n产品营收:")
for item in metrics["top_products"]:
    print(f"  {item['code']} {item['name']}  ¥{item['revenue']:,.2f}")
```

`检测异常` =
```python
by_region_amounts = defaultdict(list)
for o in orders:
    by_region_amounts[o["region"]].append(o["amount"])

anomalies = []
for region, amounts in by_region_amounts.items():
    mean = statistics.mean(amounts)
    sd = statistics.pstdev(amounts)
    if sd == 0:
        continue
    for o in orders:
        if o["region"] != region:
            continue
        z = (o["amount"] - mean) / sd
        if abs(z) >= 1.8:
            anomalies.append({
                "id": o["id"],
                "region": region,
                "product": PRODUCTS[o["product"]],
                "amount": o["amount"],
                "z": round(z, 2),
            })

anomalies.sort(key=lambda a: abs(a["z"]), reverse=True)
print(f"异常订单: {len(anomalies)} / {len(orders)}（|z| ≥ 1.8）")
for a in anomalies[:6]:
    print(f"  {a['id']}  {a['region']}  {a['product']}  ¥{a['amount']:,.2f}  z={a['z']:+.2f}")
if len(anomalies) > 6:
    print(f"  … 其余 {len(anomalies) - 6} 条")
```

`生成报告` =
```python
import json
lines = []
lines.append("=" * 56)
lines.append("Marqdo Python 经营洞察 · 周报")
lines.append("=" * 56)
lines.append(f"订单数: {len(orders)}    总营收: ¥{total_revenue:,.2f}    趋势: {trend_pct:+.2f}%")
lines.append("")
max_total = max(r["total"] for r in region_stats)
for r in region_stats:
    width = max(1, round(20 * r["total"] / max_total))
    lines.append(f"{r['region']:<3}  {r['total']:>10,.2f}  {'█' * width}")
lines.append("")
lines.append(f"异常订单: {len(anomalies)}    最大异常: {anomalies[0]['id'] if anomalies else '无'}")
lines.append("=" * 56)

report = {
    "title": "Marqdo Python 经营洞察",
    "order_count": len(orders),
    "total_revenue": total_revenue,
    "trend_pct": trend_pct,
    "regions": region_stats,
    "top_products": metrics["top_products"],
    "anomaly_count": len(anomalies),
}
report_path = "/tmp/marqdo-python-insights-report.json"
with open(report_path, "w", encoding="utf-8") as f:
    json.dump(report, f, ensure_ascii=False, indent=2)

print("\n".join(lines))
print(f"\nJSON 报告已写入: {report_path}")
```

`独立复核` =
```python
import json
import os

path = "/tmp/marqdo-python-insights-report.json"
print("独立内核中 orders 可见:", "orders" in globals())

if not os.path.exists(path):
    print("未找到报告，跳过复核")
else:
    with open(path, encoding="utf-8") as f:
        report = json.load(f)
    region_sum = round(sum(r["total"] for r in report["regions"]), 2)
    total_ok = abs(region_sum - report["total_revenue"]) < 0.01
    count_ok = report["order_count"] == 180
    regions_ok = len(report["regions"]) == 4
    print(f"报告订单数: {report['order_count']}")
    print(f"分区域合计: ¥{region_sum:,.2f} / 总额 ¥{report['total_revenue']:,.2f}")
    print(f"区域数: {len(report['regions'])}    异常数: {report['anomaly_count']}")
    print("复核通过:", total_ok and count_ok and regions_ok)
```
