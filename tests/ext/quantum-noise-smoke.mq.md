---
title: quantum noise smoke
description: bitflip p=1 after I flips |0> to |1| for all shots.
import quantum:ext/quantum/quantum.mq.md
---

# main

*qc = > quantum.circuit qubits=1*
*qc = > `qc`.i qubit=0*
*qc = > `qc`.noise kind="bitflip" p=1*
*r = > `qc`.run shots=32 seed=3*
*c1 = r[^counts][^1]*

1. `c1` == 32
  > print text=noise-ok
2. *
  > print text=noise-fail
