---
title: ext/linalg/linalg
description: Formula-first linear algebra (English). MatExpr simplify + display; ABI plugin.
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

## ensure_plugin

Load the ABI v2 `linalg` plugin once.

*p = > plugin.native_path name="linalg"*
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/linalg: native linalg plugin not found (build marqdo_plugin_linalg or marqdo ext add linalg)
  > sys.exit code=1
****

## ping

> ensure_plugin
**> linalg_ping**

## mul
    + `a`
    + `b`

Matrix product (lazy MatMul; shape-checked).

> ensure_plugin
**> linalg_mul a=`a` b=`b`**

## add
    + `a`
    + `b`

> ensure_plugin
**> linalg_add a=`a` b=`b`**

## sub
    + `a`
    + `b`

> ensure_plugin
**> linalg_sub a=`a` b=`b`**

## transpose
    + `expr`

> ensure_plugin
**> linalg_transpose expr=`expr`**

## inv
    + `expr`

Symbolic inverse (lazy).

> ensure_plugin
**> linalg_inv expr=`expr`**

## simplify
    + `expr`

Apply MatExpr rewrite rules (R1–R5).

> ensure_plugin
**> linalg_simplify expr=`expr`**

## ascii
    + `expr`

> ensure_plugin
**> linalg_ascii expr=`expr`**

## latex
    + `expr`

> ensure_plugin
**> linalg_latex expr=`expr`**

## shape
    + `expr`

> ensure_plugin
**> linalg_shape expr=`expr`**

## show
    + `expr`
    + `path`=None

Record a formula SVG via host plot channel; returns the expr map.

> ensure_plugin
**> linalg_show expr=`expr` path=`path`**

## eye
    + `n`

> ensure_plugin
**> linalg_eye n=`n`**

## zeros
    + `rows`
    + `cols`

> ensure_plugin
**> linalg_zeros rows=`rows` cols=`cols`**

## from_list
    + `data`

Dense leaf from nested list.

> ensure_plugin
**> linalg_from_list data=`data`**

## symbol
    + `name`
    + `rows`
    + `cols`

Abstract `MatrixSymbol` (`linalg_expr`). Prefer this over `matrix.symbol` (needs a receiver).

> ensure_plugin
**> linalg_symbol name=`name` rows=`rows` cols=`cols`**

# matrix

Constructor-style helpers (call after `> la.matrix` if you hold an instance; prefer top-level `symbol` / `eye` / `zeros` / `from_list`).

## symbol
    + `name`
    + `rows`
    + `cols`

**> linalg_symbol name=`name` rows=`rows` cols=`cols`**

## eye
    + `n`

**> linalg_eye n=`n`**

## zeros
    + `rows`
    + `cols`

**> linalg_zeros rows=`rows` cols=`cols`**

## from_list
    + `data`

**> linalg_from_list data=`data`**
