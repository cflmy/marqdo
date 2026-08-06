---
title: ext/agent layout scaffold
description: Requires MARQDO_AGENT_PLUGIN (set by gold harness).
> ext/agent.mq.md
> lib/fs.mq.md
---

# main

> load_native

*`cwd` = > cwd *

*`ws` = > agent root=`cwd` *

*`_n` = > `ws`.ensure_layout *

*`info` = > `ws`.probe *

*`ok` = > get value=`info` key=has_agents *

> print text=`ok`

> write_text path=skel.mq.md text={{name}}

*`_p` = > `ws`.scaffold name=demo template=skel.mq.md dest=out-demo.mq.md *

*`body` = > read_text path=out-demo.mq.md *

> print text=`body`
