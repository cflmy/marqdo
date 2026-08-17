---
title: quantum author API smoke
description: barrier, measure, append, state, draw meters.
import quantum:ext/quantum/quantum.mq.md
---

# main

*qc = > quantum.circuit qubits=2*
*qc = > `qc`.h qubit=0*
*qc = > `qc`.barrier*
*qc = > `qc`.cx control=0 target=1*
*qc = > `qc`.measure*

*st = > `qc`.state*
*dim = st[^dim]*
1. `dim` == 4
  > print text=state-ok
2. *
  > print text=state-fail

*img = > `qc`.draw path=quantum-author-draw.svg*
*kind = img[^kind]*
1. `kind` == circuit
  > print text=draw-ok
2. *
  > print text=draw-fail

*other = > quantum.circuit qubits=2*
*other = > `other`.x qubit=1*
*qc2 = > `qc`.append op=`other`*
*ops = qc2[^ops]*
*n = > len `ops`*
1. `n` == 5
  > print text=append-ok
2. *
  > print text=append-fail
