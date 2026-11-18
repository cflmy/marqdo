//! Policy compilation: reuse resolution → `.marqdo/agent/policies/*.mq.md`.

use std::fs;
use std::path::Path;

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::knowledge::ir::ReuseResult;
use crate::knowledge::util::slugify;

pub fn compile_policy(out_dir: &Path, task: &str, result: &ReuseResult) -> Result<std::path::PathBuf> {
    let dir = out_dir.join("agent/policies");
    fs::create_dir_all(&dir)?;
    let mut h = Sha256::new();
    h.update(task.as_bytes());
    let id = format!("{:x}", h.finalize())[..12].to_string();
    let stem = format!("{}-{}", slugify(&result.decision.as_str().to_ascii_lowercase()), id);
    let path = dir.join(format!("{stem}.mq.md"));

    let body = format!(
        r#"---
type: Marqdo Policy
title: Reuse policy for task
decision: {decision}
create_allowed: {create_allowed}
capability: {capability}
generated:
  by: marqdo/knowledge
---

# Reuse Policy

## Task

{task}

## Decision

**{decision}**

## Recommended

{recommended}

## Reason

{reason}

## Candidates

{candidates}

# main

This policy artifact records Engineering First Information for replay and CI.

* "policy:{id}"
"#,
        decision = result.decision.as_str(),
        create_allowed = result.create_allowed,
        capability = result.capability.as_deref().unwrap_or(""),
        task = task,
        recommended = result
            .recommended
            .iter()
            .map(|r| format!("- {r}"))
            .collect::<Vec<_>>()
            .join("\n"),
        reason = result
            .reason
            .iter()
            .map(|r| format!("- {r}"))
            .collect::<Vec<_>>()
            .join("\n"),
        candidates = result
            .candidates
            .iter()
            .take(8)
            .map(|c| format!("- {} ({:.2}) [{}]", c.title, c.score, c.kind))
            .collect::<Vec<_>>()
            .join("\n"),
        id = id,
    );
    fs::write(&path, body)?;
    Ok(path)
}
