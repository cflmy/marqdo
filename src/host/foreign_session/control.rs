//! An owned-process interrupt capability. Retirement precedes every child reap,
//! so a stale capability cannot signal a reused operating-system process id.
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct State {
    pid: Option<u32>,
    evaluating: bool,
    ready: bool,
    requested: bool,
}

#[derive(Clone, Debug, Default)]
pub struct InterruptHandle(Arc<Mutex<State>>);

impl InterruptHandle {
    pub(super) fn new(pid: u32) -> Self {
        Self(Arc::new(Mutex::new(State {
            pid: Some(pid),
            ..State::default()
        })))
    }
    pub(super) fn retire(&self) -> Option<u32> {
        if let Ok(mut state) = self.0.lock() {
            let pid = state.pid.take();
            state.evaluating = false;
            pid
        } else {
            None
        }
    }
    pub(super) fn try_wait(
        &self,
        child: &mut std::process::Child,
    ) -> std::io::Result<Option<std::process::ExitStatus>> {
        let mut state = self.0.lock().unwrap();
        let result = child.try_wait();
        if matches!(result, Ok(Some(_))) {
            state.pid = None;
            state.evaluating = false;
        }
        result
    }
    pub(super) fn begin(&self, evaluating: bool) {
        let mut state = self.0.lock().unwrap();
        state.evaluating = evaluating;
        state.ready = false;
        state.requested = false;
    }
    pub(super) fn finish(&self) {
        self.0.lock().unwrap().evaluating = false;
    }
    // SIGINT can arrive between a Node ready frame and installation of V8's
    // breakOnSigint handler. Retry the same logical request while awaiting its
    // response; never carry a retry into the next request or retired process.
    pub(super) fn retry(&self) -> Result<(), String> {
        let state = self.0.lock().map_err(|_| "interrupt lock poisoned")?;
        if state.evaluating && state.ready && state.requested {
            Self::signal(&state)?;
        }
        Ok(())
    }
    pub(super) fn ready(&self) -> Result<(), String> {
        let mut state = self.0.lock().map_err(|_| "interrupt lock poisoned")?;
        if state.ready {
            return Ok(());
        }
        state.ready = true;
        if state.requested {
            Self::signal(&state)?;
        }
        Ok(())
    }
    pub fn is_live(&self) -> bool {
        self.0.lock().is_ok_and(|s| s.pid.is_some())
    }

    /// False means idle; cancellation is coalesced within one evaluation.
    /// Before the bootstrap's ready frame it is queued rather than sent into
    /// interpreter startup or transport parsing.
    pub fn request(&self) -> Result<bool, String> {
        let mut state = self.0.lock().map_err(|_| "interrupt lock poisoned")?;
        if state.pid.is_none() {
            return Err("interrupt capability retired".into());
        }
        if !state.evaluating {
            return Ok(false);
        }
        #[cfg(not(unix))]
        return Err("non-destructive interrupt is unsupported on this platform".into());
        if !state.requested {
            if state.ready {
                Self::signal(&state)?;
            }
            state.requested = true;
        }
        Ok(true)
    }
    fn signal(state: &State) -> Result<(), String> {
        let pid = state.pid.ok_or("interrupt capability retired")?;
        #[cfg(unix)]
        {
            if unsafe { libc::kill(-(pid as i32), libc::SIGINT) } != 0 {
                return Err(format!(
                    "interrupt failed: {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            let _ = pid;
            Err("non-destructive interrupt is unsupported on this platform".into())
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use crate::host::{foreign_session, HostContext};
    use crate::value::Value;
    #[test]
    fn retired_interrupt_capability_cannot_reach_replacement_kernel() {
        for language in ["python", "node"] {
            let mut host = HostContext::default();
            let lang = Value::Text(language.into());
            let old = foreign_session::open(&mut host, &lang, None, None, None, None).unwrap();
            let Value::Handle(id) = &old else { panic!() };
            let control = host.foreign_sessions[id].interrupt_handle.clone();
            assert!(!control.request().unwrap());
            foreign_session::close(&mut host, &old).unwrap();
            let new = foreign_session::open(&mut host, &lang, None, None, None, None).unwrap();
            assert!(!control.is_live());
            assert!(control.request().unwrap_err().contains("retired"));
            foreign_session::set(&mut host, &new, &Value::Text("n".into()), &Value::Int(8))
                .unwrap();
            assert_eq!(
                foreign_session::get(&mut host, &new, &Value::Text("n".into())).unwrap(),
                Value::Int(8)
            );
            foreign_session::close(&mut host, &new).unwrap();
        }
    }
    #[test]
    fn cancellation_queued_during_bootstrap_reaches_busy_evaluation() {
        use std::time::{Duration, Instant};
        for language in ["python", "node"] {
            let mut host = HostContext::default();
            let id = foreign_session::open(
                &mut host,
                &Value::Text(language.into()),
                None,
                None,
                None,
                None,
            )
            .unwrap();
            let Value::Handle(ref name) = id else {
                panic!()
            };
            let control = host.foreign_sessions[name].interrupt_handle.clone();
            let kernel = id.clone();
            let code = if language == "python" {
                "while True: pass"
            } else {
                "while(true) {}"
            };
            let worker = std::thread::spawn(move || {
                let record = foreign_session::eval(
                    &mut host,
                    &kernel,
                    None,
                    Some(&Value::Text(code.into())),
                    None,
                    Some(&Value::Int(2)),
                )
                .unwrap();
                (host, record)
            });
            let deadline = Instant::now() + Duration::from_secs(1);
            while !control.0.lock().unwrap().evaluating && Instant::now() < deadline {
                std::thread::yield_now();
            }
            assert!(control.request().unwrap());
            assert!(
                control.request().unwrap(),
                "logical duplicates must coalesce"
            );
            let (mut host, record) = worker.join().unwrap();
            let Value::Map(fields) = record else { panic!() };
            assert_eq!(
                fields.iter().find(|(k, _)| k == "timed_out").unwrap().1,
                Value::Bool(false),
                "{language}: {fields:?}"
            );
            assert_eq!(
                fields.iter().find(|(k, _)| k == "alive").unwrap().1,
                Value::Bool(true)
            );
            foreign_session::close(&mut host, &id).unwrap();
        }
    }
}
