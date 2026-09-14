---
title: lib/math
description: High-school math — num, random, formula, plot
---

High-school math — num, random, formula, plot

## pi

pi.

*> host_pi*

## e

e.

*> host_e*

## num
    + `value`

num.

*> host_num value=`value`*

## add
    + `a`
    + `b`

add.

*> host_math_add a=`a` b=`b`*

## sub
    + `a`
    + `b`

sub.

*> host_math_sub a=`a` b=`b`*

## mul
    + `a`
    + `b`

mul.

*> host_math_mul a=`a` b=`b`*

## div
    + `a`
    + `b`

div.

*> host_math_div a=`a` b=`b`*

## pow
    + `a`
    + `b`

pow.

*> host_math_pow a=`a` b=`b`*

## neg
    + `value`

neg.

*> host_math_neg value=`value`*

## sin
    + `value`

sin.

*> host_math_sin value=`value`*

## cos
    + `value`

cos.

*> host_math_cos value=`value`*

## tan
    + `value`

tan.

*> host_math_tan value=`value`*

## asin
    + `value`

asin.

*> host_math_asin value=`value`*

## acos
    + `value`

acos.

*> host_math_acos value=`value`*

## atan
    + `value`

atan.

*> host_math_atan value=`value`*

## sqrt
    + `value`

sqrt.

*> host_math_sqrt value=`value`*

## abs
    + `value`

abs.

*> host_math_abs value=`value`*

## ln
    + `value`

ln.

*> host_math_ln value=`value`*

## exp
    + `value`

exp.

*> host_math_exp value=`value`*

## floor
    + `value`

floor.

*> host_math_floor value=`value`*

## ceil
    + `value`

ceil.

*> host_math_ceil value=`value`*

## min
    + `a`
    + `b`

min.

*> host_math_min a=`a` b=`b`*

## max
    + `a`
    + `b`

max.

*> host_math_max a=`a` b=`b`*

## random

random.

*> host_random*

## random_int
    + `min`
    + `max`

random_int.

*> host_random_int min=`min` max=`max`*

## seed
    + `value`

seed.

*> host_seed value=`value`*

## formula
    + `text`

formula.

*> host_formula value=`text`*

## simplify
    + `formula`

simplify.

*> host_simplify value=`formula`*

## expand
    + `formula`

expand.

*> host_expand value=`formula`*

## diff
    + `formula`
    + `var`

diff.

*> host_diff formula=`formula` var=`var`*

## subs
    + `formula`
    + `var`
    + `value`

subs.

*> host_subs formula=`formula` var=`var` value=`value`*

## eval
    + `formula`
    + `var`
    + `value`

eval.

*> host_eval formula=`formula` var=`var` value=`value`*

## solve
    + `formula`
    + `var`
    + `min`=None
    + `max`=None

solve.

*> host_solve formula=`formula` var=`var` min=`min` max=`max`*

## plot
    + `formula`
    + `var`
    + `min`
    + `max`
    + `steps`=None
    + `path`=None
    + `derivative`=None
    + `grid`=None

plot.

*> host_plot formula=`formula` var=`var` min=`min` max=`max` steps=`steps` path=`path` derivative=`derivative` grid=`grid`*

## plot_points
    + `xs`
    + `ys`
    + `path`=None
    + `grid`=None

plot_points.

*> host_plot_points xs=`xs` ys=`ys` path=`path` grid=`grid`*

## plot_conic
    + `kind`
    + `a`
    + `b`=None
    + `h`=None
    + `k`=None
    + `path`=None
    + `grid`=None

plot_conic.

*> host_plot_conic kind=`kind` a=`a` b=`b` h=`h` k=`k` path=`path` grid=`grid`*
