---
name: marqdo
description: Author, run and review readable Marqdo programs, libraries and executable knowledge. Use for .mq.md, marqdo call, Document/Endpoint/Resource web projects and quantum tools; read the executable programming and development handbooks.
---

# Marqdo

<!-- CORE_SYNTAX_V2.1: keep this card within the first 1200 body characters -->

## CORE SYNTAX

Occlusion is the current default. `**…**` executes a statement; `*…*` returns.
Use `[intent](actual code)` in an executable position: the label is prose and
only its payload executes. Empty parentheses are a pending None stub.
Read collections with `access(coll, key)`. Classic indexing is only for explicit
legacy sources and migration. Reading narrative references never runs effects.

| Marker | Meaning |
|---|---|
| `**…**` | Statement (assign / call / expression) |
| `*…*` | Return |
| `[intent](payload)` | Readable intent with executable payload |
| `access(c, k)` | Collection read |
| `- [intent](for items as item)` | Explicit traversal |
| `1.` / `2.` | Branches; numbered * is else |
| GFM table | Data: one column list, one row record, @ row records |

Read [the executable programming guide](../../doc/library-development.mq.md)
before authoring. Its actual callable examples demonstrate inline computation,
structured results, defaults and recoverable native file reads. Use
[the development entry](../marqdo-dev/index.mq.md) and its file tools for source
work, and [the release entry](../marqdo-release/index.mq.md) for an authorized
release. These mq.md files are the working instructions and tools; do not rebuild
a parallel command playbook in SKILL.md.

Call the actual file and heading with `marqdo call`. In a source checkout use
the built `target/debug/marqdo`; prepare its engineering plugin through the
development entry. Large structured inputs use --args-json FILE or stdin;
JSON strings stay literal during call-session replay. --require-ok makes failed
Results stop external automation while preserving their returned evidence.
Read each function's parameter/return tables and effect contract before calling.

Author programs as readable explanations: useful narrative exceeds half of
each callable unit; parameter and return tables immediately follow its heading;
inline assignments, actions and returns explain their purpose. Standard libraries
remain single-file handbooks. Multi-file projects and extensions use index.mq.md.
GFM describes data, public helpers describe operations, and components recursively
split web layouts until their content leaves are ordinary Markdown.

Native Marqdo capabilities handle file reading/writing, tables, parsing, hashing
and network reads. Add an extension for a reusable capability gap or call an
explicit existing external tool with argument arrays. Keep decisions, retries
and the execution flow in mq.md. Do not generate Python or shell programs for
already available native operations. OS processes and plugins retain their OS
authority; an fs-root does not sandbox their own access.

Use the current [web entry](../../ext/web/index.mq.md) for Document, Endpoint and
Resource artifacts, or [quantum entry](../../ext/quantum/index.mq.md) for circuit
workflows. Do not restore removed compose_* or app.configure APIs. Preserve
intentional fixtures and [classic migration context](migration-classic.md).
Validate the real calls and error paths; a narrative ratio or uploaded asset
alone is never functional evidence.
