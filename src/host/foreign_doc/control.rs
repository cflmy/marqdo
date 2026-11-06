//! Concurrent control inputs are scoped to a single recorded document action.
//! Replay supplies their elapsed-time inputs, reobserves acceptance, and stops
//! on divergence; elapsed wall time is not a deterministic foreign-state oracle.
use crate::host::foreign_session::InterruptHandle;
use crate::value::Value;
use std::collections::BTreeMap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ControlEvent {
    pub after_us: u64,
    pub results: Vec<(String, Result<bool, String>)>,
}
impl ControlEvent {
    pub fn value(&self) -> Value {
        Value::Map(vec![
            ("after_us".into(), Value::Int(self.after_us as i64)),
            (
                "results".into(),
                Value::List(
                    self.results
                        .iter()
                        .map(|(name, result)| {
                            Value::Map(vec![
                                ("kernel".into(), Value::Text(name.clone())),
                                ("accepted".into(), Value::Bool(result.is_ok())),
                                ("active".into(), Value::Bool(matches!(result, Ok(true)))),
                                (
                                    "error".into(),
                                    Value::Text(result.as_ref().err().cloned().unwrap_or_default()),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])
    }
}
#[derive(Default, Debug)]
struct State {
    kernels: BTreeMap<String, InterruptHandle>,
    generation: u64,
    action: Option<(Instant, Vec<ControlEvent>)>,
}
#[derive(Default, Debug)]
struct Shared {
    state: Mutex<State>,
    changed: Condvar,
}
#[derive(Clone, Default, Debug)]
pub struct DocumentControl(Arc<Shared>);

impl DocumentControl {
    pub(super) fn cancelled(&self) -> bool {
        self.0
            .state
            .lock()
            .unwrap()
            .action
            .as_ref()
            .is_some_and(|(_, events)| !events.is_empty())
    }
    pub(super) fn register(&self, kernel: &str, handle: InterruptHandle) {
        self.0
            .state
            .lock()
            .unwrap()
            .kernels
            .insert(kernel.into(), handle);
    }
    pub(super) fn begin(&self, plan: Vec<ControlEvent>) -> Option<JoinHandle<()>> {
        let generation = {
            let mut state = self.0.state.lock().unwrap();
            state.generation += 1;
            state.action = Some((Instant::now(), Vec::new()));
            state.generation
        };
        if plan.is_empty() {
            return None;
        }
        let controller = self.clone();
        Some(std::thread::spawn(move || {
            for event in plan {
                let mut state = controller.0.state.lock().unwrap();
                loop {
                    if state.generation != generation || state.action.is_none() {
                        return;
                    }
                    let elapsed = state.action.as_ref().unwrap().0.elapsed();
                    let remaining = Duration::from_micros(event.after_us).saturating_sub(elapsed);
                    if remaining.is_zero() {
                        break;
                    }
                    state = controller
                        .0
                        .changed
                        .wait_timeout(state, remaining.min(Duration::from_millis(200)))
                        .unwrap()
                        .0;
                }
                // Keep the action-generation check and signal under one lock.
                let _ = Self::request_locked(&mut state, Some(event.after_us));
            }
        }))
    }
    pub(super) fn finish(&self, scheduler: Option<JoinHandle<()>>) -> Vec<ControlEvent> {
        let events = self
            .0
            .state
            .lock()
            .unwrap()
            .action
            .take()
            .map(|(_, events)| events)
            .unwrap_or_default();
        self.0.changed.notify_all();
        if let Some(scheduler) = scheduler {
            let _ = scheduler.join();
        }
        events
    }
    pub fn interrupt(&self) -> Result<Value, String> {
        let mut state = self
            .0
            .state
            .lock()
            .map_err(|_| "document control lock poisoned")?;
        Self::request_locked(&mut state, None).map(|event| event.value())
    }
    fn request_locked(
        state: &mut State,
        scheduled_us: Option<u64>,
    ) -> Result<ControlEvent, String> {
        let start = state
            .action
            .as_ref()
            .ok_or("no document action in flight")?
            .0;
        let after_us = scheduled_us
            .unwrap_or_else(|| start.elapsed().as_micros().min(i64::MAX as u128) as u64);
        let results = state
            .kernels
            .iter()
            .filter(|(_, handle)| handle.is_live())
            .map(|(name, handle)| (name.clone(), handle.request()))
            .collect();
        let event = ControlEvent { after_us, results };
        state.action.as_mut().unwrap().1.push(event.clone());
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completed_action_cancels_pending_scheduler_before_next_generation() {
        let controller = DocumentControl::default();
        let scheduler = controller.begin(vec![ControlEvent {
            after_us: 60_000_000,
            results: Vec::new(),
        }]);
        let start = Instant::now();
        assert!(controller.finish(scheduler).is_empty());
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(controller.interrupt().is_err());
        let next = controller.begin(Vec::new());
        assert!(!controller.cancelled());
        assert!(controller.finish(next).is_empty());
    }
}
