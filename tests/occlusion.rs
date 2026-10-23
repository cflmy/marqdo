//! Occlusion syntax end-to-end. Grammar is declared by each document, not a
//! process environment variable, so imports and MLSP requests stay isolated.

use std::process::Command;

fn run(path: &str, markup: Option<&str>) -> (i32, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_marqdo"));
    cmd.args(["run", path]);
    if let Some(m) = markup {
        cmd.env("MARQDO_MARKUP", m);
    }
    let out = cmd.output().expect("run marqdo");
    (
        out.status.code().unwrap_or(1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn occlusion_value_is_payload() {
    let (code, stdout, stderr) = run("tests/occlusion/value.mq.md", Some("occlusion"));
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim_end(), "a=2\nb=42\nc=True");
}

#[test]
fn migrated_index_uses_access_without_a_process_mode() {
    let (code, stdout, stderr) = run("tests/occlusion/classic-index.mq.md", None);
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim_end(), "值");
}

#[test]
fn declared_occlusion_ignores_legacy_environment_switches() {
    let (code, stdout, stderr) = run("tests/occlusion/classic-index.mq.md", Some("classic"));
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim_end(), "值");
}

// ---- M2.5: label diagnostics (`label.missing` / `label.empty_payload`) ----

fn check(path: &str, markup: &str) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["check", path])
        .env("MARQDO_MARKUP", markup)
        .output()
        .expect("run marqdo check");
    (
        out.status.code().unwrap_or(1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

#[test]
fn label_missing_is_an_error() {
    let (code, text) = check("tests/occlusion/empty-label.mq.md", "occlusion");
    assert_eq!(code, 1, "text={text}");
    assert!(text.contains("遮蔽缺少描述"), "text={text}");
}

#[test]
fn label_empty_payload_is_a_warning() {
    let (code, text) = check("tests/occlusion/empty-payload.mq.md", "occlusion");
    assert_eq!(code, 0, "warnings must not fail the build: text={text}");
    assert!(text.contains("空载荷遮蔽"), "text={text}");
}

#[test]
fn label_ok_has_no_diagnostics() {
    let (code, text) = check("tests/occlusion/clean.mq.md", "occlusion");
    assert_eq!(code, 0, "text={text}");
    assert!(!text.contains("空载荷"), "text={text}");
    assert!(!text.contains("缺少描述"), "text={text}");
}

// ---- M4: L1 reference consistency (`label.ref_dangling`) ----

#[test]
fn label_ref_dangling_is_an_error() {
    let (code, text) = check("tests/occlusion/ref-dangling.mq.md", "occlusion");
    assert_eq!(code, 1, "text={text}");
    assert!(text.contains("描述引用了不存在的实体"), "text={text}");
}

#[test]
fn label_refs_that_resolve_are_fine() {
    let (code, text) = check("tests/occlusion/ref-ok.mq.md", "occlusion");
    assert_eq!(code, 0, "text={text}");
    assert!(!text.contains("不存在的实体"), "text={text}");
}

// ---- M9: two-tier label search (`labels` CLI + MLSP `labels.search`) ----

fn labels(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .arg("labels")
        .args(args)
        .env("MARQDO_MARKUP", "occlusion")
        .output()
        .expect("run marqdo labels");
    (
        out.status.code().unwrap_or(1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

#[test]
fn labels_pending_uses_builtin_groups() {
    let (code, text) = labels(&["tests/occlusion/labels.mq.md", "--count"]);
    assert_eq!(code, 0, "text={text}");
    assert_eq!(text.trim(), "2", "expected 待开发 + 模拟: {text}");
}

#[test]
fn labels_grep_replaces_builtin() {
    let (code, text) = labels(&["tests/occlusion/labels.mq.md", "--grep", "联调", "--count"]);
    assert_eq!(code, 0, "text={text}");
    assert_eq!(text.trim(), "1", "text={text}");
}

#[test]
fn labels_grep_with_builtin_unions() {
    let (code, text) = labels(&[
        "tests/occlusion/labels.mq.md",
        "--grep",
        "联调",
        "--also-builtin",
        "--count",
    ]);
    assert_eq!(code, 0, "text={text}");
    assert_eq!(text.trim(), "3", "text={text}");
}

#[test]
fn mlsp_labels_search_returns_hits() {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .arg("mlsp")
        .env("MARQDO_MARKUP", "occlusion")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn mlsp");
    let req = r#"{"id":1,"method":"labels.search","params":{"path":"tests/occlusion/labels.mq.md","also_builtin":true}}"#;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(format!("{req}\n").as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(text.contains("\"count\":2"), "text={text}");
    assert!(text.contains("客户留存率"), "text={text}");
}

// ---- M6 (T1): session-scoped staleness via `repair_apply` ----

fn mlsp_session(reqs: &[&str]) -> Vec<String> {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .arg("mlsp")
        .env("MARQDO_MARKUP", "occlusion")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn mlsp");
    let payload: String = reqs.iter().map(|r| format!("{r}\n")).collect();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(payload.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect()
}

fn repair_req(src: &str, new_line: &str) -> String {
    let src = format!("---\nmarkup: occlusion\n---\n\n{src}");
    let src_json = serde_json::to_string(&src).unwrap();
    let line_json = serde_json::to_string(new_line).unwrap();
    format!(
        r#"{{"id":1,"method":"repair_apply","params":{{"source":{src_json},"ranges":[[7,7]],"edits":[{{"op":"replace","line":7,"text":{line_json}}}],"check_labels":true}}}}"#
    )
}

#[test]
fn t1_flags_payload_change_only() {
    let src = "# main\n\n**a = [按税率算总价](100 * 0.1)**\n> print text=`a`\n";
    let out = mlsp_session(&[&repair_req(src, "**a = [按税率算总价](100 * 0.2)**")]);
    let line = out.join("\n");
    assert!(line.contains("\"stale\""), "{line}");
    assert!(line.contains("100 * 0.1"), "must show code_past: {line}");
    assert!(line.contains("100 * 0.2"), "must show code_now: {line}");
}

#[test]
fn t1_silent_when_label_also_changes() {
    let src = "# main\n\n**a = [按税率算总价](100 * 0.1)**\n> print text=`a`\n";
    let out = mlsp_session(&[&repair_req(
        src,
        "**a = [改成按两倍税率算](100 * 0.2)**",
    )]);
    let line = out.join("\n");
    assert!(line.contains("\"stale\":[]"), "label edit = author reviewed: {line}");
}

#[test]
fn t1_session_summary_reports_counters() {
    let src = "# main\n\n**a = [按税率算总价](100 * 0.1)**\n> print text=`a`\n";
    let out = mlsp_session(&[
        &repair_req(src, "**a = [按税率算总价](100 * 0.2)**"),
        r#"{"id":2,"method":"session.summary","params":{}}"#,
    ]);
    let summary = out.last().expect("summary line");
    assert!(summary.contains("\"labels_stale_seen\":1"), "{summary}");
    assert!(summary.contains("\"labels_outstanding\":1"), "{summary}");
}

// ---- M5: implicit occlusion (prose above `>` calls) ----

#[test]
fn implicit_labels_off_by_default() {
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["check", "tests/occlusion/implicit-stale.mq.md"])
        .output()
        .expect("check");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        !text.contains("调用上方的描述"),
        "must stay silent by default: {text}"
    );
}

#[test]
fn implicit_labels_opt_in_reports_mismatch() {
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "check",
            "tests/occlusion/implicit-stale.mq.md",
            "--implicit-labels",
        ])
        .output()
        .expect("check");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(text.contains("调用上方的描述提到 `税率`"), "{text}");
}

#[test]
fn implicit_labels_opt_in_stays_quiet_when_consistent() {
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args([
            "check",
            "tests/occlusion/implicit-ok.mq.md",
            "--implicit-labels",
        ])
        .output()
        .expect("check");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(!text.contains("调用上方的描述"), "{text}");
}

// ---- M3: `access` (legacy index semantics) + occlusion loop ----

#[test]
fn access_matches_legacy_index_semantics() {
    let (code, stdout, stderr) = run("tests/occlusion/access-and-loop.mq.md", Some("occlusion"));
    assert_eq!(code, 0, "stderr={stderr}");
    assert_eq!(stdout.trim_end(), "a=值\nb=二\n果子=一\n果子=二");
}

#[test]
fn access_errors_on_missing_key_like_legacy() {
    // Same failure mode as `[键](集合)`: no silent None.
    let dir = std::env::temp_dir().join("marqdo-m3-access");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("missing.mq.md");
    std::fs::write(
        &path,
        "---\nimport json:lib/json.mq.md\n---\n\n# main\n\n`表` = > json.parse text={\"键\":\"值\"}\n**x = [取不存在的键](access(`表`, \"无\"))**\n> print text=`x`\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .args(["run", path.to_str().unwrap()])
        .env("MARQDO_MARKUP", "occlusion")
        .output()
        .expect("run");
    assert_ne!(out.status.code(), Some(0));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("missing map key"), "err={err}");
}
