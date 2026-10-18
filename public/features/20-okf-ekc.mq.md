---
title: Code as documentation, documentation as knowledge — OKF and EKC
description: How Marqdo keeps one executable source of truth while projecting it into OKF-style knowledge and Engineering Knowledge Compiler artifacts
---

# main

Marqdo's constitution is code as documentation and documentation as knowledge. A `.mq.md` file is not a screenshot of a program: it is the readable source, the executable unit and the knowledge page at the same time.

That claim matters only if the knowledge stays derived from the source. OKF gives us the shape of a portable knowledge bundle; EKC gives us the engineering projection. Neither becomes a second source of truth.

---

OKF shape, Marqdo meaning.

OKF (Open Knowledge Format) describes knowledge as Markdown pages with YAML frontmatter: one concept per page, path as identity, links as graph, a deliberately small required surface. Marqdo borrows that shape, but its body is executable.

A concept page can therefore point at the exact file that proves it:

```yaml
type: Capability
title: Persistent foreign kernels
description: Run Python or Node in a named, stateful kernel
resource: public/features/19-foreign-view-cells.mq.md
status: stable
verified:
  by: marqdo-knowledge/verify
tags:
  - foreign
  - view-cells
```

The same page can be opened in an editor, linked from a catalog, queried by an agent, and executed. Unknown fields are preserved; unknown types degrade to readable prose. The page is a concept, not a copy of one.

---

What is generated, and what is not.

| Layer | Author | Role |
|---|---|---|
| Source | person or coding agent | .mq.md; the only semantic authority |
| Catalog | marqdo catalog | module, symbol, capability and graph projection |
| Task knowledge | plan promotion | task, skill and executable resource bundle |
| Static site | marqdo view output | human-readable HTML, never an authority |

Generated artifacts are marked GENERATED and can be deleted and rebuilt. The source tree still runs without them. A catalog is an index, not a language feature.

---

EKC: the Engineering Knowledge Compiler.

EKC projects the repository into an engineering knowledge system, then exposes it through a preflight before an agent writes new code:

```text
.mq.md ──► EKC ──► catalog L0–L4
                 ├─ graph
                 ├─ context pack
                 └─ Agent preflight
```

| Layer | Knowledge | Example |
|---|---|---|
| L0 Repository | language, size, roots | repository summary |
| L1 Module | imports, exports, responsibility | a foreign kernel module |
| L2 Symbol | signature, calls, fingerprint | a Python analysis function |
| L3 Capability | what the project can do | run a persistent foreign kernel |
| L4 Knowledge | decisions, constraints, failures | why the bridge is explicit |

The graph records relations such as implements, uses, depends_on, tested_by, documented_by, constrained_by, decided_by, supersedes, conflicts_with and deprecated_by. That graph is how an agent discovers existing capability before inventing a duplicate.

---

Engineering First Information: discover before create.

The reuse protocol is a hard order:

```text
REUSE  →  ADAPT  →  CREATE
```

A preflight can return existing capabilities, APIs, constraints, decisions and failure notes before a new implementation starts. If the knowledge graph is missing, the result is an explicit missing_knowledge state, not a silent fabrication.

The five principles are:

| # | Principle |
|---|---|
| 1 | Discover before Create |
| 2 | Reuse before Adapt |
| 3 | Adapt before Duplicate |
| 4 | Verify before Promote |
| 5 | Code changes update knowledge |

---

The commands.

```bash
# Project the repository into catalog + EKC artifacts
marqdo catalog .

# Ask what already exists before writing code
marqdo knowledge preflight "add a persistent foreign kernel"
marqdo find "foreign session"
marqdo reuse "implement JSON fan-out"

# See what a change would touch
marqdo impact public/features/19-foreign-view-cells.mq.md

# Validate the knowledge projection
marqdo knowledge verify
```

`marqdo catalog` keeps the human-readable `index.md` and module pages; EKC adds L0–L4 files under `.marqdo/`. Both are generated from the same source tree.

---

Every claimed capability has an executable resource.

A knowledge page that says “this project can analyze orders” must point to a `.mq.md` resource that actually runs. That is the difference between an OKF-shaped note and an executable knowledge system.

In the foreign example, the page itself is the proof: the Python call, the persistent kernel and the state bridge all run from the same Markdown source that a reader is looking at. There is no separate demo project to drift out of sync.

---

Read more.

OKF design: [OKF design](../../doc/design/okf.md)

Engineering Knowledge Compiler: [Engineering Knowledge Compiler](../../doc/design/engineering-knowledge.md)

Catalog CLI: [Catalog CLI](../../doc/design/catalog-cli.md)

OKF and Marqdo research: [OKF and Marqdo research](../../doc/research/okf-and-marqdo.md)

Foreign view-cells example: [Foreign view-cells example](19-foreign-view-cells.mq.md)
