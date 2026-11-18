//! Stub subtask surface when `exec-host` is disabled (browser wasm).

use crate::host::HostContext;

/// Placeholder handle type so `HostContext.subtasks` still type-checks.
#[derive(Debug)]
pub(crate) struct Handle;

pub fn kill_all(_ctx: &mut HostContext) {}
