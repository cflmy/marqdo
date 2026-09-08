---
title: Built-in input
description: Read one line from stdin; optional prompt. Demo via frontmatter stdin.
stdin: Ada
---

# main

The input builtin reads one line. For demos and marqdo view / static export, put sample lines in frontmatter (stdin:). CLI can also use a pipe or --stdin-file. Live view can override with the Preset input box.

*`name` = > input prompt="Name: "*

> print text=Hello `name`!
