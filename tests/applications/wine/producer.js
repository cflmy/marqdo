function scaledDecimal(s, digits) {
  var parts = s.split('.');
  var tail = parts.length > 1 ? parts[1] : '';
  if (tail.length > digits) throw Error('unsupported decimal precision');
  return Number(parts[0])*10**digits + Number(tail.padEnd(digits,'0'));
}
var lines = data.trim().split(/\r?\n/);
var header = lines[0].split(';').map(s=>s.replaceAll('"',''));
var bridge = lines.slice(1).map(line=>{
  var fields=line.split(';');
  return {quality:Number(fields[header.indexOf('quality')]),density_ppm:scaledDecimal(fields[header.indexOf('density')],6),ph_cent:scaledDecimal(fields[header.indexOf('pH')],2)};
});
bridge;
