//! Transport for the executable Lean policy. This module makes no replay
//! acceptance decisions; malformed/unavailable policy responses fail closed.
use super::super::ProtocolStep;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
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

#[derive(Serialize)]
struct StepWire {
    q: String,
    r: String,
}

// Lean's derived FromJson requires every field, including fields with a Lean
// default. Always send the complete record for init, prepare, receive and finish.
#[derive(Serialize)]
struct CommandWire<'a> {
    op: &'a str,
    steps: &'a [StepWire],
    value: &'a str,
}

#[derive(Debug)]
pub(super) struct Client {
    child: Child,
    input: ChildStdin,
    replies: Receiver<Result<Reply, String>>,
    stderr: Receiver<String>,
}

pub(super) fn request_key(session: &str, request: &Value) -> String {
    serde_json::to_string(&(session, request.to_string())).expect("strings serialize")
}
pub(super) fn response_key(response: &Result<Value, String>) -> String {
    serde_json::to_string(response).expect("protocol responses serialize")
}

fn executable() -> PathBuf {
    if let Some(path) = std::env::var_os("MARQDO_REPLAY_POLICY") {
        return path.into();
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let adjacent = parent.join("marqdo-replay-policy");
            if adjacent.is_file() {
                return adjacent;
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("verification/replay-policy/.lake/build/bin/replayPolicy")
}

impl Client {
    pub(super) fn start(
        steps: impl IntoIterator<Item = ProtocolStep>,
    ) -> Result<(Self, Reply), String> {
        Self::start_at(&executable(), steps)
    }

    fn start_at(
        path: &Path,
        steps: impl IntoIterator<Item = ProtocolStep>,
    ) -> Result<(Self, Reply), String> {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let input = child.stdin.take().ok_or("policy stdin unavailable")?;
        let output = child.stdout.take().ok_or("policy stdout unavailable")?;
        let mut errors = child.stderr.take().ok_or("policy stderr unavailable")?;
        let (sender, replies) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let reply = line
                    .map_err(|e| e.to_string())
                    .and_then(|line| serde_json::from_str(&line).map_err(|e| e.to_string()));
                let failed = reply.is_err();
                if sender.send(reply).is_err() || failed {
                    break;
                }
            }
        });
        // Drain stderr concurrently so a verbose failure cannot block the child.
        // Retain a bounded diagnostic tail rather than buffering an entire stream.
        let (error_sender, stderr) = mpsc::channel();
        std::thread::spawn(move || {
            const LIMIT: usize = 4096;
            let mut tail = Vec::new();
            let mut buffer = [0; LIMIT];
            while let Ok(count) = errors.read(&mut buffer) {
                if count == 0 {
                    break;
                }
                tail.extend_from_slice(&buffer[..count]);
                if tail.len() > LIMIT {
                    tail.drain(..tail.len() - LIMIT);
                }
            }
            let _ = error_sender.send(String::from_utf8_lossy(&tail).trim().to_string());
        });
        let steps: Vec<_> = steps
            .into_iter()
            .map(|step| StepWire {
                q: request_key(&step.session, &step.request),
                r: response_key(&step.response),
            })
            .collect();
        let mut client = Self {
            child,
            input,
            replies,
            stderr,
        };
        let reply = client.send(CommandWire {
            op: "init",
            steps: &steps,
            value: "",
        })?;
        Ok((client, reply))
    }

    pub(super) fn exchange(&mut self, op: &str, value: &str) -> Result<Reply, String> {
        self.send(CommandWire {
            op,
            steps: &[],
            value,
        })
    }

    fn send(&mut self, command: CommandWire<'_>) -> Result<Reply, String> {
        let result = (|| {
            serde_json::to_writer(&mut self.input, &command).map_err(|e| e.to_string())?;
            self.input.write_all(b"\n").map_err(|e| e.to_string())?;
            self.input.flush().map_err(|e| e.to_string())?;
            let reply = self
                .replies
                .recv_timeout(Duration::from_secs(30))
                .map_err(|e| format!("policy response: {e}"))??;
            if reply.policy
                != include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/verification/replay-policy/POLICY-SHA256"
                ))
                .trim()
            {
                return Err("policy executable does not match the compiled source identity".into());
            }
            if reply.allow == reply.reason.is_some() {
                return Err("policy response has inconsistent verdict and reason".into());
            }
            Ok(reply)
        })();
        result.map_err(|error| self.process_error(error))
    }

    fn process_error(&mut self, mut error: String) -> String {
        // Prefer the actual exit status when the process has already failed.
        let status = match self.child.try_wait() {
            Ok(Some(status)) => Some(status),
            _ => {
                let _ = self.child.kill();
                self.child.wait().ok()
            }
        };
        if let Some(status) = status {
            error.push_str(&format!("; policy process {status}"));
        }
        if let Ok(stderr) = self.stderr.recv_timeout(Duration::from_millis(100)) {
            if !stderr.is_empty() {
                error.push_str(&format!("; stderr: {stderr}"));
            }
        }
        error
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn failed_companion_reports_its_exit_status_and_decoder_error() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!(
                "policy-transport-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        std::fs::create_dir_all(&dir).unwrap();
        let executable = dir.join("failed-policy");
        std::fs::write(&executable, "#!/bin/sh\nIFS= read -r command\nprintf '%s\\n' 'PolicyCommand.value: String expected' >&2\nexit 17\n").unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        let error = Client::start_at(&executable, Vec::<ProtocolStep>::new()).unwrap_err();
        std::fs::remove_dir_all(dir).unwrap();
        assert!(error.contains("exit status: 17"), "{error}");
        assert!(
            error.contains("PolicyCommand.value: String expected"),
            "{error}"
        );
    }
}
