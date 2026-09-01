//! Stub plugin surface when `plugin-host` is disabled (browser wasm).

use std::collections::HashMap;

use crate::host::HostContext;
use crate::value::Value;

pub const ABI_VERSION: u32 = 2;
pub const ABI_VERSION_MIN: u32 = 1;

/// Hook: resolve bare `lib.member` (unused without native plugins).
pub struct LibPathCall {
    pub call: fn(*mut (), &str) -> Result<Value, String>,
    pub data: *mut (),
}

pub fn with_lib_path_call<R>(_hook: LibPathCall, body: impl FnOnce() -> R) -> R {
    body()
}

#[derive(Clone)]
pub struct RegisteredFn {
    pub params: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct PluginState;

impl PluginState {
    pub fn get(&self, _name: &str) -> Option<&RegisteredFn> {
        None
    }

    pub fn list_names(&self) -> Vec<String> {
        Vec::new()
    }
}

pub fn load(_ctx: &mut HostContext, _path: &Value) -> Result<Value, String> {
    Err("plugin.load unavailable in browser wasm".into())
}

pub fn unload(_ctx: &mut HostContext) -> Result<Value, String> {
    Err("plugin.unload unavailable in browser wasm".into())
}

pub fn list(_ctx: &HostContext) -> Result<Value, String> {
    Ok(Value::List(Vec::new()))
}

pub fn call_registered(
    _ctx: &mut HostContext,
    name: &str,
    _bound: &HashMap<String, Value>,
) -> Result<Value, String> {
    Err(format!("plugin `{name}` unavailable in browser wasm"))
}
