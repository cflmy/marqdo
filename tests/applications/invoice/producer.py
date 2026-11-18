seen = set()
bridge = []
for row in data:
    if row['status'] == 'posted' and row['id'] not in seen:
        seen.add(row['id'])
        bridge.append({k: row[k] for k in ('id', 'region', 'units', 'cents', 'discount')})
bridge.sort(key=lambda r: r['id'].encode('utf-8'))
bridge
