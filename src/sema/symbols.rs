//! Stable module/symbol identity and deterministic project declaration index.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::ast::Function;
use crate::project::ProjectRoot;

/// Project-local module identity.  An explicit frontmatter `id` wins over the
/// normalized relative path, so authors can preserve identity across moves.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(String);

impl ModuleId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Globally stable address inside a project compilation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(String);

impl SymbolId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolKind {
    Type,
    Function,
}

impl SymbolKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Function => "function",
        }
    }
}

/// Callable classification is semantic metadata, separate from `SymbolKind`.
/// This keeps stable SymbolIds independent of whether a body later gains or
/// loses a receiver dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallableKind {
    FreeFunction,
    AssociatedFunction,
    InstanceMethod,
}

impl CallableKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FreeFunction => "free_function",
            Self::AssociatedFunction => "associated_function",
            Self::InstanceMethod => "instance_method",
        }
    }
}

/// One source declaration, including a display path for human-facing tools.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolDecl {
    pub id: SymbolId,
    pub module: ModuleId,
    pub kind: SymbolKind,
    /// Present only for functions; never inferred from the display name.
    pub callable: Option<CallableKind>,
    pub name: String,
    pub declaration_path: Vec<String>,
    pub source: PathBuf,
    pub line: u32,
}

/// Deterministic read-only view of all declarations in one project.
#[derive(Debug, Clone)]
pub struct ProjectIndex {
    project_id: String,
    root: ProjectRoot,
    modules: BTreeMap<ModuleId, PathBuf>,
    symbols: Vec<SymbolDecl>,
    by_id: BTreeMap<SymbolId, usize>,
    by_name: BTreeMap<String, Vec<usize>>,
}

impl ProjectIndex {
    pub fn build(root: ProjectRoot) -> Result<Self> {
        let project_id = project_id(root.path());
        let files = root.source_files()?;
        let mut modules = BTreeMap::new();
        let mut symbols = Vec::new();

        for file in files {
            let source = fs::read_to_string(&file)
                .with_context(|| format!("read source {}", file.display()))?;
            let module_id = module_id(&source, root.path(), &file)?;
            if let Some(previous) = modules.insert(module_id.clone(), file.clone()) {
                bail!(
                    "duplicate module id `{module_id}` in {} and {}",
                    previous.display(),
                    file.display()
                );
            }
            let module = crate::load::parse_source_auto(&source)
                .with_context(|| format!("parse source {}", file.display()))?;
            for function in &module.functions {
                collect_symbols(
                    &project_id,
                    &module_id,
                    &file,
                    function,
                    None,
                    &mut Vec::new(),
                    &mut symbols,
                )?;
            }
        }

        symbols.sort_by(|a, b| a.id.cmp(&b.id));
        let mut by_id = BTreeMap::new();
        let mut by_name: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (index, symbol) in symbols.iter().enumerate() {
            if by_id.insert(symbol.id.clone(), index).is_some() {
                bail!("duplicate symbol id `{}`", symbol.id);
            }
            by_name.entry(symbol.name.clone()).or_default().push(index);
        }
        for indexes in by_name.values_mut() {
            indexes.sort_by(|a, b| symbols[*a].id.cmp(&symbols[*b].id));
        }

        Ok(Self {
            project_id,
            root,
            modules,
            symbols,
            by_id,
            by_name,
        })
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn root(&self) -> &ProjectRoot {
        &self.root
    }

    pub fn modules(&self) -> &BTreeMap<ModuleId, PathBuf> {
        &self.modules
    }

    pub fn symbols(&self) -> &[SymbolDecl] {
        &self.symbols
    }

    pub fn symbol(&self, id: &SymbolId) -> Option<&SymbolDecl> {
        self.by_id.get(id).map(|index| &self.symbols[*index])
    }

    /// Candidates are deliberately returned as a list: discovery may be fuzzy,
    /// but callers must report ambiguity instead of silently choosing one.
    pub fn symbols_named(&self, name: &str) -> Vec<&SymbolDecl> {
        self.by_name
            .get(name)
            .into_iter()
            .flatten()
            .map(|index| &self.symbols[*index])
            .collect()
    }
}

fn collect_symbols(
    project_id: &str,
    module_id: &ModuleId,
    source: &Path,
    function: &Function,
    parent: Option<&Function>,
    parent_path: &mut Vec<String>,
    out: &mut Vec<SymbolDecl>,
) -> Result<()> {
    let kind = if function.is_object() {
        SymbolKind::Type
    } else {
        SymbolKind::Function
    };
    let callable = if function.is_object() {
        None
    } else if parent.is_some_and(Function::is_object) {
        Some(if crate::ast::function_uses_receiver(function) {
            CallableKind::InstanceMethod
        } else {
            CallableKind::AssociatedFunction
        })
    } else {
        Some(CallableKind::FreeFunction)
    };
    parent_path.push(function.name.clone());
    let id = symbol_id(project_id, module_id, kind, parent_path);
    if out.iter().any(|existing| existing.id == id) {
        bail!(
            "duplicate declaration path `{}` in module `{module_id}`",
            parent_path.join("/")
        );
    }
    out.push(SymbolDecl {
        id,
        module: module_id.clone(),
        kind,
        callable,
        name: function.name.clone(),
        declaration_path: parent_path.clone(),
        source: source.to_path_buf(),
        line: function.span.line,
    });
    for child in &function.children {
        collect_symbols(
            project_id,
            module_id,
            source,
            child,
            Some(function),
            parent_path,
            out,
        )?;
    }
    parent_path.pop();
    Ok(())
}

fn module_id(source: &str, root: &Path, file: &Path) -> Result<ModuleId> {
    if let Some(id) = frontmatter_value(source, "id") {
        if id.is_empty() {
            bail!("empty frontmatter id in {}", file.display());
        }
        return Ok(ModuleId(id));
    }
    let rel = file.strip_prefix(root).with_context(|| {
        format!(
            "source {} outside project root {}",
            file.display(),
            root.display()
        )
    })?;
    let mut value = rel.to_string_lossy().replace('\\', "/");
    value.truncate(value.len().saturating_sub(".mq.md".len()));
    Ok(ModuleId(value))
}

fn project_id(root: &Path) -> String {
    frontmatter_project_id(root).unwrap_or_else(|| {
        root.file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("marqdo-project")
            .to_string()
    })
}

fn frontmatter_project_id(root: &Path) -> Option<String> {
    let path = root.join(".marqdo").join("project.toml");
    let source = fs::read_to_string(path).ok()?;
    source.lines().find_map(|line| {
        let line = line.trim();
        let value = line
            .strip_prefix("id=")
            .or_else(|| line.strip_prefix("id ="))?;
        let value = value.trim().trim_matches('"');
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn frontmatter_value(source: &str, key: &str) -> Option<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let mut lines = source.lines();
    if lines.next().map(str::trim) != Some("---") {
        return None;
    }
    let key = format!("{key}:");
    for line in lines {
        let line = line.trim();
        if line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix(&key) {
            return Some(value.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn symbol_id(
    project_id: &str,
    module_id: &ModuleId,
    kind: SymbolKind,
    declaration_path: &[String],
) -> SymbolId {
    let path = declaration_path
        .iter()
        .map(|segment| percent_encode(segment))
        .collect::<Vec<_>>()
        .join("/");
    SymbolId(format!(
        "mq://{}/{}/{}{}",
        percent_encode(project_id),
        percent_encode(module_id.as_str()),
        kind.as_str(),
        if path.is_empty() {
            String::new()
        } else {
            format!("/{path}")
        }
    ))
}

fn percent_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
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
        let root = std::env::temp_dir().join(format!("marqdo-sema-{name}-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn index_uses_explicit_module_id_and_stable_declaration_paths() {
        let root = fixture_root("ids");
        let source = r#"---
id: knowledge.policy
---
# 书店政策

## 金卡折扣

*0.90*
"#;
        fs::write(root.join("policy.mq.md"), source).unwrap();

        let index = ProjectIndex::build(ProjectRoot::new(&root).unwrap()).unwrap();
        assert_eq!(index.modules().len(), 1);
        assert_eq!(index.symbols().len(), 2);
        assert!(index.symbols()[0].id.as_str().starts_with(&format!(
            "mq://{}/knowledge.policy/function/",
            index.project_id()
        )));
        assert!(index.symbols()[0].id.as_str().ends_with(
            "%E4%B9%A6%E5%BA%97%E6%94%BF%E7%AD%96/%E9%87%91%E5%8D%A1%E6%8A%98%E6%89%A3"
        ));
        assert_eq!(index.symbols()[1].declaration_path, ["书店政策"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn duplicate_module_ids_are_rejected() {
        let root = fixture_root("duplicate-module");
        fs::write(root.join("a.mq.md"), "---\nid: duplicate\n---\n# a").unwrap();
        fs::write(root.join("b.mq.md"), "---\nid: duplicate\n---\n# b").unwrap();

        let err = ProjectIndex::build(ProjectRoot::new(&root).unwrap())
            .unwrap_err()
            .to_string();
        assert!(err.contains("duplicate module id `duplicate`"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn symbols_named_remains_ambiguous_by_design() {
        let root = fixture_root("ambiguous");
        fs::write(root.join("a.mq.md"), "# A\n\n## 查询\n\n*1*").unwrap();
        fs::write(root.join("b.mq.md"), "# B\n\n## 查询\n\n*2*").unwrap();

        let index = ProjectIndex::build(ProjectRoot::new(&root).unwrap()).unwrap();
        assert_eq!(index.symbols_named("查询").len(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn percent_encoding_preserves_unicode_boundaries() {
        assert_eq!(
            percent_encode("李白/出生 年份"),
            "%E6%9D%8E%E7%99%BD%2F%E5%87%BA%E7%94%9F%20%E5%B9%B4%E4%BB%BD"
        );
    }

    #[test]
    fn object_children_are_classified_without_changing_their_symbol_identity() {
        let root = fixture_root("callable-kind");
        fs::write(
            root.join("facts.mq.md"),
            "# 李白\n\n## 出生年份\n\n*701*\n\n## 名称\n\n*`self`*",
        )
        .unwrap();

        let index = ProjectIndex::build(ProjectRoot::new(&root).unwrap()).unwrap();
        let fact = index.symbols_named("出生年份").pop().unwrap();
        let method = index.symbols_named("名称").pop().unwrap();
        assert_eq!(fact.kind, SymbolKind::Function);
        assert_eq!(fact.callable, Some(CallableKind::AssociatedFunction));
        assert_eq!(method.callable, Some(CallableKind::InstanceMethod));
        assert!(fact.id.as_str().contains("/function/"));
        fs::remove_dir_all(root).unwrap();
    }
}
