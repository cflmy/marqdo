//! Object inheritance: `# Child = > Parent` validation and helpers.

use std::collections::HashSet;

use anyhow::Result;

use crate::ast::{Function, Module};
use crate::diagnostics::Diagnostic;

/// Import binding paths identify module instances, including repeated imports
/// of the same file under different bindings. The tree remains immutable.
pub(crate) fn module_namespace(root:&Module, wanted:&Module) -> Option<Vec<String>> {
    if std::ptr::eq(root,wanted) { return Some(Vec::new()); }
    let mut imports=root.import_modules.iter().collect::<Vec<_>>();
    imports.sort_by_key(|(name,_)| *name);
    for (bind,module) in imports {
        if let Some(mut path)=module_namespace(module,wanted) {
            path.insert(0,bind.clone()); return Some(path);
        }
    }
    None
}

pub(crate) fn object_at_namespace<'a>(root:&'a Module,path:&[String],name:&str) -> Option<(&'a Module,&'a Function)> {
    let mut owner=root;
    for segment in path { owner=owner.import_modules.get(segment)?; }
    let object=owner.functions.iter().find(|fun|fun.is_object() && fun.name==name)?;
    Some((owner,object))
}

/// Bare types prefer their defining module. An imported short-name binding
/// resolves its explicit path; a legacy imported bare name must be unique.
pub(crate) fn resolve_object<'a>(owner:&'a Module,name:&str) -> Result<(&'a Module,&'a Function),String> {
    if let Some(alias)=owner.uses.iter().find(|binding|binding.bind==name) {
        let Some((class,path))=alias.path.split_last() else { return Err("empty object alias".into()); };
        return object_at_namespace(owner,path,class).ok_or_else(||format!("unknown object type `{name}`"));
    }
    if let Some(object)=owner.functions.iter().find(|fun|fun.is_object() && fun.name==name) { return Ok((owner,object)); }
    fn imported<'a>(module:&'a Module,name:&str,out:&mut Vec<(&'a Module,&'a Function)>) {
        let mut imports=module.import_modules.iter().collect::<Vec<_>>();imports.sort_by_key(|(name,_)|*name);
        for (_,child) in imports {
            out.extend(child.functions.iter().filter(|fun|fun.is_object() && fun.name==name).map(|fun|(child,fun)));
            imported(child,name,out);
        }
    }
    let mut candidates=Vec::new(); imported(owner,name,&mut candidates);
    match candidates.as_slice() {
        [only]=>Ok(*only),
        []=>Err(format!("unknown object type `{name}`")),
        _=>Err(format!("ambiguous object type `{name}`; use an explicit import binding")),
    }
}

/// Validate that every object's `base` resolves and that the inheritance graph is acyclic.
pub fn validate_inheritance(module: &Module) -> Result<()> {
    fn modules<'a>(module: &'a Module, out: &mut Vec<&'a Module>) {
        out.push(module);
        let mut imports=module.import_modules.iter().collect::<Vec<_>>();
        imports.sort_by_key(|(name,_)| *name);
        for (_, child) in imports { modules(child,out); }
    }
    let mut all=Vec::new(); modules(module,&mut all);
    for owner in all {
        for object in owner.functions.iter().filter(|fun|fun.is_object()) {
            let mut current_owner=owner;
            let mut current=object;
            let mut seen=HashSet::new();
            let mut chain=Vec::new();
            loop {
                chain.push(current.name.clone());
                if !seen.insert((current_owner as *const Module,current as *const Function)) {
                    return Err(Diagnostic::new(owner.source.path.as_deref(),object.span,
                        format!("cyclic inheritance involving `{}` ({})",object.name,chain.join(" → ")))
                        .with_source(&owner.source).into());
                }
                let Some(base)=&current.base else { break; };
                (current_owner,current)=resolve_object(current_owner,base).map_err(|message| {
                    Diagnostic::new(current_owner.source.path.as_deref(),current.span,
                        format!("{} base type `{base}` for object `{}`: {message}",if message.starts_with("ambiguous") { "ambiguous" } else { "unknown" },current.name))
                        .with_source(&current_owner.source)
                })?;
            }
        }
    }
    Ok(())
}

/// Walk the base chain starting at `type_name` (inclusive).
pub fn inheritance_chain<'a>(module: &'a Module, type_name: &str) -> Vec<&'a Function> {
    let mut out=Vec::new();
    let Ok((mut owner,mut object))=resolve_object(module,type_name) else { return out; };
    let mut seen=HashSet::new();
    while seen.insert((owner as *const Module,object as *const Function)) {
        out.push(object);
        let Some(base)=&object.base else { break; };
        let Ok(next)=resolve_object(owner,base) else { break; };
        (owner,object)=next;
    }
    out
}

