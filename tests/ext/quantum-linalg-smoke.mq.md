---
title: quantum linalg smoke
description: Q7a density / partial_trace / expect / schmidt / purity
import quantum:ext/quantum/quantum.mq.md
import math:lib/math.mq.md
---

# main

*qc = > quantum.circuit qubits=2*
*qc = > `qc`.h qubit=0*
*qc = > `qc`.cx control=0 target=1*

*rho = > `qc`.density*
*pur = > `rho`.purity*
*lo = > math.div a=9 b=10*
1. `pur` > `lo`
  > print text=purity-ok
2. *
  > print text=purity-fail

*red = > `rho`.partial_trace keep=0*
*rpur = > `red`.purity*
*half_lo = > math.div a=4 b=10*
*half_hi = > math.div a=6 b=10*
1. `rpur` > `half_lo`
  1. `rpur` < `half_hi`
    > print text=reduce-ok
  2. *
    > print text=reduce-fail
2. *
  > print text=reduce-fail

*zz = "ZZ"*
*ezz = > `qc`.expect obs=`zz`*
*zzi = > math.div a=9 b=10*
1. `ezz` > `zzi`
  > print text=expect-ok
2. *
  > print text=expect-fail

*sch = > `qc`.schmidt cut=1*
*ent = sch[^entropy]*
*elo = > math.div a=1 b=2*
1. `ent` > `elo`
  > print text=schmidt-ok
2. *
  > print text=schmidt-fail
