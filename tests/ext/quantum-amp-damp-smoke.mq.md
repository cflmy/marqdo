---
title: quantum amplitude damping smoke
description: X then amplitude_damping p=1 returns all shots to |0>.
import quantum:ext/quantum/quantum.mq.md
---

# main

*qc = > quantum.circuit qubits=1*
*qc = > `qc`.x qubit=0*
*qc = > `qc`.noise kind="amplitude_damping" p=1*
*r = > `qc`.run shots=32 seed=7*
*c0 = r[^counts][^0]*

1. `c0` == 32
  > print text=amp-ok
2. *
  > print text=amp-fail
