//! Shared call binding for execution (G02 BoundCall).
//!
//! Tree and Bytecode must agree: explicit import paths and same-module bare
//! names bind; bare names never silently pick an imported symbol; true
//! dynamics are marked separately.

use crate::ast::{CallExpr, Function, Module};
use crate::diagnostics::Span;
use crate::host::HostFn;

/// Why a call site is not a static BoundCall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicReason {
    MethodReceiver,
    CallFn,
    Plugin,
}

impl DynamicReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MethodReceiver => "method_receiver",
            Self::CallFn => "call_fn",
            Self::Plugin => "plugin",
        }
    }
}

/// Runtime resolution shared by Tree (and mirrored by Bytecode rules).
#[derive(Debug)]
pub enum ExecTarget<'a> {
    /// Function in the calling module (or nested children).
    Local(&'a Function),
    /// `bind.member…` through `module.import_modules`.
    Import {
        bind: String,
        lib: &'a Module,
        fun: &'a Function,
        path: Vec<String>,
    },
    Builtin(&'static str),
    Host(HostFn),
    /// Plugin / method / call_fn — not a static BoundCall.
    Dynamic {
        reason: DynamicReason,
        name: String,
    },
    Unresolved {
        message: String,
        #[allow(dead_code)]
        span_hint: Option<Span>,
    },
}

pub fn call_display(call: &CallExpr) -> String {
    if let Some(path) = &call.path {
        return path.join(".");
    }
    if let Some(recv) = &call.receiver {
        return format!("{recv}.{}", call.callee);
    }
    call.callee.clone()
}

fn find_in_children<'a>(fun: &'a Function, name: &str) -> Option<&'a Function> {
    for child in &fun.children {
        if child.name == name {
            return Some(child);
        }
        if let Some(f) = find_in_children(child, name) {
            return Some(f);
        }
    }
    None
}

/// Same-module bare-name lookup (never searches imports).
pub fn lookup_local<'a>(
    module: &'a Module,
    current: &'a Function,
    name: &str,
) -> Option<&'a Function> {
    if let Some(f) = find_in_children(current, name) {
        return Some(f);
    }
    module.functions.iter().find(|f| f.name == name)
}

fn walk_path<'a>(lib: &'a Module, path: &[String]) -> Result<&'a Function, String> {
    let display = path.join(".");
    if path.is_empty() {
        return Err("empty knowledge path".into());
    }
    let mut parent_is_object = false;
    let mut node = lib
        .functions
        .iter()
        .find(|f| f.name == path[0])
        .ok_or_else(|| format!("unknown `{display}`"))?;
    for seg in &path[1..] {
        parent_is_object = node.is_object();
        node = node
            .children
            .iter()
            .find(|c| c.name == *seg)
            .ok_or_else(|| format!("unknown `{display}`"))?;
    }
    if parent_is_object && node.level != 1 && crate::ast::function_uses_receiver(node) {
        return Err(format!(
            "`{display}` is an instance method; call it as `var`.{}",
            node.name
        ));
    }
    Ok(node)
}

fn resolve_import_path<'a>(module: &'a Module, path: &[String]) -> ExecTarget<'a> {
    let display = path.join(".");
    if path.len() < 2 {
        return ExecTarget::Unresolved {
            message: format!("library path `{display}`: need at least `lib.member`"),
            span_hint: None,
        };
    }
    let bind = &path[0];
    let Some(lib) = module.import_modules.get(bind) else {
        return ExecTarget::Unresolved {
            message: format!("unknown library `{bind}` (import it in frontmatter)"),
            span_hint: None,
        };
    };
    match walk_path(lib, &path[1..]) {
        Ok(fun) => ExecTarget::Import {
            bind: bind.clone(),
            lib,
            fun,
            path: path.to_vec(),
        },
        Err(message) => ExecTarget::Unresolved {
            message,
            span_hint: None,
        },
    }
}

/// Resolve a call for execution. Import bare-name leakage is forbidden (G02).
pub fn resolve_for_exec<'a>(
    module: &'a Module,
    current: &'a Function,
    call: &CallExpr,
) -> ExecTarget<'a> {
    if call.receiver.is_some() {
        return ExecTarget::Dynamic {
            reason: DynamicReason::MethodReceiver,
            name: call_display(call),
        };
    }

    if let Some(path) = &call.path {
        if path.len() == 2 && module.import_modules.contains_key(&path[0]) {
            return resolve_import_path(module, path);
        }
        if path.len() == 2 {
            // Possible local-variable method; caller checks env.
            return ExecTarget::Dynamic {
                reason: DynamicReason::MethodReceiver,
                name: call_display(call),
            };
        }
        return resolve_import_path(module, path);
    }

    if let Some(u) = module.uses.iter().find(|u| u.bind == call.callee) {
        return resolve_import_path(module, &u.path);
    }

    match call.callee.as_str() {
        "print" | "input" | "len" | "str" | "int" | "type" | "trim" | "split" | "join" | "at" => {
            return ExecTarget::Builtin(match call.callee.as_str() {
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
            });
        }
        "call_fn" => {
            return ExecTarget::Dynamic {
                reason: DynamicReason::CallFn,
                name: "call_fn".into(),
            };
        }
        other if HostFn::from_name(other).is_some() => {
            return ExecTarget::Host(HostFn::from_name(other).unwrap());
        }
        _ => {}
    }

    if let Some(fun) = lookup_local(module, current, &call.callee) {
        return ExecTarget::Local(fun);
    }

    ExecTarget::Dynamic {
        reason: DynamicReason::Plugin,
        name: call.callee.clone(),
    }
}

pub fn is_static_bound(target: &ExecTarget<'_>) -> bool {
    matches!(
        target,
        ExecTarget::Local(_)
            | ExecTarget::Import { .. }
            | ExecTarget::Builtin(_)
            | ExecTarget::Host(_)
    )
}
