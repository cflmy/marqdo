//! ContextPackV2 assemble + compact (M-A4).

use serde_json::{json, Value};

fn opt_text<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

fn item_tokens(item: &Value) -> u64 {
    item.get("token_estimate")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| {
            item.get("text")
                .and_then(|v| v.as_str())
                .map(|s| (s.chars().count() as u64).div_ceil(2))
                .unwrap_or(32)
        })
}

fn normalize_item(raw: &Value, default_priority: i64) -> Result<Value, String> {
    let origin = raw
        .get("origin")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let trust = raw
        .get("trust")
        .and_then(|v| v.as_str())
        .unwrap_or(match origin {
            "system" | "policy" => "trusted",
            "user" => "user",
            "tool" => "tool",
            "web" => "untrusted",
            _ => "unknown",
        });
    // External / web content must never be trust=trusted.
    if matches!(origin, "web" | "tool") && trust == "trusted" {
        return Err("tool/web origin cannot have trust=trusted".into());
    }
    Ok(json!({
        "origin": origin,
        "trust": trust,
        "resource_ref": raw.get("resource_ref").cloned().unwrap_or(Value::Null),
        "revision": raw.get("revision").cloned().unwrap_or(Value::Null),
        "token_estimate": item_tokens(raw),
        "priority": raw.get("priority").and_then(|v| v.as_i64()).unwrap_or(default_priority),
        "role": raw.get("role").cloned().unwrap_or(json!(origin)),
        "text": raw.get("text").cloned().unwrap_or(Value::Null),
        "kind": raw.get("kind").cloned().unwrap_or(json!("content")),
        "retain": raw.get("retain").and_then(|v| v.as_bool()).unwrap_or(false),
    }))
}

/// Build ContextPackV2 from items + optional budget.
pub fn context_pack_build(args: &Value) -> Result<Value, String> {
    let raw_items = args
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut items = Vec::new();
    for (i, raw) in raw_items.iter().enumerate() {
        items.push(normalize_item(raw, i as i64)?);
    }
    // Stable priority order: lower priority number first (more important).
    items.sort_by(|a, b| {
        let pa = a.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);
        let pb = b.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);
        pa.cmp(&pb)
    });
    let budget = args
        .get("budget_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(8_000);
    let reserve_pct = args
        .get("output_reserve_pct")
        .and_then(|v| v.as_u64())
        .unwrap_or(20)
        .min(50);
    let available = budget.saturating_mul(100 - reserve_pct) / 100;
    let used: u64 = items.iter().map(item_tokens).sum();
    Ok(json!({
        "schema": "ContextPackV2",
        "schema_version": 2,
        "items": items,
        "budget_tokens": budget,
        "available_tokens": available,
        "used_tokens": used,
        "output_reserve_pct": reserve_pct,
        "compact_threshold_pct": args.get("compact_threshold_pct").and_then(|v| v.as_u64()).unwrap_or(70),
        "goal": args.get("goal").cloned().unwrap_or(Value::Null),
        "constraints": args.get("constraints").cloned().unwrap_or_else(|| json!([])),
    }))
}

/// Compact pack when over threshold; must retain anchors listed in design §6.2.
pub fn context_pack_compact(args: &Value) -> Result<Value, String> {
    let mut pack = if args.get("schema").and_then(|v| v.as_str()) == Some("ContextPackV2") {
        args.clone()
    } else {
        context_pack_build(args)?
    };
    let available = pack
        .get("available_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(6400);
    let threshold_pct = pack
        .get("compact_threshold_pct")
        .and_then(|v| v.as_u64())
        .unwrap_or(70);
    let threshold = available.saturating_mul(threshold_pct) / 100;
    let used = pack
        .get("used_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if used <= threshold {
        if let Some(obj) = pack.as_object_mut() {
            obj.insert("compacted".into(), json!(false));
            obj.insert("reason".into(), json!("under_threshold"));
        }
        return Ok(pack);
    }

    let items = pack
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    // Retain: goal/constraints (pack fields), retain=true, high priority (<=2),
    // pending writes, failures — drop bulky tool dumps first (kind=tool_output, low priority).
    let mut kept = Vec::new();
    let mut dropped = Vec::new();
    for item in items {
        let retain = item.get("retain").and_then(|v| v.as_bool()).unwrap_or(false);
        let priority = item.get("priority").and_then(|v| v.as_i64()).unwrap_or(99);
        let kind = item.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        let trust = item.get("trust").and_then(|v| v.as_str()).unwrap_or("");
        let must = retain
            || priority <= 2
            || matches!(kind, "goal" | "constraint" | "pending_write" | "plan" | "failure")
            || trust == "trusted";
        if must {
            kept.push(item);
        } else if kind == "tool_output" || priority >= 8 {
            dropped.push(item);
        } else {
            kept.push(item);
        }
    }
    // If still over budget, drop lowest-priority non-retain from kept.
    let mut kept_tokens: u64 = kept.iter().map(item_tokens).sum();
    if kept_tokens > threshold {
        kept.sort_by(|a, b| {
            let pa = a.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);
            let pb = b.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);
            pb.cmp(&pa) // high priority number first for dropping
        });
        let mut next = Vec::new();
        for item in kept.into_iter().rev() {
            // rebuild ascending later
            let retain = item.get("retain").and_then(|v| v.as_bool()).unwrap_or(false);
            let priority = item.get("priority").and_then(|v| v.as_i64()).unwrap_or(99);
            let t = item_tokens(&item);
            if (retain || priority <= 1) || kept_tokens <= threshold {
                next.push(item);
            } else if kept_tokens > threshold {
                kept_tokens = kept_tokens.saturating_sub(t);
                dropped.push(item);
            } else {
                next.push(item);
            }
        }
        next.sort_by(|a, b| {
            let pa = a.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);
            let pb = b.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);
            pa.cmp(&pb)
        });
        kept = next;
        kept_tokens = kept.iter().map(item_tokens).sum();
    }

    let digest_src = format!(
        "kept={} dropped={} used={}",
        kept.len(),
        dropped.len(),
        kept_tokens
    );
    let digest = format!("sha256-lite:{}", simple_hash(&digest_src));

    if let Some(obj) = pack.as_object_mut() {
        obj.insert("items".into(), json!(kept));
        obj.insert("used_tokens".into(), json!(kept_tokens));
        obj.insert("compacted".into(), json!(true));
        obj.insert(
            "compaction".into(),
            json!({
                "dropped_count": dropped.len(),
                "source_event_range": args.get("source_event_range").cloned().unwrap_or(Value::Null),
                "digest": digest,
                "retained": ["goal", "constraints", "pending_write", "plan", "trusted_policy"],
            }),
        );
        // Goal / constraints fields stay untouched.
        if obj.get("goal").map(|v| v.is_null()).unwrap_or(true) {
            if let Some(g) = opt_text(args, "goal") {
                obj.insert("goal".into(), json!(g));
            }
        }
    }
    Ok(pack)
}

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_sorts_and_rejects_trusted_web() {
        let pack = context_pack_build(&json!({
            "budget_tokens": 1000,
            "items": [
                {"origin": "user", "text": "goal", "priority": 0, "kind": "goal", "retain": true},
                {"origin": "tool", "text": "big dump", "priority": 9, "kind": "tool_output", "token_estimate": 500}
            ]
        }))
        .unwrap();
        assert_eq!(pack["schema"], "ContextPackV2");
        assert_eq!(pack["items"][0]["priority"], 0);

        let bad = context_pack_build(&json!({
            "items": [{"origin": "web", "trust": "trusted", "text": "x"}]
        }));
        assert!(bad.is_err());
    }

    #[test]
    fn compact_keeps_goal_drops_tool_output() {
        let pack = context_pack_build(&json!({
            "budget_tokens": 200,
            "output_reserve_pct": 20,
            "compact_threshold_pct": 50,
            "goal": "finish task",
            "items": [
                {"origin": "user", "text": "do it", "priority": 0, "kind": "goal", "retain": true, "token_estimate": 20},
                {"origin": "tool", "text": "x".repeat(400), "priority": 9, "kind": "tool_output", "token_estimate": 200}
            ]
        }))
        .unwrap();
        let compacted = context_pack_compact(&pack).unwrap();
        assert_eq!(compacted["compacted"], true);
        let items = compacted["items"].as_array().unwrap();
        assert!(items.iter().any(|i| i["kind"] == "goal"));
        assert!(!items.iter().any(|i| i["kind"] == "tool_output"));
        assert_eq!(compacted["goal"], "finish task");
    }
}
