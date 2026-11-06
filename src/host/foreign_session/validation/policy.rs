//! Transport for the executable Lean policy. This module makes no replay
//! acceptance decisions; malformed/unavailable policy responses fail closed.
use super::super::ProtocolStep;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Reply {
    pub policy: String,
    pub allow: bool,
    pub matched: usize,
    pub remaining: usize,
    pub reason: Option<String>,
}

#[derive(Debug)]
pub(super) struct Client {
    child: Child,
    input: ChildStdin,
    replies: Receiver<Result<Reply,String>>,
}

pub(super) fn request_key(session: &str, request: &Value) -> String {
    serde_json::to_string(&(session,request.to_string())).expect("strings serialize")
}
pub(super) fn response_key(response: &Result<Value,String>) -> String {
    serde_json::to_string(response).expect("protocol responses serialize")
}

fn executable() -> PathBuf {
    if let Some(path) = std::env::var_os("MARQDO_REPLAY_POLICY") { return path.into() }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let adjacent = parent.join("marqdo-replay-policy");
            if adjacent.is_file() { return adjacent }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("verification/replay-policy/.lake/build/bin/replayPolicy")
}

impl Client {
    pub(super) fn start(steps: impl IntoIterator<Item=ProtocolStep>) -> Result<(Self,Reply),String> {
        let path=executable();
        let mut child=Command::new(&path).stdin(Stdio::piped()).stdout(Stdio::piped())
            .stderr(Stdio::null()).spawn().map_err(|e|format!("{}: {e}",path.display()))?;
        let input=child.stdin.take().ok_or("policy stdin unavailable")?;
        let output=child.stdout.take().ok_or("policy stdout unavailable")?;
        let (sender,replies)=mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let reply=line.map_err(|e|e.to_string())
                    .and_then(|line|serde_json::from_str(&line).map_err(|e|e.to_string()));
                let failed=reply.is_err();
                if sender.send(reply).is_err() || failed { break }
            }
        });
        let steps: Vec<_>=steps.into_iter().map(|step|json!({
            "q":request_key(&step.session,&step.request),"r":response_key(&step.response)
        })).collect();
        let mut client=Self {child,input,replies};
        let reply=client.exchange(json!({"op":"init","steps":steps}))?;
        Ok((client,reply))
    }

    pub(super) fn exchange(&mut self, command: Value) -> Result<Reply,String> {
        serde_json::to_writer(&mut self.input,&command).map_err(|e|e.to_string())?;
        self.input.write_all(b"\n").map_err(|e|e.to_string())?;
        self.input.flush().map_err(|e|e.to_string())?;
        let reply=self.replies.recv_timeout(Duration::from_secs(30))
            .map_err(|e|format!("policy response: {e}"))??;
        if reply.policy != include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/verification/replay-policy/POLICY-SHA256")).trim() {
            return Err("policy executable does not match the compiled source identity".into())
        }
        if reply.allow == reply.reason.is_some() {
            return Err("policy response has inconsistent verdict and reason".into())
        }
        Ok(reply)
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        let _=self.child.kill();
        let _=self.child.wait();
    }
}
