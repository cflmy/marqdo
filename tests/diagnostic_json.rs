//! T1.4 诊断机器面：真实错误路径产出的 Diagnostic 可序列化（`run --json` 出口）。
//!
//! 验收（three-problems-plan.md T1.4）：人类文本零回归 + JSON `code`/`span` 非空。

use std::path::Path;

use marqdo::diagnostics::Diagnostic;
use marqdo::RunOptions;

#[test]
fn error_fixture_downcasts_and_serializes() {
    let err = marqdo::run_file(
        Path::new("tests/errors/div-zero.mq.md"),
        &RunOptions::default(),
    )
    .expect_err("div-zero 应失败");
    let diag: Diagnostic = err
        .downcast()
        .expect("错误应为 diagnostics::Diagnostic（机器面出口的前提）");
    let v = diag.to_json();
    assert_eq!(v["code"], "marqdo.error"); // code 分类细化随 Phase 2/3
    assert_eq!(v["severity"], "error");
    assert!(
        v["span"]["line"].as_u64().unwrap() > 0,
        "JSON span.line 必须非空"
    );
    assert!(v["message"]
        .as_str()
        .unwrap()
        .contains("division by zero"));
    // 人类文本零回归
    assert!(diag.format_message().contains("division by zero"));
}
