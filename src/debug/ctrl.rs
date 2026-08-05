//! Interactive debug controller for tree-walk (view + tests).

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugAction {
    Continue,
    Step,
    Stop,
}

#[derive(Debug, Clone)]
pub struct DebugPause {
    pub line: u32,
    pub fun: String,
    pub locals: Vec<(String, String)>,
    pub stdout: String,
}

#[derive(Debug, Clone)]
pub enum DebugSnapshot {
    Idle,
    Running,
    Paused(DebugPause),
    Done {
        ok: bool,
        stdout: String,
        stderr: String,
    },
}

struct Gate {
    action: Mutex<Option<DebugAction>>,
    cv: Condvar,
}

/// Shared between the interpreter thread and HTTP/CLI control.
pub struct DebugController {
    breakpoints: Mutex<HashSet<u32>>,
    /// Pause at the next statement (step).
    step_once: AtomicBool,
    stop: AtomicBool,
    /// Skip re-breaking on this line once after continue-from-pause (same entry).
    suppress_line: Mutex<Option<u32>>,
    gate: Gate,
    snapshot: Mutex<DebugSnapshot>,
    /// Notifies waiters that snapshot changed.
    snap_cv: Condvar,
}

impl DebugController {
    pub fn new(breakpoints: HashSet<u32>) -> Arc<Self> {
        Arc::new(Self {
            breakpoints: Mutex::new(breakpoints),
            step_once: AtomicBool::new(false),
            stop: AtomicBool::new(false),
            suppress_line: Mutex::new(None),
            gate: Gate {
                action: Mutex::new(None),
                cv: Condvar::new(),
            },
            snapshot: Mutex::new(DebugSnapshot::Idle),
            snap_cv: Condvar::new(),
        })
    }

    pub fn set_breakpoints(&self, lines: HashSet<u32>) {
        *self.breakpoints.lock().unwrap() = lines;
    }

    /// Pause at the next statement (used when starting with no breakpoints).
    pub fn request_step(&self) {
        self.step_once.store(true, Ordering::SeqCst);
    }

    pub fn breakpoints(&self) -> HashSet<u32> {
        self.breakpoints.lock().unwrap().clone()
    }

    pub fn snapshot(&self) -> DebugSnapshot {
        self.snapshot.lock().unwrap().clone()
    }

    pub fn set_running(&self) {
        *self.snapshot.lock().unwrap() = DebugSnapshot::Running;
        self.snap_cv.notify_all();
    }

    pub fn set_done(&self, ok: bool, stdout: String, stderr: String) {
        *self.snapshot.lock().unwrap() = DebugSnapshot::Done {
            ok,
            stdout,
            stderr,
        };
        self.snap_cv.notify_all();
        // Unblock any waiter in on_stmt
        self.send_action(DebugAction::Stop);
    }

    pub fn send_action(&self, action: DebugAction) {
        // Clear stale Paused so wait_until_* does not return the previous pause.
        {
            let mut snap = self.snapshot.lock().unwrap();
            if matches!(&*snap, DebugSnapshot::Paused(_)) {
                *snap = DebugSnapshot::Running;
                self.snap_cv.notify_all();
            }
        }
        let mut g = self.gate.action.lock().unwrap();
        *g = Some(action);
        self.gate.cv.notify_all();
    }

    /// Wait until snapshot is Paused or Done (or timeout).
    pub fn wait_until_paused_or_done(&self, timeout: Duration) -> DebugSnapshot {
        let mut snap = self.snapshot.lock().unwrap();
        let deadline = std::time::Instant::now() + timeout;
        loop {
            match &*snap {
                DebugSnapshot::Paused(_) | DebugSnapshot::Done { .. } => return snap.clone(),
                _ => {}
            }
            let now = std::time::Instant::now();
            if now >= deadline {
                return snap.clone();
            }
            let (s, _) = self
                .snap_cv
                .wait_timeout(snap, deadline - now)
                .unwrap();
            snap = s;
        }
    }

    /// Called at the start of each statement.
    pub fn on_stmt(
        &self,
        line: u32,
        fun: &str,
        locals: &HashMap<String, Value>,
        stdout: &str,
    ) -> Result<(), String> {
        if self.stop.load(Ordering::SeqCst) {
            return Err("debug stopped".into());
        }

        let step = self.step_once.swap(false, Ordering::SeqCst);
        let is_bp = self.breakpoints.lock().unwrap().contains(&line);
        let suppressed = *self.suppress_line.lock().unwrap() == Some(line);
        if suppressed {
            *self.suppress_line.lock().unwrap() = None;
        }
        let should_pause = step || (is_bp && !suppressed);
        if !should_pause {
            return Ok(());
        }

        let mut locals_v: Vec<(String, String)> = locals
            .iter()
            .map(|(k, v)| (k.clone(), v.as_display()))
            .collect();
        locals_v.sort_by(|a, b| a.0.cmp(&b.0));

        {
            let mut snap = self.snapshot.lock().unwrap();
            *snap = DebugSnapshot::Paused(DebugPause {
                line,
                fun: fun.to_string(),
                locals: locals_v,
                stdout: stdout.to_string(),
            });
            self.snap_cv.notify_all();
        }

        // Wait for Continue / Step / Stop
        let mut guard = self.gate.action.lock().unwrap();
        loop {
            while guard.is_none() {
                guard = self.gate.cv.wait(guard).unwrap();
            }
            let action = guard.take().unwrap();
            match action {
                DebugAction::Continue => {
                    // Don't immediately re-break on this same line entry.
                    *self.suppress_line.lock().unwrap() = Some(line);
                    return Ok(());
                }
                DebugAction::Step => {
                    self.step_once.store(true, Ordering::SeqCst);
                    *self.suppress_line.lock().unwrap() = Some(line);
                    return Ok(());
                }
                DebugAction::Stop => {
                    self.stop.store(true, Ordering::SeqCst);
                    return Err("debug stopped".into());
                }
            }
        }
    }
}

/// Serialize snapshot for JSON APIs.
pub fn snapshot_json(s: &DebugSnapshot) -> serde_json::Value {
    match s {
        DebugSnapshot::Idle => serde_json::json!({ "status": "idle" }),
        DebugSnapshot::Running => serde_json::json!({ "status": "running" }),
        DebugSnapshot::Paused(p) => {
            let locals: serde_json::Map<String, serde_json::Value> = p
                .locals
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            serde_json::json!({
                "status": "paused",
                "line": p.line,
                "fun": p.fun,
                "locals": locals,
                "stdout": p.stdout,
            })
        }
        DebugSnapshot::Done {
            ok,
            stdout,
            stderr,
        } => serde_json::json!({
            "status": "done",
            "ok": ok,
            "stdout": stdout,
            "stderr": stderr,
        }),
    }
}

/// Channel pair helper if needed by callers.
#[allow(dead_code)]
pub fn action_channel() -> (Sender<DebugAction>, Receiver<DebugAction>) {
    mpsc::channel()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn pause_continue_and_locals() {
        let ctrl = DebugController::new(HashSet::from([2]));
        let c2 = ctrl.clone();
        let handle = thread::spawn(move || {
            let mut locals = HashMap::new();
            locals.insert("x".into(), Value::Int(1));
            c2.on_stmt(1, "main", &locals, "").unwrap(); // no bp
            locals.insert("x".into(), Value::Int(2));
            c2.on_stmt(2, "main", &locals, "hi\n").unwrap(); // pauses
            locals.insert("x".into(), Value::Int(3));
            c2.on_stmt(3, "main", &locals, "hi\n").unwrap();
            c2.set_done(true, "hi\n".into(), String::new());
        });

        let snap = ctrl.wait_until_paused_or_done(Duration::from_secs(2));
        match snap {
            DebugSnapshot::Paused(p) => {
                assert_eq!(p.line, 2);
                assert_eq!(p.fun, "main");
                assert!(p.locals.iter().any(|(k, v)| k == "x" && v == "2"));
                assert_eq!(p.stdout, "hi\n");
            }
            other => panic!("expected paused, got {other:?}"),
        }
        ctrl.send_action(DebugAction::Continue);
        let snap = ctrl.wait_until_paused_or_done(Duration::from_secs(2));
        match snap {
            DebugSnapshot::Done { ok, stdout, .. } => {
                assert!(ok);
                assert_eq!(stdout, "hi\n");
            }
            other => panic!("expected done, got {other:?}"),
        }
        handle.join().unwrap();
    }

    #[test]
    fn step_once() {
        let ctrl = DebugController::new(HashSet::new());
        ctrl.request_step();
        let c2 = ctrl.clone();
        let handle = thread::spawn(move || {
            let locals = HashMap::new();
            c2.on_stmt(10, "main", &locals, "").unwrap();
            c2.set_done(true, String::new(), String::new());
        });
        let snap = ctrl.wait_until_paused_or_done(Duration::from_secs(2));
        assert!(matches!(snap, DebugSnapshot::Paused(p) if p.line == 10));
        ctrl.send_action(DebugAction::Continue);
        let _ = ctrl.wait_until_paused_or_done(Duration::from_secs(2));
        handle.join().unwrap();
    }

    #[test]
    fn interpreter_pauses_on_breakpoint() {
        use crate::interp::Interpreter;
        use crate::parse::parse_source;

        let src = "# main\n\n*`x` = 1*\n> print text=`x`\n*`x` = 2*\n> print text=`x`\n";
        let module = parse_source(src).expect("parse");
        // Line 4 is first `> print`
        let ctrl = DebugController::new(HashSet::from([4u32]));
        let c2 = ctrl.clone();
        let handle = thread::spawn(move || {
            let mut interp = Interpreter::with_capture(None, false).with_debug(c2.clone());
            let r = interp.run_module(&module);
            let out = interp.captured_stdout.clone();
            match r {
                Ok(_) => c2.set_done(true, out, String::new()),
                Err(e) => c2.set_done(false, out, e.to_string()),
            }
        });

        let snap = ctrl.wait_until_paused_or_done(Duration::from_secs(2));
        match snap {
            DebugSnapshot::Paused(p) => {
                assert_eq!(p.line, 4, "locals={:?}", p.locals);
                assert!(p.locals.iter().any(|(k, v)| k == "x" && v == "1"));
            }
            other => panic!("expected pause on line 4, got {other:?}"),
        }
        ctrl.send_action(DebugAction::Continue);
        let snap = ctrl.wait_until_paused_or_done(Duration::from_secs(2));
        match snap {
            DebugSnapshot::Done { ok, stdout, stderr } => {
                assert!(ok, "stderr={stderr}");
                assert!(stdout.contains('1') && stdout.contains('2'), "{stdout:?}");
            }
            other => panic!("expected done, got {other:?}"),
        }
        handle.join().unwrap();
    }
}
