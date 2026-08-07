---
title: Standard library — overview
description: How to import official lib modules
---

# main

Official libraries live under lib/ (alias std/). Import them in YAML frontmatter with a greater-than line, for example lib/text.mq.md or lib/fs.mq.md.

English-named files export English APIs; Chinese-named files export Chinese APIs. JSON is special: only lib/json.mq.md (shared by both languages).

No extra allow flags: importing a module means you intend to use it. Under marqdo view or static export, exit soft-fails and long sleeps are clamped so the console stays up.

Modules: text (text / 文本), table (table / 表), files (fs / 文件), time (time / 时间), system (sys / 系统), json (json only), net (net / 网络), math (math / 数学), foreign (foreign / 外联), plugin (plugin / 插件), writeback (writeback / 自写回), subtask (subtask / 子任务).

From **v0.1.2**, official `lib/*.mq.md` is **embedded in the `marqdo` binary** (disk `lib/` or `MARQDO_LIB` still overrides). Standalone `.exe` works without a separate stdlib zip.

Official optional extensions live under ext/ (not stdlib): ext/llm, ext/agent — see features/05-extensions.mq.md and doc/design/ext-llm.md / ext-agent.md.

Sibling pages in this folder document each module. Chinese overview: 00-索引.mq.md.

> print text=stdlib overview ok
