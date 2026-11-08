//! Inspect real single-file standard library signatures; no index module is required.
//! --json emits compiler metadata; --check validates source effect notes;
//! --write refreshes only effect notes, preserving authored prose and examples.
use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let write = args.iter().any(|arg| arg == "--write");
    let json = args.iter().any(|arg| arg == "--json");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut sources: Vec<_> = fs::read_dir(root.join("lib"))?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<std::io::Result<_>>()?;
    sources.retain(|p| p.is_file() && p.to_string_lossy().ends_with(".mq.md"));
    sources.sort();
    let mut changed = 0;
    let mut tools = Vec::new();
    for source in sources {
        let module = marqdo::load::load_module(&source)?;
        let compilation = marqdo::sema::compile_file_closure(&source)?;
        let content = fs::read_to_string(&source)?;
        let mut lines: Vec<_> = content.lines().map(str::to_owned).collect();
        let mut edits = Vec::new();
        let zh = source
            .file_name()
            .unwrap()
            .to_string_lossy()
            .chars()
            .any(|c| !c.is_ascii());
        for (index, function) in module.functions.iter().enumerate() {
            let artifact = compilation
                .semantics
                .artifacts
                .iter()
                .find(|a| {
                    a.kind == "function"
                        && a.source.line == function.span.line
                        && same_source(&compilation.semantics.root, &a.source.file, &source)
                })
                .context("missing compiler function artifact")?;
            let signature = artifact
                .signature
                .as_ref()
                .context("missing compiler signature")?;
            let effects: Vec<_> = artifact
                .effects
                .values()
                .into_iter()
                .filter(|e| *e != "pure")
                .collect();
            let effects = if effects.is_empty() {
                "pure".to_owned()
            } else {
                effects.join(",")
            };
            let expected = if zh {
                format!("调用效应：{effects}。")
            } else {
                format!("Effects: {effects}.")
            };
            let start = function.span.line as usize;
            let end = module
                .functions
                .get(index + 1)
                .map(|f| (f.span.line as usize).saturating_sub(1))
                .unwrap_or(lines.len());
            let notes: Vec<_> = (start..end)
                .filter(|i| {
                    lines[*i].starts_with("Effects: ") || lines[*i].starts_with("调用效应：")
                })
                .collect();
            if notes.len() != 1 {
                bail!(
                    "{} {} needs one effect note in its source",
                    source.display(),
                    function.name
                );
            }
            if lines[notes[0]] != expected {
                changed += 1;
                if write {
                    edits.push((notes[0], expected));
                } else if !json {
                    eprintln!("stale effect note: {} {}", source.display(), function.name);
                }
            }
            tools.push(serde_json::json!({
                "name": function.name,
                "parameters": signature.params.iter().map(|p| serde_json::json!({"name":p.name,"type":p.ty,"required":p.required,"default_digest":p.default_digest.as_ref().map(|d| &d.0),"unit":p.unit})).collect::<Vec<_>>(),
                "returns": signature.returns,
                "effects": artifact.effects.values(),
                "source": source.strip_prefix(&root)?,
                "line": function.span.line
            }));
        }
        if !edits.is_empty() {
            for (line, text) in edits {
                lines[line] = text;
            }
            fs::write(&source, lines.join("\n") + "\n")?;
        }
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&tools)?);
    } else if changed > 0 && !write {
        bail!("{changed} effect notes differ from compiler metadata; run with --write");
    } else {
        println!(
            "{} source effect notes {}; {} public functions",
            changed,
            if write { "updated" } else { "need updating" },
            tools.len()
        );
    }
    Ok(())
}
fn same_source(root: &Path, relative: &str, source: &Path) -> bool {
    root.join(relative).canonicalize().ok() == source.canonicalize().ok()
}
