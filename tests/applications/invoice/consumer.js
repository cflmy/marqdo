var totals = new Map();
for (var row of bridge) {
  var value = totals.get(row.region) || [0, 0];
  value[0] += 1;
  value[1] += Math.floor(row.units * row.cents * (10000-row.discount) / 10000);
  totals.set(row.region, value);
}
var answer = [...totals].sort((a,b) => Buffer.compare(Buffer.from(a[0]),Buffer.from(b[0])))
  .map(([region, value]) => ({region, invoices: value[0], net_cents: value[1]}));
answer;
