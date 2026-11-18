---
markup: occlusion
title: ABI-native durable agent workflow test
import:
  workflow: ext/ai/workflow.mq.md
  json: lib/json.mq.md
  sys: lib/sys.mq.md
---

# main

Offline test: the same Marqdo program advances through model choice, user
input and terminal state with plugin-owned durable checkpoints.

**steps = > json.parse text=[{"id":"intent","kind":"model","options":["chat","office"]},{"id":"confirm","kind":"input","question":"Continue?","options":["yes","no"]},{"id":"end","kind":"return"}]**
**opened = > workflow.open id="abi_flow_ci_20261118" steps=`steps` source_revision="flow-v1"**
**o = > json.get value=`opened` key="ok"**
1. `o`
  **_ = 1**
2. *
  > sys.exit code=21

**first = > workflow.next id="abi_flow_ci_20261118" source_revision="flow-v1"**
**first_state = > json.get value=`first` key="status"**
1. `first_state` == "waiting_model"
  **_ = 1**
2. *
  > sys.exit code=22

**c1 = > workflow.commit id="abi_flow_ci_20261118" step_id="intent" response="chat" source_revision="flow-v1"**
**done1 = > json.get value=`c1` key="ok"**
1. `done1`
  **_ = 1**
2. *
  > sys.exit code=23

**second = > workflow.next id="abi_flow_ci_20261118" source_revision="flow-v1"**
**second_state = > json.get value=`second` key="status"**
1. `second_state` == "waiting_input"
  **_ = 1**
2. *
  > sys.exit code=24

**c2 = > workflow.commit id="abi_flow_ci_20261118" step_id="confirm" response="yes" source_revision="flow-v1"**
**done2 = > json.get value=`c2` key="ok"**
1. `done2`
  **_ = 1**
2. *
  > sys.exit code=25

**c3 = > workflow.commit id="abi_flow_ci_20261118" step_id="end" response="finished" source_revision="flow-v1"**
**complete = > workflow.next id="abi_flow_ci_20261118" source_revision="flow-v1"**
**status = > json.get value=`complete` key="status"**
1. `status` == "succeeded"
  **_ = 1**
2. *
  > sys.exit code=26

**partial_steps = > json.parse text=[{"id":"children","kind":"parallel","tasks":[]}]**
**new_batch = > workflow.open id="abi_flow_partial_ci_20261118" steps=`partial_steps` source_revision="flow-v1"**
**parts = > json.parse text=[{"ok":true,"value":{"code":0,"value":"useful"}},{"ok":false,"error":"timeout"}]**
**stopped = > workflow.commit id="abi_flow_partial_ci_20261118" step_id="children" response=`parts` source_revision="flow-v1"**
**reloaded = > workflow.next id="abi_flow_partial_ci_20261118" source_revision="flow-v1"**
**partial = > json.get value=`reloaded` key="partial"**
**survivors = > json.get value=`partial` key="succeeded"**
1. `survivors` == 1
  **_ = 1**
2. *
  > sys.exit code=27

Return *"durable-agent-flow-ok"*.
