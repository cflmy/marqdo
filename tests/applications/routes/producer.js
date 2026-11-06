var minimum = new Map();
for (var edge of data.edges) {
  if (edge.weight >= 0) {
    var key = JSON.stringify([edge.from, edge.to]);
    minimum.set(key, Math.min(minimum.has(key) ? minimum.get(key) : edge.weight, edge.weight));
  }
}
var edges = [...minimum].map(([key,weight]) => ({from: JSON.parse(key)[0], to: JSON.parse(key)[1], weight}));
edges.sort((a,b) => Buffer.compare(Buffer.from(a.from),Buffer.from(b.from)) || Buffer.compare(Buffer.from(a.to),Buffer.from(b.to)));
var bridge = {source:data.source, nodes:[...data.nodes].sort((a,b)=>Buffer.compare(Buffer.from(a),Buffer.from(b))), edges};
bridge;
