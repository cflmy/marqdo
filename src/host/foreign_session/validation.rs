//! Production adapter to the executable Lean replay policy.
//! All replay decisions come from ReplayPolicy.Cursor, with no Rust fallback.
use super::ProtocolStep;
use serde_json::Value;
use std::collections::VecDeque;
mod policy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivergenceKind { Missing, Request, Reply, Surplus, Phase, Policy }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divergence {
    pub kind: DivergenceKind,
    pub matched_prefix: usize,
    pub detail: Option<String>,
}
impl std::fmt::Display for Divergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.kind {
            DivergenceKind::Missing | DivergenceKind::Request => "replay: protocol request differs before dispatch",
            DivergenceKind::Reply => "replay: protocol response differs",
            DivergenceKind::Surplus => "replay: recorded execution prefix was not consumed",
            DivergenceKind::Phase => "replay: invalid protocol validation phase",
            DivergenceKind::Policy => "replay: verified policy unavailable",
        })?;
        if let Some(detail) = &self.detail { write!(f, ": {detail}")? }
        Ok(())
    }
}
#[derive(Debug)]
pub struct ReplayCursor {
    client: Option<policy::Client>,
    progress: (usize, usize),
    stopped: Option<Divergence>,
}
impl From<VecDeque<ProtocolStep>> for ReplayCursor {
    fn from(expected: VecDeque<ProtocolStep>) -> Self {
        let count = expected.len();
        match policy::Client::start(expected) {
            Ok((client, reply)) if reply.allow && reply.matched == 0 && reply.remaining == count =>
                Self { client: Some(client), progress: (0, count), stopped: None },
            result => {
                let detail = match result { Err(e) => e, Ok(_) => "invalid policy initialization".into() };
                Self { client: None, progress: (0,count), stopped: Some(Divergence {
                    kind: DivergenceKind::Policy, matched_prefix: 0, detail: Some(detail) }) }
            }
        }
    }
}
impl From<Vec<ProtocolStep>> for ReplayCursor {
    fn from(expected: Vec<ProtocolStep>) -> Self { VecDeque::from(expected).into() }
}
/// Private construction, no Clone: actual dispatch consumes this snapshot.
pub(super) struct DispatchPermit { session: String, request: Value }
impl DispatchPermit {
    pub(super) fn session(&self) -> &str { &self.session }
    pub(super) fn request(&self) -> &Value { &self.request }
}
impl ReplayCursor {
    fn unavailable(&mut self, detail: String) -> Divergence {
        self.client.take();
        let error = Divergence { kind: DivergenceKind::Policy, matched_prefix: self.progress.0, detail: Some(detail) };
        self.stopped = Some(error.clone());
        error
    }
    fn command(&mut self, op: &str, value: &str) -> Result<(), Divergence> {
        if let Some(error) = &self.stopped { return Err(error.clone()) }
        let result = self.client.as_mut().ok_or_else(|| "no policy process".to_string())
            .and_then(|client| client.exchange(op, value));
        let reply = result.map_err(|e| self.unavailable(e))?;
        self.progress = (reply.matched, reply.remaining);
        if reply.allow { return Ok(()) }
        let kind = match reply.reason.as_deref() {
            Some("missing") => DivergenceKind::Missing,
            Some("request") => DivergenceKind::Request,
            Some("reply") => DivergenceKind::Reply,
            Some("surplus") => DivergenceKind::Surplus,
            Some("phase") => DivergenceKind::Phase,
            _ => return Err(self.unavailable("unknown policy stop reason".into())),
        };
        let error = Divergence { kind, matched_prefix: reply.matched, detail: None };
        self.stopped = Some(error.clone());
        Err(error)
    }
    fn prepare(&mut self, session: &str, request: &Value) -> Result<DispatchPermit, Divergence> {
        self.command("prepare", &policy::request_key(session,request))?;
        Ok(DispatchPermit { session: session.into(), request: request.clone() })
    }
    pub(super) fn accept_response(&mut self, response: &Result<Value,String>) -> Result<(), Divergence> {
        self.command("receive", &policy::response_key(response))
    }
    pub fn progress(&self) -> (usize,usize) { self.progress }
    /// Called only at the outer action boundary; consumes the cursor.
    pub fn finish(mut self) -> Result<(),Divergence> { self.command("finish", "") }
}
pub(super) fn prepare(cursor: Option<&mut ReplayCursor>, session: &str, request: &Value)
    -> Result<DispatchPermit,Divergence> {
    if let Some(cursor) = cursor { return cursor.prepare(session,request) }
    Ok(DispatchPermit { session: session.into(), request: request.clone() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn step(reply: Result<Value, String>) -> ProtocolStep {
        ProtocolStep {
            session: "s".into(),
            request: json!({"op":"get"}),
            response: reply,
        }
    }
    #[test]
    fn local_returns_and_ordinary_failures_preserve_residual_history() {
        let mut cursor: ReplayCursor =
            vec![step(Err("foreign failure".into())), step(Ok(json!(7)))].into();
        assert_eq!(cursor.progress(), (0, 2));
        let permit = cursor.prepare("s", &json!({"op":"get"})).unwrap();
        assert_eq!(permit.session(), "s");
        cursor
            .accept_response(&Err("foreign failure".into()))
            .unwrap();
        assert_eq!(cursor.progress(), (1, 1));
        cursor.prepare("s", &json!({"op":"get"})).unwrap();
        cursor.accept_response(&Ok(json!(7))).unwrap();
        cursor.finish().unwrap();
    }
    #[test]
    fn divergence_is_sticky_and_matching_reply_cannot_revive_it() {
        let mut cursor: ReplayCursor = vec![step(Ok(json!(7)))].into();
        cursor.prepare("s", &json!({"op":"get"})).unwrap();
        let first = cursor.accept_response(&Ok(json!(8))).unwrap_err();
        assert_eq!(first.kind, DivergenceKind::Reply);
        assert_eq!(
            cursor.prepare("s", &json!({"op":"get"})).err().unwrap(),
            first
        );
        assert_eq!(cursor.accept_response(&Ok(json!(7))).unwrap_err(), first);
        assert_eq!(cursor.finish().unwrap_err(), first);
    }
    #[test]
    fn permit_is_a_snapshot_and_phase_errors_fail_closed() {
        let mut request = json!({"op":"get"});
        let mut cursor: ReplayCursor = vec![step(Ok(json!(7)))].into();
        let permit = cursor.prepare("s", &request).unwrap();
        request["op"] = json!("write");
        assert_ne!(permit.request(), &request);
        assert_eq!(
            cursor.prepare("s", permit.request()).err().unwrap().kind,
            DivergenceKind::Phase
        );
        assert!(cursor.accept_response(&Ok(json!(7))).is_err());
    }
    #[test]
    fn mismatching_and_missing_requests_never_issue_a_permit() {
        let mut cursor: ReplayCursor = vec![step(Ok(json!(7)))].into();
        let error = cursor
            .prepare("other-session", &json!({"op":"get"}))
            .err()
            .unwrap();
        assert_eq!(error.kind, DivergenceKind::Request);
        assert_eq!(cursor.progress(), (0, 1));
        assert_eq!(
            cursor.prepare("s", &json!({"op":"get"})).err().unwrap(),
            error
        );
        let mut empty: ReplayCursor = Vec::<ProtocolStep>::new().into();
        assert_eq!(
            empty.prepare("s", &json!({"op":"get"})).err().unwrap().kind,
            DivergenceKind::Missing
        );
        assert!(empty.finish().is_err());
    }
}
