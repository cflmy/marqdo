var groups = new Map();
for (var row of bridge) {
  var g=groups.get(row.quality) || {quality:row.quality,count:0,sum_density_ppm:0,min_ph_cent:row.ph_cent,max_ph_cent:row.ph_cent};
  g.count++;g.sum_density_ppm+=row.density_ppm;
  g.min_ph_cent=Math.min(g.min_ph_cent,row.ph_cent);g.max_ph_cent=Math.max(g.max_ph_cent,row.ph_cent);
  groups.set(row.quality,g);
}
var answer=[...groups.values()].sort((a,b)=>a.quality-b.quality);
answer;
