---
markup: occlusion
title: ext/linalg/linalg
description: Formula-first linear algebra (English). MatExpr simplify + display; ABI plugin.
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
---

Formula-first linear algebra (English). MatExpr simplify + display; ABI plugin.

Use [the linear-algebra workflow](index.mq.md) to build a matrix from readable formula data, factor it once and solve through its receiver. This leaf owns the shared native matrix expression API. Keep dimensions, tolerance units, lazy expressions and materialized data distinct in new operations. Native ABI changes belong in marqdo_plugin_linalg; numeric validation should compare results and residuals rather than accepting a nonempty handle.

## ensure_plugin

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | No business value on completion; a missing native plugin reports the cause and exits this run. |

Load the ABI v2 `linalg` plugin once.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="linalg"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/linalg: native linalg plugin not found (build marqdo_plugin_linalg or marqdo ext add linalg)"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
Return *None*.

## ping

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Plugin health check. |

Plugin health check.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Plugin health check.](linalg_ping())*.

## mul

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | Left matrix or numeric operand |
| b | any | required | Right matrix, right-hand side or numeric operand |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matrix product (lazy MatMul; shape-checked). |

Use `a`, `b` as the inputs for this operation.

Matrix product (lazy MatMul; shape-checked).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Matrix product (lazy MatMul; shape-checked).](linalg_mul(a=`a`, b=`b`))*.

## add

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | Left matrix or numeric operand |
| b | any | required | Right matrix, right-hand side or numeric operand |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matrix / expression addition. |

Use `a`, `b` as the inputs for this operation.

Matrix / expression addition.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Matrix / expression addition.](linalg_add(a=`a`, b=`b`))*.

## sub

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | Left matrix or numeric operand |
| b | any | required | Right matrix, right-hand side or numeric operand |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matrix / expression subtraction. |

Use `a`, `b` as the inputs for this operation.

Matrix / expression subtraction.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Matrix / expression subtraction.](linalg_sub(a=`a`, b=`b`))*.

## transpose

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Transpose expression. |

Use `expr` as the inputs for this operation.

Transpose expression.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Transpose expression.](linalg_transpose(expr=`expr`))*.

## inv

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Symbolic inverse (lazy). |

Use `expr` as the inputs for this operation.

Symbolic inverse (lazy).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Symbolic inverse (lazy).](linalg_inv(expr=`expr`))*.

## simplify

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Apply MatExpr rewrite rules (R1–R5). |

Use `expr` as the inputs for this operation.

Apply MatExpr rewrite rules (R1–R5).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Apply MatExpr rewrite rules (R1–R5).](linalg_simplify(expr=`expr`))*.

## ascii

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |
| raw | any | False | Whether to keep the unsimplified expression tree while rendering |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Display ASCII (default simplify first). Pass `raw=True` to keep the written tree. |

Use `expr`, `raw`=False as the inputs for this operation.

Display ASCII (default simplify first). Pass `raw=True` to keep the written tree.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Display ASCII (default simplify first). Pass raw=True to keep the written tree.](linalg_ascii(expr=`expr`, raw=`raw`))*.

## latex

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |
| raw | any | False | Whether to keep the unsimplified expression tree while rendering |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | LaTeX display. |

Use `expr`, `raw`=False as the inputs for this operation.

LaTeX display.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[LaTeX display.](linalg_latex(expr=`expr`, raw=`raw`))*.

## shape

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matrix shape (rows, cols). |

Use `expr` as the inputs for this operation.

Matrix shape (rows, cols).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Matrix shape (rows, cols).](linalg_shape(expr=`expr`))*.

## show

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |
| path | any | None | Explicit filesystem path; honor the documented filesystem root and operation effects |
| raw | any | False | Whether to keep the unsimplified expression tree while rendering |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Record a formula SVG via host plot channel (default simplify). Returns the expr map. |

Use `expr`, `path`=None, `raw`=False as the inputs for this operation.

Record a formula SVG via host plot channel (default simplify). Returns the expr map.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Record a formula SVG via host plot channel (default simplify). Returns the expr map.](linalg_show(expr=`expr`, path=`path`, raw=`raw`))*.

## eye

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| n | any | required | Matrix order |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Identity matrix of order n. |

Use `n` as the inputs for this operation.

Identity matrix of order n.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Identity matrix of order n.](linalg_eye(n=`n`))*.

## zeros

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| rows | any | required | Input row records to insert or process |
| cols | any | required | Matrix column count |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Zero matrix. |

Use `rows`, `cols` as the inputs for this operation.

Zero matrix.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Zero matrix.](linalg_zeros(rows=`rows`, cols=`cols`))*.

## from_list

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | any | required | Structured input supplied to this adapter or resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Dense leaf from nested list. |

Use `data` as the inputs for this operation.

Dense leaf from nested list.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Dense leaf from nested list.](linalg_from_list(data=`data`))*.

## from_formula

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| formula | any | required | Symbolic formula accepted by the documented parser |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Absorb a `$$` numeric matrix formula (or nested list / dense map) as a dense leaf. |

Use `formula` as the inputs for this operation.

Absorb a `$$` numeric matrix formula (or nested list / dense map) as a dense leaf.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Absorb a $$ numeric matrix formula (or nested list / dense map) as a dense leaf.](linalg_from_formula(formula=`formula`))*.

## explicit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |
| raw | any | False | Whether to keep the unsimplified expression tree while rendering |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Evaluate to dense (default simplify first). |

Use `expr`, `raw`=False as the inputs for this operation.

Evaluate to dense (default simplify first).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Evaluate to dense (default simplify first).](linalg_explicit(expr=`expr`, raw=`raw`))*.

## det

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Determinant. |

Use `expr` as the inputs for this operation.

Determinant.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Determinant.](linalg_det(expr=`expr`))*.

## trace

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Trace. |

Use `expr` as the inputs for this operation.

Trace.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Trace.](linalg_trace(expr=`expr`))*.

## solve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | None | Left matrix or numeric operand |
| b | any | required | Right matrix, right-hand side or numeric operand |
| factor | any | None | Reusable factorization returned by factorize |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Solve `a x = b` (Gaussian elimination), or reuse `factorize kind=lu` via `factor` + `b`. |

Use `a`=None, `b`, `factor`=None as the inputs for this operation.

Solve `a x = b` (Gaussian elimination), or reuse `factorize kind=lu` via `factor` + `b`.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Solve a x = b (Gaussian elimination), or reuse factorize kind=lu via factor + b.](linalg_solve(a=`a`, b=`b`, factor=`factor`))*.

## matmul

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | Left matrix or numeric operand |
| b | any | required | Right matrix, right-hand side or numeric operand |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Dense matrix product → `linalg_dense`. |

Use `a`, `b` as the inputs for this operation.

Dense matrix product → `linalg_dense`.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Dense matrix product → linalg_dense.](linalg_matmul(a=`a`, b=`b`))*.

## kron

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | Left matrix or numeric operand |
| b | any | required | Right matrix, right-hand side or numeric operand |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Lazy Kronecker product (does not expand). |

Use `a`, `b` as the inputs for this operation.

Lazy Kronecker product (does not expand).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Lazy Kronecker product (does not expand).](linalg_kron(a=`a`, b=`b`))*.

## block

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| blocks | any | required | Nested row list of block matrix expressions |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Block matrix from a nested list of expressions. |

Use `blocks` as the inputs for this operation.

Block matrix from a nested list of expressions.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Block matrix from a nested list of expressions.](linalg_block(blocks=`blocks`))*.

## collapse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Block-level collapse (`Block*Block` → block of products). |

Use `expr` as the inputs for this operation.

Block-level collapse (`Block*Block` → block of products).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Block-level collapse (Block*Block → block of products).](linalg_collapse(expr=`expr`))*.

## factorize

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| matrix | any | required | Matrix value supplied for evaluation or factorization |
| kind | any | "lu" | Operation or factorization kind listed in this function's behavior |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Dense factorization → `linalg_factor` (`kind` = `lu` &#124; `qr` &#124; `svd` &#124; `eig` &#124; `chol`). |

Use `matrix`, `kind`="lu" as the inputs for this operation.

Dense factorization → `linalg_factor` (`kind` = `lu` | `qr` | `svd` | `eig` | `chol`).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Dense factorization → linalg_factor (kind = lu | qr | svd | eig | chol).](linalg_factorize(matrix=`matrix`, kind=`kind`))*.

## draw

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| factor | any | required | Reusable factorization returned by factorize |
| kind | any | None | Operation or factorization kind listed in this function's behavior |
| path | any | None | Explicit filesystem path; honor the documented filesystem root and operation effects |
| theme | any | "light" | Rendering theme token supported by the implementation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Structure SVG (`eig` &#124; `svd` &#124; `qr` &#124; `lu` &#124; `ge` &#124; `chol` &#124; `structure` &#124; `heatmap` &#124; `hinton`). Theme tokens match quantum Q8: `light` &#124; `dark` &#124; `bw`. |

Use `factor`, `kind`=None, `path`=None, `theme`="light" as the inputs for this operation.

Structure SVG (`eig` | `svd` | `qr` | `lu` | `ge` | `chol` | `structure` | `heatmap` | `hinton`). Theme tokens match quantum Q8: `light` | `dark` | `bw`.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Structure SVG (eig | svd | qr | lu | ge | chol | structure | heatmap | hinton). Theme tokens match quantum Q8: light | dark | bw.](linalg_draw(factor=`factor`, kind=`kind`, path=`path`, theme=`theme`))*.

## lstsq

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | Left matrix or numeric operand |
| b | any | required | Right matrix, right-hand side or numeric operand |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Least squares `min &#124;&#124;a x - b&#124;&#124;` (QR if tall, SVD if wide). |

Use `a`, `b` as the inputs for this operation.

Least squares `min ||a x - b||` (QR if tall, SVD if wide).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Least squares min ||a x - b|| (QR if tall, SVD if wide).](linalg_lstsq(a=`a`, b=`b`))*.

## norm

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |
| ord | any | "fro" | Norm order accepted by the native norm implementation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matrix norm: `fro` &#124; `1` &#124; `inf` &#124; `2`. Complex dense supports `fro` only. |

Use `expr`, `ord`="fro" as the inputs for this operation.

Matrix norm: `fro` | `1` | `inf` | `2`. Complex dense supports `fro` only.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Matrix norm: fro | 1 | inf | 2. Complex dense supports fro only.](linalg_norm(expr=`expr`, ord=`ord`))*.

## cond

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Spectral condition number via SVD (`ord=2`). |

Use `expr` as the inputs for this operation.

Spectral condition number via SVD (`ord=2`).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Spectral condition number via SVD (ord=2).](linalg_cond(expr=`expr`))*.

## rank

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expr | any | required | Matrix expression produced by this library, or an accepted symbolic expression |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Numerical rank (SVD). |

Use `expr` as the inputs for this operation.

Numerical rank (SVD).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Numerical rank (SVD).](linalg_rank(expr=`expr`))*.

## declare

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Declare several matrix symbols from an `@` record table (`name` / `rows` / `cols`). Returns a map env. |

Use `table` as the inputs for this operation.

Declare several matrix symbols from an `@` record table (`name` / `rows` / `cols`). Returns a map env.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Declare several matrix symbols from an @ record table (name / rows / cols). Returns a map env.](linalg_declare(table=`table`, type_tag="matrix"))*.

## symbol

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | required | Name registered or looked up by this operation |
| rows | any | required | Input row records to insert or process |
| cols | any | required | Matrix column count |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Abstract matrix symbol. Returns a `# matrix` instance (supports `+` / `*` and methods). |

Use `name`, `rows`, `cols` as the inputs for this operation.

Abstract matrix symbol. Returns a `# matrix` instance (supports `+` / `*` and methods).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Matrix](matrix(name=`name`, rows=`rows`, cols=`cols`))*.

# matrix

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | None | Name registered or looked up by this operation |
| rows | any | None | Input row records to insert or process |
| cols | any | None | Matrix column count |
| data | any | None | Structured input supplied to this adapter or resource |
| formula | any | None | Symbolic formula accepted by the documented parser |
| n | any | None | Matrix order |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matrix value (`_type=matrix`). Prefer infix `A + B` / `A * B`, or methods `` `A`.T ``. |

Use `name`=None, `rows`=None, `cols`=None, `data`=None, `formula`=None, `n`=None as the inputs for this operation.

Matrix value (`_type=matrix`). Prefer infix `A + B` / `A * B`, or methods `` `A`.T ``.

We **[Load the native plugin required by this operation](ensure_plugin())**.
1. `formula`
  Return *[Matrix value (_type=matrix). Prefer infix A + B / A * B, or methods  A.T .](linalg_from_formula(formula=`formula`))*.
2. `data`
  Return *[Matrix value (_type=matrix). Prefer infix A + B / A * B, or methods  A.T .](linalg_from_list(data=`data`))*.
3. `name`
  Return *[Matrix value (_type=matrix). Prefer infix A + B / A * B, or methods  A.T .](linalg_symbol(name=`name`, rows=`rows`, cols=`cols`))*.
4. `n`
  Return *[Matrix value (_type=matrix). Prefer infix A + B / A * B, or methods  A.T .](linalg_eye(n=`n`))*.
5. `rows`
  Return *[Matrix value (_type=matrix). Prefer infix A + B / A * B, or methods  A.T .](linalg_zeros(rows=`rows`, cols=`cols`))*.
6. *
  We **[Print the prepared message](print(text="ext/linalg: matrix needs", name="/", data="/", formula="/", n="/", rows=""))**.
  Return *None*.

## T

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Transpose (method). |

Transpose (method).

Return *[Transpose (method).](linalg_transpose(expr=`self`))*.

## T_ascii

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Transpose then display ASCII (auto-simplified). Handy for lecture identities. |

Transpose then display ASCII (auto-simplified). Handy for lecture identities.

We retain **`Pt` = [Transpose then display ASCII (auto-simplified). Handy for lecture identities.](linalg_transpose(expr=`self`))**.
Return *[Transpose then display ASCII (auto-simplified). Handy for lecture identities.](linalg_ascii(expr=`Pt`, raw=False))*.

## add

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| b | any | required | Right matrix, right-hand side or numeric operand |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matrix / expression addition. |

Use `b` as the inputs for this operation.

Matrix / expression addition.

Return *[Matrix / expression addition.](linalg_add(a=`self`, b=`b`))*.

## sub

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| b | any | required | Right matrix, right-hand side or numeric operand |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matrix / expression subtraction. |

Use `b` as the inputs for this operation.

Matrix / expression subtraction.

Return *[Matrix / expression subtraction.](linalg_sub(a=`self`, b=`b`))*.

## mul

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| b | any | required | Right matrix, right-hand side or numeric operand |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matrix / expression product. |

Use `b` as the inputs for this operation.

Matrix / expression product.

Return *[Matrix / expression product.](linalg_mul(a=`self`, b=`b`))*.

## inv

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Inverse. |

Inverse.

Return *[Inverse.](linalg_inv(expr=`self`))*.

## simplify

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Simplify MatExpr. |

Simplify MatExpr.

Return *[Simplify MatExpr.](linalg_simplify(expr=`self`))*.

## ascii

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | ASCII display. |

ASCII display.

Return *[ASCII display.](linalg_ascii(expr=`self`, raw=False))*.

## latex

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | LaTeX display. |

LaTeX display.

Return *[LaTeX display.](linalg_latex(expr=`self`, raw=False))*.

## explicit

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Evaluate to an explicit matrix. |

Evaluate to an explicit matrix.

Return *[Evaluate to an explicit matrix.](linalg_explicit(expr=`self`, raw=False))*.

## det

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Determinant. |

Determinant.

Return *[Determinant.](linalg_det(expr=`self`))*.

## trace

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Trace. |

Trace.

Return *[Trace.](linalg_trace(expr=`self`))*.

## show

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | None | Explicit filesystem path; honor the documented filesystem root and operation effects |
| raw | any | False | Whether to keep the unsimplified expression tree while rendering |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Show / export the expression. |

Use `path`=None, `raw`=False as the inputs for this operation.

Show / export the expression.

Return *[Show / export the expression.](linalg_show(expr=`self`, path=`path`, raw=`raw`))*.

## eye

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| n | any | required | Matrix order |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Identity matrix of order n. |

Use `n` as the inputs for this operation.

Identity matrix of order n.

Return *[Identity matrix of order n.](linalg_eye(n=`n`))*.

## zeros

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| rows | any | required | Input row records to insert or process |
| cols | any | required | Matrix column count |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Zero matrix. |

Use `rows`, `cols` as the inputs for this operation.

Zero matrix.

Return *[Zero matrix.](linalg_zeros(rows=`rows`, cols=`cols`))*.

## from_list

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | any | required | Structured input supplied to this adapter or resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matrix from nested list. |

Use `data` as the inputs for this operation.

Matrix from nested list.

Return *[Matrix from nested list.](linalg_from_list(data=`data`))*.

## from_formula

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| formula | any | required | Symbolic formula accepted by the documented parser |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matrix from formula text. |

Use `formula` as the inputs for this operation.

Matrix from formula text.

Return *[Matrix from formula text.](linalg_from_formula(formula=`formula`))*.
