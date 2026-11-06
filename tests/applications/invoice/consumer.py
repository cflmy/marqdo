from collections import defaultdict
totals = defaultdict(lambda: [0, 0])
for row in bridge:
    totals[row['region']][0] += 1
    totals[row['region']][1] += row['units'] * row['cents'] * (10000-row['discount']) // 10000
answer = [{'region': k, 'invoices': v[0], 'net_cents': v[1]}
          for k, v in sorted(totals.items(), key=lambda kv: kv[0].encode('utf-8'))]
answer
