var seen = new Set();
var bridge = [];
for (var row of data) {
  if (row.status === 'posted' && !seen.has(row.id)) {
    seen.add(row.id);
    bridge.push({id: row.id, region: row.region, units: row.units, cents: row.cents, discount: row.discount});
  }
}
bridge.sort((a, b) => Buffer.compare(Buffer.from(a.id), Buffer.from(b.id)));
bridge;
