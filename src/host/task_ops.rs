//! Bounded orchestration over existing subtask resources.
use crate::{
    host::{HostContext, HostFn},
    value::Value,
};
use std::collections::HashMap;
#[cfg(feature = "exec-host")]
fn arg<'a>(b: &'a HashMap<String, Value>, k: &str) -> Result<&'a Value, String> {
    b.get(k).ok_or_else(|| format!("missing {k}"))
}
pub(super) fn call(
    ctx: &mut HostContext,
    id: HostFn,
    b: &HashMap<String, Value>,
) -> Result<Value, String> {
    #[cfg(feature = "exec-host")]
    {
        fn integer(b: &HashMap<String, Value>, key: &str) -> Result<u64, String> {
            match arg(b, key)? {
                Value::Int(n) if *n >= 0 => Ok(*n as u64),
                _ => Err(format!("{key} must be nonnegative integer")),
            }
        }
        match id {
            HostFn::SubtaskWaitTimeout => {
                let answer = super::subtask::wait_timeout(
                    ctx,
                    arg(b, "id")?,
                    integer(b, "timeout_ms")?,
                    matches!(b.get("cancel"), Some(Value::Bool(true))),
                );
                Ok(super::result::capture("subtask.wait_timeout", answer))
            }
            HostFn::SubtaskCancelAll => {
                let count = ctx.subtasks.len();
                super::subtask::kill_all(ctx);
                Ok(Value::Int(count as i64))
            }
            HostFn::SubtaskRunMany => super::subtask::run_many(
                ctx,
                arg(b, "tasks")?,
                integer(b, "limit")? as usize,
                integer(b, "timeout_ms")?,
            ),
            _ => Err("not a task operation".into()),
        }
    }
    #[cfg(not(feature = "exec-host"))]
    {
        let _ = (ctx, id, b);
        Err("subtask unavailable in this build".into())
    }
}
