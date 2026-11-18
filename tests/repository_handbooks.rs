//! Real CLI data transport, executable handbooks and repository authoring checks.
use std::{fs,path::PathBuf,process::{Command,Stdio},io::Write};
fn root()->PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }
fn native()->Command { let mut c=Command::new(env!("CARGO_BIN_EXE_marqdo"));c.current_dir(root());c }
#[test]
fn legacy_quoted_text_keeps_raw_backslashes_and_source() {
    let work=std::env::temp_dir().join(format!("mq-native-authoring-{}",std::process::id()));
    fs::create_dir_all(&work).unwrap();
    let source="---\nmarkup: occlusion\n---\n\n## 加一\n\n输入\u{0060}n\u{0060}之后，返回*[结果](n + 1)*。\n\n\"quotes\" 'single' \\\\ $HOME 😀\n";
    let out=native().args(["call","lib/fs.mq.md","write_text","path=\"source.mq.md\"",
       &format!("text={}",format!("\"{source}\"")),"--effects","fs.write","--fs-root",work.to_str().unwrap(),"--json"]).output().unwrap();
    assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stdout));
    assert_eq!(fs::read_to_string(work.join("source.mq.md")).unwrap(),source);
    let out=native().args(["call","lib/fs.mq.md","write_text","path=\"raw.txt\"",
        "text='C:\\new\\file'","--effects","fs.write","--fs-root",work.to_str().unwrap(),"--json"]).output().unwrap();
    assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stdout));
    assert_eq!(fs::read_to_string(work.join("raw.txt")).unwrap(),"C:\\new\\file");
    let parsed=marqdo::call::parse_call_args(&[r#"text="C:\new\file\q""#.into()]).unwrap();
    assert_eq!(parsed[0].1,marqdo::value::Value::Text(r"C:\new\file\q".into()));
    fs::remove_dir_all(work).unwrap();
}

fn call_stdin(args:&[&str],data:&str)->std::process::Output {
    let mut cmd=native();cmd.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child=cmd.spawn().unwrap();child.stdin.take().unwrap().write_all(data.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn call_json_stdin_carries_large_literal_source_and_rejects_ambiguous_data() {
    let work=std::env::temp_dir().join(format!("mq-native-json-{}",std::process::id()));fs::create_dir_all(&work).unwrap();
    let source=format!("$literal\n{}\n\"q\" \\\\ @literal", "源码与文档 😀\n".repeat(20000));
    let args=["call","lib/fs.mq.md","write_text","--args-json","-","--effects","fs.write","--fs-root",work.to_str().unwrap(),"--json"];
    let out=call_stdin(&args,&serde_json::json!({"path":"large.mq.md","text":source}).to_string());
    assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stdout));
    assert_eq!(fs::read_to_string(work.join("large.mq.md")).unwrap(),source);
    for data in ["[]",r#"{"text":"a","text":"b"}"#] {
        let out=call_stdin(&["call","lib/bytes.mq.md","from_text","--args-json","-","--json"],data);
        assert!(!out.status.success(),"ambiguous data accepted: {data}");
    }
    let out=call_stdin(&["call","lib/bytes.mq.md","from_text","text=\"named\"","--args-json","-","--json"],r#"{"text":"json"}"#);
    assert!(!out.status.success(),"mixed argument channels accepted");
    fs::remove_dir_all(work).unwrap();
}
#[test]
fn literal_json_strings_remain_literal_in_a_call_session() {
    let work=std::env::temp_dir().join(format!("mq-native-json-session-{}",std::process::id()));fs::create_dir_all(&work).unwrap();
    let args=["call","lib/bytes.mq.md","from_text","--args-json","-","--session","literal","--toget","data","--json"];
    let mut cmd=native();cmd.args(args).env("MARQDO_CALL_SESSION_DIR",&work).stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child=cmd.spawn().unwrap();child.stdin.take().unwrap().write_all(br#"{"text":"$not_a_reference"}"#).unwrap();
    let out=child.wait_with_output().unwrap();assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stdout));
    let file=work.join("literal.json");
    let mut journal:serde_json::Value=serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    assert_eq!(journal["steps"][0]["literal_args"],true);
    journal["bindings"].as_object_mut().unwrap().remove("data");
    journal["journal_bindings"]["data"]=serde_json::json!(1);
    fs::write(&file,serde_json::to_vec(&journal).unwrap()).unwrap();
    let out=native().env("MARQDO_CALL_SESSION_DIR",&work)
        .args(["call","lib/bytes.mq.md","to_text","data=$data","--session","literal","--json"]).output().unwrap();
    assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stdout));
    let answer:serde_json::Value=serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(answer["result"],"$not_a_reference");
    fs::remove_dir_all(work).unwrap();
}

#[test]
fn json_argument_envelope_allows_independent_lists_but_preserves_value_domain() {
    let parsed=marqdo::call::parse_call_args_json(r#"{"a":[1],"b":[2,3]}"#).unwrap();
    assert_eq!(parsed.len(),2);
    for bad in [r#"{"a":{"x":[1],"y":[2,3]}}"#,r#"{"a":9007199254740992}"#,r#"{"a":{"x":1,"x":2}}"#] {
        assert!(marqdo::call::parse_call_args_json(bad).is_err(),"{bad}");
    }
}

#[test]
fn require_ok_preserves_failed_result_and_stops_external_execution_flow() {
    let args=["call","lib/result.mq.md","error","code=\"fixture\"","message=\"stop here\"","--json"];
    let ordinary=native().args(args).output().unwrap();
    assert!(ordinary.status.success());
    let checked=native().args(args).arg("--require-ok").output().unwrap();
    assert!(!checked.status.success());
    let value:serde_json::Value=serde_json::from_slice(&checked.stdout).unwrap();
    assert_eq!(value["result"]["ok"],false);
    assert_eq!(value["result"]["error"]["code"],"fixture");
    let success=native().args(["call","lib/result.mq.md","ok","value=False","--require-ok","--json"]).output().unwrap();
    assert!(success.status.success());
}

#[test]
fn native_writeback_call_uses_entry_source_and_the_native_filesystem_root() {
    use marqdo::{Backend,call::{CallEffects,CallOptions,call_file_with}};
    let work=std::env::temp_dir().join(format!("mq-call-writeback-{}",std::process::id()));
    fs::create_dir_all(work.join("restricted")).unwrap();
    let import=root().join("lib/writeback.mq.md").display().to_string().replace('\\',"/");
    let file=work.join("task.mq.md");
    let source=format!("---\nmarkup: occlusion\nimport:\n  wb: {import}\n---\n\n## task\n\nWe **written = [Record the real source-local result](wb.record(value=\"observed\", key=\"summary\"))**.\nReturn *[Read the recorded slot](wb.get(key=\"summary\"))*.\n");
    fs::write(&file,&source).unwrap();
    let opts=||CallOptions{effects:CallEffects::parse("fs.write").unwrap(),fs_root:Some(work.clone()),..Default::default()};
    for _ in 0..2 {
        let value=call_file_with(&file,"task",&[],Backend::Tree,opts()).unwrap();
        assert!(value.as_display().contains("observed"),"{value:?}");
    }
    let recorded=fs::read_to_string(&file).unwrap();
    assert_eq!(recorded.matches("<!-- marqdo-out summary").count(),1);
    assert!(call_file_with(&file,"task",&[],Backend::Tree,CallOptions{fs_root:Some(work.join("restricted")),..opts()}).is_err());
    assert_eq!(fs::read_to_string(&file).unwrap(),recorded);
    fs::remove_dir_all(work).unwrap();
}

#[test]
fn native_review_receipts_and_stale_batch_preflight_preserve_exact_sources() {
    let work=std::env::temp_dir().join(format!("mq-native-review-{}",std::process::id()));
    fs::create_dir_all(&work).unwrap();
    let first="\u{feff}source\r\n$literal 😀\r\n";
    fs::write(work.join("first.txt"),first).unwrap();
    fs::write(work.join("second.txt"),"keep second\n").unwrap();
    let run=|function:&str,args:serde_json::Value|{
        let out=call_stdin(&["call","skills/marqdo-dev/files.mq.md",function,"--args-json","-","--effects","fs.write","--fs-root",work.to_str().unwrap(),"--json"],&args.to_string());
        assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stdout));
        serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()["result"].clone()
    };
    let observed=run("read_many",serde_json::json!({"paths":["first.txt","second.txt"]}));
    assert_eq!(observed[0]["source"],first);
    let digest=call_stdin(&["call","lib/hash.mq.md","sha256","--args-json","-","--json"],&serde_json::json!({"text":first}).to_string());
    assert!(digest.status.success());
    let digest:serde_json::Value=serde_json::from_slice(&digest.stdout).unwrap();
    assert_eq!(observed[0]["sha256"],digest["result"]);
    fs::write(work.join("first.txt"),"concurrently changed\n").unwrap();
    let result=run("apply",serde_json::json!({"updates":[
        {"path":"second.txt","source":"must not be written\n","expected_sha256":observed[1]["sha256"]},
        {"path":"first.txt","source":"stale proposal\n","expected_sha256":observed[0]["sha256"]}
    ]}));
    assert_eq!(result["ok"],false);
    assert_eq!(result["error"]["code"],"authoring.changed");
    assert_eq!(fs::read_to_string(work.join("first.txt")).unwrap(),"concurrently changed\n");
    assert_eq!(fs::read_to_string(work.join("second.txt")).unwrap(),"keep second\n");
    fs::remove_dir_all(work).unwrap();
}

#[test]
fn readable_syntax_tour_is_a_callable_program_on_both_backends() {
    use marqdo::{Backend,call::{CallEffects,CallOptions,call_file_with},value::Value};
    for backend in [Backend::Tree,Backend::Bytecode] {
        let report=call_file_with(&root().join("examples/syntax-tour/index.mq.md"),"main",&[],backend,
            CallOptions{effects:CallEffects::parse("output.write").unwrap(),..Default::default()}).unwrap();
        let Value::Map(fields)=report else {panic!("expected actual demonstration values")};
        for (name,expected) in [("name_length",6),("sum",3),("comma_sum",3),("nested_sum",13),("spaced_name",32)] {
            assert_eq!(fields.iter().find(|(k,_)|k==name).unwrap().1,Value::Int(expected),"{name}");
        }
        assert_eq!(fields.iter().find(|(k,_)|k=="unfinished").unwrap().1,Value::None);
    }
}

#[test]
fn native_public_file_examples_preserve_caller_files_on_both_backends() {
    use marqdo::{Backend,call::{CallEffects,CallOptions,call_file_with},value::Value};
    let work=std::env::temp_dir().join(format!("mq-public-files-{}",std::process::id()));
    let kept=[("out/demo.txt","caller source"),("out/demo-copy.txt","caller copy"),
        ("m9-demo/readme.txt","caller English"),("m9-演示/说明.txt","caller Chinese")];
    for (name,text) in kept {
        let path=work.join(name);fs::create_dir_all(path.parent().unwrap()).unwrap();fs::write(path,text).unwrap();
    }
    for backend in [Backend::Tree,Backend::Bytecode] {
        for file in ["public/features/09-path-fs.mq.md","public/features/09-路径文件.mq.md",
            "public/features/16-fs-mid2.mq.md","public/features/16-文件mid2.mq.md"] {
            let observed=call_file_with(&root().join(file),"main",&[],backend,
                CallOptions{effects:CallEffects::parse("fs.write").unwrap(),fs_root:Some(work.clone()),..Default::default()}).unwrap();
            let Value::Map(fields)=observed else {panic!("expected actual file observations")};
            if file.contains("/09-") {
                assert_eq!(fields.iter().find(|(k,_)|k=="copied_text").unwrap().1,Value::Text("path-m2".into()));
            } else {
                let expected=if file.contains("16-fs"){2}else{3};
                assert_eq!(fields.iter().find(|(k,_)|k=="size").unwrap().1,Value::Int(expected));
                assert_eq!(fields.iter().find(|(k,_)|k=="entries").unwrap().1,Value::Int(2));
            }
            for (name,text) in kept {assert_eq!(fs::read_to_string(work.join(name)).unwrap(),text);}
            assert_eq!(fs::read_dir(&work).unwrap().count(),3,"temporary tree leaked: {file}");
        }
    }
    fs::remove_dir_all(work).unwrap();
}

#[test]
fn native_excerpt_keeps_one_based_locations_and_clips_long_unicode_lines() {
    let work=std::env::temp_dir().join(format!("mq-native-excerpt-{}",std::process::id()));
    fs::create_dir_all(&work).unwrap();
    fs::write(work.join("lines.txt"),format!("before\r\n{}\r\nafter\r\n","文".repeat(2500))).unwrap();
    for backend in ["tree","bytecode"] {
        let args=["call","skills/marqdo-dev/files.mq.md","excerpt","--backend",backend,"--args-json","-","--effects","fs.read","--fs-root",work.to_str().unwrap(),"--json"];
        let output=call_stdin(&args,r#"{"path":"lines.txt","start":2,"count":2}"#);
        assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stdout));
        let actual:serde_json::Value=serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(actual["result"]["total_lines"],3);
        assert_eq!(actual["result"]["lines"][0]["line"],2);
        assert_eq!(actual["result"]["lines"][0]["text"].as_str().unwrap().chars().count(),2000);
        assert_eq!(actual["result"]["lines"][1]["text"],"after");
        let output=call_stdin(&args,r#"{"path":"lines.txt","count":201}"#);
        let actual:serde_json::Value=serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(actual["result"]["ok"],false);
    }
    fs::remove_dir_all(work).unwrap();
}

#[test]
fn request_query_resource_defaults_remain_native_values_on_both_backends() {
    for backend in ["tree","bytecode"] {
        for (file,function,args) in [
            ("ext/data/view.mq.md","query",r#"{"source":"articles"}"#),
            ("ext/data/视图.mq.md","查询",r#"{"源":"articles"}"#),
        ] {
            let out=call_stdin(&["call",file,function,"--backend",backend,"--args-json","-","--json"],args);
            assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stdout));
            let body:serde_json::Value=serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(body["result"]["_type"],"data_query");
            assert_eq!(body["result"]["detail"],false);
            assert_eq!(body["result"]["tenant_scope"],false);
            assert_eq!(body["result"]["limit"],50);
            assert!(body["result"]["where"].is_null());
        }
    }
}
