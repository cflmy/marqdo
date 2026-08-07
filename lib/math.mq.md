---
title: lib/math
description: High-school math — num, random, formula, plot
---

## pi

**> host_pi**

## e

**> host_e**

## num
    + `value`

**> host_num value=`value`**

## add
    + `a`
    + `b`

**> host_math_add a=`a` b=`b`**

## sub
    + `a`
    + `b`

**> host_math_sub a=`a` b=`b`**

## mul
    + `a`
    + `b`

**> host_math_mul a=`a` b=`b`**

## div
    + `a`
    + `b`

**> host_math_div a=`a` b=`b`**

## pow
    + `a`
    + `b`

**> host_math_pow a=`a` b=`b`**

## neg
    + `value`

**> host_math_neg value=`value`**

## sin
    + `value`

**> host_math_sin value=`value`**

## cos
    + `value`

**> host_math_cos value=`value`**

## tan
    + `value`

**> host_math_tan value=`value`**

## asin
    + `value`

**> host_math_asin value=`value`**

## acos
    + `value`

**> host_math_acos value=`value`**

## atan
    + `value`

**> host_math_atan value=`value`**

## sqrt
    + `value`

**> host_math_sqrt value=`value`**

## abs
    + `value`

**> host_math_abs value=`value`**

## ln
    + `value`

**> host_math_ln value=`value`**

## exp
    + `value`

**> host_math_exp value=`value`**

## floor
    + `value`

**> host_math_floor value=`value`**

## ceil
    + `value`

**> host_math_ceil value=`value`**

## min
    + `a`
    + `b`

**> host_math_min a=`a` b=`b`**

## max
    + `a`
    + `b`

**> host_math_max a=`a` b=`b`**

## random

**> host_random**

## random_int
    + `min`
    + `max`

**> host_random_int min=`min` max=`max`**

## seed
    + `value`

**> host_seed value=`value`**

## formula
    + `text`

**> host_formula value=`text`**

## simplify
    + `formula`

**> host_simplify value=`formula`**

## expand
    + `formula`

**> host_expand value=`formula`**

## diff
    + `formula`
    + `var`

**> host_diff formula=`formula` var=`var`**

## subs
    + `formula`
    + `var`
    + `value`

**> host_subs formula=`formula` var=`var` value=`value`**

## eval
    + `formula`
    + `var`
    + `value`

**> host_eval formula=`formula` var=`var` value=`value`**

## solve
    + `formula`
    + `var`
    + `min`=None
    + `max`=None

**> host_solve formula=`formula` var=`var` min=`min` max=`max`**

## plot
    + `formula`
    + `var`
    + `min`
    + `max`
    + `steps`=None
    + `path`=None
    + `derivative`=None
    + `grid`=None

**> host_plot formula=`formula` var=`var` min=`min` max=`max` steps=`steps` path=`path` derivative=`derivative` grid=`grid`**

## plot_points
    + `xs`
    + `ys`
    + `path`=None
    + `grid`=None

**> host_plot_points xs=`xs` ys=`ys` path=`path` grid=`grid`**

## plot_conic
    + `kind`
    + `a`
    + `b`=None
    + `h`=None
    + `k`=None
    + `path`=None
    + `grid`=None

**> host_plot_conic kind=`kind` a=`a` b=`b` h=`h` k=`k` path=`path` grid=`grid`**
