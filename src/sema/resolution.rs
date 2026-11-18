//! Ordered declaration lookup, independent of the declaration representation.
use super::bind::{call_display, DynamicReason};
use crate::{ast::CallExpr, diagnostics::Span, host::HostFn};

pub(crate) trait NameView {
    type Function: Copy;
    type Module: Clone;
    fn roots(&self, module: &Self::Module) -> Vec<Self::Function>;
    fn children(&self, function: Self::Function) -> Vec<Self::Function>;
    fn parent(&self, module: &Self::Module, function: Self::Function) -> Option<Self::Function>;
    fn same(&self, left: Self::Function, right: Self::Function) -> bool;
    fn name(&self, function: Self::Function) -> String;
    fn level(&self, function: Self::Function) -> u8;
    fn is_object(&self, function: Self::Function) -> bool;
    fn uses_receiver(&self, function: Self::Function) -> bool;
    fn import(&self, module: &Self::Module, name: &str) -> Option<Self::Module>;
    fn used_path(&self, module: &Self::Module, name: &str) -> Option<Vec<String>>;
}
#[derive(Debug)]
pub enum Target<F, M> {
    Local(F),
    Import {
        bind: String,
        lib: M,
        fun: F,
        path: Vec<String>,
    },
    Builtin(&'static str),
    Host(HostFn),
    Dynamic {
        reason: DynamicReason,
        name: String,
    },
    Unresolved {
        message: String,
        span_hint: Option<Span>,
    },
}
fn ancestors<V: NameView>(
    view: &V,
    nodes: Vec<V::Function>,
    current: V::Function,
    path: &mut Vec<V::Function>,
) -> bool {
    for node in nodes {
        if view.same(node, current) {
            return true;
        }
        path.push(node);
        if ancestors(view, view.children(node), current, path) {
            return true;
        }
        path.pop();
    }
    false
}
pub(crate) fn lookup<V: NameView>(
    view: &V,
    module: &V::Module,
    current: V::Function,
    name: &str,
) -> Option<V::Function> {
    let mut cursor = Some(current);
    while let Some(node) = cursor {
        if let Some(f) = view
            .children(node)
            .into_iter()
            .find(|f| view.name(*f) == name)
        {
            return Some(f);
        }
        cursor = view.parent(module, node);
    }
    view.roots(module)
        .into_iter()
        .find(|f| view.name(*f) == name)
}
fn walk<V: NameView>(view: &V, module: &V::Module, path: &[String]) -> Result<V::Function, String> {
    let display = path.join(".");
    let first = path.first().ok_or("empty knowledge path")?;
    let mut node = view
        .roots(module)
        .into_iter()
        .find(|f| view.name(*f) == *first)
        .ok_or_else(|| format!("unknown `{display}`"))?;
    let mut parent_is_object = false;
    for segment in &path[1..] {
        parent_is_object = view.is_object(node);
        node = view
            .children(node)
            .into_iter()
            .find(|f| view.name(*f) == *segment)
            .ok_or_else(|| format!("unknown `{display}`"))?;
    }
    if parent_is_object && view.level(node) != 1 && view.uses_receiver(node) {
        return Err(format!(
            "`{display}` is an instance method; call it as `var`.{}",
            view.name(node)
        ));
    }
    Ok(node)
}
fn imported<V: NameView>(
    view: &V,
    module: &V::Module,
    path: &[String],
) -> Target<V::Function, V::Module> {
    let unresolved = |message| Target::Unresolved {
        message,
        span_hint: None,
    };
    if path.len() < 2 {
        return unresolved(format!(
            "library path `{}`: need at least `lib.member`",
            path.join(".")
        ));
    }
    let bind = &path[0];
    let Some(lib) = view.import(module, bind) else {
        return unresolved(format!(
            "unknown library `{bind}` (import it in frontmatter)"
        ));
    };
    match walk(view, &lib, &path[1..]) {
        Ok(fun) => Target::Import {
            bind: bind.clone(),
            lib,
            fun,
            path: path.to_vec(),
        },
        Err(message) => unresolved(message),
    }
}
pub(crate) fn resolve<V: NameView>(
    view: &V,
    module: &V::Module,
    current: V::Function,
    call: &CallExpr,
) -> Target<V::Function, V::Module> {
    if call.receiver.is_some() {
        return Target::Dynamic {
            reason: DynamicReason::MethodReceiver,
            name: call_display(call),
        };
    }
    if let Some(path) = &call.path {
        if path
            .first()
            .is_some_and(|p| view.import(module, p).is_some())
        {
            return imported(view, module, path);
        }
        if path.len() >= 2 {
            return match walk(view, module, path) {
                Ok(fun) => Target::Local(fun),
                Err(_) if path.len() == 2 => Target::Dynamic {
                    reason: DynamicReason::MethodReceiver,
                    name: call_display(call),
                },
                Err(message) => Target::Unresolved {
                    message,
                    span_hint: None,
                },
            };
        }
        return Target::Unresolved {
            message: format!(
                "library path `{}`: need at least `lib.member`",
                path.join(".")
            ),
            span_hint: None,
        };
    }
    if let Some(path) = view.used_path(module, &call.callee) {
        return imported(view, module, &path);
    }
    if crate::arguments::builtin_signature(&call.callee).is_some() {
        if call.callee == "call_fn" {
            return Target::Dynamic {
                reason: DynamicReason::CallFn,
                name: "call_fn".into(),
            };
        }
        let name = match call.callee.as_str() {
            "print" => "print",
            "input" => "input",
            "len" => "len",
            "str" => "str",
            "int" => "int",
            "type" => "type",
            "trim" => "trim",
            "split" => "split",
            "join" => "join",
            "at" => "at",
            _ => unreachable!(),
        };
        return Target::Builtin(name);
    }
    if let Some(host) = HostFn::from_name(&call.callee) {
        return Target::Host(host);
    }
    if let Some(fun) = lookup(view, module, current, &call.callee) {
        return Target::Local(fun);
    }
    Target::Dynamic {
        reason: DynamicReason::Plugin,
        name: call.callee.clone(),
    }
}

#[derive(Default)]
pub(crate) struct AstView<'a>(std::marker::PhantomData<&'a crate::ast::Module>);
impl<'a> NameView for AstView<'a> {
    type Function = &'a crate::ast::Function;
    type Module = &'a crate::ast::Module;
    fn roots(&self, m: &Self::Module) -> Vec<Self::Function> {
        m.functions.iter().collect()
    }
    fn children(&self, f: Self::Function) -> Vec<Self::Function> {
        f.children.iter().collect()
    }
    fn parent(&self, m: &Self::Module, f: Self::Function) -> Option<Self::Function> {
        let mut path = Vec::new();
        if ancestors(self, self.roots(m), f, &mut path) {
            path.last().copied()
        } else {
            None
        }
    }
    fn same(&self, a: Self::Function, b: Self::Function) -> bool {
        std::ptr::eq(a, b)
    }
    fn name(&self, f: Self::Function) -> String {
        f.name.clone()
    }
    fn level(&self, f: Self::Function) -> u8 {
        f.level
    }
    fn is_object(&self, f: Self::Function) -> bool {
        f.is_object()
    }
    fn uses_receiver(&self, f: Self::Function) -> bool {
        crate::ast::function_uses_receiver(f)
    }
    fn import(&self, m: &Self::Module, name: &str) -> Option<Self::Module> {
        m.import_modules.get(name)
    }
    fn used_path(&self, m: &Self::Module, name: &str) -> Option<Vec<String>> {
        m.uses
            .iter()
            .find(|u| u.bind == name)
            .map(|u| u.path.clone())
    }
}
