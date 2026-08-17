---
title: quantum bell smoke
description: H+CX Bell state probabilities ≈ 1/2 each.
import quantum:ext/quantum/quantum.mq.md
import math:lib/math.mq.md
---

# main

*qc = > quantum.circuit qubits=2*
*qc = > `qc`.h qubit=0*
*qc = > `qc`.cx control=0 target=1*
*p = > `qc`.probabilities*

*p00 = p[^00]*
*p11 = p[^11]*
*lo = > math.div a=2 b=5*
*hi = > math.div a=3 b=5*

1. `p00` > `lo`
  1. `p00` < `hi`
    > print text=p00-ok
  2. *
    > print text=p00-fail
2. *
  > print text=p00-fail

1. `p11` > `lo`
  1. `p11` < `hi`
    > print text=p11-ok
  2. *
    > print text=p11-fail
2. *
  > print text=p11-fail

*ping = > quantum_ping*
*ok = ping[^ok]*
1. `ok`
  > print text=ping-ok
2. *
  > print text=ping-fail
