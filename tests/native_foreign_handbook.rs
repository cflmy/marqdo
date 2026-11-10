//! Native analysis and real persistent view cells must agree on observable values.
use marqdo::{Backend,call::{CallEffects,CallOptions,call_file_with,parse_call_args_json},value::Value};
use std::path::PathBuf;
fn root()->PathBuf {PathBuf::from(env!("CARGO_MANIFEST_DIR"))}
fn field<'a>(value:&'a Value,key:&str)->&'a Value {
    let Value::Map(fields)=value else {panic!("expected map: {value:?}")};
    &fields.iter().find(|(k,_)|k==key).unwrap_or_else(||panic!("missing {key}: {value:?}")).1
}
fn list(value:&Value)->&[Value] {let Value::List(values)=value else {panic!("expected list: {value:?}")};values}
fn number(value:&Value)->f64 {
    match value {Value::Int(n)=>*n as f64,Value::Num(n)=>*n,_=>panic!("expected number: {value:?}")}
}
fn analyze(data:&str,backend:Backend)->Value {
    call_file_with(&root().join("examples/foreign-native-report/index.mq.md"),"analyze_orders",
        &parse_call_args_json(data).unwrap(),backend,CallOptions::default()).unwrap()
}
#[test]
fn native_grouping_handles_nonintegral_means_equal_groups_and_empty_input() {
    for backend in [Backend::Tree,Backend::Bytecode] {
        let report=analyze(r#"{"orders":[{"region":"b","amount":3},{"region":"a","amount":2},{"region":"a","amount":2}]}"#,backend);
        assert_eq!(number(field(&report,"total")),7.);
        assert_eq!(number(field(&report,"count")),3.);
        assert!((number(field(&report,"average"))-7./3.).abs()<1e-12);
        let rows=list(field(&report,"regions"));
        assert_eq!(rows.len(),2);
        assert_eq!(field(&rows[0],"region"),&Value::Text("a".into()));
        for row in rows {assert_eq!(number(field(row,"z_span")),0.);}
        let empty=analyze(r#"{"orders":[]}"#,backend);
        assert!(list(field(&empty,"regions")).is_empty());
        assert_eq!(number(field(&empty,"total")),0.);
        assert_eq!(number(field(&empty,"average")),0.);
    }
}
#[test]
fn public_views_run_three_actual_cells_with_shared_and_isolated_formatters() {
    if !std::process::Command::new("node").arg("--version").output().is_ok_and(|o|o.status.success()) {
        eprintln!("Node Intl unavailable; native report remains covered independently");return;
    }
    for file in ["public/features/19-foreign-view-cells.mq.md","public/features/19-外联视图单元.mq.md"] {
        let path=root().join(file);
        let result=call_file_with(&path,"main",&[],Backend::Tree,CallOptions{
            effects:CallEffects::parse("process.exec").unwrap(),fs_root:Some(root()),..Default::default()
        }).unwrap();
        let report=field(&result,"report");
        assert_eq!(number(field(report,"total")),950.);
        assert!((number(field(report,"average"))-950./6.).abs()<1e-12);
        let formatted=field(&result,"formatting");
        assert!(field(formatted,"total").as_display().contains("950.00"));
        assert!(field(formatted,"average").as_display().contains("158.33"));
        let mut session=marqdo::host::foreign_doc::DocumentSession::open(&path,&root()).unwrap();
        let actual=session.run_all(false).unwrap();
        assert_eq!(field(&actual,"ok"),&Value::Bool(true),"{file}: {actual:?}");
        assert_eq!(number(field(&actual,"ran")),3.);
        assert_eq!(number(field(&actual,"failed")),0.);
        let records=list(field(&actual,"records"));
        assert_eq!(records.len(),3);
        for record in records {assert_eq!(field(record,"ok"),&Value::Bool(true),"{record:?}");}
        assert!(field(&records[1],"stdout").as_display().contains("158.33"));
        assert!(field(&records[2],"stdout").as_display().contains("950.00"));
        drop(session);
    }
}

#[test]
fn showcase_uses_native_reports_and_keeps_one_shot_and_persistent_formats_equivalent() {
    if !std::process::Command::new("node").arg("--version").output().is_ok_and(|o|o.status.success()) {
        eprintln!("Node unavailable; native report cases are independent");return;
    }
    for backend in [Backend::Tree,Backend::Bytecode] {
        let observed=call_file_with(&root().join("examples/oa2026-showcase/foreign-node.mq.md"),"main",&[],backend,
            CallOptions{effects:CallEffects::parse("process.exec").unwrap(),fs_root:Some(root()),..Default::default()}).unwrap();
        assert_eq!(field(&observed,"ok"),&Value::Bool(true),"{observed:?}");
        let values=field(&observed,"value");
        assert_eq!(number(field(field(values,"before"),"total")),870.);
        assert_eq!(number(field(field(values,"after"),"total")),950.);
        let ranked=list(field(values,"updated_ranking"));
        for (row,total) in ranked.iter().zip([420.,300.,230.]) {assert_eq!(number(field(row,"total")),total);}
        let once=field(values,"one_shot");let initial=field(values,"persistent_initial");
        for key in ["total","average"] {assert_eq!(field(once,key),field(initial,key));}
        assert!(field(field(values,"persistent_updated"),"average").as_display().contains("158.33"));
    }
}

#[test]
fn public_foreign_guides_execute_batches_and_persistent_state_in_both_languages() {
    if !std::process::Command::new("node").arg("--version").output().is_ok_and(|o|o.status.success()) {
        eprintln!("Node unavailable; native report cases remain independent");return;
    }
    for backend in [Backend::Tree,Backend::Bytecode] {
        for file in ["public/stdlib/foreign.mq.md","public/stdlib/外联.mq.md"] {
            let captured=marqdo::run_file_capture(&root().join(file),&marqdo::RunOptions{backend,fs_root:Some(root()),..Default::default()}).unwrap();
            assert!(captured.stdout.contains("4.00"),"{file}: {}",captured.stdout);
            assert!(captured.stdout.contains("21.00") && captured.stdout.contains("0.08"),"{file}: {}",captured.stdout);
            assert!(!captured.stdout.contains("NaN"),"{file}: {}",captured.stdout);
        }
        for file in ["public/00-welcome.mq.md","public/00-欢迎.mq.md"] {
            let captured=marqdo::run_file_capture(&root().join(file),&marqdo::RunOptions{backend,fs_root:Some(root()),..Default::default()}).unwrap();
            assert!(captured.stdout.contains("950.00"),"{file}: {}",captured.stdout);
        }
    }
}
