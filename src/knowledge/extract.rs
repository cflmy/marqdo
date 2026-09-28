//! Extract Engineering Knowledge IR from a Marqdo source tree.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::ast::Function;
use crate::knowledge::fingerprint::{collect_calls, fingerprint_hash, fingerprint_text, ops_hint};
use crate::knowledge::ir::{
    Confidence, Evidence, KnowledgeGraph, KnowledgeItem, KnowledgeKind, Lifecycle, ModuleInfo,
    RepositoryInfo, SymbolInfo,
};
use crate::knowledge::util::{
    extract_fm_field, extract_fm_list, extract_nested_by, extract_section, first_paragraph,
    module_stem, slugify,
};
use crate::lex::{classify_source, LineKind};
use crate::parse::parse_source;

pub struct ExtractOptions {
    pub path: PathBuf,
    pub out_dir: PathBuf,
}

pub fn extract_graph(opts: &ExtractOptions) -> Result<KnowledgeGraph> {
    let root = if opts.path.is_file() {
        opts.path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        opts.path.clone()
    };
    let root = root.canonicalize().unwrap_or(root);
    let out_canon = opts.out_dir.canonicalize().unwrap_or(opts.out_dir.clone());
    let version = env!("CARGO_PKG_VERSION");

    let mut files = Vec::new();
    if opts.path.is_file() {
        files.push(opts.path.canonicalize().unwrap_or(opts.path.clone()));
    } else {
        collect_mq_md(&root, &root, &out_canon, &mut files)?;
        files.sort();
    }

    let mut modules = Vec::new();
    let mut symbols = Vec::new();
    let mut knowledge = Vec::new();
    let mut had_err = false;

    for abs in &files {
        let rel = abs
            .strip_prefix(&root)
            .unwrap_or(abs)
            .to_string_lossy()
            .replace('\\', "/");
        if rel.contains("/.marqdo/") || rel.starts_with(".marqdo/") {
            continue;
        }
        match inspect_source(abs, &rel) {
            Ok((m, syms, ks)) => {
                modules.push(m);
                symbols.extend(syms);
                knowledge.extend(ks);
            }
            Err(e) => {
                eprintln!("knowledge: skip {rel}: {e:#}");
                had_err = true;
            }
        }
    }

    // Also scan authored knowledge markdown (non-.mq.md) under knowledge/ or doc/adr/
    collect_authored_knowledge(&root, &out_canon, &mut knowledge)?;

    // Wire called_by
    let mut call_index: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for s in &symbols {
        for c in &s.calls {
            let short = c.rsplit('.').next().unwrap_or(c).to_string();
            call_index.entry(short).or_default().push(s.id.clone());
            call_index.entry(c.clone()).or_default().push(s.id.clone());
        }
    }
    for s in &mut symbols {
        let mut cb = call_index.remove(&s.name).unwrap_or_default();
        cb.extend(call_index.remove(&s.id).unwrap_or_default());
        cb.sort();
        cb.dedup();
        s.called_by = cb;
    }

    // Detect tests: paths containing /tests/ or name starting with test_
    for s in &mut symbols {
        if s.resource.contains("/tests/") || s.resource.starts_with("tests/") {
            // module-level test file — mark symbols as tests for same-named exports later
        }
        if s.name.starts_with("test_") || s.name.starts_with("测试") {
            s.kind = "test".into();
        }
    }
    let test_syms: Vec<(String, String)> = symbols
        .iter()
        .filter(|s| s.kind == "test" || s.resource.contains("/tests/") || s.resource.starts_with("tests/"))
        .map(|s| (s.name.clone(), s.id.clone()))
        .collect();
    for s in &mut symbols {
        for (tname, tid) in &test_syms {
            if tname.contains(&s.name) || s.tests.iter().any(|t| t == tid) {
                if !s.tests.contains(tid) && tid != &s.id {
                    s.tests.push(tid.clone());
                }
            }
        }
    }

    let capabilities = crate::knowledge::capability::derive_capabilities(&symbols, &modules);
    let mut edges = crate::knowledge::graph::build_edges(&modules, &symbols, &capabilities, &knowledge);
    crate::knowledge::duplicate::annotate_duplicates(&symbols, &mut edges, &capabilities);

    // Re-derive capability duplicate links after annotate (annotate mutates via returned list)
    let capabilities =
        crate::knowledge::duplicate::apply_duplicate_flags(capabilities, &symbols);

    let repository = RepositoryInfo {
        title: root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("marqdo-project")
            .to_string(),
        root: root.to_string_lossy().to_string(),
        languages: vec!["marqdo".into()],
        module_count: modules.len(),
        symbol_count: symbols.len(),
        capability_count: capabilities.len(),
        knowledge_count: knowledge.len(),
    };

    if had_err {
        eprintln!("knowledge: extraction completed with some skipped files");
    }

    Ok(KnowledgeGraph {
        repository,
        modules,
        symbols,
        capabilities,
        knowledge,
        edges,
        generated_by: format!("marqdo/{version}"),
    })
}

fn collect_mq_md(
    root: &Path,
    dir: &Path,
    out_canon: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name == "target"
                || name == ".git"
                || name == "node_modules"
                || name == "docker"
                || name == "data"
                || name == ".cursor"
            {
                continue;
            }
            if let Ok(canon) = p.canonicalize() {
                if canon == *out_canon {
                    continue;
                }
            }
            collect_mq_md(root, &p, out_canon, out)?;
        } else if p
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.ends_with(".mq.md"))
            .unwrap_or(false)
        {
            out.push(p);
        }
    }
    Ok(())
}

fn collect_authored_knowledge(
    root: &Path,
    out_canon: &Path,
    out: &mut Vec<KnowledgeItem>,
) -> Result<()> {
    walk_knowledge_md(root, root, out_canon, out)
}

fn walk_knowledge_md(
    root: &Path,
    dir: &Path,
    out_canon: &Path,
    out: &mut Vec<KnowledgeItem>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name == "target"
                || name == ".git"
                || name == ".marqdo"
                || name == "docker"
                || name == "data"
                || name == ".cursor"
            {
                continue;
            }
            if let Ok(canon) = p.canonicalize() {
                if canon == *out_canon {
                    continue;
                }
            }
            walk_knowledge_md(root, &p, out_canon, out)?;
            continue;
        }
        let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        // Authored knowledge pages: .md with engineering type, not .mq.md (those go through inspect)
        if !name.ends_with(".md") || name.ends_with(".mq.md") {
            continue;
        }
        let source = fs::read_to_string(&p)?;
        let Some(ty) = extract_fm_field(&source, "type") else {
            continue;
        };
        let kind = KnowledgeKind::parse(&ty);
        match kind {
            KnowledgeKind::Decision
            | KnowledgeKind::Constraint
            | KnowledgeKind::Failure
            | KnowledgeKind::AntiPattern
            | KnowledgeKind::Pattern
            | KnowledgeKind::Migration
            | KnowledgeKind::Fact => {}
            _ => continue,
        }
        let rel = p
            .strip_prefix(root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        out.push(knowledge_from_source(&source, &rel, kind));
    }
    Ok(())
}

fn knowledge_from_source(source: &str, rel: &str, kind: KnowledgeKind) -> KnowledgeItem {
    let title = extract_fm_field(source, "title")
        .or_else(|| {
            source.lines().find_map(|l| {
                l.trim()
                    .strip_prefix("# ")
                    .map(|s| s.trim().to_string())
            })
        })
        .unwrap_or_else(|| rel.to_string());
    let id = extract_fm_field(source, "id").unwrap_or_else(|| {
        format!(
            "{}/{}",
            kind.subdir(),
            slugify(&title)
        )
    });
    let status = Lifecycle::parse(
        &extract_fm_field(source, "status").unwrap_or_else(|| "stable".into()),
    );
    let applies_to = extract_fm_list(source, "applies_to");
    let supersedes = extract_fm_list(source, "supersedes");
    let conflicts_with = extract_fm_list(source, "conflicts_with");
    let related = extract_fm_list(source, "related");
    let stale_after = extract_fm_field(source, "stale_after");
    let summary = first_paragraph(source);
    let mut evidence = Evidence::default();
    evidence.sources.push(rel.to_string());
    KnowledgeItem {
        id,
        kind,
        title,
        resource: rel.to_string(),
        status,
        body_summary: summary,
        applies_to,
        supersedes,
        conflicts_with,
        related,
        stale_after,
        evidence,
        confidence: Confidence {
            level: "high".into(),
            implementation: "authored".into(),
            tests: "unknown".into(),
            runtime: "unknown".into(),
        },
        source_text: Some(source.to_string()),
    }
}

fn inspect_source(
    abs: &Path,
    rel: &str,
) -> Result<(ModuleInfo, Vec<SymbolInfo>, Vec<KnowledgeItem>)> {
    let source = fs::read_to_string(abs).with_context(|| format!("read {}", abs.display()))?;
    let imports = extract_frontmatter_imports(&source);
    let title = extract_fm_field(&source, "title")
        .unwrap_or_else(|| rel.trim_end_matches(".mq.md").to_string());
    let verified_by = extract_nested_by(&source, "verified");
    let sources = extract_fm_list(&source, "sources");
    let status = Lifecycle::parse(
        &extract_fm_field(&source, "status").unwrap_or_else(|| "stable".into()),
    );
    let responsibility = first_paragraph(&source);
    let responsibility = if responsibility.is_empty() {
        None
    } else {
        Some(responsibility)
    };

    let mut knowledge = Vec::new();
    if let Some(ty) = extract_fm_field(&source, "type") {
        let kind = KnowledgeKind::parse(&ty);
        match kind {
            KnowledgeKind::Decision
            | KnowledgeKind::Constraint
            | KnowledgeKind::Failure
            | KnowledgeKind::AntiPattern
            | KnowledgeKind::Pattern
            | KnowledgeKind::Migration
            | KnowledgeKind::Fact => {
                knowledge.push(knowledge_from_source(&source, rel, kind));
            }
            _ => {}
        }
    }

    let module_id = rel.trim_end_matches(".mq.md").to_string();
    let mut symbols = Vec::new();

    match parse_source(&source) {
        Ok(module) => {
            for f in &module.functions {
                push_function_symbols(f, &module_id, rel, &source, &mut symbols);
            }
        }
        Err(_) => {
            for line in classify_source(&source) {
                if line.kind != LineKind::Code {
                    continue;
                }
                let t = line.text.trim();
                if let Some(rest) = t.strip_prefix("# ") {
                    if !rest.starts_with('#') {
                        let name = rest
                            .split('=')
                            .next()
                            .unwrap_or(rest)
                            .trim()
                            .to_string();
                        let id = format!("{module_id}::{name}");
                        let fp = fingerprint_text(&name, &[], &[], &[]);
                        symbols.push(SymbolInfo {
                            id,
                            name,
                            module: module_id.clone(),
                            resource: rel.to_string(),
                            kind: "fn".into(),
                            visibility: "public".into(),
                            params: Vec::new(),
                            calls: Vec::new(),
                            called_by: Vec::new(),
                            tests: Vec::new(),
                            fingerprint: fingerprint_hash(&fp),
                            fingerprint_text: fp,
                            use_when: None,
                            do_not_use_when: None,
                            related: Vec::new(),
                            reuse: extract_fm_list(&source, "reuse"),
                            capability: extract_fm_field(&source, "capability"),
                            status: status.clone(),
                            evidence: Evidence {
                                sources: vec![rel.to_string()],
                                ..Evidence::default()
                            },
                            confidence: Confidence::default(),
                        });
                    }
                }
            }
        }
    }

    let exports: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();
    let module = ModuleInfo {
        id: module_id,
        resource: rel.to_string(),
        title,
        imports,
        exports,
        responsibility,
        verified_by,
        sources,
        status,
    };
    Ok((module, symbols, knowledge))
}

fn push_function_symbols(
    f: &Function,
    module_id: &str,
    rel: &str,
    source: &str,
    out: &mut Vec<SymbolInfo>,
) {
    let name = f.name.clone();
    let id = format!("{module_id}::{name}");
    let params: Vec<String> = f.params.iter().map(|p| p.name.clone()).collect();
    let calls = collect_calls(f);
    let ops = ops_hint(f);
    let fp = fingerprint_text(&name, &params, &calls, &ops);
    let kind = if f.is_object() {
        "object".to_string()
    } else {
        "fn".to_string()
    };
    // Prefer function-local sections if present in source near heading — fall back to file-level
    let use_when = extract_section(source, "Use when").or_else(|| prose_field(source, "Use when"));
    let do_not = extract_section(source, "Do not use when")
        .or_else(|| prose_field(source, "Do not use when"));
    let related = extract_fm_list(source, "related");
    let reuse = extract_fm_list(source, "reuse");
    let capability = extract_fm_field(source, "capability");
    let status = Lifecycle::parse(
        &extract_fm_field(source, "status").unwrap_or_else(|| "stable".into()),
    );

    out.push(SymbolInfo {
        id,
        name,
        module: module_id.to_string(),
        resource: rel.to_string(),
        kind,
        visibility: "public".into(),
        params,
        calls,
        called_by: Vec::new(),
        tests: Vec::new(),
        fingerprint: fingerprint_hash(&fp),
        fingerprint_text: fp,
        use_when,
        do_not_use_when: do_not,
        related,
        reuse,
        capability,
        status,
        evidence: Evidence {
            sources: vec![rel.to_string()],
            ..Evidence::default()
        },
        confidence: Confidence {
            level: "high".into(),
            implementation: "verified".into(),
            tests: "unknown".into(),
            runtime: "unknown".into(),
        },
    });

    for child in &f.children {
        push_function_symbols(child, module_id, rel, source, out);
    }
}

fn prose_field(source: &str, label: &str) -> Option<String> {
    let prefix = format!("{label}:");
    for line in source.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix(&prefix) {
            let v = rest.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn extract_frontmatter_imports(source: &str) -> Vec<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let mut imports = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return imports;
    }
    for line in lines.iter().skip(1) {
        let t = line.trim();
        if t == "---" {
            break;
        }
        if let Ok(Some(crate::parse::ImportLine::File(imp))) =
            crate::parse::parse_import_line(t)
        {
            imports.push(format!("import {}:{}", imp.bind, imp.path));
        } else if let Ok(Some(crate::parse::ImportLine::Member(u))) =
            crate::parse::parse_import_line(t)
        {
            imports.push(format!("import {}:{}", u.bind, u.path.join(".")));
        }
    }
    imports
}

#[allow(dead_code)]
pub fn stem_of(resource: &str) -> String {
    module_stem(resource)
}
