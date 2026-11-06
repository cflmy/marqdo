var index = new Map();
for (var row of bridge) {
  for (var word of row.tokens) {
    var value = index.get(word) || {count:0,docs:new Set()};
    value.count++; value.docs.add(row.id); index.set(word,value);
  }
}
var answer = [...index].sort((a,b)=>Buffer.compare(Buffer.from(a[0]),Buffer.from(b[0])))
  .map(([word,v])=>({word,occurrences:v.count,documents:v.docs.size}));
answer;
