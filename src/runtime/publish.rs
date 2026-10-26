//! Publish journal + expected_generation CAS (M-R6).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishPhase {
    Staging,
    Validated,
    GenerationRenamed,
    CurrentSwitched,
    DbProjected,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishJournal {
    pub kb_id: String,
    pub expected_generation: Option<String>,
    pub new_generation: String,
    pub phase: PublishPhase,
    pub staging_dir: PathBuf,
    pub generation_dir: PathBuf,
    pub current_path: PathBuf,
    pub source_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishOutcome {
    Published { generation: String },
    Conflict { current: String, expected: String },
    Recovered { generation: String },
    Aborted { reason: String },
}

impl PublishJournal {
    pub fn begin(
        out_dir: impl AsRef<Path>,
        kb_id: impl Into<String>,
        expected_generation: Option<String>,
        new_generation: impl Into<String>,
        source_digest: impl Into<String>,
    ) -> Result<Self, String> {
        let out_dir = out_dir.as_ref();
        let new_generation = new_generation.into();
        let staging_dir = out_dir.join(".publish-staging").join(&new_generation);
        let generation_dir = out_dir.join("generations").join(&new_generation);
        let current_path = out_dir.join("CURRENT");
        fs::create_dir_all(&staging_dir).map_err(|e| e.to_string())?;
        let j = Self {
            kb_id: kb_id.into(),
            expected_generation,
            new_generation,
            phase: PublishPhase::Staging,
            staging_dir,
            generation_dir,
            current_path,
            source_digest: source_digest.into(),
        };
        j.write_journal(out_dir)?;
        Ok(j)
    }

    fn journal_path(out_dir: &Path) -> PathBuf {
        out_dir.join("publish_journal.json")
    }

    fn write_journal(&self, out_dir: &Path) -> Result<(), String> {
        let path = Self::journal_path(out_dir);
        let tmp = path.with_extension("json.tmp");
        let body = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&tmp, body).map_err(|e| e.to_string())?;
        // Best-effort fsync via File.
        let f = fs::OpenOptions::new()
            .write(true)
            .open(&tmp)
            .map_err(|e| e.to_string())?;
        let _ = f.sync_all();
        drop(f);
        fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(out_dir: impl AsRef<Path>) -> Result<Option<Self>, String> {
        let path = Self::journal_path(out_dir.as_ref());
        if !path.is_file() {
            return Ok(None);
        }
        let body = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let j: Self = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        Ok(Some(j))
    }

    /// CAS: refuse if CURRENT does not match expected_generation.
    pub fn assert_expected_current(&self) -> Result<String, PublishOutcome> {
        let current = fs::read_to_string(&self.current_path)
            .unwrap_or_default()
            .trim()
            .to_string();
        if let Some(exp) = &self.expected_generation {
            if &current != exp {
                return Err(PublishOutcome::Conflict {
                    current,
                    expected: exp.clone(),
                });
            }
        }
        Ok(current)
    }

    pub fn mark(&mut self, out_dir: &Path, phase: PublishPhase) -> Result<(), String> {
        self.phase = phase;
        self.write_journal(out_dir)
    }

    /// Atomic publish: staging tree already complete → rename into generations → CURRENT.
    pub fn commit_filesystem(&mut self, out_dir: &Path) -> Result<PublishOutcome, String> {
        match self.assert_expected_current() {
            Err(o) => return Ok(o),
            Ok(_) => {}
        }
        self.mark(out_dir, PublishPhase::Validated)?;

        if self.generation_dir.exists() {
            fs::remove_dir_all(&self.generation_dir).map_err(|e| e.to_string())?;
        }
        fs::create_dir_all(self.generation_dir.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::rename(&self.staging_dir, &self.generation_dir).map_err(|e| e.to_string())?;
        self.mark(out_dir, PublishPhase::GenerationRenamed)?;

        let tmp = self.current_path.with_extension("tmp");
        {
            let mut f = fs::File::create(&tmp).map_err(|e| e.to_string())?;
            writeln!(f, "{}", self.new_generation).map_err(|e| e.to_string())?;
            let _ = f.sync_all();
        }
        fs::rename(&tmp, &self.current_path).map_err(|e| e.to_string())?;
        self.mark(out_dir, PublishPhase::CurrentSwitched)?;

        // Clear journal after successful CURRENT switch; DB projection is product-side.
        let _ = fs::remove_file(Self::journal_path(out_dir));
        Ok(PublishOutcome::Published {
            generation: self.new_generation.clone(),
        })
    }

    /// Crash recovery: if CURRENT already points at new_generation, treat as done;
    /// if generation exists but CURRENT old, finish CURRENT switch; else abort staging.
    pub fn recover(out_dir: impl AsRef<Path>) -> Result<PublishOutcome, String> {
        let out_dir = out_dir.as_ref();
        let Some(mut j) = Self::load(out_dir)? else {
            return Ok(PublishOutcome::Aborted {
                reason: "no_journal".into(),
            });
        };
        let current = fs::read_to_string(&j.current_path)
            .unwrap_or_default()
            .trim()
            .to_string();
        if current == j.new_generation {
            let _ = fs::remove_file(Self::journal_path(out_dir));
            return Ok(PublishOutcome::Recovered {
                generation: j.new_generation,
            });
        }
        match j.phase {
            PublishPhase::GenerationRenamed | PublishPhase::Validated
                if j.generation_dir.is_dir() =>
            {
                let tmp = j.current_path.with_extension("tmp");
                fs::write(&tmp, format!("{}\n", j.new_generation)).map_err(|e| e.to_string())?;
                fs::rename(&tmp, &j.current_path).map_err(|e| e.to_string())?;
                let _ = fs::remove_file(Self::journal_path(out_dir));
                Ok(PublishOutcome::Recovered {
                    generation: j.new_generation,
                })
            }
            PublishPhase::Staging | PublishPhase::Aborted => {
                let _ = fs::remove_dir_all(&j.staging_dir);
                j.mark(out_dir, PublishPhase::Aborted)?;
                let _ = fs::remove_file(Self::journal_path(out_dir));
                Ok(PublishOutcome::Aborted {
                    reason: "incomplete_staging".into(),
                })
            }
            other => Ok(PublishOutcome::Aborted {
                reason: format!("unhandled_phase:{other:?}"),
            }),
        }
    }
}

/// Stable logical document identity independent of generation directory name.
pub fn logical_document_id(project_id: &str, relative_path: &str) -> String {
    let mut h = Sha256::new();
    h.update(project_id.as_bytes());
    h.update(b"\0");
    h.update(relative_path.replace('\\', "/").as_bytes());
    format!("doc:{:x}", h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cas_conflict_and_commit() {
        let root = std::env::temp_dir().join(format!("mq-pub-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("generations")).unwrap();
        fs::write(root.join("CURRENT"), "gen_old\n").unwrap();

        let mut j = PublishJournal::begin(&root, "kb1", Some("gen_old".into()), "gen_new", "digest")
            .unwrap();
        fs::write(j.staging_dir.join("manifest.json"), "{}").unwrap();
        let out = j.commit_filesystem(&root).unwrap();
        assert_eq!(
            out,
            PublishOutcome::Published {
                generation: "gen_new".into()
            }
        );
        assert_eq!(
            fs::read_to_string(root.join("CURRENT")).unwrap().trim(),
            "gen_new"
        );

        let mut j2 =
            PublishJournal::begin(&root, "kb1", Some("gen_old".into()), "gen_x", "d").unwrap();
        fs::write(j2.staging_dir.join("x"), "1").unwrap();
        let conflict = j2.commit_filesystem(&root).unwrap();
        assert!(matches!(conflict, PublishOutcome::Conflict { .. }));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn recover_after_generation_rename() {
        let root = std::env::temp_dir().join(format!("mq-pub2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("generations/gen_r")).unwrap();
        fs::write(root.join("CURRENT"), "old\n").unwrap();
        let mut j =
            PublishJournal::begin(&root, "kb", Some("old".into()), "gen_r", "d").unwrap();
        // Simulate crash after generation rename: staging already moved.
        let _ = fs::remove_dir_all(&j.staging_dir);
        j.phase = PublishPhase::GenerationRenamed;
        j.write_journal(&root).unwrap();
        let out = PublishJournal::recover(&root).unwrap();
        assert_eq!(
            out,
            PublishOutcome::Recovered {
                generation: "gen_r".into()
            }
        );
        assert_eq!(
            fs::read_to_string(root.join("CURRENT")).unwrap().trim(),
            "gen_r"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn logical_id_stable_across_generation_paths() {
        let a = logical_document_id("proj", "docs/a.mq.md");
        let b = logical_document_id("proj", "docs/a.mq.md");
        assert_eq!(a, b);
        assert_ne!(a, logical_document_id("proj", "docs/b.mq.md"));
    }
}
