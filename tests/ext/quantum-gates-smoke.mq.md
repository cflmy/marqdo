---
title: quantum gates + run smoke
description: Z sandwich, Ry(pi), seeded Bell run counts.
> ext/quantum/quantum.mq.md
> lib/math.mq.md
> lib/json.mq.md
---

# main

H-Z-H on |0⟩ → |1⟩.

*`qc` = > quantum.circuit qubits=1 *
*`qc` = > `qc`.h qubit=0 *
*`qc` = > `qc`.z qubit=0 *
*`qc` = > `qc`.h qubit=0 *
*`p` = > `qc`.probabilities *
*`p1` = > json.get value=`p` key="1" *
*`lo` = > math.div a=9 b=10 *

1. `p1` > `lo`
  > print text=z-sandwich-ok
2. *
  > print text=z-sandwich-fail

Ry(π) on |0⟩ → |1⟩.

*`pi` = > math.pi *
*`ry` = > quantum.circuit qubits=1 *
*`ry` = > `ry`.ry qubit=0 theta=`pi` *
*`rp` = > `ry`.probabilities *
*`rp1` = > json.get value=`rp` key="1" *

1. `rp1` > `lo`
  > print text=ry-ok
2. *
  > print text=ry-fail

Bell run with fixed seed: both 00 and 11 appear; shots sum.

*`bell` = > quantum.circuit qubits=2 *
*`bell` = > `bell`.h qubit=0 *
*`bell` = > `bell`.cx control=0 target=1 *
*`res` = > `bell`.run shots=200 seed=42 *
*`counts` = > json.get value=`res` key=counts *
*`c00` = > json.get value=`counts` key="00" *
*`c11` = > json.get value=`counts` key="11" *
*`shots` = > json.get value=`res` key=shots *

1. `c00`
  1. `c11`
    > print text=run-ok
  2. *
    > print text=run-fail
2. *
  > print text=run-fail

1. `shots` == 200
  > print text=shots-ok
2. *
  > print text=shots-fail
