---
markup: occlusion
title: Named timer controls
import:
  controls: ../controls.mq.md
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Structured View rows for this component |

This region describes the tick output and its two actions. The client increments retained state only when the named timer actually calls back. Start replaces a timer with the same id; Stop preserves the count. This layout performs no sleeping or counting on the server, and its native text output can be tested independently of timer scheduling.

We **label = [Describe the timer label](component.markdown(text="计时（秒）："))** and **counter = [Describe the initial count](controls.text(id="tick", value="0"))** and **start = [Describe the explicit start action](controls.button(id="start-tick", label="开始"))** and **stop = [Describe the explicit stop action](controls.button(id="stop-tick", label="停止", secondary=True))** and **content = [Keep label before count](table.concat(a=`label`, b=`counter`))** and **actions = [Keep the two actions together](table.concat(a=`start`, b=`stop`))** and **children = [Place actions beside their output](table.concat(a=`content`, b=`actions`))**.
Return *[Wrap this one timer responsibility](component.wrap(children=`children`, kind="row", slot="timer"))*.
