from itertools import groupby
answer = []
for quality, group in groupby(sorted(bridge,key=lambda r:r['quality']),key=lambda r:r['quality']):
    rows = list(group)
    answer.append({'quality':quality,'count':len(rows),'sum_density_ppm':sum(r['density_ppm'] for r in rows),
                   'min_ph_cent':min(r['ph_cent'] for r in rows),'max_ph_cent':max(r['ph_cent'] for r in rows)})
answer
