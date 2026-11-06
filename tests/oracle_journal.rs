use std::process::Command;

#[test]
fn document_oracle_values_both_backends() {
    for backend in ["tree", "bytecode"] {
        let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
            .args(["run", "tests/lib/foreign-oracle-journal.mq.md", "--backend", backend])
            .output().unwrap();
        assert!(out.status.success(), "{backend}: {}", String::from_utf8_lossy(&out.stderr));
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "True\nFalse\n2", "{backend}");
    }
}

#[test]
fn failed_responses_are_values_on_both_backends() {
    for backend in ["tree", "bytecode"] {
        let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
            .args(["run", "tests/lib/foreign-oracle-failure.mq.md", "--backend", backend])
            .output().unwrap();
        assert!(out.status.success(), "{backend}: {}", String::from_utf8_lossy(&out.stderr));
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "eval\nFalse\nFalse", "{backend}");
    }
}

#[test]
fn copied_code_values_are_refused_on_both_backends_and_lifetimes() {
    for backend in ["tree", "bytecode"] {
        for path in ["tests/lib/foreign-copy-code-refuse.mq.md", "tests/lib/foreign-session-copy-code-refuse.mq.md"] {
            let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
                .args(["run", path, "--backend", backend]).output().unwrap();
            assert!(!out.status.success(), "{backend} {path}: accepted code payload");
            assert!(String::from_utf8_lossy(&out.stderr).contains("outside V_safe"), "{}", String::from_utf8_lossy(&out.stderr));
            assert!(out.stdout.is_empty(), "refused input must not run the source");
        }
    }
}

#[test]
fn explicit_names_is_a_decode_observation_on_both_backends() {
    for backend in ["tree", "bytecode"] {
        let out=Command::new(env!("CARGO_BIN_EXE_marqdo"))
            .args(["run","tests/lib/foreign-oracle-names.mq.md","--backend",backend]).output().unwrap();
        assert!(out.status.success(),"{backend}: {}",String::from_utf8_lossy(&out.stderr));
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(),"decode\nzzz","{backend}");
    }
}
