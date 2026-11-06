---
markup: occlusion
title: Lexical sibling calls, recursion and nearest declaration
---

# main

**a = > first**
> print text=`a`
**b = > outer**
> print text=`b`
**c = > helper**
> print text=`c`

## first

**answer = > second**
*answer*

## second

**answer = > factorial n=5**
*answer*

## factorial
    + `n`

1. `n` < 2
  *1*
2. *
  **previous = n - 1**
  **part = > factorial n=`previous`**
  **answer = n * part**
  *answer*

## outer

**answer = > helper**

### helper

*41*
*answer*

## helper

*99*
