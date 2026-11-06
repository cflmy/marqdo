use crate::host::{foreign_session as session, json::value_to_json, oracle, HostContext};
use crate::value::Value;
use serde_json::{json, Value as Json};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn directory() -> PathBuf {
    let p = std::env::var_os("MARQDO_CAUSAL_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("target/causal-replay"));
    std::fs::create_dir_all(&p).unwrap();
    p
}
fn steps(protocol: &[session::ProtocolStep]) -> Vec<Json> {
    protocol
        .iter()
        .map(|p| json!({"session":p.session,"request":p.request,"response":p.response}))
        .collect()
}
pub(crate) fn evidence(ctx: &mut HostContext) -> Json {
    let journal = value_to_json(&Value::List(ctx.oracle_observations.clone())).unwrap();
    let protocol = steps(&ctx.foreign_protocol_steps);
    let divergence = ctx.foreign_replay_divergence.clone();
    let live = ctx.foreign_sessions.len();
    // Inspection is explicitly outside replay and outside the admitted journal.
    ctx.foreign_replay_expected = None;
    ctx.foreign_replay_divergence = None;
    let n = if live == 1 {
        session::get(ctx, &Value::Handle("s1".into()), &Value::Text("n".into()))
            .ok()
            .map(|v| value_to_json(&v).unwrap())
    } else {
        None
    };
    json!({"journal":journal,"protocol":protocol,"divergence":divergence,"live":live,"n":n})
}

#[test]
fn observation_journal_erases_writes_but_a_context_distinguishes_them() {
    for lang in ["python", "node"] {
        let out = directory();
        let path = out.join(format!("journal-only-{lang}.marker"));
        let literal = serde_json::to_string(&path.display().to_string()).unwrap();
        let code = if lang == "python" {
            format!("from pathlib import Path\nPath({literal}).write_text(str(ticket))\nNone")
        } else {
            format!("require('fs').writeFileSync({literal},String(ticket)); null;")
        };
        let mut journals = Vec::new();
        let mut histories = Vec::new();
        let mut contents = Vec::new();
        let mut reads = Vec::new();
        let mut first_history = Vec::new();
        for ticket in [7, 8] {
            let mut ctx = HostContext::default();
            let id =
                session::open(&mut ctx, &Value::Text(lang.into()), None, None, None, None).unwrap();
            session::set(
                &mut ctx,
                &id,
                &Value::Text("ticket".into()),
                &Value::Int(ticket),
            )
            .unwrap();
            let reply = session::eval(
                &mut ctx,
                &id,
                None,
                Some(&Value::Text(code.clone())),
                None,
                None,
            )
            .unwrap();
            let observed = oracle::eval_observation(&reply).unwrap();
            oracle::append(&mut ctx, "eval", &observed).unwrap();
            journals.push(Value::List(ctx.oracle_observations.clone()));
            histories.push(steps(&ctx.foreign_protocol_steps));
            if ticket == 7 {
                first_history = ctx.foreign_protocol_steps.clone();
            }
            contents.push(std::fs::read_to_string(&path).unwrap());
            reads.push(session::get(&mut ctx, &id, &Value::Text("ticket".into())).unwrap());
            session::close(&mut ctx, &id).unwrap();
        }
        let checked = oracle::replay_check(&journals[0], &journals[1]).unwrap();
        assert_eq!(journals[0], journals[1]);
        assert_eq!(contents, ["7", "8"]);
        assert_ne!(reads[0], reads[1]);
        assert_eq!(value_to_json(&checked).unwrap()["ok"], true);
        // Control requests use a different wrapper than eval. A mismatching
        // setter must not retire the live kernel or overwrite its prior value.
        let mut ctx = HostContext::default();
        let id =
            session::open(&mut ctx, &Value::Text(lang.into()), None, None, None, None).unwrap();
        session::set(
            &mut ctx,
            &id,
            &Value::Text("ticket".into()),
            &Value::Int(99),
        )
        .unwrap();
        ctx.foreign_protocol_steps.clear();
        ctx.foreign_replay_expected = Some(first_history.into());
        let error =
            session::set(&mut ctx, &id, &Value::Text("ticket".into()), &Value::Int(8)).unwrap_err();
        let dispatched = ctx.foreign_protocol_steps.len();
        let live = ctx.foreign_sessions.len();
        ctx.foreign_replay_expected = None;
        ctx.foreign_replay_divergence = None;
        let retained = session::get(&mut ctx, &id, &Value::Text("ticket".into())).unwrap();
        assert_eq!(dispatched, 0);
        assert_eq!(live, 1);
        assert_eq!(retained, Value::Int(99));
        session::close(&mut ctx, &id).unwrap();
        std::fs::write(out.join(format!("journal-only-{lang}.json")),serde_json::to_vec_pretty(&json!({"kind":"journal-erasure","language":lang,"journals":journals.iter().map(|j|value_to_json(j).unwrap()).collect::<Vec<_>>(),"histories":histories,"contents":contents,"reads":reads.iter().map(|v|value_to_json(v).unwrap()).collect::<Vec<_>>(),"journal_check":value_to_json(&checked).unwrap(),"setter_guard":{"error":error,"dispatched":dispatched,"live":live,"retained":value_to_json(&retained).unwrap()}})).unwrap()).unwrap();
    }
}

#[test]
fn first_protocol_divergence_stops_host_suffix_and_preserves_current_effects() {
    let lengths = std::env::var("MARQDO_CAUSAL_PREFIXES")
        .unwrap_or("0,4".into())
        .split(',')
        .map(|s| s.parse::<usize>().unwrap())
        .collect::<Vec<_>>();
    let seeds = std::env::var("MARQDO_CAUSAL_SEEDS")
        .unwrap_or("3".into())
        .split(',')
        .map(|s| s.parse::<i64>().unwrap())
        .collect::<Vec<_>>();
    let mut count = 0;
    for lang in ["python", "node"] {
        for &length in &lengths {
            for &seed in &seeds {
                for backend in ["tree", "bytecode"] {
                    for mode in ["reply", "request"] {
                        let out = directory();
                        let id = format!("{lang}-{length}-{seed}-{backend}-{mode}");
                        let path = out.join(format!("{id}.mq.md"));
                        let input = out.join(format!("{id}.input"));
                        let marker = out.join(format!("{id}.suffix"));
                        let q = |s: &str| serde_json::to_string(s).unwrap();
                        let code = if lang == "python" {
                            format!(
                                "from pathlib import Path\nn = 99\nint(Path({}).read_text())",
                                q(&input.display().to_string())
                            )
                        } else {
                            format!(
                                "n = 99; Number(require('fs').readFileSync({},'utf8'));",
                                q(&input.display().to_string())
                            )
                        };
                        let prefix=(0..length).map(|i|format!("**p{i} = > foreign.session_eval id=`s` source=\"n += 1; n\"**\n")).collect::<String>();
                        let source=format!("---\nmarkup: occlusion\nimport:\n  foreign: {}\n  fs: {}\n---\n# main\n**s = > foreign.session_open lang={}**\n**> foreign.session_set id=`s` name=\"n\" value={seed}**\n{prefix}**r = > foreign.session_eval id=`s` source={}**\n**> fs.write_text path={} text=\"suffix\"**\n*r*\n",root().join("lib/foreign.mq.md").display(),root().join("lib/fs.mq.md").display(),q(lang),q(&code),q(&marker.display().to_string()));
                        std::fs::write(&path, &source).unwrap();
                        std::fs::write(&input, "1").unwrap();
                        let module = crate::load::load_module(&path).unwrap();
                        let expected = if backend == "tree" {
                            let mut tree =
                                crate::interp::Interpreter::with_capture(Some(&path), false);
                            tree.run_module(&module).unwrap();
                            tree.application_evidence().1
                        } else {
                            let program =
                                crate::bytecode::compile_module(Some(&path), &module).unwrap();
                            let mut vm = crate::bytecode::Vm::with_capture(Some(&path));
                            vm.run(&program).unwrap();
                            vm.application_evidence().1
                        };
                        assert_eq!(std::fs::read_to_string(&marker).unwrap(), "suffix");
                        std::fs::remove_file(&marker).unwrap();
                        let actual_source = if mode == "request" {
                            source.replace(&q(&code), &q(&code.replace("n = 99", "n = 77")))
                        } else {
                            std::fs::write(&input, "2").unwrap();
                            source.clone()
                        };
                        std::fs::write(&path, &actual_source).unwrap();
                        let actual = crate::load::load_module(&path).unwrap();
                        let mut host = HostContext::for_capture(Some(&path), Default::default());
                        host.foreign_replay_expected = Some(expected.clone().into());
                        let (error, observed) = if backend == "tree" {
                            let mut tree =
                                crate::interp::Interpreter::with_capture(Some(&path), false)
                                    .with_host(host);
                            let error = tree.run_module(&actual).err().map(|e| e.to_string());
                            (error, tree.causal_replay_evidence())
                        } else {
                            let program =
                                crate::bytecode::compile_module(Some(&path), &actual).unwrap();
                            let mut vm =
                                crate::bytecode::Vm::with_capture(Some(&path)).with_host(host);
                            let error = vm.run(&program).err().map(|e| e.to_string());
                            (error, vm.causal_replay_evidence())
                        };
                        let record = json!({"kind":"causal-stop","id":id,"language":lang,"prefix":length,"seed":seed,"backend":backend,"mode":mode,"expected":steps(&expected),"observed":observed,"error":error,"suffix_exists":marker.exists(),"source":actual_source,"source_sha256":format!("{:x}",Sha256::digest(actual_source.as_bytes()))});
                        std::fs::write(
                            out.join(format!("{id}.json")),
                            serde_json::to_vec_pretty(&record).unwrap(),
                        )
                        .unwrap();
                        assert!(
                            error.is_some(),
                            "{id}: replay divergence returned a soft result: {record}"
                        );
                        assert!(!marker.exists(), "{id}: host suffix ran after divergence");
                        assert_eq!(
                            observed["n"],
                            if mode == "reply" {
                                json!(99)
                            } else {
                                json!(seed + length as i64)
                            }
                        );
                        assert_eq!(observed["live"], 1);
                        assert_eq!(observed["journal"].as_array().unwrap().len(), length);
                        assert_eq!(
                            observed["protocol"].as_array().unwrap().len(),
                            length + if mode == "reply" { 2 } else { 1 }
                        );
                        count += 1;
                    }
                }
            }
        }
    }
    println!(
        "CAUSAL_METRIC:{}",
        json!({"cases":count,"document_runs":count*2})
    );
}
