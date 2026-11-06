//! Source-grounded declaration identities and the flat view of name resolution.
use super::Program;
use crate::sema::resolution::{NameView, Target};
use std::collections::{HashMap, HashSet};

pub(crate) struct FlatView<'a>(pub &'a Program);
impl NameView for FlatView<'_> {
    type Function = usize;
    type Module = Vec<String>;
    fn roots(&self, m: &Self::Module) -> Vec<usize> {
        self.0
            .functions
            .iter()
            .enumerate()
            .filter(|(i, f)| &f.module_path == m && self.0.parents[*i].is_none())
            .map(|(i, _)| i)
            .collect()
    }
    fn children(&self, f: usize) -> Vec<usize> {
        self.0.children[f].clone()
    }
    fn parent(&self, _: &Self::Module, f: usize) -> Option<usize> {
        self.0.parents[f]
    }
    fn same(&self, a: usize, b: usize) -> bool {
        a == b
    }
    fn name(&self, f: usize) -> String {
        self.0.functions[f].source_ast.name.clone()
    }
    fn level(&self, f: usize) -> u8 {
        self.0.functions[f].source_ast.level
    }
    fn is_object(&self, f: usize) -> bool {
        self.level(f) == 1
    }
    fn uses_receiver(&self, f: usize) -> bool {
        self.0.functions[f].source_ast.receiver_aliases
    }
    fn import(&self, m: &Self::Module, name: &str) -> Option<Self::Module> {
        let owner = self.0.functions.iter().find(|f| &f.module_path == m)?;
        if !owner.source_ast.imports.iter().any(|n| n == name) {
            return None;
        }
        let mut path = m.clone();
        path.push(name.into());
        Some(path)
    }
    fn used_path(&self, m: &Self::Module, name: &str) -> Option<Vec<String>> {
        self.0
            .functions
            .iter()
            .find(|f| &f.module_path == m)?
            .source_ast
            .uses
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, p)| p.clone())
    }
}
pub(crate) fn resolve(
    program: &Program,
    caller: usize,
    call: &crate::ast::CallExpr,
) -> Target<usize, Vec<String>> {
    crate::sema::resolution::resolve(
        &FlatView(program),
        &program.functions[caller].module_path,
        caller,
        call,
    )
}
pub(crate) fn lookup(program: &Program, caller: usize, name: &str) -> Option<usize> {
    crate::sema::resolution::lookup(
        &FlatView(program),
        &program.functions[caller].module_path,
        caller,
        name,
    )
}
pub(crate) fn verify(program: &Program) -> Result<(), String> {
    let objects = program
        .functions
        .iter()
        .enumerate()
        .filter(|(id, f)| program.parents[*id].is_none() && f.source_ast.level == 1)
        .map(|(id, f)| (f.source_ast.name.clone(), id))
        .collect::<Vec<_>>();
    if program.objects != objects {
        return Err("object names differ from source declarations".into());
    }
    let methods = objects
        .iter()
        .flat_map(|(_, owner)| {
            program.children[*owner].iter().map(|child| {
                (
                    *owner,
                    program.functions[*child].source_ast.name.clone(),
                    *child,
                )
            })
        })
        .collect::<Vec<_>>();
    if program.methods != methods {
        return Err("method names differ from source declarations".into());
    }
    let mut identities = HashSet::new();
    let mut scopes = HashMap::new();
    let mut namespaces = HashMap::new();
    for f in &program.functions {
        let source = &f.source_ast;
        if source.declaration_path.is_empty()
            || !identities.insert((f.module_path.clone(), source.declaration_path.clone()))
        {
            return Err("duplicate or missing source declaration identity".into());
        }
        if scopes
            .insert(f.module_scope, f.module_path.clone())
            .is_some_and(|p| p != f.module_path)
            || namespaces
                .insert(f.module_path.clone(), f.module_scope)
                .is_some_and(|s| s != f.module_scope)
            || (f.module_scope == 0) != f.module_path.is_empty()
        {
            return Err("source module scope mapping differs".into());
        }
        let owner = program
            .functions
            .iter()
            .find(|g| g.module_path == f.module_path)
            .unwrap();
        if source.module_root_count != owner.source_ast.module_root_count
            || source.imports != owner.source_ast.imports
            || source.uses != owner.source_ast.uses
        {
            return Err("inconsistent source module header".into());
        }
    }
    for (id, f) in program.functions.iter().enumerate() {
        let source = &f.source_ast;
        let parent_path = &source.declaration_path[..source.declaration_path.len() - 1];
        match program.parents[id] {
            Some(parent)
                if program.functions[parent].source_ast.declaration_path == parent_path =>
            {
                ()
            }
            None if parent_path.is_empty() => (),
            _ => return Err("flat parent differs from source declaration".into()),
        }
        if program.children[id].len() != source.child_count {
            return Err("source child inventory differs".into());
        }
        for (ordinal, child) in program.children[id].iter().enumerate() {
            let mut expected = source.declaration_path.clone();
            expected.push(ordinal);
            if program.functions[*child].source_ast.declaration_path != expected {
                return Err("source child order differs".into());
            }
        }
        let roots = FlatView(program).roots(&f.module_path);
        if roots.len() != source.module_root_count
            || roots
                .iter()
                .enumerate()
                .any(|(i, r)| program.functions[*r].source_ast.declaration_path != [i])
        {
            return Err("source root inventory/order differs".into());
        }
    }
    Ok(())
}
