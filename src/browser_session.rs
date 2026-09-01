//! In-memory browser / WASM session: boot `# main`, then call handlers with shared entry env.

use anyhow::{bail, Result};

use crate::ast::Module;
use crate::host::json::{json_to_value, value_to_json};
use crate::host::{HostCaps, HostContext};
use crate::interp::Interpreter;
use crate::load::load_module_from_source;
use crate::value::Value;

/// Persistent Marqdo program for the browser host bridge.
pub struct BrowserSession {
    module: Module,
    interp: Interpreter,
}

impl BrowserSession {
    /// Parse source, run `# main`, keep entry bindings for later [`Self::call`].
    pub fn boot(source: &str) -> Result<(Self, Value)> {
        if source.trim().is_empty() {
            bail!("source is empty");
        }
        let module = load_module_from_source(source)?;
        let caps = HostCaps {
            fs_write: false,
            exec: false,
            net: false,
            plugin: false,
        };
        let mut host = HostContext::for_capture(None, caps);
        host.set_entry_source(None, source);
        host.sleep_limit_ms = Some(0);
        let mut interp = Interpreter::with_capture(None, false).with_host(host);
        let value = interp.run_module(&module)?;
        Ok((Self { module, interp }, value))
    }

    /// Invoke `name` with optional JSON object of named args; updates entry bindings.
    pub fn call(&mut self, name: &str, args_json: &str) -> Result<Value> {
        let args = parse_named_args(args_json)?;
        self.interp.captured_stdout.clear();
        self.interp.invoke_in_entry(&self.module, name, &args)
    }

    pub fn take_stdout(&mut self) -> String {
        std::mem::take(&mut self.interp.captured_stdout)
    }
}

fn parse_named_args(args_json: &str) -> Result<Vec<(String, Value)>> {
    let t = args_json.trim();
    if t.is_empty() || t == "{}" || t == "null" {
        return Ok(Vec::new());
    }
    let v: serde_json::Value =
        serde_json::from_str(t).map_err(|e| anyhow::anyhow!("call args json: {e}"))?;
    match v {
        serde_json::Value::Object(map) => {
            let mut out = Vec::new();
            for (k, j) in map {
                out.push((k, json_to_value(&j).map_err(|e| anyhow::anyhow!(e))?));
            }
            Ok(out)
        }
        serde_json::Value::Null => Ok(Vec::new()),
        _ => bail!("call args must be a JSON object"),
    }
}

/// Serialize a runtime [`Value`] to JSON (for WASM bridge responses).
pub fn value_as_json(v: &Value) -> Result<serde_json::Value, String> {
    value_to_json(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_and_call_share_count() {
        let src = r##"---
import json:lib/json.mq.md
---

# main

*`count` = 0*

`wire` =

| @ | 选择器 | 事件 | 调用 |
|---|--------|------|------|
| 1 | "#btn" | click | bump |

**`wire`**

## bump
*`count` = count + 1*
*`label` = > str count*
*`patch` = > json.set map=None key="#out" value=label*
**> json.set map=None key="set_text" value=patch**
"##;
        let (mut sess, wire) = BrowserSession::boot(src).expect("boot");
        assert!(matches!(wire, Value::List(_)), "wire={wire:?}");
        let _v1 = sess.call("bump", "{}").expect("call1");
        let v2 = sess.call("bump", "{}").expect("call2");
        let j2 = value_as_json(&v2).unwrap();
        assert_eq!(j2["set_text"]["#out"], "2");
    }

    #[test]
    fn fetch_mq_load_returns_fetch_effect() {
        let src = std::fs::read_to_string("examples/browser-hello/fetch.mq.md")
            .expect("fetch.mq.md");
        let (mut sess, wire) = BrowserSession::boot(&src).expect("boot");
        assert!(matches!(wire, Value::List(_)));
        let v = sess.call("load", "{}").expect("load");
        let j = value_as_json(&v).unwrap();
        assert_eq!(j["set_text"]["#status"], "loading…");
        assert_eq!(j["fetch"]["then"], "on_uuid");
        assert!(j["fetch"]["url"].as_str().unwrap().contains("httpbin"));
        let done = sess
            .call(
                "on_uuid",
                r#"{"ok":true,"status":200,"body":"{\"uuid\":\"x\"}"}"#,
            )
            .expect("on_uuid");
        let jd = value_as_json(&done).unwrap();
        assert!(jd["set_text"]["#status"].as_str().unwrap().contains("uuid"));
    }
}
