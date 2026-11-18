//! M-R7 smoke: admission → cancel CAS → event resume → publish journal recover.
//! Uses local fake/dry-run paths only (no live LLM).

use marqdo::runtime::{
    cancel_task, AdmissionClass, ControlService, InvokeRequestV2, PublishJournal, PublishOutcome,
    Scheduler, SchedulerConfig, Supervisor, SupervisorConfig, TaskState, TaskStore,
};
use marqdo::runtime::protocol::{EntryRef, InvokeContext, LeaseRef};
use serde_json::json;
use std::sync::Arc;

#[test]
fn mr07_admit_cancel_event_resume() {
    let dir = std::env::temp_dir().join(format!("mq-mr07-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let store = Arc::new(TaskStore::with_persist(dir.join("store")).unwrap());
    let sched = Arc::new(Scheduler::new(SchedulerConfig::default()));
    let ctl = ControlService::new(Arc::clone(&store), Arc::clone(&sched), dir.join("cancel"));

    let (t, depth) = ctl
        .admit(
            "task_smoke",
            "run_smoke",
            "ws",
            Some("sess".into()),
            "idem-smoke",
            &json!({"prompt": "hi"}),
            AdmissionClass::Execute,
        )
        .unwrap();
    assert!(depth >= 1);
    assert_eq!(t.state, TaskState::Queued);

    let again = ctl
        .admit(
            "task_other",
            "run_other",
            "ws",
            None,
            "idem-smoke",
            &json!({"prompt": "hi"}),
            AdmissionClass::Execute,
        )
        .unwrap();
    assert_eq!(again.0.task_id, t.task_id);

    let conflict = ctl.admit(
        "task_x",
        "run_x",
        "ws",
        None,
        "idem-smoke",
        &json!({"prompt": "different"}),
        AdmissionClass::Execute,
    );
    assert!(conflict.is_err());

    let _ = store
        .append_event(
            &t.task_id,
            &t.run_id,
            &t.attempt_id,
            "message.delta",
            json!({"text": "a"}),
        )
        .unwrap();
    let e2 = store
        .append_event(
            &t.task_id,
            &t.run_id,
            &t.attempt_id,
            "message.delta",
            json!({"text": "b"}),
        )
        .unwrap();
    let resumed = ctl.events_since(&t.task_id, 1).unwrap();
    assert_eq!(resumed.len(), 1);
    assert_eq!(resumed[0].seq, e2.seq);

    let c = cancel_task(&ctl, &t.task_id).unwrap();
    assert_eq!(c.state, TaskState::CancelRequested);
    assert_eq!(
        std::fs::read_to_string(ctl.cancel_token_path(&t.task_id)).unwrap(),
        "cancelled"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mr07_supervisor_dry_run_slots_and_publish_recover() {
    std::env::set_var("MARQDO_RUNTIME_DRY_RUN", "1");
    let root = std::env::temp_dir().join(format!("mq-mr07b-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let bin = std::env::current_exe().unwrap();
    let sup = Supervisor::new(
        SupervisorConfig {
            execute_slots: 2,
            ..SupervisorConfig::default()
        },
        root.join("stage"),
        &bin,
        root.join("cancel"),
        root.join("events"),
    );
    sup.warm().unwrap();
    let req = |id: &str| InvokeRequestV2 {
        schema: "InvokeRequestV2".into(),
        schema_version: 2,
        task_id: id.into(),
        run_id: format!("run_{id}"),
        attempt_id: format!("a_{id}"),
        lease: LeaseRef {
            id: format!("l_{id}"),
            epoch: 1,
        },
        entry: EntryRef {
            module: "agent/qdagent.mq.md".into(),
            function: "run".into(),
        },
        args_ref: None,
        context: InvokeContext {
            workspace_id: "ws".into(),
            snapshot_id: None,
            selection_ref: None,
            capabilities_ref: None,
            deadline_at: "2099-01-01T00:00:00Z".into(),
            fs_root: Some(root.display().to_string()),
        },
    };
    let a = sup.dispatch(req("a")).unwrap();
    let b = sup.dispatch(req("b")).unwrap();
    assert_eq!(sup.active_count(), 2);
    assert!(sup.dispatch(req("c")).is_err());
    sup.release(&a);
    sup.release(&b);

    // Publish journal recovery after generation rename (CURRENT still old).
    let pub_root = root.join("kb");
    std::fs::create_dir_all(pub_root.join("generations/g1")).unwrap();
    std::fs::write(pub_root.join("CURRENT"), "old\n").unwrap();
    let body = serde_json::json!({
        "kb_id": "kb",
        "expected_generation": "old",
        "new_generation": "g1",
        "phase": "generation_renamed",
        "staging_dir": pub_root.join(".publish-staging/g1"),
        "generation_dir": pub_root.join("generations/g1"),
        "current_path": pub_root.join("CURRENT"),
        "source_digest": "d"
    });
    std::fs::write(
        pub_root.join("publish_journal.json"),
        serde_json::to_vec_pretty(&body).unwrap(),
    )
    .unwrap();
    let out = PublishJournal::recover(&pub_root).unwrap();
    assert_eq!(
        out,
        PublishOutcome::Recovered {
            generation: "g1".into()
        }
    );

    let caps = marqdo::workbench::probe_capabilities(&json!({}));
    assert_eq!(caps["features"]["runtime.task_control_v2"], 1);
    assert_eq!(caps["features"]["runtime.event_replay"], 1);
    assert_eq!(caps["features"]["runtime.isolated_execution"], 1);

    let _ = std::fs::remove_dir_all(&root);
    std::env::remove_var("MARQDO_RUNTIME_DRY_RUN");
}
