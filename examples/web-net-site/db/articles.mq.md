---
markup: occlusion
title: Article schema and seed handbook
import:
  document: lib/document.mq.md
---

This file explains callable use; [articles.tables.md](articles.tables.md) holds the readable literal tables. Set --fs-root to the site directory and grant fs.read. Reading returns data without running prose, opening a database or changing a page.

## schema

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Native table value, preserving literal types and column geometry |

Declare article fields, native types and nullability. The initializer consumes this table to ensure structure; it must not guess a migration or erase existing rows.

Call independently with fs.read on the site root. The section must match exactly and uniquely. Malformed tables or dynamic expressions retain native document diagnostics. A returned style declaration still needs compilation; returned database data still needs an explicit initializer. This value is neither HTML nor an applied browser effect.

Return *[读取 schema 的实际字面表](document.read_table(path="db/articles.tables.md", title="schema"))*.

## seed

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Native table value, preserving literal types and column geometry |

Provide initial demonstration articles with their original ids, stable slugs and content. Insert only after the database initializer confirms an empty table; this reader never changes existing articles.

Call independently with fs.read on the site root. The section must match exactly and uniquely. Malformed tables or dynamic expressions retain native document diagnostics. A returned style declaration still needs compilation; returned database data still needs an explicit initializer. This value is neither HTML nor an applied browser effect.

Return *[读取 seed 的实际字面表](document.read_table(path="db/articles.tables.md", title="seed"))*.
