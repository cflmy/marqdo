//! Knowledge compilation from agent episodes / failure writebacks.

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::knowledge::util::slugify;

#[derive(Debug, Clone)]
pub struct LearnInput {
    pub task: String,
    pub failed_approach: Option<String>,
    pub failure: Option<String>,
    pub correct_approach: Option<String>,
    pub decision_title: Option<String>,
    pub decision_body: Option<String>,
    pub constraint_title: Option<String>,
    pub constraint_body: Option<String>,
}

pub fn learn(out_dir: &Path, input: &LearnInput) -> Result<Vec<std::path::PathBuf>> {
    let mut written = Vec::new();

    if input.failed_approach.is_some() || input.failure.is_some() {
        let dir = out_dir.join("knowledge/failures");
        fs::create_dir_all(&dir)?;
        let stem = slugify(&format!("fail-{}", &input.task));
        let path = dir.join(format!("{stem}.md"));
        let body = format!(
            r#"---
type: Failure
task: {task}
status: resolved
generated:
  by: marqdo/knowledge-learn
---

# Do not repeat: {task}

## Failed approach

{failed}

## Failure

{failure}

## Correct approach

{correct}

## Introduced by

agent episode learn
"#,
            task = input.task,
            failed = input.failed_approach.as_deref().unwrap_or("_unspecified_"),
            failure = input.failure.as_deref().unwrap_or("_unspecified_"),
            correct = input.correct_approach.as_deref().unwrap_or("_unspecified_"),
        );
        fs::write(&path, body)?;
        written.push(path);
    }

    if let (Some(title), Some(body)) = (&input.decision_title, &input.decision_body) {
        let dir = out_dir.join("knowledge/decisions");
        fs::create_dir_all(&dir)?;
        let stem = slugify(title);
        let path = dir.join(format!("{stem}.md"));
        let page = format!(
            r#"---
type: Decision
title: {title}
status: accepted
generated:
  by: marqdo/knowledge-learn
---

# {title}

## Decision

{body}
"#,
            title = title,
            body = body,
        );
        fs::write(&path, page)?;
        written.push(path);
    }

    if let (Some(title), Some(body)) = (&input.constraint_title, &input.constraint_body) {
        let dir = out_dir.join("knowledge/constraints");
        fs::create_dir_all(&dir)?;
        let stem = slugify(title);
        let path = dir.join(format!("{stem}.md"));
        let page = format!(
            r#"---
type: Constraint
title: {title}
status: stable
generated:
  by: marqdo/knowledge-learn
---

# {title}

{body}
"#,
            title = title,
            body = body,
        );
        fs::write(&path, page)?;
        written.push(path);
    }

    // Episode receipt
    let ep = out_dir.join("agent/episodes");
    fs::create_dir_all(&ep)?;
    let ep_path = ep.join(format!("{}.json", slugify(&input.task)));
    let receipt = serde_json::json!({
        "task": input.task,
        "learned": written.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
    });
    fs::write(&ep_path, serde_json::to_string_pretty(&receipt)?)?;
    written.push(ep_path);

    Ok(written)
}
