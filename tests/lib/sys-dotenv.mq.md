---
title: sys dotenv
import sys:lib/sys.mq.md
import fs:lib/fs.mq.md
---

# main

> fs.write_text path=dotenv-fixture.env text=MARQDO_DOTENV_FIXTURE=loaded

> sys.load_dotenv path=dotenv-fixture.env

*`v` = > sys.env_get name=MARQDO_DOTENV_FIXTURE *

> print text=`v`
