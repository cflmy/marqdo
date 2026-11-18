---
markup: occlusion
title: Draft and list editor
import:
  controls: ../controls.mq.md
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Structured View rows for this component |

The editor contains one draft input, three draft actions, the export actions and an initially empty native list. The client owns records, stable identities and delegated deletion; no authored list HTML is stored here. Save persists only the draft under this origin. Copy and Download export current item text while list state remains in browser session memory.

We **heading = [Read the plain editor heading](component.markdown(text="## 待办"))** and **input = [Describe the exact draft target](controls.input(id="draft", hint="新项…"))** and **children = [Place input after its heading](table.concat(a=`heading`, b=`input`))**.

`actions` =

| @ | id | label | secondary |
|---|---|---|---|
| 1 | add | 添加 | False |
| 2 | save | 存草稿 | True |
| 3 | load | 读草稿 | True |
| 4 | copy-log | 复制列表 | True |
| 5 | dl-list | 下载列表 | True |

- [Describe each native action leaf](for actions as action)
  We **button = [Keep this action separate from state](controls.button(id=access(action, "id"), label=access(action, "label"), secondary=access(action, "secondary")))** and **children = [Preserve the authored action order](table.concat(a=`children`, b=`button`))**.
We **empty = [Start the empty native list](table.empty_list())** and **attrs = [Retain the delegated list target](table.put(in=None, at="id", value="items"))** and **list = [Describe the list container](component.wrap(children=`empty`, kind="list", attrs=`attrs`, slot="items"))**.
Return *[Keep the list after its editing controls](table.concat(a=`children`, b=`list`))*.
