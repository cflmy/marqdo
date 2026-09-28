//! Reuse ratio / duplication rate metrics.

use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

#[derive(Debug, Clone, Default)]
pub struct ReuseMetrics {
    pub generated_functions: u64,
    pub reused: u64,
    pub adapted: u64,
    pub created: u64,
    pub duplicated: u64,
}

impl ReuseMetrics {
    pub fn reuse_ratio(&self) -> f64 {
        if self.generated_functions == 0 {
            return 0.0;
        }
        self.reused as f64 / self.generated_functions as f64
    }

    pub fn adapt_ratio(&self) -> f64 {
        if self.generated_functions == 0 {
            return 0.0;
        }
        self.adapted as f64 / self.generated_functions as f64
    }

    pub fn novel_ratio(&self) -> f64 {
        if self.generated_functions == 0 {
            return 0.0;
        }
        self.created as f64 / self.generated_functions as f64
    }

    pub fn duplication_rate(&self) -> f64 {
        if self.created == 0 {
            return 0.0;
        }
        self.duplicated as f64 / self.created as f64
    }

    pub fn to_json(&self) -> Value {
        json!({
            "generated_functions": self.generated_functions,
            "reused": self.reused,
            "adapted": self.adapted,
            "created": self.created,
            "duplicated": self.duplicated,
            "reuse_ratio": self.reuse_ratio(),
            "adapt_ratio": self.adapt_ratio(),
            "novel_ratio": self.novel_ratio(),
            "duplication_rate": self.duplication_rate(),
        })
    }
}

pub fn metrics_path(out_dir: &Path) -> std::path::PathBuf {
    out_dir.join("agent/episodes/reuse_metrics.json")
}

pub fn load_metrics(out_dir: &Path) -> ReuseMetrics {
    let path = metrics_path(out_dir);
    let Ok(raw) = fs::read_to_string(path) else {
        return ReuseMetrics::default();
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return ReuseMetrics::default();
    };
    ReuseMetrics {
        generated_functions: v.get("generated_functions").and_then(|x| x.as_u64()).unwrap_or(0),
        reused: v.get("reused").and_then(|x| x.as_u64()).unwrap_or(0),
        adapted: v.get("adapted").and_then(|x| x.as_u64()).unwrap_or(0),
        created: v.get("created").and_then(|x| x.as_u64()).unwrap_or(0),
        duplicated: v.get("duplicated").and_then(|x| x.as_u64()).unwrap_or(0),
    }
}

pub fn save_metrics(out_dir: &Path, m: &ReuseMetrics) -> Result<()> {
    let path = metrics_path(out_dir);
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    fs::write(path, serde_json::to_string_pretty(&m.to_json())?)?;
    Ok(())
}

pub fn record_decision(out_dir: &Path, decision: &str, duplicated: bool) -> Result<ReuseMetrics> {
    let mut m = load_metrics(out_dir);
    m.generated_functions += 1;
    match decision.to_ascii_uppercase().as_str() {
        "REUSE" => m.reused += 1,
        "ADAPT" => m.adapted += 1,
        "CREATE" => {
            m.created += 1;
            if duplicated {
                m.duplicated += 1;
            }
        }
        _ => {}
    }
    save_metrics(out_dir, &m)?;
    Ok(m)
}
