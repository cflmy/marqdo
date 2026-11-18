//! Application compositions with precomputed independent SQL/Floyd oracles.
use crate::host::{
    foreign_doc::{packing, DocumentSession},
    json::{json_to_value, value_to_json},
};
use crate::value::Value;
use serde_json::{json, Value as Json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn cases() -> Vec<Json> {
    let path = std::env::var_os("MARQDO_APPLICATION_CASES")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("tests/applications/CASES.json"));
    let rows: Vec<Json> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert!(!rows.is_empty());
    rows
}
fn out() -> PathBuf {
    let path = std::env::var_os("MARQDO_APPLICATION_ARTIFACT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("target/applications"));
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn code(case: &Json, stage: &str) -> String {
    let lang = if stage == "producer" {
        case["producer"].as_str().unwrap()
    } else {
        case["consumer"].as_str().unwrap()
    };
    let ext = if lang == "python" { "py" } else { "js" };
    std::fs::read_to_string(
        root()
            .join("tests/applications")
            .join(case["family"].as_str().unwrap())
            .join(format!("{stage}.{ext}")),
    )
    .unwrap()
}
fn failure(lang: &str) -> &'static str {
    if lang == "python" {
        "retained = answer\nraise RuntimeError('application_expected_failure')"
    } else {
        "globalThis.retained = answer; throw new Error('application_expected_failure');"
    }
}
fn mutate(lang: &str) -> &'static str {
    if lang == "python" {
        "bridge = None\nNone"
    } else {
        "bridge = null; null;"
    }
}
fn field<'a>(value: &'a Value, name: &str) -> &'a Value {
    let Value::Map(fields) = value else {
        panic!("expected record: {value:?}")
    };
    &fields.iter().find(|(k, _)| k == name).unwrap().1
}
fn save(path: &Path, value: &Json) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

#[test]
fn compositions_preserve_data_failure_recovery_and_copy_isolation_on_both_backends() {
    let out = out();
    let rows = cases();
    for case in &rows {
        let id = case["id"].as_str().unwrap();
        let q = |s: &str| serde_json::to_string(s).unwrap();
        let input = q(&case["input"].to_string());
        let family = root().join("lib");
        let consumer = case["consumer"].as_str().unwrap();
        let source = format!(
            r#"---
markup: occlusion
import:
  foreign: {foreign}
  json: {json}
---
# main
**data = > json.parse text={input}**
**p = > foreign.session_open lang={producer}**
**c = > foreign.session_open lang={consumer}**
**> foreign.session_set id=`p` name="data" value=`data`**
**first = > evaluate target=`p` source={produce}**
**bridge = > foreign.session_get id=`p` name="bridge"**
**> foreign.session_set id=`c` name="bridge" value=`bridge`**
**second = > evaluate target=`c` source={consume}**
**failure = > evaluate target=`c` source={failure}**
**kept = > foreign.session_get id=`c` name="retained"**
**cleared = > evaluate target=`c` source={mutate}**
**empty = > foreign.session_get id=`c` name="bridge"**
**original = > foreign.session_get id=`p` name="bridge"**
**answer = > foreign.session_get id=`c` name="answer"**
**> foreign.session_close id=`p`**
**> foreign.session_close id=`c`**
*answer*

## evaluate
    + `target`
    + `source`
**record = > foreign.session_eval id=`target` source=`source`**
*record*
"#,
            foreign = family.join("foreign.mq.md").display(),
            json = family.join("json.mq.md").display(),
            producer = q(case["producer"].as_str().unwrap()),
            consumer = q(consumer),
            produce = q(&code(case, "producer")),
            consume = q(&code(case, "consumer")),
            failure = q(failure(consumer)),
            mutate = q(mutate(consumer))
        );
        let path = out.join(format!("{id}.mq.md"));
        std::fs::write(&path, &source).unwrap();
        let module = crate::load::load_module(&path).unwrap();
        let mut executions = Vec::new();
        for backend in ["tree", "bytecode"] {
            let (result, stdout, (journal, protocol, live)) = if backend == "tree" {
                let mut tree = crate::interp::Interpreter::with_capture(Some(&path), false);
                let result = tree.run_module(&module);
                (
                    result,
                    tree.captured_stdout.clone(),
                    tree.application_evidence(),
                )
            } else {
                let program = crate::bytecode::compile_module(Some(&path), &module).unwrap();
                let mut vm = crate::bytecode::Vm::with_capture(Some(&path));
                let result = vm.run(&program);
                (
                    result,
                    vm.captured_stdout.clone(),
                    vm.application_evidence(),
                )
            };
            let error = result.as_ref().err().map(|e| e.to_string());
            let actual = result.ok().map(|v| value_to_json(&v).unwrap());
            executions.push(json!({"backend":backend,"ok":error.is_none(),"error":error,"result":actual,"stdout":stdout,"live_sessions":live,
                "journal":value_to_json(&Value::List(journal)).unwrap(),"protocol":protocol.iter().map(|p|json!({"session":p.session,"request":p.request,"response":p.response})).collect::<Vec<_>>()}));
        }
        let result = json!({"version":1,"kind":"document","case":case,"source":source,"source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())),"executions":executions});
        save(&out.join(format!("{id}-document.json")), &result);
        for execution in result["executions"].as_array().unwrap() {
            assert_eq!(execution["result"], case["answer"], "{id}: {execution}");
            assert_eq!(execution["ok"], true, "{id}: {execution}");
            assert_eq!(execution["stdout"], "");
            assert_eq!(execution["live_sessions"], 0);
        }
    }
    println!(
        "APPLICATION_METRIC:{}",
        json!({"cases":rows.len(),"executions":rows.len()*2})
    );
}

#[test]
fn application_action_histories_replay_raw_and_packed_after_failure() {
    let out = out();
    let rows = cases();
    for case in &rows {
        let id = case["id"].as_str().unwrap();
        let consumer = case["consumer"].as_str().unwrap();
        let source=format!("---\nimport:\n  foreign: {}\n---\n# main\n\n`会话` = > foreign.session\n\n`produce` =\n```{}\n{}\n```\n\n`consume` =\n```{}\n{}\n```\n\n`failure` =\n```{}\n{}\n```\n\n`mutate` =\n```{}\n{}\n```\n",root().join("lib/foreign.mq.md").display(),case["producer"].as_str().unwrap(),code(case,"producer"),consumer,code(case,"consumer"),consumer,failure(consumer),consumer,mutate(consumer));
        let path = out.join(format!("{id}-interactive.mq.md"));
        std::fs::write(&path, &source).unwrap();
        let mut session = DocumentSession::open(&path, &root()).unwrap();
        let p = session
            .cells
            .iter()
            .find(|c| c.name == "produce")
            .unwrap()
            .kernel
            .clone();
        let c = session
            .cells
            .iter()
            .find(|c| c.name == "consume")
            .unwrap()
            .kernel
            .clone();
        let expected = json_to_value(&case["answer"]).unwrap();
        session
            .set_value(&p, "data", json_to_value(&case["input"]).unwrap())
            .unwrap();
        let first = session.run_cell("produce", None).unwrap();
        assert_eq!(field(&first, "ok"), &Value::Bool(true), "{id}: {first:?}");
        let bridge = session.pull_value(&p, "bridge").unwrap();
        assert_eq!(value_to_json(&bridge).unwrap(), case["bridge"]);
        session.set_value(&c, "bridge", bridge.clone()).unwrap();
        let second = session.run_cell("consume", None).unwrap();
        assert_eq!(field(&second, "result"), &expected, "{id}: {second:?}");
        let failed = session.run_cell("failure", None).unwrap();
        assert_eq!(field(&failed, "ok"), &Value::Bool(false));
        assert_eq!(
            session
                .cells
                .iter()
                .find(|c| c.name == "failure")
                .unwrap()
                .last_result,
            Value::None
        );
        assert_eq!(session.pull_value(&c, "retained").unwrap(), expected);
        session.run_cell("mutate", None).unwrap();
        assert_eq!(
            session
                .cells
                .iter()
                .find(|c| c.name == "mutate")
                .unwrap()
                .last_result,
            Value::None
        );
        assert_eq!(session.pull_value(&c, "bridge").unwrap(), Value::None);
        assert_eq!(session.pull_value(&p, "bridge").unwrap(), bridge);
        assert_eq!(session.pull_value(&c, "answer").unwrap(), expected);
        let snapshot = session.export_snapshot().unwrap();
        session.close();
        let packed = packing::pack(&snapshot).unwrap();
        assert_eq!(
            value_to_json(&packing::unpack(&packed).unwrap()).unwrap(),
            value_to_json(&snapshot).unwrap()
        );
        let raw_check =
            DocumentSession::check_snapshot_isolated(&path, &root(), &snapshot).unwrap();
        let packed_check =
            DocumentSession::check_snapshot_isolated(&path, &root(), &packed).unwrap();
        save(
            &out.join(format!("{id}-interactive.json")),
            &json!({"version":1,"kind":"interactive","case":case,"source":source,"source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())),"snapshot":value_to_json(&snapshot).unwrap(),"packed":value_to_json(&packed).unwrap(),"raw_check":value_to_json(&raw_check).unwrap(),"packed_check":value_to_json(&packed_check).unwrap()}),
        );
        assert_eq!(
            field(&raw_check, "ok"),
            &Value::Bool(true),
            "{id}: {raw_check:?}"
        );
        assert_eq!(
            field(&packed_check, "ok"),
            &Value::Bool(true),
            "{id}: {packed_check:?}"
        );
    }
    println!(
        "APPLICATION_REPLAY_METRIC:{}",
        json!({"cases":rows.len(),"strict_replays":rows.len()*2})
    );
}

#[test]
fn explicit_null_and_stdout_remain_distinct_and_only_success_can_persist() {
    for lang in ["python", "node"] {
        let (good, bad) = if lang == "python" {
            (
                "print('audit')\nNone",
                "print('failed audit')\nraise RuntimeError('expected_failure')",
            )
        } else {
            (
                "console.log('audit'); null;",
                "console.log('failed audit'); throw Error('expected_failure');",
            )
        };
        let path = out().join(format!("null-persist-{lang}-{}.mq.md", std::process::id()));
        std::fs::write(&path,format!("---\nimport:\n  foreign: {}\n---\n# main\n\n`会话` = > foreign.session\n\n`good` =\n```{lang}\n{good}\n```\n\n`bad` =\n```{lang}\n{bad}\n```\n",root().join("lib/foreign.mq.md").display())).unwrap();
        let mut session = DocumentSession::open(&path, &root()).unwrap();
        assert!(session.persist_cell("good", "empty").is_err());
        session.run_cell("good", None).unwrap();
        let cell = session.cells.iter().find(|c| c.name == "good").unwrap();
        assert_eq!(cell.last_result, Value::None);
        assert_eq!(cell.last_stdout, "audit\n");
        assert_eq!(session.oracle_journal.last().unwrap().value, Value::None);
        assert_eq!(session.oracle_journal.last().unwrap().result, Value::None);
        session.persist_cell("good", "empty").unwrap();
        session.run_cell("bad", None).unwrap();
        assert_eq!(
            session
                .cells
                .iter()
                .find(|c| c.name == "bad")
                .unwrap()
                .last_result,
            Value::None
        );
        assert!(session.persist_cell("bad", "must_not_write").is_err());
        session.close();
    }
}
