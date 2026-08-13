---
title: quantum draw probs/bloch smoke
description: draw kind=probs and kind=bloch return matching kind tags.
import quantum:ext/quantum/quantum.mq.md
---

# main

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

*`qc` = > quantum.circuit qubits=2 steps=`steps` *
*`img` = > `qc`.draw kind=probs path=quantum-draw-probs.svg *
*`kind` = `img`[^kind] *

1. `kind` == probs
  > print text=probs-ok
2. *
  > print text=probs-fail

*`qc1` = > quantum.circuit qubits=1 *
*`qc1` = > `qc1`.h qubit=0 *
*`bloch` = > `qc1`.draw kind=bloch path=quantum-draw-bloch.svg *
*`bk` = `bloch`[^kind] *

1. `bk` == bloch
  > print text=bloch-ok
2. *
  > print text=bloch-fail
