//! Shared call binding for execution (G02 BoundCall).
//!
//! Tree and Bytecode must agree: explicit import paths and same-module bare
//! names bind; bare names never silently pick an imported symbol; true
//! dynamics are marked separately.

use crate::ast::{CallExpr, Function, Module};

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

pub type ExecTarget<'a> = super::resolution::Target<&'a Function, &'a Module>;

pub fn call_display(call: &CallExpr) -> String {
    if let Some(path) = &call.path {
        return path.join(".");
    }
    if let Some(recv) = &call.receiver {
        return format!("{recv}.{}", call.callee);
    }
    call.callee.clone()
}

/// Same-module bare names never search imports.
pub fn lookup_local<'a>(module: &'a Module, current: &'a Function, name: &str) -> Option<&'a Function> {
    super::resolution::lookup(&super::resolution::AstView::default(), &module, current, name)
}
pub fn resolve_for_exec<'a>(module: &'a Module, current: &'a Function, call: &CallExpr) -> ExecTarget<'a> {
    match call.for_evaluation() {
        Ok(normalized) => super::resolution::resolve(&super::resolution::AstView::default(), &module, current, &normalized),
        Err(message) => ExecTarget::Unresolved { message, span_hint: None },
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
