//! Project boundaries and deterministic source discovery.
//!
//! The executable-knowledge compiler must never let each consumer decide what
//! a project contains.  This module establishes one filesystem boundary and a
//! stable, sorted list of Marqdo source files for `sema`, EKC, view, and MLSP.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// Canonical project boundary used by project-wide compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRoot {
    root: PathBuf,
}

impl ProjectRoot {
    /// Find the nearest explicit project marker or Git root.  A standalone
    /// source file without either marker forms a one-directory project.
    pub fn discover(start: &Path) -> Result<Self> {
        let start = if start.is_file() {
            start.parent().unwrap_or_else(|| Path::new("."))
        } else {
            start
        };
        let start = start
            .canonicalize()
            .with_context(|| format!("resolve project start {}", start.display()))?;
        let fallback = start.clone();
        let mut current = start.as_path();
        loop {
            if current.join(".marqdo").join("project.toml").is_file()
                || current.join(".git").exists()
            {
                return Ok(Self {
                    root: current.to_path_buf(),
                });
            }
            match current.parent() {
                Some(parent) => current = parent,
                None => {
                    return Ok(Self { root: fallback });
                }
            }
        }
    }

    /// Construct an explicitly scoped project root.
    pub fn new(root: &Path) -> Result<Self> {
        let root = root
            .canonicalize()
            .with_context(|| format!("resolve project root {}", root.display()))?;
        if !root.is_dir() {
            bail!("project root {} is not a directory", root.display());
        }
        Ok(Self { root })
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Discover every project-local `.mq.md` file in lexicographic relative
    /// path order.  Generated and dependency directories are intentionally
    /// excluded so a cache cannot silently become source knowledge.
    pub fn source_files(&self) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        collect_sources(&self.root, &self.root, &mut out)?;
        out.sort_by(|a, b| relative_key(&self.root, a).cmp(&relative_key(&self.root, b)));
        Ok(out)
    }
}

const EXCLUDED_DIRS: &[&str] = &[
    ".git",
    ".marqdo",
    "target",
    "node_modules",
    ".cursor",
    ".agents",
];

fn collect_sources(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("read project directory {}", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            // A source symlink is only admitted if its resolved target remains
            // within the project boundary.  This prevents index escape.
            let Ok(canonical) = path.canonicalize() else {
                continue;
            };
            if !canonical.starts_with(root) {
                continue;
            }
            if canonical.is_dir() {
                collect_sources(root, &canonical, out)?;
            } else if is_mq_md(&canonical) {
                out.push(canonical);
            }
            continue;
        }
        if file_type.is_dir() {
            let name = entry.file_name();
            if EXCLUDED_DIRS.iter().any(|excluded| name == *excluded) {
                continue;
            }
            collect_sources(root, &path, out)?;
        } else if file_type.is_file() && is_mq_md(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn is_mq_md(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".mq.md"))
}

fn relative_key(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("marqdo-project-{name}-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn source_discovery_is_sorted_and_skips_generated_dirs() {
        let root = fixture_root("sources");
        fs::create_dir_all(root.join("z")).unwrap();
        fs::create_dir_all(root.join("a")).unwrap();
        fs::create_dir_all(root.join("target")).unwrap();
        fs::write(root.join("z/b.mq.md"), "# z").unwrap();
        fs::write(root.join("a/a.mq.md"), "# a").unwrap();
        fs::write(root.join("target/ignored.mq.md"), "# ignored").unwrap();
        fs::write(root.join("README.md"), "not source").unwrap();

        let project = ProjectRoot::new(&root).unwrap();
        let files = project.source_files().unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|p| {
                p.strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        assert_eq!(names, ["a/a.mq.md", "z/b.mq.md"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn nearest_marker_wins() {
        let root = fixture_root("discover");
        let nested = root.join("nested/work");
        fs::create_dir_all(nested.join(".marqdo")).unwrap();
        fs::write(nested.join(".marqdo/project.toml"), "id = \"nested\"").unwrap();
        fs::create_dir_all(nested.join("src")).unwrap();
        let file = nested.join("src/main.mq.md");
        fs::write(&file, "# main").unwrap();

        let project = ProjectRoot::discover(&file).unwrap();
        assert_eq!(project.path(), nested.canonicalize().unwrap());
        fs::remove_dir_all(root).unwrap();
    }
}
