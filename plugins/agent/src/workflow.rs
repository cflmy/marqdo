//! Plugin-only durable step workflow.
//! No interpreter-frame serialization: every model/tool decision is a checkpointed
//! transition. This state machine issues actions but never executes arbitrary code.

use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use serde_json::{json, Value};

use crate::{choice, parallel, resume};

const MAX_STEPS: usize = 128;

struct Guard(PathBuf);
impl Drop for Guard {
    fn drop(&mut self) { let _ = fs::remove_file(&self.0); }
}

fn field<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v.get(key).and_then(Value::as_str).filter(|x| !x.is_empty())
        .ok_or_else(|| format!("workflow requires text {key}"))
}

fn safe_key(s: &str) -> bool {
    !s.is_empty() && s.len() <= 100
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn storage(args: &Value) -> Result<PathBuf, String> {
    let rel = args.get("resume_dir").and_then(Value::as_str)
        .unwrap_or(".marqdo/agent-resume");
    let path = PathBuf::from(rel);
    if path.is_absolute() || path.components().any(|part|
        matches!(part, std::path::Component::ParentDir | std::path::Component::Prefix(_)))
    {
        return Err("resume_dir must be relative with no parent traversal".into());
    }
    let base = crate::host_query_json("cwd").ok()
        .and_then(|v| v.as_str().map(PathBuf::from))
        .unwrap_or(std::env::current_dir().map_err(|e| e.to_string())?);
    let dir = base.join(path);
    fs::create_dir_all(&dir).map_err(|e| format!("workflow mkdir: {e}"))?;
    Ok(dir)
}

fn lock(args: &Value, id: &str) -> Result<Guard, String> {
    if !safe_key(id) {
        return Err("workflow id must be <=100 ASCII letters, numbers, - or _".into());
    }
    let path = storage(args)?.join(format!("{id}.flow-lock"));
    OpenOptions::new().write(true).create_new(true).open(&path)
        .map_err(|e| format!("workflow busy or stale lock; review before recovering: {e}"))?;
    Ok(Guard(path))
}

fn checkpoint_args(args: &Value, id: &str) -> Value {
    json!({"id": id, "resume_dir": args.get("resume_dir")})
}

fn read(args: &Value, id: &str) -> Result<Value, String> {
    resume::resume_load(&checkpoint_args(args,id))
}

fn context(args: &Value) -> Value {
    json!({
        "source_revision": args.get("source_revision").cloned().unwrap_or(Value::Null),
        "session_id": args.get("session_id").cloned().unwrap_or(Value::Null),
        "principal_ref": args.get("principal_ref").cloned().unwrap_or(Value::Null)
    })
}

fn check_context(args: &Value, ckpt: &Value) -> bool {
    ckpt["meta"]["identity"] == context(args)
}

fn validate_steps(args: &Value) -> Result<Value, String> {
    let steps = args.get("steps").and_then(Value::as_array)
        .ok_or_else(|| "steps must be a list".to_string())?;
    if steps.is_empty() || steps.len() > MAX_STEPS {
        return Err("steps must contain 1..=128 items".into());
    }
    let mut seen = std::collections::HashSet::new();
    for s in steps {
        let id = field(s, "id")?;
        if !safe_key(id) || !seen.insert(id) {
            return Err("step ids must be unique safe identifiers".into());
        }
        let kind = field(s, "kind")?;
        if !matches!(kind, "model" | "input" | "skill" | "parallel" | "return") {
            return Err(format!("unsupported step kind: {kind}"));
        }
        if matches!(kind, "model" | "input") {
            let options = s.get("options").and_then(Value::as_array)
                .ok_or_else(|| "choice step requires options list".to_string())?;
            let _ = choice::parse_choice(&json!({"reply":"","options":options}))?;
            if kind == "input" { let _ = field(s, "question")?; }
            if kind == "model" { let _ = field(s, "prompt")?; }
        }
        if kind == "skill" { let _ = field(s, "skill")?; }
        if kind == "parallel" && !s.get("tasks").is_some_and(Value::is_array) {
            return Err("parallel step requires tasks array".into());
        }
    }
    // Resolve all branch targets against frozen, forward-only step IDs.
    for (i, s) in steps.iter().enumerate() {
        let targets = s.get("next").and_then(Value::as_str).into_iter()
            .chain(s.get("routes").and_then(Value::as_object)
                .into_iter().flat_map(|m| m.values().filter_map(Value::as_str)));
        for target in targets {
            let position = steps.iter().position(|x|x["id"]==target)
                .ok_or_else(|| format!("unknown workflow target: {target}"))?;
            if position <= i {
                return Err("workflow routes must move forward to avoid loops".into());
            }
        }
        if let Some(routes)=s.get("routes").and_then(Value::as_object) {
            if !matches!(s["kind"].as_str(),Some("model"|"input")) {
                return Err("only choice steps may declare routes".into());
            }
            for key in routes.keys() {
                if !s["options"].as_array().is_some_and(|opts|
                    opts.iter().any(|x|x.as_str()==Some(key.as_str()))) {
                    return Err("route label is not a declared choice".into());
                }
            }
        }
    }
    Ok(Value::Array(steps.clone()))
}

fn advance_cursor(step:&Value, value:&Value, steps:&[Value], current:usize)->Result<usize,String> {
    let target=if let Some(routes)=step.get("routes").and_then(Value::as_object) {
        let label=value.as_str().ok_or("routed choice must return a text label")?;
        Some(routes.get(label).and_then(Value::as_str)
            .ok_or("chosen option is missing a declared route")?)
    } else {
        step.get("next").and_then(Value::as_str)
    };
    match target {
        None=>Ok(current+1),
        Some(name)=> {
            let n=steps.iter().position(|s|s["id"]==name)
                .ok_or("unknown target after checkpoint recovery")?;
            if n<=current {return Err("cyclic route refused".into())}
            Ok(n)
        }
    }
}

fn state(ckpt: &Value) -> Result<(&Value,usize),String> {
    let meta = ckpt.get("meta").ok_or("missing workflow metadata")?;
    if meta.get("kind").and_then(Value::as_str) != Some("MarqdoAgentFlowV1") {
        return Err("checkpoint is not a workflow".into());
    }
    let index = meta.get("cursor").and_then(Value::as_u64)
        .ok_or("missing workflow cursor")? as usize;
    Ok((meta,index))
}

fn save(args: &Value, id: &str, meta: &Value, status: &str) -> Result<(),String> {
    let result = resume::resume_save(&json!({
        "id": id,
        "resume_dir": args.get("resume_dir"),
        "checkpoint_v2": true,
        "status": status,
        "meta": meta,
        "completed_steps": meta["completed"],
        "step_id": meta["steps"].get(meta["cursor"].as_u64().unwrap_or(0) as usize)
            .and_then(|x| x.get("id")).cloned().unwrap_or(Value::Null)
    }))?;
    if result["ok"] != true { return Err("workflow checkpoint write failed".into()); }
    Ok(())
}

/// Register a bounded, frozen plan. Re-opening never resets a progressed plan.
pub fn open(args: &Value) -> Result<Value,String> {
    let id = field(args,"id")?;
    let _guard = lock(args,id)?;
    let revision = field(args,"source_revision")?;
    if revision.len()>256 { return Err("source_revision too long".into()); }
    let steps = validate_steps(args)?;
    let existing = read(args,id)?;
    let identity = context(args);
    if existing["found"] == true {
        if existing["ok"] == true
            && existing["meta"]["identity"] == identity
            && existing["meta"]["steps"] == steps
        {
            return next_doc(&existing,id,true);
        }
        return Ok(json!({"ok":false,"status":"conflict","error":"workflow.id_or_revision_reused"}));
    }
    let meta = json!({
        "kind":"MarqdoAgentFlowV1",
        "identity":identity,
        "steps":steps,
        "cursor":0,
        "completed":[],
    });
    save(args,id,&meta,"running")?;
    Ok(json!({"ok":true,"id":id,"status":"created","cursor":0}))
}

fn next_doc(ckpt: &Value, id: &str, reused: bool) -> Result<Value,String> {
    let (meta,index)=state(ckpt)?;
    let steps = meta["steps"].as_array().ok_or("invalid steps")?;
    if index>=steps.len() {
        return Ok(json!({"ok":true,"id":id,"status":"succeeded",
            "cursor":index,"completed":meta["completed"],"reused":reused}));
    }
    let step=&steps[index];
    let status=ckpt["status"].as_str().unwrap_or("running");
    if status=="blocked" || status=="uncertain" || status=="cancelled" {
        return Ok(json!({"ok":false,"id":id,"status":status,
            "cursor":index,"step":step,"completed":meta["completed"],
            "partial": meta.get("partial").cloned().unwrap_or(Value::Null)}));
    }
    // Never auto-dispatch effectful or unclassified child documents.
    // Those must be executed using the existing EffectLedger policy and an
    // independently verified receipt, not an arbitrary model instruction.
    if (step["kind"]=="skill" || step["kind"]=="parallel")
        && step["effect_class"]!="read"
    {
        return Ok(json!({"ok":false,"id":id,"status":"manual_review",
            "cursor":index,"step":step,
            "reason":"automatic child dispatch requires an explicit read-only effect class"}));
    }
    let waiting=match step["kind"].as_str().unwrap_or("") {
        "model"=>"waiting_model",
        "input"=>"waiting_input",
        "skill" | "parallel"=>"ready_tool",
        "return"=>"ready_return",
        _=>"blocked"
    };
    Ok(json!({"ok":true,"id":id,"status":waiting,
        "cursor":index,"step":step,"completed":meta["completed"],"reused":reused}))
}

/// Query the durable next action. This is non-blocking and does not invoke LLM.
pub fn next(args: &Value) -> Result<Value,String> {
    let id=field(args,"id")?;
    let ckpt=read(args,id)?;
    if ckpt["ok"]!=true || !check_context(args,&ckpt) {
        return Ok(json!({"ok":false,"id":id,"status":"not_found_or_mismatched"}));
    }
    next_doc(&ckpt,id,false)
}

/// Read a workflow summary without advancing it or running any step.
/// The caller-supplied identity must match the checkpoint, exactly like next.
pub fn inspect(args:&Value)->Result<Value,String> {
    let id=field(args,"id")?;
    let ckpt=read(args,id)?;
    if ckpt["ok"]!=true || !check_context(args,&ckpt) {
        return Ok(json!({"ok":false,"id":id,"status":"not_found_or_mismatched"}));
    }
    let (meta,index)=state(&ckpt)?;
    let steps=meta["steps"].as_array().ok_or("invalid steps")?;
    let completed=meta["completed"].as_array().ok_or("invalid completed")?;
    // `status` is the durable checkpoint status; `action` is what tick would
    // observe next (waiting_model, waiting_input, ready_tool, ...).
    let action=next_doc(&ckpt,id,false)?.get("status").cloned().unwrap_or(Value::Null);
    Ok(json!({
        "ok":true,"id":id,
        "status":ckpt["status"].as_str().unwrap_or("running"),
        "action":action,
        "cursor":index,
        "total_steps":steps.len(),
        "completed_steps":completed.len(),
        "completed":meta["completed"],
        "step":steps.get(index).cloned().unwrap_or(Value::Null),
        "partial":meta.get("partial").cloned().unwrap_or(Value::Null),
        "identity":meta["identity"],
        "updated_at":ckpt.get("updated_at").cloned().unwrap_or(Value::Null)
    }))
}

/// List workflow checkpoints in the (host-relative) resume directory.
/// Non-workflow checkpoints and corrupt entries are omitted. When the caller
/// supplies an attested session or principal, only matching runs are returned.
pub fn list(args:&Value)->Result<Value,String> {
    let dir=storage(args)?;
    let session=args.get("session_id").and_then(Value::as_str);
    let principal=args.get("principal_ref").and_then(Value::as_str);
    let mut items=Vec::new();
    let rd=fs::read_dir(&dir).map_err(|e|format!("workflow list: {e}"))?;
    for ent in rd.flatten() {
        let path=ent.path();
        if path.extension().and_then(|x|x.to_str())!=Some("json") { continue; }
        if path.file_name().and_then(|x|x.to_str())
            .map(|x|x.starts_with('.')).unwrap_or(false) { continue; }
        let Ok(text)=fs::read_to_string(&path) else { continue };
        let Ok(doc)=serde_json::from_str::<Value>(&text) else { continue };
        let meta=&doc["meta"];
        if meta["kind"].as_str()!=Some("MarqdoAgentFlowV1") { continue; }
        if session.is_some() && meta["identity"]["session_id"].as_str()!=session { continue; }
        if principal.is_some() && meta["identity"]["principal_ref"].as_str()!=principal { continue; }
        let steps=meta["steps"].as_array().map(|x|x.len()).unwrap_or(0);
        let completed=meta["completed"].as_array().map(|x|x.len()).unwrap_or(0);
        items.push(json!({
            "id":doc.get("id").cloned().unwrap_or(Value::Null),
            "status":doc.get("status").cloned().unwrap_or(Value::Null),
            "cursor":meta.get("cursor").cloned().unwrap_or(Value::Null),
            "total_steps":steps,
            "completed_steps":completed,
            "source_revision":meta["identity"]["source_revision"],
            "updated_at":doc.get("updated_at").cloned().unwrap_or(Value::Null)
        }));
    }
    items.sort_by(|a,b| a["id"].as_str().cmp(&b["id"].as_str()));
    Ok(json!({"ok":true,"items":items,"resume_dir":dir.to_string_lossy()}))
}

/// Commit a verified result for the exact current step. All mutations are
/// serialized with a lock; a completed step cannot be committed a second time.
pub fn commit(args: &Value) -> Result<Value,String> {
    let id=field(args,"id")?;
    let _guard=lock(args,id)?;
    let step_id=field(args,"step_id")?;
    let ckpt=read(args,id)?;
    if ckpt["ok"]!=true || !check_context(args,&ckpt) {
        return Ok(json!({"ok":false,"status":"not_found_or_mismatched"}));
    }
    let (meta,index)=state(&ckpt)?;
    let steps=meta["steps"].as_array().ok_or("invalid steps")?;
    let response=args.get("response").ok_or("missing response")?;
    if index>=steps.len() {
        let previous = meta["completed"].as_array().ok_or("invalid completed")?;
        if previous.iter().any(|x|x["step_id"]==step_id && x["value"]==*response) {
            return Ok(json!({"ok":true,"status":"succeeded","reused":true}));
        }
        return Ok(json!({"ok":false,"status":"already_finished"}));
    }
    let step=&steps[index];
    if step["id"]!=step_id {
        if meta["completed"].as_array().is_some_and(|list|
            list.iter().any(|x|x["step_id"]==step_id && x["value"]==*response))
        {
            return Ok(json!({"ok":true,"status":"already_committed","reused":true}));
        }
        return Ok(json!({"ok":false,"status":"step_conflict"}));
    }
    if ckpt["status"]!="running" {
        return Ok(json!({"ok":false,"status":"blocked","reason":ckpt["status"]}));
    }
    let kind=field(step,"kind")?;
    let value = match kind {
        "model" | "input" => {
            let reply = response.as_str().ok_or("model/input response must be text")?;
            let valid=choice::parse_choice(&json!({"options":step["options"],"reply":reply}))?;
            if valid["ok"]!=true {
                return Ok(json!({"ok":false,"status":"invalid_choice",
                    "error":valid["error"]}));
            }
            valid["value"].clone()
        }
        "skill" => {
            // A completed wait is not equivalent to a successful child process.
            let exit = response.get("value").and_then(|v| v.get("code"))
                .or_else(|| response.get("code")).and_then(Value::as_i64);
            if response["ok"]==false || exit != Some(0) {
                return Ok(json!({"ok":false,
                    "status":if exit.is_none() {"uncertain"} else {"failed"},
                    "reason":"child process exit code zero is required before commit"}));
            }
            response.clone()
        }
        "parallel" => {
            let report=if response["schema"]=="AgentParallelResultV1" {
                response.clone()
            } else {
                parallel::report(&json!({"results":response}))?
            };
            if report["ok"]!=true {
                let status=if report["uncertain"].as_u64().unwrap_or(0)>0 {
                    "uncertain"
                } else {"blocked"};
                // Save the full manifest of successful and failed siblings before
                // returning. Do not run failed/unknown children automatically.
                let mut frozen=meta.clone();
                frozen["partial"]=report.clone();
                save(args,id,&frozen,status)?;
                return Ok(json!({"ok":false,"status":status,
                    "report":report,"reason":"child results retained for reconciliation"}));
            }
            report
        }
        "return" => response.clone(),
        _ => return Err("invalid step kind".into()),
    };
    let target=advance_cursor(step,&value,steps,index)?;
    let mut new_meta=meta.clone();
    new_meta["cursor"]=json!(target);
    new_meta["completed"].as_array_mut().ok_or("completed not array")?
        .push(json!({"step_id":step_id,"value":value}));
    let terminal=target==steps.len();
    save(args,id,&new_meta,if terminal {"succeeded"} else {"running"})?;
    Ok(json!({"ok":true,"id":id,
        "status":if terminal {"succeeded"} else {"advanced"},
        "cursor":target,"value":value,"reused":false}))
}

/// Produce a plan containing only previously failed *read-only* child tasks.
/// "uncertain" children and effectful plans cannot be blindly retried.
pub fn retry_plan(args: &Value) -> Result<Value,String> {
    let id=field(args,"id")?;
    let ckpt=read(args,id)?;
    if ckpt["ok"]!=true || !check_context(args,&ckpt) {
        return Ok(json!({"ok":false,"status":"not_found_or_mismatched"}));
    }
    let (meta,index)=state(&ckpt)?;
    if ckpt["status"]!="blocked" {
        return Ok(json!({"ok":false,"status":"not_retryable"}));
    }
    let step=&meta["steps"][index];
    if step["kind"]!="parallel" || step["effect_class"]!="read"
        || meta["partial"]["uncertain"].as_u64().unwrap_or(1)!=0
    {
        return Ok(json!({"ok":false,"status":"manual_reconciliation_required"}));
    }
    let Some(items)=meta["partial"]["items"].as_array() else {
        return Err("partial report has no items".into());
    };
    let Some(tasks)=step["tasks"].as_array() else {
        return Err("parallel plan has no tasks".into());
    };
    if items.len()!=tasks.len() {
        return Err("partial report task count mismatch".into());
    }
    let mut failed=Vec::new();
    let mut indices=Vec::new();
    for (i,item) in items.iter().enumerate() {
        if item["index"]!=i {
            return Err("out-of-order child result index".into());
        }
        if item["status"]=="failed" {
            failed.push(tasks[i].clone());
            indices.push(i);
        } else if item["status"]!="succeeded" {
            return Ok(json!({"ok":false,"status":"manual_reconciliation_required"}));
        }
    }
    Ok(json!({"ok":true,"status":"retry_read_only",
        "id":id,"step_id":step["id"],"tasks":failed,"indices":indices}))
}

/// Merge only the failed child slots with verified new outcomes.
/// Successful results and their order remain untouched.
pub fn retry_commit(args: &Value) -> Result<Value,String> {
    let id=field(args,"id")?;
    let _guard=lock(args,id)?;
    let ckpt=read(args,id)?;
    if ckpt["ok"]!=true || !check_context(args,&ckpt) {
        return Ok(json!({"ok":false,"status":"not_found_or_mismatched"}));
    }
    let (meta,index)=state(&ckpt)?;
    if ckpt["status"]!="blocked" {
        return Ok(json!({"ok":false,"status":"not_retryable"}));
    }
    let step=&meta["steps"][index];
    if step["kind"]!="parallel" || step["effect_class"]!="read"
        || meta["partial"]["uncertain"].as_u64().unwrap_or(1)!=0
    {
        return Ok(json!({"ok":false,"status":"manual_reconciliation_required"}));
    }
    let items=meta["partial"]["items"].as_array()
        .ok_or("invalid partial report")?;
    let incoming=args.get("results").and_then(Value::as_array)
        .ok_or("retry_commit requires results list")?;
    let expected=items.iter().filter(|it|it["status"]=="failed").count();
    if incoming.len()!=expected {
        return Err("retry result count differs from failed child count".into());
    }
    let mut next=Vec::with_capacity(items.len());
    let mut replacement=incoming.iter();
    for item in items {
        if item["status"]=="succeeded" {
            next.push(item["result"].clone());
        } else if item["status"]=="failed" {
            next.push(replacement.next().ok_or("missing retry result")?.clone());
        } else {
            return Ok(json!({"ok":false,"status":"manual_reconciliation_required"}));
        }
    }
    let report=parallel::report(&json!({"results":next}))?;
    let mut updated=meta.clone();
    if report["ok"]==true {
        let target=advance_cursor(step,&report,
            updated["steps"].as_array().ok_or("steps missing")?,index)?;
        updated["cursor"]=json!(target);
        updated["partial"]=Value::Null;
        updated["completed"].as_array_mut().ok_or("completed not list")?
            .push(json!({"step_id":step["id"],"value":report}));
        let finished=target==updated["steps"].as_array().ok_or("steps missing")?.len();
        save(args,id,&updated,if finished {"succeeded"} else {"running"})?;
        Ok(json!({"ok":true,"status":if finished {"succeeded"} else {"advanced"},
            "cursor":target,"result":report}))
    } else {
        updated["partial"]=report.clone();
        save(args,id,&updated,if report["uncertain"].as_u64().unwrap_or(0)>0 {
            "uncertain" } else {"blocked"})?;
        Ok(json!({"ok":false,"status":"blocked","report":report}))
    }
}

pub fn cancel(args: &Value) -> Result<Value,String> {
    let id=field(args,"id")?;
    let _guard=lock(args,id)?;
    let ckpt=read(args,id)?;
    if ckpt["ok"]!=true || !check_context(args,&ckpt) {
        return Ok(json!({"ok":false,"status":"not_found_or_mismatched"}));
    }
    if ckpt["status"]=="succeeded" {
        return Ok(json!({"ok":false,"status":"already_finished"}));
    }
    let (meta,_)=state(&ckpt)?;
    save(args,id,meta,"cancelled")?;
    Ok(json!({"ok":true,"id":id,"status":"cancelled"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64,Ordering};
    static SEQ: AtomicU64=AtomicU64::new(0);
    fn fixture()->Value {
        let n=SEQ.fetch_add(1,Ordering::Relaxed);
        json!({"id":format!("wf-{}-{n}",std::process::id()),
            "resume_dir":format!("target/workflow-test-{}-{n}",std::process::id()),
            "source_revision":"v1",
            "steps":[
                {"id":"choose","kind":"model","prompt":"Choose a skill","options":["chat","office"]},
                {"id":"answer","kind":"input","question":"OK?","options":["yes","no"]},
                {"id":"end","kind":"return"}
            ]})
    }

    #[test]
    fn nonblocking_model_then_input_then_terminal() {
        let a=fixture();
        assert_eq!(open(&a).unwrap()["status"],"created");
        assert_eq!(next(&a).unwrap()["status"],"waiting_model");
        let mut c=a.clone();
        c["step_id"]=json!("choose");
        c["response"]=json!("office");
        assert_eq!(commit(&c).unwrap()["status"],"advanced");
        assert_eq!(next(&a).unwrap()["status"],"waiting_input");
        assert_eq!(commit(&c).unwrap()["status"],"already_committed");
        c["step_id"]=json!("answer");
        c["response"]=json!("yes");
        assert_eq!(commit(&c).unwrap()["status"],"advanced");
        c["step_id"]=json!("end");
        c["response"]=json!("ok");
        assert_eq!(commit(&c).unwrap()["status"],"succeeded");
        assert_eq!(next(&a).unwrap()["status"],"succeeded");
        assert_eq!(open(&a).unwrap()["status"],"succeeded");
    }

    #[test]
    fn unknown_and_write_effects_never_auto_dispatch() {
        let mut a=fixture();
        a["steps"]=json!([{"id":"publish","kind":"skill","skill":"office"}]);
        open(&a).unwrap();
        assert_eq!(next(&a).unwrap()["status"],"manual_review");
    }

    #[test]
    fn routes_skip_unselected_skill_branches() {
        let mut a=fixture();
        a["steps"]=json!([
            {"id":"decide","kind":"model","prompt":"Pick","options":["chat","office"],
              "routes":{"chat":"chat_skill","office":"office_skill"}},
            {"id":"chat_skill","kind":"skill","skill":"chat","effect_class":"read","next":"done"},
            {"id":"office_skill","kind":"skill","skill":"office","effect_class":"read","next":"done"},
            {"id":"done","kind":"return"}
        ]);
        open(&a).unwrap();
        let mut c=a.clone();
        c["step_id"]=json!("decide");
        c["response"]=json!("office");
        assert_eq!(commit(&c).unwrap()["cursor"],2);
        assert_eq!(next(&a).unwrap()["step"]["id"],"office_skill");
        c["step_id"]=json!("office_skill");
        c["response"]=json!({"ok":true,"value":{"code":0}});
        assert_eq!(commit(&c).unwrap()["cursor"],3);
        assert_eq!(next(&a).unwrap()["step"]["id"],"done");
    }

    #[test]
    fn source_change_conflict_and_invalid_model_choice() {
        let a=fixture();
        open(&a).unwrap();
        let mut bad=a.clone();
        bad["source_revision"]=json!("v2");
        assert_eq!(next(&bad).unwrap()["status"],"not_found_or_mismatched");
        assert_eq!(open(&bad).unwrap()["status"],"conflict");
        let mut c=a.clone();
        c["step_id"]=json!("choose");
        c["response"]=json!("chat or office");
        assert_eq!(commit(&c).unwrap()["status"],"invalid_choice");
        assert_eq!(next(&a).unwrap()["status"],"waiting_model");
    }

    #[test]
    fn partial_children_remain_durable_and_unknown_requires_reconciliation() {
        let mut a=fixture();
        a["steps"]=json!([{"id":"batch","kind":"parallel","tasks":[{"skill":"a"},{"skill":"b"}]}]);
        open(&a).unwrap();
        let mut c=a.clone();
        c["step_id"]=json!("batch");
        c["response"]=json!([
            {"ok":true,"value":{"code":0,"value":"useful"}},
            {"ok":false,"error":"connection lost"}
        ]);
        assert_eq!(commit(&c).unwrap()["status"],"blocked");
        let after=next(&a).unwrap();
        assert_eq!(after["status"],"blocked");
        assert_eq!(after["partial"]["succeeded"],1);
        assert_eq!(after["partial"]["failed"],1);
        assert_eq!(commit(&c).unwrap()["status"],"blocked");
    }

    #[test]
    fn retries_only_failed_read_only_children_without_redoing_successes() {
        let mut a=fixture();
        a["steps"]=json!([{"id":"batch","kind":"parallel","effect_class":"read",
            "tasks":[{"path":"a.mq.md"},{"path":"b.mq.md"}]}]);
        open(&a).unwrap();
        let mut c=a.clone();
        c["step_id"]=json!("batch");
        c["response"]=json!([
            {"ok":true,"value":{"code":0,"value":"keep"}},
            {"ok":false,"error":"temporary timeout"}
        ]);
        assert_eq!(commit(&c).unwrap()["status"],"blocked");
        let plan=retry_plan(&a).unwrap();
        assert_eq!(plan["indices"],json!([1]));
        assert_eq!(plan["tasks"].as_array().unwrap().len(),1);
        let mut retry=a.clone();
        retry["results"]=json!([{"ok":true,"value":{"code":0,"value":"fixed"}}]);
        assert_eq!(retry_commit(&retry).unwrap()["status"],"succeeded");
        let out=next(&a).unwrap();
        assert_eq!(out["completed"][0]["value"]["items"][0]["result"]["value"]["value"],"keep");
        assert_eq!(out["completed"][0]["value"]["items"][1]["result"]["value"]["value"],"fixed");
    }

    #[test]
    fn inspect_and_list_are_read_only_and_session_scoped() {
        let mut a=fixture();
        a["session_id"]=json!("s1");
        a["principal_ref"]=json!("u1");
        open(&a).unwrap();
        let seen=inspect(&a).unwrap();
        assert_eq!(seen["status"],"running");
        assert_eq!(seen["action"],"waiting_model");
        assert_eq!(seen["total_steps"],3);
        assert_eq!(seen["completed_steps"],0);
        assert_eq!(seen["step"]["id"],"choose");
        // Inspection must not consume the current step.
        assert_eq!(next(&a).unwrap()["status"],"waiting_model");
        let listed=list(&a).unwrap();
        assert!(listed["items"].as_array().unwrap()
            .iter().any(|x|x["id"]==a["id"]));
        let mut other=a.clone();
        other["session_id"]=json!("s2");
        assert_eq!(inspect(&other).unwrap()["status"],"not_found_or_mismatched");
        assert!(list(&other).unwrap()["items"].as_array().unwrap().is_empty());
    }

    #[test]
    fn explicit_cancel_stops_future_commits() {
        let a=fixture();
        open(&a).unwrap();
        assert_eq!(cancel(&a).unwrap()["status"],"cancelled");
        let mut c=a.clone();
        c["step_id"]=json!("choose");
        c["response"]=json!("chat");
        assert_eq!(commit(&c).unwrap()["status"],"blocked");
    }
}
