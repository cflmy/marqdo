---
title: sys dotenv
> lib/sys.mq.md
> lib/fs.mq.md
---

# main

> write_text path=dotenv-fixture.env text=MARQDO_DOTENV_FIXTURE=loaded

> load_dotenv path=dotenv-fixture.env

*`v` = > env_get name=MARQDO_DOTENV_FIXTURE *

> print text=`v`
