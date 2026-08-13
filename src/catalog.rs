//! OKF-compatible catalog generation: `marqdo catalog` / `sync` (O1+O3).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::lex::{classify_source, LineKind};
use crate::parse::parse_source;

pub struct CatalogOptions {
    pub path: PathBuf,
    pub out_dir: PathBuf,
}

struct ModuleInfo {
    id: String,
    resource: String,
    title: String,
    imports: Vec<String>,
    exports: Vec<String>,
    /// Optional OKF trust fields copied from source frontmatter.
    verified_by: Option<String>,
    sources: Vec<String>,
}

struct ConceptInfo {
    /// Relative id e.g. `concepts/tasks/<sig>`
    id: String,
    concept_type: String,
    title: String,
    status: String,
    resource: Option<String>,
    skill: Option<String>,
    description: Option<String>,
    verified_by: Option<String>,
    /// Absolute path of source concept page (to copy).
    abs: PathBuf,
    /// Relative path under out_dir e.g. `concepts/tasks/x.md`
    out_rel: String,
}

pub fn write_catalog(opts: CatalogOptions) -> Result<()> {
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

    let mut files = Vec::new();
    if opts.path.is_file() {
        files.push(opts.path.canonicalize().unwrap_or(opts.path.clone()));
    } else {
        collect_mq_md(&root, &root, &out_canon, &mut files)?;
        files.sort();
    }

    fs::create_dir_all(opts.out_dir.join("modules"))
        .with_context(|| format!("create {}", opts.out_dir.display()))?;
    fs::create_dir_all(opts.out_dir.join("concepts"))
        .with_context(|| format!("create concepts under {}", opts.out_dir.display()))?;

    let version = env!("CARGO_PKG_VERSION");
    let mut modules = Vec::new();
    let mut had_err = false;

    for abs in &files {
        let rel = abs
            .strip_prefix(&root)
            .unwrap_or(abs)
            .to_string_lossy()
            .replace('\\', "/");
        // Skip agent-kb resources (executable skills) from Module table — they appear as concepts.
        if rel.contains("/.marqdo/agent-kb/") || rel.starts_with(".marqdo/agent-kb/") {
            continue;
        }
        match inspect_file(abs, &rel) {
            Ok(info) => modules.push(info),
            Err(e) => {
                eprintln!("catalog: skip {rel}: {e:#}");
                had_err = true;
            }
        }
    }

    let mut concepts = Vec::new();
    collect_agent_concepts(&root, &out_canon, &mut concepts)?;
    concepts.sort_by(|a, b| a.id.cmp(&b.id));

    for c in &concepts {
        let dest = opts.out_dir.join(&c.out_rel);
        if let Some(p) = dest.parent() {
            fs::create_dir_all(p)?;
        }
        let body = fs::read_to_string(&c.abs)
            .with_context(|| format!("read concept {}", c.abs.display()))?;
        fs::write(&dest, body)?;
    }

    let catalog_yaml = render_catalog_yaml(&modules, &concepts, version);
    fs::write(opts.out_dir.join("catalog.yaml"), catalog_yaml)?;

    let index_md = render_index_md(&modules, &concepts, version);
    fs::write(opts.out_dir.join("index.md"), index_md)?;

    for m in &modules {
        let stem = module_stem(&m.resource);
        let path = opts.out_dir.join("modules").join(format!("{stem}.md"));
        fs::write(&path, render_module_md(m, &modules, version))?;
    }

    eprintln!(
        "marqdo catalog: {} module(s), {} concept(s) → {}",
        modules.len(),
        concepts.len(),
        opts.out_dir.display()
    );
    if had_err {
        anyhow::bail!("catalog completed with errors");
    }
    Ok(())
}

fn module_stem(resource: &str) -> String {
    resource
        .replace('/', "__")
        .replace('\\', "__")
        .trim_end_matches(".mq.md")
        .to_string()
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

/// Find `**/.marqdo/agent-kb/concepts/**/*.md` under scan root.
fn collect_agent_concepts(
    root: &Path,
    out_canon: &Path,
    out: &mut Vec<ConceptInfo>,
) -> Result<()> {
    walk_for_concepts(root, root, out_canon, out)
}

fn walk_for_concepts(
    root: &Path,
    dir: &Path,
    out_canon: &Path,
    out: &mut Vec<ConceptInfo>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            if let Ok(canon) = p.canonicalize() {
                if canon == *out_canon {
                    continue;
                }
            }
            walk_for_concepts(root, &p, out_canon, out)?;
            continue;
        }
        let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.ends_with(".md") || name.ends_with(".mq.md") {
            continue;
        }
        let rel = p
            .strip_prefix(root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        let marker = "/.marqdo/agent-kb/concepts/";
        let idx = if let Some(i) = rel.find(marker) {
            i + marker.len()
        } else if let Some(rest) = rel.strip_prefix(".marqdo/agent-kb/concepts/") {
            // path relative from agent-kb concepts
            let _ = rest;
            0
        } else {
            continue;
        };
        let under = if idx == 0 {
            rel.strip_prefix(".marqdo/agent-kb/concepts/")
                .unwrap_or("")
                .to_string()
        } else {
            rel[idx..].to_string()
        };
        if under.is_empty() {
            continue;
        }
        let source = fs::read_to_string(&p)?;
        let concept_type =
            extract_fm_field(&source, "type").unwrap_or_else(|| "Marqdo Concept".into());
        let title = extract_fm_field(&source, "title").unwrap_or_else(|| under.clone());
        let status = extract_fm_field(&source, "status").unwrap_or_else(|| "stable".into());
        let resource = extract_fm_field(&source, "resource");
        let skill = extract_fm_field(&source, "skill");
        let description = extract_fm_field(&source, "description");
        let verified_by = extract_nested_by(&source, "verified");
        let id = format!("concepts/{}", under.trim_end_matches(".md"));
        let out_rel = format!("concepts/{under}");
        out.push(ConceptInfo {
            id,
            concept_type,
            title,
            status,
            resource,
            skill,
            description,
            verified_by,
            abs: p,
            out_rel,
        });
    }
    Ok(())
}

fn inspect_file(abs: &Path, rel: &str) -> Result<ModuleInfo> {
    let source = fs::read_to_string(abs)?;
    let imports = extract_frontmatter_imports(&source);
    let title = extract_frontmatter_title(&source)
        .unwrap_or_else(|| rel.trim_end_matches(".mq.md").to_string());
    let verified_by = extract_nested_by(&source, "verified");
    let sources = extract_fm_list(&source, "sources");

    let mut exports = Vec::new();
    match parse_source(&source) {
        Ok(module) => {
            for f in &module.functions {
                exports.push(f.name.clone());
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
                        exports.push(rest.to_string());
                    }
                }
            }
        }
    }

    let id = rel.trim_end_matches(".mq.md").to_string();
    Ok(ModuleInfo {
        id,
        resource: rel.to_string(),
        title,
        imports,
        exports,
        verified_by,
        sources,
    })
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

fn extract_frontmatter_title(source: &str) -> Option<String> {
    extract_fm_field(source, "title")
}

fn extract_fm_field(source: &str, key: &str) -> Option<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return None;
    }
    let prefix = format!("{key}:");
    for line in lines.iter().skip(1) {
        let t = line.trim();
        if t == "---" {
            break;
        }
        if let Some(rest) = t.strip_prefix(&prefix) {
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    None
}

/// `verified:\n  by: foo` → Some("foo")
fn extract_nested_by(source: &str, block: &str) -> Option<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return None;
    }
    let mut in_block = false;
    for line in lines.iter().skip(1) {
        let t = line.trim_end();
        if t.trim() == "---" {
            break;
        }
        if t.trim() == format!("{block}:") || t.trim().starts_with(&format!("{block}:")) {
            // inline `verified: { by: x }` not supported; block form:
            if t.trim() == format!("{block}:") {
                in_block = true;
                continue;
            }
        }
        if in_block {
            let trimmed = t.trim();
            if trimmed.starts_with("by:") {
                return Some(
                    trimmed
                        .trim_start_matches("by:")
                        .trim()
                        .trim_matches('"')
                        .to_string(),
                );
            }
            if !trimmed.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
        }
    }
    None
}

fn extract_fm_list(source: &str, key: &str) -> Vec<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut in_list = false;
    let header = format!("{key}:");
    for line in lines.iter().skip(1) {
        let t = line.trim_end();
        if t.trim() == "---" {
            break;
        }
        if t.trim() == header || t.trim() == format!("{key}: []") {
            if t.trim().ends_with("[]") {
                return Vec::new();
            }
            in_list = true;
            continue;
        }
        if in_list {
            let trimmed = t.trim();
            if let Some(rest) = trimmed.strip_prefix("- ") {
                out.push(rest.trim().trim_matches('"').to_string());
            } else if !trimmed.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
        }
    }
    out
}

fn yaml_escape(s: &str) -> String {
    if s.is_empty()
        || s.contains(':')
        || s.contains('#')
        || s.contains('"')
        || s.contains('\'')
        || s.contains('\n')
        || s.starts_with(' ')
    {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

fn import_path(imp: &str) -> &str {
    imp.split(" as ").next().unwrap_or(imp).trim()
}

fn resolve_import_module<'a>(imp: &str, modules: &'a [ModuleInfo]) -> Option<&'a ModuleInfo> {
    let path = import_path(imp);
    modules.iter().find(|m| {
        m.resource == path
            || m.resource.ends_with(path)
            || m.resource.ends_with(&format!("/{path}"))
    })
}

fn render_catalog_yaml(modules: &[ModuleInfo], concepts: &[ConceptInfo], version: &str) -> String {
    let mut out = String::new();
    out.push_str("# GENERATED by marqdo — do not edit by hand\n");
    out.push_str("type: Marqdo Catalog\n");
    out.push_str("title: marqdo-project\n");
    out.push_str("generated:\n");
    out.push_str(&format!("  by: marqdo/{version}\n"));
    out.push_str("modules:\n");
    for m in modules {
        out.push_str(&format!("  - id: {}\n", yaml_escape(&m.id)));
        out.push_str(&format!("    resource: {}\n", yaml_escape(&m.resource)));
        out.push_str(&format!("    title: {}\n", yaml_escape(&m.title)));
        if m.imports.is_empty() {
            out.push_str("    imports: []\n");
        } else {
            out.push_str("    imports:\n");
            for i in &m.imports {
                out.push_str(&format!("      - {}\n", yaml_escape(i)));
            }
        }
        if m.exports.is_empty() {
            out.push_str("    exports: []\n");
        } else {
            out.push_str("    exports:\n");
            for e in &m.exports {
                out.push_str(&format!("      - name: {}\n", yaml_escape(e)));
                out.push_str("        kind: fn\n");
            }
        }
        if let Some(ref v) = m.verified_by {
            out.push_str("    verified:\n");
            out.push_str(&format!("      by: {}\n", yaml_escape(v)));
        }
        if !m.sources.is_empty() {
            out.push_str("    sources:\n");
            for s in &m.sources {
                out.push_str(&format!("      - {}\n", yaml_escape(s)));
            }
        }
    }
    out.push_str("concepts:\n");
    if concepts.is_empty() {
        // keep key present for consumers
        out.push_str("  []\n");
    } else {
        for c in concepts {
            out.push_str(&format!("  - id: {}\n", yaml_escape(&c.id)));
            out.push_str(&format!("    type: {}\n", yaml_escape(&c.concept_type)));
            out.push_str(&format!("    title: {}\n", yaml_escape(&c.title)));
            out.push_str(&format!("    status: {}\n", yaml_escape(&c.status)));
            out.push_str(&format!("    page: {}\n", yaml_escape(&c.out_rel)));
            if let Some(ref r) = c.resource {
                out.push_str(&format!("    resource: {}\n", yaml_escape(r)));
            }
            if let Some(ref s) = c.skill {
                out.push_str(&format!("    skill: {}\n", yaml_escape(s)));
            }
            if let Some(ref d) = c.description {
                out.push_str(&format!("    description: {}\n", yaml_escape(d)));
            }
            if let Some(ref v) = c.verified_by {
                out.push_str("    verified:\n");
                out.push_str(&format!("      by: {}\n", yaml_escape(v)));
            }
        }
    }
    out
}

fn render_index_md(modules: &[ModuleInfo], concepts: &[ConceptInfo], version: &str) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("type: Marqdo Catalog\n");
    out.push_str("title: marqdo catalog\n");
    out.push_str("generated:\n");
    out.push_str(&format!("  by: marqdo/{version}\n"));
    out.push_str("---\n\n");
    out.push_str("# Catalog\n\n");
    out.push_str("> Generated by Marqdo. Do not edit by hand.\n\n");
    out.push_str("## Modules\n\n");
    out.push_str("| Module | Resource | Imports |\n");
    out.push_str("|--------|----------|--------|\n");
    for m in modules {
        let stem = module_stem(&m.resource);
        let imports_cell = if m.imports.is_empty() {
            "—".to_string()
        } else {
            m.imports
                .iter()
                .map(|i| {
                    if let Some(dep) = resolve_import_module(i, modules) {
                        format!(
                            "[{}](modules/{}.md)",
                            dep.title,
                            module_stem(&dep.resource)
                        )
                    } else {
                        format!("`{i}`")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        out.push_str(&format!(
            "| [{}](modules/{stem}.md) | `{}` | {imports_cell} |\n",
            m.title, m.resource
        ));
    }

    out.push_str("\n## Agent knowledge (OKF concepts)\n\n");
    if concepts.is_empty() {
        out.push_str("_No `.marqdo/agent-kb` concepts found under the scan root._\n");
    } else {
        out.push_str("| Concept | Type | Status | Page |\n");
        out.push_str("|---------|------|--------|------|\n");
        for c in concepts {
            out.push_str(&format!(
                "| {} | `{}` | {} | [{}]({}) |\n",
                c.title, c.concept_type, c.status, c.id, c.out_rel
            ));
        }
        out.push_str(
            "\nThese pages are copied from local `agent-kb` for human browsing alongside modules.\n",
        );
    }
    out
}

fn render_module_md(m: &ModuleInfo, modules: &[ModuleInfo], version: &str) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("type: Marqdo Module\n");
    out.push_str(&format!("title: {}\n", m.title));
    out.push_str(&format!("resource: {}\n", m.resource));
    if m.imports.is_empty() {
        out.push_str("depends: []\n");
    } else {
        out.push_str("depends:\n");
        for i in &m.imports {
            out.push_str(&format!("  - {i}\n"));
        }
    }
    if m.exports.is_empty() {
        out.push_str("exports: []\n");
    } else {
        out.push_str("exports:\n");
        for e in &m.exports {
            out.push_str(&format!("  - {e}\n"));
        }
    }
    out.push_str("generated:\n");
    out.push_str(&format!("  by: marqdo/{version}\n"));
    if let Some(ref v) = m.verified_by {
        out.push_str("verified:\n");
        out.push_str(&format!("  by: {v}\n"));
    }
    if !m.sources.is_empty() {
        out.push_str("sources:\n");
        for s in &m.sources {
            out.push_str(&format!("  - {s}\n"));
        }
    }
    out.push_str("---\n\n");
    out.push_str(&format!("# {}\n\n", m.title));
    out.push_str(&format!(
        "> Auto-generated from [`{}`](../../{}). Do not edit by hand.\n\n",
        m.resource, m.resource
    ));
    out.push_str("## Dependencies\n\n");
    if m.imports.is_empty() {
        out.push_str("_None_\n\n");
    } else {
        for i in &m.imports {
            if let Some(dep) = resolve_import_module(i, modules) {
                out.push_str(&format!(
                    "- [{}]({}.md) (`{}`)\n",
                    dep.title,
                    module_stem(&dep.resource),
                    i
                ));
            } else {
                out.push_str(&format!("- `{i}`\n"));
            }
        }
        out.push('\n');
    }
    out.push_str("## Exports\n\n");
    if m.exports.is_empty() {
        out.push_str("_None_\n");
    } else {
        out.push_str("| Symbol | Kind |\n|--------|------|\n");
        for e in &m.exports {
            out.push_str(&format!("| `{e}` | fn |\n"));
        }
    }
    out
}
