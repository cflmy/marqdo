var distance = new Map(bridge.nodes.map(n=>[n,Infinity]));
distance.set(bridge.source,0);
var unvisited = new Set(bridge.nodes);
while (unvisited.size) {
  var best = [...unvisited].reduce((a,b)=>distance.get(a)<=distance.get(b)?a:b);
  unvisited.delete(best);
  if (!Number.isFinite(distance.get(best))) break;
  for (var edge of bridge.edges) {
    if (edge.from === best && distance.get(best)+edge.weight < distance.get(edge.to))
      distance.set(edge.to,distance.get(best)+edge.weight);
  }
}
var answer = bridge.nodes.map(node=>({node,distance:Number.isFinite(distance.get(node))?distance.get(node):null}));
answer;
