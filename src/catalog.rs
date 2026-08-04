//! OKF-compatible catalog generation: `marqdo catalog` / `sync`.

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

    let mut files = Vec::new();
    if opts.path.is_file() {
        files.push(opts.path.canonicalize().unwrap_or(opts.path.clone()));
    } else {
        collect_mq_md(&root, &root, &mut files)?;
        files.sort();
    }

    fs::create_dir_all(opts.out_dir.join("modules"))
        .with_context(|| format!("create {}", opts.out_dir.display()))?;

    let version = env!("CARGO_PKG_VERSION");
    let mut modules = Vec::new();
    let mut had_err = false;

    for abs in &files {
        let rel = abs
            .strip_prefix(&root)
            .unwrap_or(abs)
            .to_string_lossy()
            .replace('\\', "/");
        match inspect_file(abs, &rel) {
            Ok(info) => modules.push(info),
            Err(e) => {
                eprintln!("catalog: skip {rel}: {e:#}");
                had_err = true;
            }
        }
    }

    let catalog_yaml = render_catalog_yaml(&modules, version);
    fs::write(opts.out_dir.join("catalog.yaml"), catalog_yaml)?;

    let index_md = render_index_md(&modules, version);
    fs::write(opts.out_dir.join("index.md"), index_md)?;

    for m in &modules {
        let stem = m.resource.replace('/', "__").replace('\\', "__");
        let stem = stem.trim_end_matches(".mq.md");
        let path = opts.out_dir.join("modules").join(format!("{stem}.md"));
        fs::write(&path, render_module_md(m, version))?;
    }

    eprintln!(
        "marqdo catalog: {} module(s) → {}",
        modules.len(),
        opts.out_dir.display()
    );
    if had_err {
        anyhow::bail!("catalog completed with errors");
    }
    Ok(())
}

fn collect_mq_md(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            collect_mq_md(root, &p, out)?;
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

fn inspect_file(abs: &Path, rel: &str) -> Result<ModuleInfo> {
    let source = fs::read_to_string(abs)?;
    let imports = extract_frontmatter_imports(&source);
    let title = extract_frontmatter_title(&source)
        .unwrap_or_else(|| rel.trim_end_matches(".mq.md").to_string());

    let mut exports = Vec::new();
    // Prefer parse for accurate top-level fn names
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
        if let Some(rest) = t.strip_prefix('>') {
            let path = rest.trim();
            if path.ends_with(".mq.md") {
                imports.push(path.to_string());
            }
        }
    }
    imports
}

fn extract_frontmatter_title(source: &str) -> Option<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return None;
    }
    for line in lines.iter().skip(1) {
        let t = line.trim();
        if t == "---" {
            break;
        }
        if let Some(v) = t.strip_prefix("title:") {
            return Some(v.trim().trim_matches('"').to_string());
        }
    }
    None
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

fn render_catalog_yaml(modules: &[ModuleInfo], version: &str) -> String {
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
    }
    out
}

fn render_index_md(modules: &[ModuleInfo], version: &str) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("type: Marqdo Catalog\n");
    out.push_str("title: marqdo catalog\n");
    out.push_str("generated:\n");
    out.push_str(&format!("  by: marqdo/{version}\n"));
    out.push_str("---\n\n");
    out.push_str("# Catalog\n\n");
    out.push_str("> Generated by Marqdo. Do not edit by hand.\n\n");
    out.push_str("| Module | Resource | Imports |\n");
    out.push_str("|--------|----------|--------|\n");
    for m in modules {
        let stem = m.resource.replace('/', "__");
        let stem = stem.trim_end_matches(".mq.md");
        out.push_str(&format!(
            "| [{}](modules/{stem}.md) | `{}` | {} |\n",
            m.title,
            m.resource,
            if m.imports.is_empty() {
                "—".into()
            } else {
                m.imports.join(", ")
            }
        ));
    }
    out
}

fn render_module_md(m: &ModuleInfo, version: &str) -> String {
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
            out.push_str(&format!("- `{i}`\n"));
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
