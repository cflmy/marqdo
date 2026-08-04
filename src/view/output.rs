//! Static HTML export: `marqdo view output`.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::view::html::{page_file, page_index, LinkMode};
use crate::view::{build_file_view, build_root, ViewRoot};

pub struct OutputOptions {
    pub path: PathBuf,
    pub out_dir: PathBuf,
    pub no_exec: bool,
}

pub fn write_static(opts: OutputOptions) -> Result<()> {
    let root = build_root(&opts.path)?;
    fs::create_dir_all(&opts.out_dir)
        .with_context(|| format!("create {}", opts.out_dir.display()))?;

    let pages_root = opts.out_dir.join("pages");
    let mut first_page_html: Option<String> = None;

    for rel_pb in &root.files {
        let rel = rel_pb.to_string_lossy().replace('\\', "/");
        let abs = resolve_abs(&root, &rel)?;
        let mut vm = build_file_view(&abs, &rel, &[])?;
        if opts.no_exec {
            vm.stdout.clear();
            vm.stderr = "execution skipped (--no-exec)".into();
            vm.ok = true;
        }
        let links = LinkMode::Static {
            from: Some(rel.clone()),
        };
        let html = page_file(&root.files, &rel, &vm, &links);
        let out_file = pages_root.join(format!("{rel}.html"));
        if let Some(parent) = out_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out_file, html).with_context(|| format!("write {}", out_file.display()))?;

        if first_page_html.is_none() {
            // index.html = first file (same content, links rooted at OUT_DIR)
            first_page_html = Some(page_file(
                &root.files,
                &rel,
                &vm,
                &LinkMode::Static { from: None },
            ));
        }
    }

    let index_html = first_page_html.unwrap_or_else(|| {
        page_index(&root.files, root.only_file.as_deref(), &LinkMode::Static { from: None })
    });
    let index_path = opts.out_dir.join("index.html");
    fs::write(&index_path, index_html)
        .with_context(|| format!("write {}", index_path.display()))?;

    eprintln!(
        "marqdo view output: {} page(s) → {}",
        root.files.len(),
        opts.out_dir.display()
    );
    Ok(())
}

fn resolve_abs(root: &ViewRoot, rel: &str) -> Result<PathBuf> {
    let abs = root.root.join(rel);
    Ok(abs.canonicalize().unwrap_or(abs))
}
