---
title: linalg factor smoke (L4)
description: eig factorize, structure SVG, LU reuse solve.
import la:ext/linalg/linalg.mq.md
import math:lib/math.mq.md
---

# main

`M` =
$$
\begin{bmatrix}2&0\\0&3\end{bmatrix}
$$

*`M` = > la.from_formula formula=`M`*
*`eps` = > math.div a=1 b=1000*

*`f` = > la.factorize matrix=`M` kind="eig"*
*`kind` = f[^kind]*
1. `kind` == eig
  > print text=eig-kind-ok
2. *
  > print text=eig-kind-fail

*`evals` = f[^eigenvalues]*
*`ev0` = > at value=`evals` index=0*
*`e0` = > math.sub a=`ev0` b=3*
*`e0` = > math.abs value=`e0`*
1. `e0` < `eps`
  > print text=eig-val-ok
2. *
  > print text=eig-val-fail

*`img` = > la.draw factor=`f` kind="eig" path="linalg-factor-eig.svg"*
*`ik` = img[^kind]*
1. `ik` == eig
  > print text=draw-kind-ok
2. *
  > print text=draw-kind-fail

*`svg` = img[^svg]*
*`parts` = > split value=`svg` sep="data-linalg=\"eig\""*
*`pn` = > len value=`parts`*
1. `pn` > 1
  > print text=draw-mark-ok
2. *
  > print text=draw-mark-fail

`A` =
$$
\begin{bmatrix}2&1\\1&3\end{bmatrix}
$$

`b` =
$$
\begin{pmatrix}5\\5\end{pmatrix}
$$

*`A` = > la.from_formula formula=`A`*
*`b` = > la.from_formula formula=`b`*
*`lu` = > la.factorize matrix=`A` kind="lu"*
*`sol` = > la.solve factor=`lu` b=`b`*
*`x` = sol[^x]*
*`data` = x[^data]*
*`row0` = > at value=`data` index=0*
*`x0` = > at value=`row0` index=0*
*`xe` = > math.sub a=`x0` b=2*
*`xe` = > math.abs value=`xe`*
1. `xe` < `eps`
  > print text=lu-solve-ok
2. *
  > print text=lu-solve-fail

*`svd` = > la.factorize matrix=`M` kind="svd"*
*`sk` = svd[^ascii]*
1. `sk` == SVD
  > print text=svd-ok
2. *
  > print text=svd-fail
