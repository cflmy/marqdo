//! Compile KnowledgeGraph into `.marqdo/` projections.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::json;

use crate::knowledge::extract::{extract_graph, ExtractOptions};
use crate::knowledge::ir::KnowledgeGraph;
use crate::knowledge::util::{module_stem, slugify, yaml_escape};

#[derive(Debug, Clone)]
pub struct KnowledgeOptions {
    pub path: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
}

pub fn compile(opts: KnowledgeOptions) -> Result<KnowledgeGraph> {
    let extract_opts = ExtractOptions {
        path: opts.path.clone(),
        out_dir: opts.out_dir.clone(),
    };
    let graph = extract_graph(&extract_opts)?;
    write_projections(&opts.out_dir, &graph)?;
    eprintln!(
        "marqdo knowledge: {} module(s), {} symbol(s), {} capability(ies), {} knowledge → {}",
        graph.modules.len(),
        graph.symbols.len(),
        graph.capabilities.len(),
        graph.knowledge.len(),
        opts.out_dir.display()
    );
    Ok(graph)
}

pub fn load_graph(out_dir: &Path) -> Result<KnowledgeGraph> {
    let graph_path = out_dir.join("graph").join("graph.json");
    let edges_path = out_dir.join("graph").join("edges.json");
    if !graph_path.exists() {
        anyhow::bail!(
            "no knowledge graph at {} — run `marqdo knowledge` first",
            graph_path.display()
        );
    }
    // Prefer re-extract for freshness when source path known; for query commands
    // we rebuild from JSON snapshot fields + re-scan if catalog sibling exists.
    // Snapshot load: reconstruct minimal IR from written pages + edges.json.
    let edges_raw = fs::read_to_string(&edges_path).unwrap_or_else(|_| "{\"edges\":[]}".into());
    let edges_v: serde_json::Value = serde_json::from_str(&edges_raw)?;
    let graph_raw = fs::read_to_string(&graph_path)?;
    let graph_v: serde_json::Value = serde_json::from_str(&graph_raw)?;

    // Re-compile from recorded root if present in graph.json
    if let Some(root) = graph_v
        .pointer("/repository/root")
        .and_then(|v| v.as_str())
    {
        let root_path = Path::new(root);
        if root_path.exists() {
            return extract_graph(&ExtractOptions {
                path: root_path.to_path_buf(),
                out_dir: out_dir.to_path_buf(),
            });
        }
    }

    // Fallback: empty-ish graph with edges only
    let mut edges = Vec::new();
    if let Some(arr) = edges_v.get("edges").and_then(|e| e.as_array()) {
        for e in arr {
            let kind = match e.get("kind").and_then(|k| k.as_str()).unwrap_or("") {
                "implements" => crate::knowledge::ir::EdgeKind::Implements,
                "uses" => crate::knowledge::ir::EdgeKind::Uses,
                "depends_on" => crate::knowledge::ir::EdgeKind::DependsOn,
                "tested_by" => crate::knowledge::ir::EdgeKind::TestedBy,
                "documented_by" => crate::knowledge::ir::EdgeKind::DocumentedBy,
                "constrained_by" => crate::knowledge::ir::EdgeKind::ConstrainedBy,
                "decided_by" => crate::knowledge::ir::EdgeKind::DecidedBy,
                "supersedes" => crate::knowledge::ir::EdgeKind::Supersedes,
                "conflicts_with" => crate::knowledge::ir::EdgeKind::ConflictsWith,
                "deprecated_by" => crate::knowledge::ir::EdgeKind::DeprecatedBy,
                "derived_from" => crate::knowledge::ir::EdgeKind::DerivedFrom,
                "failed_by" => crate::knowledge::ir::EdgeKind::FailedBy,
                _ => crate::knowledge::ir::EdgeKind::RelatedTo,
            };
            edges.push(crate::knowledge::ir::Edge {
                from: e.get("from").and_then(|v| v.as_str()).unwrap_or("").into(),
                to: e.get("to").and_then(|v| v.as_str()).unwrap_or("").into(),
                kind,
                note: e.get("note").and_then(|v| v.as_str()).map(|s| s.to_string()),
            });
        }
    }

    Ok(KnowledgeGraph {
        repository: crate::knowledge::ir::RepositoryInfo {
            title: graph_v
                .pointer("/repository/title")
                .and_then(|v| v.as_str())
                .unwrap_or("marqdo-project")
                .into(),
            root: graph_v
                .pointer("/repository/root")
                .and_then(|v| v.as_str())
                .unwrap_or(".")
                .into(),
            languages: vec!["marqdo".into()],
            module_count: 0,
            symbol_count: 0,
            capability_count: 0,
            knowledge_count: 0,
        },
        modules: Vec::new(),
        symbols: Vec::new(),
        capabilities: Vec::new(),
        knowledge: Vec::new(),
        edges,
        generated_by: graph_v
            .pointer("/generated/by")
            .and_then(|v| v.as_str())
            .unwrap_or("marqdo")
            .into(),
    })
}

pub fn ensure_compiled(path: &Path, out_dir: &Path) -> Result<KnowledgeGraph> {
    let graph_path = out_dir.join("graph").join("graph.json");
    if graph_path.exists() {
        // Always refresh from source when path given
        return compile(KnowledgeOptions {
            path: path.to_path_buf(),
            out_dir: out_dir.to_path_buf(),
        });
    }
    compile(KnowledgeOptions {
        path: path.to_path_buf(),
        out_dir: out_dir.to_path_buf(),
    })
}

fn write_projections(out_dir: &Path, graph: &KnowledgeGraph) -> Result<()> {
    let dirs = [
        "catalog/modules",
        "catalog/symbols",
        "catalog/capabilities",
        "knowledge/facts",
        "knowledge/decisions",
        "knowledge/constraints",
        "knowledge/patterns",
        "knowledge/failures",
        "knowledge/migrations",
        "graph",
        "agent/policies",
        "agent/skills",
        "agent/contexts",
        "agent/episodes",
        "tests/reuse",
        "tests/knowledge",
        "tests/retrieval",
    ];
    for d in dirs {
        fs::create_dir_all(out_dir.join(d))
            .with_context(|| format!("create {}/{}", out_dir.display(), d))?;
    }

    // L0 repository
    fs::write(
        out_dir.join("catalog/repository.mq.md"),
        render_repository(graph),
    )?;

    // L1 modules
    for m in &graph.modules {
        let stem = module_stem(&m.resource);
        fs::write(
            out_dir.join("catalog/modules").join(format!("{stem}.md")),
            render_module(m, graph),
        )?;
    }

    // L2 symbols
    for s in &graph.symbols {
        let stem = slugify(&s.id.replace("::", "__").replace('/', "__"));
        fs::write(
            out_dir.join("catalog/symbols").join(format!("{stem}.md")),
            render_symbol(s),
        )?;
    }

    // L3 capabilities
    for c in &graph.capabilities {
        let stem = slugify(&c.id);
        fs::write(
            out_dir
                .join("catalog/capabilities")
                .join(format!("{stem}.md")),
            render_capability(c),
        )?;
    }

    // L4 knowledge
    for k in &graph.knowledge {
        let dir = out_dir.join("knowledge").join(k.kind.subdir());
        let stem = slugify(&k.id.rsplit('/').next().unwrap_or(&k.id));
        let path = dir.join(format!("{stem}.md"));
        let body = if let Some(ref src) = k.source_text {
            // Ensure generated marker without destroying authored content
            if src.contains("GENERATED by marqdo") {
                src.clone()
            } else {
                format!(
                    "<!-- projected by marqdo knowledge from {} -->\n{src}",
                    k.resource
                )
            }
        } else {
            render_knowledge(k)
        };
        fs::write(&path, body)?;
    }

    // Graph JSON
    let graph_json = serde_json::to_string_pretty(&graph.to_graph_json())?;
    fs::write(out_dir.join("graph/graph.json"), graph_json)?;
    let edges_json = serde_json::to_string_pretty(&graph.to_edges_json())?;
    fs::write(out_dir.join("graph/edges.json"), edges_json)?;

    // Search index
    let index = json!({
        "type": "Marqdo Knowledge Index",
        "generated": { "by": graph.generated_by },
        "entries": graph.index_map(),
    });
    fs::write(
        out_dir.join("graph/index.json"),
        serde_json::to_string_pretty(&index)?,
    )?;

    // Root index.mq.md
    fs::write(out_dir.join("index.mq.md"), render_index_mq(graph))?;

    // Augment root catalog.yaml if present (append engineering summary)
    write_engineering_summary(out_dir, graph)?;

    Ok(())
}

fn write_engineering_summary(out_dir: &Path, graph: &KnowledgeGraph) -> Result<()> {
    let path = out_dir.join("engineering.yaml");
    let mut out = String::new();
    out.push_str("# GENERATED by marqdo knowledge — do not edit by hand\n");
    out.push_str("type: Marqdo Engineering Knowledge\n");
    out.push_str(&format!("title: {}\n", yaml_escape(&graph.repository.title)));
    out.push_str("generated:\n");
    out.push_str(&format!("  by: {}\n", yaml_escape(&graph.generated_by)));
    out.push_str("layers:\n");
    out.push_str(&format!("  modules: {}\n", graph.modules.len()));
    out.push_str(&format!("  symbols: {}\n", graph.symbols.len()));
    out.push_str(&format!("  capabilities: {}\n", graph.capabilities.len()));
    out.push_str(&format!("  knowledge: {}\n", graph.knowledge.len()));
    out.push_str(&format!("  edges: {}\n", graph.edges.len()));
    out.push_str("capabilities:\n");
    if graph.capabilities.is_empty() {
        out.push_str("  []\n");
    } else {
        for c in &graph.capabilities {
            out.push_str(&format!("  - id: {}\n", yaml_escape(&c.id)));
            out.push_str(&format!("    name: {}\n", yaml_escape(&c.name)));
            out.push_str(&format!("    status: {}\n", c.status.as_str()));
            if !c.implemented_by.is_empty() {
                out.push_str("    implemented_by:\n");
                for s in &c.implemented_by {
                    out.push_str(&format!("      - {}\n", yaml_escape(s)));
                }
            }
            if !c.possible_duplicates.is_empty() {
                out.push_str("    possible_duplicates:\n");
                for s in &c.possible_duplicates {
                    out.push_str(&format!("      - {}\n", yaml_escape(s)));
                }
            }
        }
    }
    out.push_str("knowledge:\n");
    if graph.knowledge.is_empty() {
        out.push_str("  []\n");
    } else {
        for k in &graph.knowledge {
            out.push_str(&format!("  - id: {}\n", yaml_escape(&k.id)));
            out.push_str(&format!("    type: {}\n", k.kind.as_str()));
            out.push_str(&format!("    title: {}\n", yaml_escape(&k.title)));
            out.push_str(&format!("    status: {}\n", k.status.as_str()));
        }
    }
    fs::write(path, out)?;
    Ok(())
}

fn render_repository(graph: &KnowledgeGraph) -> String {
    let r = &graph.repository;
    format!(
        r#"---
type: Marqdo Repository
title: {title}
generated:
  by: {by}
status: stable
---

# {title}

> GENERATED by Marqdo Engineering Knowledge Compiler. Do not edit by hand.

## Repository

| Field | Value |
|-------|-------|
| Languages | {langs} |
| Modules | {modules} |
| Symbols | {symbols} |
| Capabilities | {caps} |
| Knowledge | {know} |
| Edges | {edges} |

## Layers

- L0 Repository — this page
- L1 Modules — `catalog/modules/`
- L2 Symbols — `catalog/symbols/`
- L3 Capabilities — `catalog/capabilities/`
- L4 Engineering Knowledge — `knowledge/`
"#,
        title = r.title,
        by = graph.generated_by,
        langs = r.languages.join(", "),
        modules = r.module_count,
        symbols = r.symbol_count,
        caps = r.capability_count,
        know = r.knowledge_count,
        edges = graph.edges.len(),
    )
}

fn render_module(m: &crate::knowledge::ir::ModuleInfo, graph: &KnowledgeGraph) -> String {
    let mut exports = String::new();
    if m.exports.is_empty() {
        exports.push_str("_None_\n");
    } else {
        exports.push_str("| Symbol | Kind |\n|--------|------|\n");
        for e in &m.exports {
            let kind = graph
                .symbols
                .iter()
                .find(|s| s.module == m.id && s.name == *e)
                .map(|s| s.kind.as_str())
                .unwrap_or("fn");
            exports.push_str(&format!("| `{e}` | {kind} |\n"));
        }
    }
    let resp = m.responsibility.as_deref().unwrap_or("_unspecified_");
    format!(
        r#"---
type: Marqdo Module
title: {title}
resource: {resource}
status: {status}
generated:
  by: {by}
---

# {title}

> Auto-generated from `{resource}`. Do not edit by hand.

## Responsibility

{resp}

## Exports

{exports}
"#,
        title = m.title,
        resource = m.resource,
        status = m.status.as_str(),
        by = graph.generated_by,
        resp = resp,
        exports = exports,
    )
}

fn render_symbol(s: &crate::knowledge::ir::SymbolInfo) -> String {
    let params = if s.params.is_empty() {
        "[]".into()
    } else {
        s.params
            .iter()
            .map(|p| format!("  - {p}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let use_when = s.use_when.as_deref().unwrap_or("_unspecified_");
    let do_not = s.do_not_use_when.as_deref().unwrap_or("_unspecified_");
    format!(
        r#"---
type: Function
name: {name}
module: {module}
symbol: {name}
resource: {resource}
status: {status}
visibility: {vis}
fingerprint: {fp}
capability: {cap}
generated:
  by: marqdo/knowledge
---

# {name}

## Use when

{use_when}

## Do not use when

{do_not}

## Signature

params:
{params}

## Calls

{calls}

## Fingerprint

```
{fpt}
```
"#,
        name = s.name,
        module = s.module,
        resource = s.resource,
        status = s.status.as_str(),
        vis = s.visibility,
        fp = s.fingerprint,
        cap = s.capability.as_deref().unwrap_or(""),
        use_when = use_when,
        do_not = do_not,
        params = params,
        calls = if s.calls.is_empty() {
            "_None_".into()
        } else {
            s.calls
                .iter()
                .map(|c| format!("- `{c}`"))
                .collect::<Vec<_>>()
                .join("\n")
        },
        fpt = s.fingerprint_text,
    )
}

fn render_capability(c: &crate::knowledge::ir::CapabilityInfo) -> String {
    let use_when = c.use_when.as_deref().unwrap_or(&c.description);
    let do_not = c.do_not_use_when.as_deref().unwrap_or("_unspecified_");
    format!(
        r#"---
type: Capability
id: {id}
name: {name}
status: {status}
generated:
  by: marqdo/knowledge
---

# {name}

{desc}

## Use when

{use_when}

## Do not use when

{do_not}

## Implemented by

{impls}

## Possible duplicates

{dups}
"#,
        id = c.id,
        name = c.name,
        status = c.status.as_str(),
        desc = c.description,
        use_when = use_when,
        do_not = do_not,
        impls = if c.implemented_by.is_empty() {
            "_None_".into()
        } else {
            c.implemented_by
                .iter()
                .map(|s| format!("- `{s}`"))
                .collect::<Vec<_>>()
                .join("\n")
        },
        dups = if c.possible_duplicates.is_empty() {
            "_None_".into()
        } else {
            c.possible_duplicates
                .iter()
                .map(|s| format!("- `{s}`"))
                .collect::<Vec<_>>()
                .join("\n")
        },
    )
}

fn render_knowledge(k: &crate::knowledge::ir::KnowledgeItem) -> String {
    format!(
        r#"---
type: {ty}
id: {id}
title: {title}
status: {status}
generated:
  by: marqdo/knowledge
---

# {title}

{summary}
"#,
        ty = k.kind.as_str(),
        id = k.id,
        title = k.title,
        status = k.status.as_str(),
        summary = k.body_summary,
    )
}

fn render_index_mq(graph: &KnowledgeGraph) -> String {
    format!(
        r#"---
type: Marqdo Engineering Knowledge
title: Engineering Knowledge Index
generated:
  by: {by}
---

# Engineering Knowledge Index

> GENERATED. Source of truth remains `*.mq.md`. This tree is a Knowledge Projection.

## Counts

| Layer | Count |
|-------|------:|
| Modules | {modules} |
| Symbols | {symbols} |
| Capabilities | {caps} |
| Knowledge | {know} |
| Edges | {edges} |

## Commands

```
marqdo knowledge
marqdo find "query"
marqdo reuse "task"
marqdo duplicate
marqdo impact PATH
marqdo conflicts
marqdo stale
marqdo verify
```
"#,
        by = graph.generated_by,
        modules = graph.modules.len(),
        symbols = graph.symbols.len(),
        caps = graph.capabilities.len(),
        know = graph.knowledge.len(),
        edges = graph.edges.len(),
    )
}

