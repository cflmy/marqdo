import heapq
adjacency = {node: [] for node in bridge['nodes']}
for edge in bridge['edges']:
    adjacency[edge['from']].append((edge['to'], edge['weight']))
distance = {bridge['source']: 0}
pending = [(0, bridge['source'])]
while pending:
    cost, node = heapq.heappop(pending)
    if distance[node] != cost:
        continue
    for target, weight in adjacency[node]:
        candidate = cost + weight
        if target not in distance or candidate < distance[target]:
            distance[target] = candidate
            heapq.heappush(pending, (candidate, target))
answer = [{'node': n, 'distance': distance.get(n)} for n in bridge['nodes']]
answer
