---
markup: occlusion
title: ABI-native durable agent workflow test
import:
  workflow: ext/ai/workflow.mq.md
  json: lib/json.mq.md
  sys: lib/sys.mq.md
  time: lib/time.mq.md
---

# main

Offline test: the same Marqdo program advances through model choice, user
input and terminal state with plugin-owned durable checkpoints. The run ID and
checkpoint directory include the current millisecond so repeated local runs
start clean instead of reopening yesterday's completed workflow.

**now = > time.now_ms**
**stamp = > json.stringify value=`now`**
**rid = "abi_flow_ci_" + stamp**
**dir = ".marqdo/ai-native-workflow-" + stamp**

**steps = > json.parse text=[{"id":"intent","kind":"model","prompt":"Choose next skill","options":["chat","office"]},{"id":"confirm","kind":"input","question":"Continue?","options":["yes","no"]},{"id":"end","kind":"return"}]**
**opened = > workflow.open id=`rid` steps=`steps` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**o = > json.get value=`opened` key="ok"**
1. `o`
  **_ = 1**
2. *
  > sys.exit code=21

**seen = > workflow.inspect id=`rid` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**seen_status = > json.get value=`seen` key="action"**
**seen_total = > json.get value=`seen` key="total_steps"**
1. `seen_status` == "waiting_model"
  1. `seen_total` == 3
    **_ = 1**
  2. *
    > sys.exit code=30
2. *
  > sys.exit code=31

**listed = > workflow.list session_id="ci-session" resume_dir=`dir`**
**listed_ok = > json.get value=`listed` key="ok"**
**listed_items = > json.get value=`listed` key="items"**
**listed_n = > len value=`listed_items`**
1. `listed_ok`
  1. `listed_n` > 0
    **_ = 1**
  2. *
    > sys.exit code=32
2. *
  > sys.exit code=33

**first = > workflow.next id=`rid` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**first_state = > json.get value=`first` key="status"**
1. `first_state` == "waiting_model"
  **_ = 1**
2. *
  > sys.exit code=22

**c1 = > workflow.commit id=`rid` step_id="intent" response="chat" source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**done1 = > json.get value=`c1` key="ok"**
1. `done1`
  **_ = 1**
2. *
  > sys.exit code=23

**progress = > workflow.inspect id=`rid` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**progress_done = > json.get value=`progress` key="completed_steps"**
1. `progress_done` == 1
  **_ = 1**
2. *
  > sys.exit code=34

**second = > workflow.tick id=`rid` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**second_state = > json.get value=`second` key="status"**
1. `second_state` == "waiting_input"
  **_ = 1**
2. *
  > sys.exit code=24

**c2 = > workflow.commit id=`rid` step_id="confirm" response="yes" source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**done2 = > json.get value=`c2` key="ok"**
1. `done2`
  **_ = 1**
2. *
  > sys.exit code=25

**c3 = > workflow.tick id=`rid` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**complete = > workflow.next id=`rid` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**status = > json.get value=`complete` key="status"**
1. `status` == "succeeded"
  **_ = 1**
2. *
  > sys.exit code=26

**partial_steps = > json.parse text=[{"id":"children","kind":"parallel","effect_class":"read","tasks":[{"path":"worker-a.mq.md"},{"path":"worker-b.mq.md"}]}]**
**prid = "abi_flow_partial_ci_" + stamp**
**new_batch = > workflow.open id=`prid` steps=`partial_steps` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**parts = > json.parse text=[{"ok":true,"value":{"code":0,"value":"useful"}},{"ok":false,"error":"timeout"}]**
**stopped = > workflow.commit id=`prid` step_id="children" response=`parts` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**reloaded = > workflow.next id=`prid` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**partial = > json.get value=`reloaded` key="partial"**
**survivors = > json.get value=`partial` key="succeeded"**
1. `survivors` == 1
  **_ = 1**
2. *
  > sys.exit code=27

**retry_plan = > workflow.retry_plan id=`prid` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**allowed = > json.get value=`retry_plan` key="ok"**
1. `allowed`
  **_ = 1**
2. *
  > sys.exit code=28
**repaired = > json.parse text=[{"ok":true,"value":{"code":0,"value":"repaired"}}]**
**merged = > workflow.retry_commit id=`prid` results=`repaired` source_revision="flow-v1" session_id="ci-session" resume_dir=`dir`**
**merged_ok = > json.get value=`merged` key="ok"**
1. `merged_ok`
  **_ = 1**
2. *
  > sys.exit code=29

Return *"durable-agent-flow-ok"*.
