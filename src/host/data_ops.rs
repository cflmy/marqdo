//! Recoverable parsing and timezone operations at the public data boundary.
use crate::{
    host::{HostContext, HostFn},
    value::Value,
};
use std::collections::HashMap;
fn arg<'a>(b: &'a HashMap<String, Value>, k: &str) -> Result<&'a Value, String> {
    b.get(k).ok_or_else(|| format!("missing {k}"))
}
pub(super) fn call(
    _ctx: &mut HostContext,
    id: HostFn,
    b: &HashMap<String, Value>,
) -> Result<Value, String> {
    match id {
        HostFn::TomlStringify => super::toml_ops::stringify(arg(b, "value")?),
        HostFn::TomlTryParse => Ok(super::result::capture(
            "toml.parse",
            super::toml_ops::parse(arg(b, "text")?).map_err(|e| format!("invalid_toml: {e}")),
        )),
        HostFn::DatetimeLocal => Ok(super::result::capture(
            "datetime.parse_local",
            super::datetime::local(arg(b, "text")?, arg(b, "zone")?, arg(b, "ambiguity")?),
        )),
        HostFn::DatetimeCalendarAdd => Ok(super::result::capture(
            "datetime.add_calendar",
            super::datetime::calendar_add(arg(b, "dt")?, arg(b, "days")?, arg(b, "ambiguity")?),
        )),
        _ => Err("not a data operation".into()),
    }
}
