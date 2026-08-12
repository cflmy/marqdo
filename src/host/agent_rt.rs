//! Runtime helpers for ABI `host_query` and shared map/list ops.

use std::path::{Path, PathBuf};

use crate::host::HostContext;
use crate::value::Value;

impl HostContext {
    pub fn set_entry_source(&mut self, path: Option<&Path>, source: &str) {
        self.entry_source = Some(source.to_string());
        self.entry_path = path.map(|p| p.to_path_buf());
    }

    pub fn push_call_frame(&mut self, name: &str) {
        self.call_stack.push(name.to_string());
    }

    pub fn pop_call_frame(&mut self) {
        self.call_stack.pop();
    }
}

pub fn module_source(ctx: &HostContext) -> Result<Value, String> {
    Ok(match &ctx.entry_source {
        Some(s) => Value::Text(s.clone()),
        None => Value::Text(String::new()),
    })
}

pub fn call_site(ctx: &HostContext, line: Option<u32>) -> Result<Value, String> {
    let path = ctx
        .entry_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let function = ctx
        .call_stack
        .last()
        .cloned()
        .unwrap_or_else(|| "main".into());
    let line_v = match line {
        Some(n) => Value::Int(n as i64),
        None => Value::None,
    };
    Ok(Value::Map(vec![
        ("path".into(), Value::Text(path)),
        ("function".into(), Value::Text(function)),
        ("line".into(), line_v),
    ]))
}

pub fn marqdo_skill(ctx: &HostContext) -> Result<Value, String> {
    let text = load_marqdo_skill(ctx)?;
    Ok(Value::Text(text))
}

fn load_marqdo_skill(ctx: &HostContext) -> Result<String, String> {
    if let Ok(p) = std::env::var("MARQDO_SKILL") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return std::fs::read_to_string(&path)
                .map_err(|e| format!("MARQDO_SKILL read {}: {e}", path.display()));
        }
        let skill = path.join("SKILL.md");
        if skill.is_file() {
            return read_skill_pack(&path);
        }
    }

    for root in skill_search_roots(ctx) {
        let pack = root.join("skills").join("marqdo");
        if pack.join("SKILL.md").is_file() {
            return read_skill_pack(&pack);
        }
        if root.join("SKILL.md").is_file() && root.ends_with("marqdo") {
            return read_skill_pack(&root);
        }
    }
    Ok(String::new())
}

fn read_skill_pack(dir: &Path) -> Result<String, String> {
    let mut out = String::new();
    for name in ["SKILL.md", "reference.md", "examples.md"] {
        let p = dir.join(name);
        if p.is_file() {
            let body = std::fs::read_to_string(&p)
                .map_err(|e| format!("skill read {}: {e}", p.display()))?;
            if !out.is_empty() {
                out.push_str("\n\n---\n\n");
            }
            out.push_str(&format!("# file: {name}\n\n"));
            out.push_str(&body);
        }
    }
    Ok(out)
}

fn skill_search_roots(ctx: &HostContext) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.to_path_buf());
            if let Some(parent) = dir.parent() {
                roots.push(parent.to_path_buf());
                if let Some(repo) = parent.parent() {
                    roots.push(repo.to_path_buf());
                }
            }
        }
    }
    roots.push(ctx.cwd.clone());
    if let Some(p) = &ctx.entry_path {
        if let Some(parent) = p.parent() {
            roots.push(parent.to_path_buf());
            if let Some(repo) = parent.parent().and_then(|p| p.parent()) {
                roots.push(repo.to_path_buf());
            }
        }
    }
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        roots.push(PathBuf::from(manifest));
    }
    roots
}

pub fn map_set(map: &Value, key: &Value, value: &Value) -> Result<Value, String> {
    crate::host::collection::map_set(map, key, value)
}

pub fn list_append(list: &Value, item: &Value) -> Result<Value, String> {
    crate::host::collection::list_append(list, item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_set_updates() {
        let m = Value::Map(vec![("a".into(), Value::Int(1))]);
        let m2 = map_set(&m, &Value::Text("a".into()), &Value::Int(2)).unwrap();
        assert_eq!(m2, Value::Map(vec![("a".into(), Value::Int(2))]));
    }
}
