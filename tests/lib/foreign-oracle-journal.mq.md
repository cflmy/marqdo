---
markup: occlusion
title: document oracle journal is a first-class value
import:
  foreign: lib/foreign.mq.md
  json: lib/json.mq.md
---

# main

**s = > foreign.session_open lang="python"**
**> foreign.session_set id=`s` name="n" value=2**
**> foreign.session_eval id=`s` source="n + 1"**
**> foreign.session_get id=`s` name="n"**
**> foreign.session_close id=`s`**
**j = > foreign.oracle_journal**
**same = > foreign.replay_check expected=`j` observed=`j`**
**> print text=[ok](access(`same`, "ok"))**
**> json.parse text="3"**
**later = > foreign.oracle_journal**
**changed = > foreign.replay_check expected=`j` observed=`later`**
**> print text=[ok](access(`changed`, "ok"))**
**> print text=[index](access(`changed`, "index"))**
