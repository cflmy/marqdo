---
markup: occlusion
title: lib/math
description: High-school math — num, random, formula, plot
---

High-school math — num, random, formula, plot

Compute numbers, symbolic expressions and plots.

Behavior and limits: Number contracts include symbolic formulas. Random values require random.read. Plotting emits an artifact and needs output.write; linear algebra and quantum workloads stay in ext.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/math.rs](../src/host/math.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## pi

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Return the mathematical constant pi |

Effects: pure.

Return the mathematical constant pi.

Return *[Return the mathematical constant pi](host_pi())*.

## e

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Return Euler constant e |

Effects: pure.

Return Euler constant e.

Return *[Return Euler constant e](host_e())*.

## num

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Convert a numeric value or numeric text to a number |

Use `value` as the inputs for this operation.

Effects: pure.

Convert a numeric value or numeric text to a number.

Return *[Convert a numeric value or numeric text to a number](> host_num value=`value`)*.

## add

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | First operand; its domain is described by this operation |
| b | any | required | Second operand; its domain is described by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Add the two numeric or symbolic operands |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Add the two numeric or symbolic operands.

Return *[Add the two numeric or symbolic operands](> host_math_add a=`a` b=`b`)*.

## sub

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | First operand; its domain is described by this operation |
| b | any | required | Second operand; its domain is described by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Subtract b from a |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Subtract b from a.

Return *[Subtract b from a](> host_math_sub a=`a` b=`b`)*.

## mul

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | First operand; its domain is described by this operation |
| b | any | required | Second operand; its domain is described by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Multiply a by b |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Multiply a by b.

Return *[Multiply a by b](> host_math_mul a=`a` b=`b`)*.

## div

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | First operand; its domain is described by this operation |
| b | any | required | Second operand; its domain is described by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Divide a by b; zero divisor is an error |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Divide a by b; zero divisor is an error.

Return *[Divide a by b; zero divisor is an error](> host_math_div a=`a` b=`b`)*.

## pow

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | First operand; its domain is described by this operation |
| b | any | required | Second operand; its domain is described by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Raise a to the power b |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Raise a to the power b.

Return *[Raise a to the power b](> host_math_pow a=`a` b=`b`)*.

## neg

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Negate a numeric or symbolic value |

Use `value` as the inputs for this operation.

Effects: pure.

Negate a numeric or symbolic value.

Return *[Negate a numeric or symbolic value](> host_math_neg value=`value`)*.

## sin

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Compute sine; numeric angles are radians |

Use `value` as the inputs for this operation.

Effects: pure.

Compute sine; numeric angles are radians.

Return *[Compute sine; numeric angles are radians](> host_math_sin value=`value`)*.

## cos

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Compute cosine; numeric angles are radians |

Use `value` as the inputs for this operation.

Effects: pure.

Compute cosine; numeric angles are radians.

Return *[Compute cosine; numeric angles are radians](> host_math_cos value=`value`)*.

## tan

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Compute tangent; numeric angles are radians |

Use `value` as the inputs for this operation.

Effects: pure.

Compute tangent; numeric angles are radians.

Return *[Compute tangent; numeric angles are radians](> host_math_tan value=`value`)*.

## asin

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Compute arcsine in radians |

Use `value` as the inputs for this operation.

Effects: pure.

Compute arcsine in radians.

Return *[Compute arcsine in radians](> host_math_asin value=`value`)*.

## acos

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Compute arccosine in radians |

Use `value` as the inputs for this operation.

Effects: pure.

Compute arccosine in radians.

Return *[Compute arccosine in radians](> host_math_acos value=`value`)*.

## atan

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Compute arctangent in radians |

Use `value` as the inputs for this operation.

Effects: pure.

Compute arctangent in radians.

Return *[Compute arctangent in radians](> host_math_atan value=`value`)*.

## sqrt

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Compute the square root of value |

Use `value` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/math.mq.md sqrt value=9 --effects none --json
```

Compute the square root of value.

Return *[Compute the square root of value](> host_math_sqrt value=`value`)*.

## abs

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Return absolute magnitude |

Use `value` as the inputs for this operation.

Effects: pure.

Return absolute magnitude.

Return *[Return absolute magnitude](> host_math_abs value=`value`)*.

## ln

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Compute natural logarithm |

Use `value` as the inputs for this operation.

Effects: pure.

Compute natural logarithm.

Return *[Compute natural logarithm](> host_math_ln value=`value`)*.

## exp

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Compute the exponential of value |

Use `value` as the inputs for this operation.

Effects: pure.

Compute the exponential of value.

Return *[Compute the exponential of value](> host_math_exp value=`value`)*.

## floor

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Round toward negative infinity |

Use `value` as the inputs for this operation.

Effects: pure.

Round toward negative infinity.

Return *[Round toward negative infinity](> host_math_floor value=`value`)*.

## ceil

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Round toward positive infinity |

Use `value` as the inputs for this operation.

Effects: pure.

Round toward positive infinity.

Return *[Round toward positive infinity](> host_math_ceil value=`value`)*.

## min

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | First operand; its domain is described by this operation |
| b | any | required | Second operand; its domain is described by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Return the smaller operand |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Return the smaller operand.

Return *[Return the smaller operand](> host_math_min a=`a` b=`b`)*.

## max

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | First operand; its domain is described by this operation |
| b | any | required | Second operand; its domain is described by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Return the larger operand |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Return the larger operand.

Return *[Return the larger operand](> host_math_max a=`a` b=`b`)*.

## random

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Return a pseudo-random number; seed controls reproducibility |

Effects: random.read.

Return a pseudo-random number; seed controls reproducibility.

Return *[Return a pseudo-random number; seed controls reproducibility](host_random())*.

## random_int

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| min | any | required | Inclusive lower bound or start of the search/plot interval |
| max | any | required | Upper bound or end of the search/plot interval |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Return a pseudo-random integer in the documented bounds |

Use `min`, `max` as the inputs for this operation.

Effects: random.read.

Return a pseudo-random integer in the documented bounds.

Return *[Return a pseudo-random integer in the documented bounds](> host_random_int min=`min` max=`max`)*.

## seed

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Integer seed for reproducible random sequences |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Set this run pseudo-random generator seed; not suitable for secrets |

Use `value` as the inputs for this operation.

Effects: pure.

Set this run pseudo-random generator seed; not suitable for secrets.

Return *[Set this run pseudo-random generator seed; not suitable for secrets](> host_seed value=`value`)*.

## formula

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | any | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Parse expression text into a symbolic formula for later algebra or evaluation; invalid syntax produces a diagnostic |

Use `text` as the inputs for this operation.

Effects: pure.

Parse expression text into a symbolic formula for later algebra or evaluation; invalid syntax produces a diagnostic.

Return *[Parse expression text into a symbolic formula for later algebra or evaluation; invalid syntax produces a diagnostic](> host_formula value=`text`)*.

## simplify

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| formula | any | required | Symbolic formula returned by formula or a prior symbolic operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Return a simplified formula without changing the input; this is symbolic algebra, not numeric evaluation |

Use `formula` as the inputs for this operation.

Effects: pure.

Return a simplified formula without changing the input; this is symbolic algebra, not numeric evaluation.

Return *[Return a simplified formula without changing the input; this is symbolic algebra, not numeric evaluation](> host_simplify value=`formula`)*.

## expand

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| formula | any | required | Symbolic formula returned by formula or a prior symbolic operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Expand algebraic products and powers and return a new symbolic formula |

Use `formula` as the inputs for this operation.

Effects: pure.

Expand algebraic products and powers and return a new symbolic formula.

Return *[Expand algebraic products and powers and return a new symbolic formula](> host_expand value=`formula`)*.

## diff

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| formula | any | required | Symbolic formula returned by formula or a prior symbolic operation |
| var | any | required | Symbol name to differentiate, substitute, evaluate or solve for |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Differentiate the formula with respect to the named symbol and return a symbolic derivative |

Use `formula`, `var` as the inputs for this operation.

Effects: pure.

Differentiate the formula with respect to the named symbol and return a symbolic derivative.

Return *[Differentiate the formula with respect to the named symbol and return a symbolic derivative](> host_diff formula=`formula` var=`var`)*.

## subs

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| formula | any | required | Symbolic formula returned by formula or a prior symbolic operation |
| var | any | required | Symbol name to differentiate, substitute, evaluate or solve for |
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Substitute a symbol with the supplied value; return a number if fully reduced, otherwise a formula |

Use `formula`, `var`, `value` as the inputs for this operation.

Effects: pure.

Substitute a symbol with the supplied value; return a number if fully reduced, otherwise a formula.

Return *[Substitute a symbol with the supplied value; return a number if fully reduced, otherwise a formula](> host_subs formula=`formula` var=`var` value=`value`)*.

## eval

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| formula | any | required | Symbolic formula returned by formula or a prior symbolic operation |
| var | any | required | Symbol name to differentiate, substitute, evaluate or solve for |
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Evaluate a formula at one numeric variable value; unresolved symbols produce a diagnostic |

Use `formula`, `var`, `value` as the inputs for this operation.

Effects: pure.

Evaluate a formula at one numeric variable value; unresolved symbols produce a diagnostic.

Return *[Evaluate a formula at one numeric variable value; unresolved symbols produce a diagnostic](> host_eval formula=`formula` var=`var` value=`value`)*.

## solve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| formula | any | required | Symbolic formula returned by formula or a prior symbolic operation |
| var | any | required | Symbol name to differentiate, substitute, evaluate or solve for |
| min | any | None | Inclusive lower bound or start of the search/plot interval |
| max | any | None | Upper bound or end of the search/plot interval |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Return roots for the named symbol within min and max; omitted bounds use -100 and 100 |

Use `formula`, `var`, `min`=None, `max`=None as the inputs for this operation.

Effects: pure.

Return roots for the named symbol within min and max; omitted bounds use -100 and 100.

Return *[Return roots for the named symbol within min and max; omitted bounds use -100 and 100](> host_solve formula=`formula` var=`var` min=`min` max=`max`)*.

## plot

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| formula | any | required | Symbolic formula returned by formula or a prior symbolic operation |
| var | any | required | Symbol name to differentiate, substitute, evaluate or solve for |
| min | any | required | Inclusive lower bound or start of the search/plot interval |
| max | any | required | Upper bound or end of the search/plot interval |
| steps | any | None | Number of samples across the plotting interval |
| path | any | None | Path relative to the granted filesystem root |
| derivative | any | None | Whether to include the derivative in the plot |
| grid | any | None | Whether to draw a coordinate grid |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Sample the formula over the interval and return SVG text; an optional path also writes the SVG under the granted root |

Use `formula`, `var`, `min`, `max`, `steps`=None, `path`=None, `derivative`=None, `grid`=None as the inputs for this operation.

Effects: output.write.

Sample the formula over the interval and return SVG text; an optional path also writes the SVG under the granted root.

Return *[Sample the formula over the interval and return SVG text; an optional path also writes the SVG under the granted root](> host_plot formula=`formula` var=`var` min=`min` max=`max` steps=`steps` path=`path` derivative=`derivative` grid=`grid`)*.

## plot_points

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| xs | any | required | Input list or collection, as required by this operation |
| ys | any | required | Y-coordinate list paired with xs |
| path | any | None | Path relative to the granted filesystem root |
| grid | any | None | Whether to draw a coordinate grid |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Plot paired x/y lists and return SVG text; unequal coordinate lengths are rejected |

Use `xs`, `ys`, `path`=None, `grid`=None as the inputs for this operation.

Effects: output.write.

Plot paired x/y lists and return SVG text; unequal coordinate lengths are rejected.

Return *[Plot paired x/y lists and return SVG text; unequal coordinate lengths are rejected](> host_plot_points xs=`xs` ys=`ys` path=`path` grid=`grid`)*.

## plot_conic

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| kind | any | required | Conic type accepted by plot_conic |
| a | any | required | First operand; its domain is described by this operation |
| b | any | None | Second operand; its domain is described by this operation |
| h | any | None | Horizontal shift of the conic center |
| k | any | None | Vertical shift of the conic center |
| path | any | None | Path relative to the granted filesystem root |
| grid | any | None | Whether to draw a coordinate grid |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Draw the selected conic with its dimensions and center offsets and return SVG text |

Use `kind`, `a`, `b`=None, `h`=None, `k`=None, `path`=None, `grid`=None as the inputs for this operation.

Effects: output.write.

Draw the selected conic with its dimensions and center offsets and return SVG text.

Return *[Draw the selected conic with its dimensions and center offsets and return SVG text](> host_plot_conic kind=`kind` a=`a` b=`b` h=`h` k=`k` path=`path` grid=`grid`)*.
