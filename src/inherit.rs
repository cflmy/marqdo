//! Object inheritance: `# Child = > Parent` validation and helpers.

use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::ast::{Function, Module};
use crate::diagnostics::{Diagnostic, Span};

/// Validate that every object's `base` resolves and that the inheritance graph is acyclic.
pub fn validate_inheritance(module: &Module) -> Result<()> {
    let mut by_name: HashMap<String, (Span, Option<String>)> = HashMap::new();
    collect_objects(module, &mut by_name);
    // Deterministic order: source line, then name (stable gold diagnostics).
    let mut names: Vec<String> = by_name.keys().cloned().collect();
    names.sort_by(|a, b| {
        let (sa, _) = &by_name[a];
        let (sb, _) = &by_name[b];
        sa.line.cmp(&sb.line).then_with(|| a.cmp(b))
    });

    for name in &names {
        let (span, base) = &by_name[name];
        let Some(base) = base else { continue };
        if !by_name.contains_key(base) {
            return Err(Diagnostic::new(
                None,
                *span,
                format!("unknown base type `{base}` for object `{name}`"),
            )
            .into());
        }
    }

    for name in &names {
        let (span, base) = &by_name[name];
        if base.is_none() {
            continue;
        }
        let mut path = Vec::new();
        let mut cur = name.clone();
        loop {
            if path.iter().any(|p| p == &cur) {
                path.push(cur);
                return Err(Diagnostic::new(
                    None,
                    *span,
                    format!(
                        "cyclic inheritance involving `{name}` ({})",
                        path.join(" → ")
                    ),
                )
                .into());
            }
            path.push(cur.clone());
            let Some((_, Some(next))) = by_name.get(&cur) else {
                break;
            };
            cur = next.clone();
        }
    }
    Ok(())
}

fn collect_objects(module: &Module, out: &mut HashMap<String, (Span, Option<String>)>) {
    for fun in &module.functions {
        if fun.is_object() {
            out.entry(fun.name.clone())
                .or_insert((fun.span, fun.base.clone()));
        }
    }
    for lib in module.import_modules.values() {
        collect_objects(lib, out);
    }
}

/// Walk the base chain starting at `type_name` (inclusive).
pub fn inheritance_chain<'a>(module: &'a Module, type_name: &str) -> Vec<&'a Function> {
    let mut out = Vec::new();
    let mut cur = type_name.to_string();
    let mut seen = HashSet::new();
    while seen.insert(cur.clone()) {
        let Some(obj) = find_object(module, &cur) else {
            break;
        };
        out.push(obj);
        match &obj.base {
            Some(b) => cur = b.clone(),
            None => break,
        }
    }
    out
}

fn find_object<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    module
        .functions
        .iter()
        .find(|f| f.name == name && f.is_object())
        .or_else(|| {
            module.import_modules.values().find_map(|lib| {
                lib.functions
                    .iter()
                    .find(|f| f.name == name && f.is_object())
            })
        })
}
