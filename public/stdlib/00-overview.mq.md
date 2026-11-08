---
markup: occlusion
title: Standard library — overview
description: How to import official lib modules
---

# main

Version 2.6.0: each standard library is normally one lib/name.mq.md source file, which is its own handbook. No extra index is required. Read the parameter/default and return tables below its headings to call it; multi-file extensions use indexes to organize modules. The 61 public sources provide 650 functions.

Import with a nested YAML import mapping. File, network and process effects still need authorization at the execution entry. marqdo call defaults to pure functions; file operations require explicit --effects and --fs-root. Contract documentation never grants permission.

New bytes, archive, Result, document and collection workflows retain explicit platform limits. Check one-based paths versus legacy zero-based indexes before using arrays. Real workflows live in examples/stdlib-handbook; the development guide is doc/library-development.mq.md.

Official libraries live under lib/ (alias std/). Use a nested import mapping in YAML frontmatter, for example a module alias mapped to lib/text.mq.md or lib/fs.mq.md.

English-named files export English APIs; Chinese-named files export Chinese APIs. JSON is special: only lib/json.mq.md (shared by both languages).

Importing selects the module; it does not replace effect authorization at the execution entry. Under marqdo view or static export, exit soft-fails and long sleeps are clamped so the console stays up.

Modules: text (text / 文本), table (table / 表), files (fs / 文件), time (time / 时间), system (sys / 系统), json (json only), net (net / 网络), math (math / 数学), foreign (foreign / 外联), plugin (plugin / 插件), writeback (writeback / 自写回), subtask (subtask / 子任务). Foreign v2 adds structured invoke records, JSON calls, parallel fan-out, persistent Python/Node kernels, an explicit state bridge and document-native view cells — see stdlib/foreign.mq.md and features/19-foreign-view-cells.mq.md.

From v0.1.2, official `lib/*.mq.md` is embedded in the `marqdo` binary (disk `lib/` or `MARQDO_LIB` still overrides). Standalone `.exe` works without a separate stdlib zip.

Official optional extensions live under ext/ (not stdlib): ext/llm, ext/agent — see features/05-extensions.mq.md and doc/design/ext-llm.md / ext-agent.md.

Sibling pages in this folder document each module. Chinese overview: 00-索引.mq.md.

> print text=stdlib overview ok
