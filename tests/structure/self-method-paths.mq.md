---
markup: occlusion
title: Bare self and 自 method paths retain the receiver
import:
  json: lib/json.mq.md
---

# Counter

**state = > json.parse text={"n":41}**
*state*

## read

*> json.get value=`self` key="n"*

## via

*> self.read*

## alias

*> 自.read*

## announce

> self.via
*7*

# main

**c = > Counter**
**x = > `c`.via**
> print text=`x`
**y = > `c`.alias**
> print text=`y`
**z = > `c`.announce**
> print text=`z`
