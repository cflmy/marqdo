minimum = {}
for edge in data['edges']:
    if edge['weight'] >= 0:
        key = (edge['from'], edge['to'])
        minimum[key] = min(minimum.get(key, edge['weight']), edge['weight'])
bridge = {'source': data['source'], 'nodes': sorted(data['nodes'], key=lambda n:n.encode('utf-8')),
          'edges': [{'from': a, 'to': b, 'weight': w} for (a,b), w in sorted(minimum.items(), key=lambda kv: (kv[0][0].encode('utf-8'), kv[0][1].encode('utf-8')))]}
bridge
