//! Fail-closed choice parsing for small-model routing.
//! This module only extracts a value; it never resolves or executes a SKILL.

use serde_json::{json, Value};

fn canonical(s: &str) -> String {
    s.trim()
        .trim_matches(|c: char| matches!(c, '"' | '\'' | '`' | '“' | '”' | '‘' | '’'))
        .trim()
        .to_lowercase()
}

fn strip_fence(s: &str) -> &str {
    let s = s.trim();
    if !s.starts_with("```") {
        return s;
    }
    let Some(start) = s.find('\n') else {
        return s;
    };
    let body = &s[start + 1..];
    let Some(end) = body.find("```") else {
        return s;
    };
    if !body[end + 3..].trim().is_empty() {
        return s;
    }
    body[..end].trim()
}

fn strip_label(s: &str) -> &str {
    let s = s.trim();
    let Some((head, tail)) = s.split_once([':', '：']) else {
        return s;
    };
    if matches!(
        head.trim().to_lowercase().as_str(),
        "choice" | "option" | "selected" | "选择" | "选项" | "答案"
    ) {
        tail.trim()
    } else {
        s
    }
}

fn single_choice(options: &[String], reply: &str) -> Option<String> {
    let raw = strip_label(strip_fence(reply));
    let value = canonical(raw);
    // Never infer a routing decision from arbitrary prose.
    let matches: Vec<&String> = options
        .iter()
        .filter(|opt| canonical(opt) == value)
        .collect();
    if matches.len() == 1 {
        return Some(matches[0].clone());
    }
    // A bare number denotes a 1-based index into the allowed list.
    if let Ok(index) = value.parse::<usize>() {
        if index > 0 && index <= options.len() {
            return Some(options[index - 1].clone());
        }
    }
    None
}

/// Parse one permitted option from a small model response.
/// Returns a status rather than treating arbitrary prose as a tool target.
pub fn parse_choice(args: &Value) -> Result<Value, String> {
    let reply = args
        .get("reply")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing text `reply`".to_string())?;
    let options = args
        .get("options")
        .and_then(Value::as_array)
        .ok_or_else(|| "options must be an array of strings".to_string())?;
    if options.is_empty() || options.len() > 64 {
        return Err("options must contain 1..=64 entries".to_string());
    }
    let mut allowed = Vec::with_capacity(options.len());
    for option in options {
        let name = option
            .as_str()
            .ok_or_else(|| "each option must be a string".to_string())?;
        if name.trim().is_empty() || name.len() > 128 {
            return Err("each option must contain 1..=128 UTF-8 bytes".to_string());
        }
        if allowed.iter().any(|p: &String| canonical(p) == canonical(name)) {
            return Err("options must be unique after normalization".to_string());
        }
        allowed.push(name.to_string());
    }
    if let Some(value) = single_choice(&allowed, reply) {
        Ok(json!({"ok": true, "value": value, "status": "accepted"}))
    } else {
        Ok(json!({
            "ok": false,
            "value": null,
            "status": "invalid",
            "error": {
                "code": "model.invalid_choice",
                "category": "invalid_output",
                "retryable": false,
                "recovery_action": "repair_output"
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(reply: &str) -> Value {
        parse_choice(&json!({"options": ["chat", "office"], "reply": reply})).unwrap()
    }

    #[test]
    fn accepts_exact_label_fence_and_number() {
        assert_eq!(check("chat")["value"], "chat");
        assert_eq!(check(" CHAT ")["value"], "chat");
        assert_eq!(check("选择：office")["value"], "office");
        assert_eq!(check("```text\nchat\n```")["value"], "chat");
        assert_eq!(check("2")["value"], "office");
    }

    #[test]
    fn refuses_prose_conflicts_and_out_of_range() {
        for reply in ["请选择 chat", "chat 或 office", "choice: chat，因为更合适", "3", ""] {
            let result = check(reply);
            assert_eq!(result["ok"], false, "accepted {reply:?}");
            assert_eq!(result["value"], Value::Null);
        }
    }

    #[test]
    fn rejects_invalid_options_before_parsing() {
        for options in [
            json!([]),
            json!(["chat", "CHAT"]),
            json!(["chat", 10]),
            json!([""]),
        ] {
            assert!(parse_choice(&json!({"options": options, "reply": "chat"})).is_err());
        }
    }

    #[test]
    fn never_extracts_arbitrary_skill_path() {
        assert_eq!(check("../../private.mq.md")["ok"], false);
    }
}
