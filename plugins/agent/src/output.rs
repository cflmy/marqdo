//! AgentOutputV1 + ResponsePolicy (M-A4).

use serde_json::{json, Value};

fn opt_text<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Normalize / build an AgentOutputV1 document.
pub fn output_build(args: &Value) -> Result<Value, String> {
    let status = opt_text(args, "status").unwrap_or("succeeded");
    if !matches!(
        status,
        "succeeded" | "partial" | "failed" | "needs_input" | "cancelled"
    ) {
        return Err(format!("invalid status={status}"));
    }
    let answer = args.get("answer").cloned().unwrap_or_else(|| {
        json!({
            "format": opt_text(args, "format").unwrap_or("markdown"),
            "text": opt_text(args, "text").unwrap_or(""),
        })
    });
    let validation = args.get("validation").cloned().unwrap_or_else(|| {
        json!({
            "status": "not_run",
            "report_ref": Value::Null
        })
    });
    let vstatus = validation
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("not_run");
    if !matches!(
        vstatus,
        "passed" | "failed" | "not_applicable" | "not_run"
    ) {
        return Err(format!("invalid validation.status={vstatus}"));
    }
    // Never claim passed without an explicit report when status says passed.
    if vstatus == "passed" && validation.get("report_ref").map(|v| v.is_null()).unwrap_or(true) {
        if args.get("allow_passed_without_report").and_then(|v| v.as_bool()) != Some(true) {
            return Err("validation.status=passed requires report_ref".into());
        }
    }

    let finish = opt_text(args, "finish_reason").unwrap_or(match status {
        "succeeded" => "completed",
        "partial" => "length_limited",
        "cancelled" => "cancelled",
        "needs_input" => "needs_input",
        _ => "error",
    });

    Ok(json!({
        "schema": "AgentOutputV1",
        "schema_version": 1,
        "status": status,
        "answer": answer,
        "artifacts": args.get("artifacts").cloned().unwrap_or_else(|| json!([])),
        "citations": args.get("citations").cloned().unwrap_or_else(|| json!([])),
        "validation": validation,
        "finish_reason": finish,
        "recovery_options": args.get("recovery_options").cloned().unwrap_or_else(|| json!([])),
        "channels": args.get("channels").cloned().unwrap_or_else(|| json!({
            "answer": true,
            "artifact": true,
            "progress": true,
            "diagnostic": true
        })),
    }))
}

/// Apply ResponsePolicy budgets to an answer text → possibly partial output.
pub fn output_apply_policy(args: &Value) -> Result<Value, String> {
    let mut out = if args.get("schema").and_then(|v| v.as_str()) == Some("AgentOutputV1") {
        args.clone()
    } else {
        output_build(args)?
    };
    let policy = args.get("policy").cloned().unwrap_or(json!({}));
    let detail = policy
        .get("detail")
        .and_then(|v| v.as_str())
        .unwrap_or("standard");
    let max_chars = match policy.get("max_chars").and_then(|v| v.as_u64()) {
        Some(n) if n > 0 => n as usize,
        _ => match detail {
            "concise" => 250,
            "detailed" => 8_000,
            _ => 700,
        },
    };
    let max_tokens = policy
        .get("max_output_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let artifact_allowed = policy
        .get("artifact_allowed")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let text = out
        .pointer("/answer/text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    // Soft char budget (CJK-aware enough: count Unicode scalars).
    let chars: Vec<char> = text.chars().collect();
    let mut finish = out
        .get("finish_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("completed")
        .to_string();
    let mut status = out
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("succeeded")
        .to_string();
    let mut answer_text = text.clone();
    if chars.len() > max_chars {
        answer_text = chars.into_iter().take(max_chars).collect();
        // Prefer not to cut mid-code fence if possible — soft trim at last newline.
        if let Some(pos) = answer_text.rfind('\n') {
            if pos > max_chars / 2 {
                answer_text.truncate(pos);
            }
        }
        finish = "length_limited".into();
        if status == "succeeded" {
            status = "partial".into();
        }
    }
    // Token budget is advisory here (chars/4 approx) when max_output_tokens set.
    if max_tokens > 0 {
        let approx = answer_text.chars().count().div_ceil(2); // rough CJK token estimate
        if approx > max_tokens {
            let keep = max_tokens.saturating_mul(2);
            answer_text = answer_text.chars().take(keep).collect();
            finish = "length_limited".into();
            status = "partial".into();
        }
    }

    if let Some(obj) = out.as_object_mut() {
        obj.insert("status".into(), json!(status));
        obj.insert("finish_reason".into(), json!(finish));
        if let Some(ans) = obj.get_mut("answer").and_then(|v| v.as_object_mut()) {
            ans.insert("text".into(), json!(answer_text));
        }
        if !artifact_allowed {
            obj.insert("artifacts".into(), json!([]));
        }
        obj.insert(
            "response_policy".into(),
            json!({
                "detail": detail,
                "max_chars": max_chars,
                "max_output_tokens": max_tokens,
                "artifact_allowed": artifact_allowed,
            }),
        );
    }
    Ok(out)
}

/// Split a channel event. Only `answer` is meant for chat body append.
pub fn output_channel_event(args: &Value) -> Result<Value, String> {
    let channel = opt_text(args, "channel").ok_or_else(|| "missing channel".to_string())?;
    if !matches!(channel, "answer" | "artifact" | "progress" | "diagnostic") {
        return Err(format!("invalid channel={channel}"));
    }
    let delta = opt_text(args, "delta").unwrap_or("");
    Ok(json!({
        "schema": "AgentChannelEventV1",
        "schema_version": 1,
        "channel": channel,
        "delta": delta,
        "append_to_chat": channel == "answer",
        "resource_id": args.get("resource_id").cloned().unwrap_or(Value::Null),
        "meta": args.get("meta").cloned().unwrap_or(json!({})),
    }))
}

/// Project `# 结论` (or first non-empty paragraph) from a marqdo document for chat.
pub fn output_project_conclusion(args: &Value) -> Result<Value, String> {
    let doc = opt_text(args, "document")
        .or_else(|| opt_text(args, "text"))
        .unwrap_or("");
    let mut in_conclusion = false;
    let mut buf = String::new();
    for line in doc.lines() {
        let t = line.trim();
        if t == "# 结论" || t.eq_ignore_ascii_case("# conclusion") {
            in_conclusion = true;
            continue;
        }
        if in_conclusion {
            if t.starts_with("# ") {
                break;
            }
            if t.is_empty() {
                if !buf.is_empty() {
                    buf.push('\n');
                }
                continue;
            }
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(t);
        }
    }
    if buf.trim().is_empty() {
        // Fallback: first non-frontmatter paragraph.
        let mut past_fm = !doc.starts_with("---");
        let mut fm_done = past_fm;
        for line in doc.lines() {
            if !fm_done {
                if line.trim() == "---" {
                    if past_fm {
                        fm_done = true;
                    } else {
                        past_fm = true;
                    }
                }
                continue;
            }
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') || t.starts_with('|') {
                continue;
            }
            buf = t.to_string();
            break;
        }
    }
    Ok(json!({
        "ok": true,
        "text": buf.trim(),
        "channel": "answer",
        "source": "conclusion_projection",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_policy_limits() {
        let out = output_build(&json!({
            "status": "succeeded",
            "text": "短答",
            "validation": {"status": "not_applicable", "report_ref": null}
        }))
        .unwrap();
        assert_eq!(out["schema"], "AgentOutputV1");
        assert_eq!(out["answer"]["text"], "短答");

        let long: String = "字".repeat(400);
        let limited = output_apply_policy(&json!({
            "status": "succeeded",
            "text": long,
            "validation": {"status": "not_applicable"},
            "policy": {"detail": "concise", "max_chars": 100}
        }))
        .unwrap();
        assert_eq!(limited["status"], "partial");
        assert_eq!(limited["finish_reason"], "length_limited");
        assert!(limited["answer"]["text"].as_str().unwrap().chars().count() <= 100);
    }

    #[test]
    fn channel_answer_only_appends_chat() {
        let e = output_channel_event(&json!({"channel": "progress", "delta": "检索中"})).unwrap();
        assert_eq!(e["append_to_chat"], false);
        let a = output_channel_event(&json!({"channel": "answer", "delta": "hi"})).unwrap();
        assert_eq!(a["append_to_chat"], true);
    }

    #[test]
    fn project_conclusion() {
        let doc = "---\ntitle: t\n---\n\n# 任务\n问\n\n# 结论\n答案是 42。\n\n# main\nprint\n";
        let p = output_project_conclusion(&json!({"document": doc})).unwrap();
        assert!(p["text"].as_str().unwrap().contains("42"));
    }
}
