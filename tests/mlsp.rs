//! MLSP for AI（T3.1/T3.2 + T2.6）：`locate` / `syntax` / `validate` / `repair_targets` / `schema`。
//!
//! 验收对照 `doc/roadmap/three-problems-plan.md` Phase 3：
//! AI 不必背语法细节（syntax/locate 可查）、schema 可导出、修复靶点有界（越界必拒）。

use marqdo::mlsp::handle_line;

fn call(req: &str) -> serde_json::Value {
    serde_json::from_str(&handle_line(req)).expect("MLSP 响应必须是合法 JSON")
}

#[test]
fn syntax_query_returns_construct_cards() {
    let resp = call(r#"{"id":1,"method":"syntax","params":{"query":"返回"}}"#);
    assert_eq!(resp["ok"], true, "{resp}");
    let ids: Vec<&str> = resp["result"]["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"italic-return"), "「返回」应命中斜体返回卡片: {resp}");
    assert!(resp["result"]["cards"][0]["manual"]
        .as_str()
        .unwrap()
        .contains("core-surface"));
}

#[test]
fn syntax_unknown_query_structured_error() {
    let resp = call(r#"{"id":2,"method":"syntax","params":{"query":"量子纠缠语法"}}"#);
    assert_eq!(resp["ok"], false);
    assert_eq!(resp["error"]["code"], "mlsp.no_match");
}

#[test]
fn syntax_cards_carry_rule_text_self_sufficient() {
    // 「AI 不必学语法」的前提：卡片自带语义/戒律/样例原文（不必回读文档），
    // 且空查询列出全部核心构造（three-problems.md §3.2「单构造戒律」）。
    let resp = call(r#"{"id":9,"method":"syntax","params":{}}"#);
    assert_eq!(resp["ok"], true, "{resp}");
    let cards = resp["result"]["cards"].as_array().unwrap();
    assert_eq!(
        cards.len(),
        marqdo::parse::CORE_CONSTRUCTS.len(),
        "空查询应列出全部核心构造: {resp}"
    );
    for c in cards {
        assert!(
            c["rule"].as_str().unwrap_or("").len() > 2,
            "卡片缺戒律原文（不可自持）: {c}"
        );
        assert!(
            c["doc_quote"].as_str().unwrap_or("").starts_with('|'),
            "卡片缺行原文: {c}"
        );
        assert!(
            !c["semantics"].as_str().unwrap_or("").is_empty(),
            "卡片缺语义: {c}"
        );
    }
}

#[test]
fn locate_symbol_returns_span_and_contract() {
    let src = "# main\n\n## 加一\n\n对于输入变量`n`,执行加一。\n\n| 参数 | 类型 | 说明 |\n|------|------|------|\n| n | number | 输入值 |\n\n*n+1*\n";
    let req = serde_json::json!({"id":3,"method":"locate","params":{"source": src, "symbol": "加一"}});
    let resp = call(&req.to_string());
    assert_eq!(resp["ok"], true, "{resp}");
    assert_eq!(resp["result"]["unit"], "加一");
    assert_eq!(resp["result"]["contract"]["params"][0]["type"], "number");
    assert!(resp["result"]["span"]["line"].as_u64().unwrap() >= 1);
}

#[test]
fn validate_reports_contract_drift() {
    let src = "# main\n\n## 加一\n\n对于输入变量`n`,执行加一。\n\n| 参数 | 类型 | 说明 |\n|------|------|------|\n| nn | number | 输入值 |\n\n*n+1*\n";
    let req = serde_json::json!({"id":4,"method":"validate","params":{"source": src}});
    let resp = call(&req.to_string());
    assert_eq!(resp["ok"], true, "{resp}");
    assert_eq!(resp["result"]["ok"], false, "漂移必须让 validate 不通过: {resp}");
    let codes: Vec<&str> = resp["result"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"contract.param_drift"), "{resp}");
}

#[test]
fn schema_exports_progressive_contract() {
    let req = serde_json::json!({
        "id": 5,
        "method": "schema",
        "params": {"path": concat!(env!("CARGO_MANIFEST_DIR"), "/tests/contracts/ok_contract.mq.md"), "unit": "加一"}
    });
    let resp = call(&req.to_string());
    assert_eq!(resp["ok"], true, "{resp}");
    assert_eq!(resp["result"]["contract"]["params"][0]["name"], "n");
    assert_eq!(resp["result"]["contract"]["params"][0]["type"], "number");
    assert_eq!(resp["result"]["contract"]["returns"]["type"], "number");
    assert!(resp["result"]["contract"]["doc_anchor"]
        .as_str()
        .unwrap()
        .contains("#L"));
}

#[test]
fn repair_targets_are_bounded() {
    let resp = call(
        r#"{"id":6,"method":"repair_targets","params":{"code":"contract.arg_mismatch","doc_anchor":"a.mq.md#L9-L15","suggestion":"把实参改为 number"}}"#,
    );
    assert_eq!(resp["ok"], true, "{resp}");
    let repair = &resp["result"]["repair"];
    assert_eq!(repair["strategy"], "anchored-local");
    assert_eq!(repair["bounded"], true);
    assert_eq!(repair["on_violation"], "abstain");
    assert_eq!(resp["result"]["targets"][0]["doc_anchor"], "a.mq.md#L9-L15");
}

#[test]
fn unknown_method_and_bad_json_are_structured() {
    let resp = call(r#"{"id":7,"method":"divine","params":{}}"#);
    assert_eq!(resp["ok"], false);
    assert_eq!(resp["error"]["code"], "mlsp.unknown_method");

    let resp = call("not json at all");
    assert_eq!(resp["ok"], false);
    assert_eq!(resp["error"]["code"], "mlsp.bad_request");
}
