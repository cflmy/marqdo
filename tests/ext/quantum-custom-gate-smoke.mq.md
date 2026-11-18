---
markup: occlusion
title: quantum custom gate from formula matrix
description: $$ H matrix $$ → gate matrix= → apply → probs match built-in H.
import:
  quantum: ext/quantum/circuit/index.mq.md
  gate: ext/quantum/gate/index.mq.md
  math: lib/math.mq.md
---

# main

`H_matrix` =
$$
\frac{1}{\sqrt{2}}\begin{pmatrix}1&1\\1&-1\end{pmatrix}
$$

**U = > gate.gate matrix=`H_matrix` name="U"**
**qc = > quantum.circuit qubits=1**
**qc = > `qc`.apply gate=`U` qubits=0**
**p = > `qc`.probabilities**
**p0 = [0](access(p, "0"))**
**lo = > math.div a=4 b=10**
**hi = > math.div a=6 b=10**

1. `p0` > `lo`
  1. `p0` < `hi`
    > print text=custom-ok
  2. *
    > print text=custom-fail
2. *
  > print text=custom-fail
