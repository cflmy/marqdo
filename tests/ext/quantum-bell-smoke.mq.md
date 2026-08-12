---
title: quantum bell smoke
description: H+CX Bell state probabilities ≈ 1/2 each.
> ext/quantum/quantum.mq.md
> lib/math.mq.md
> lib/json.mq.md
---

# main

*`qc` = > quantum.circuit qubits=2 *
*`qc` = > `qc`.h qubit=0 *
*`qc` = > `qc`.cx control=0 target=1 *
*`p` = > `qc`.probabilities *

*`p00` = > json.get value=`p` key="00" *
*`p11` = > json.get value=`p` key="11" *
*`lo` = > math.div a=2 b=5 *
*`hi` = > math.div a=3 b=5 *

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

*`ping` = > quantum_ping *
*`ok` = > json.get value=`ping` key=ok *
1. `ok`
  > print text=ping-ok
2. *
  > print text=ping-fail
