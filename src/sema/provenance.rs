//! Optional source / applicability facts from module frontmatter (G05).
//!
//! Authors declare `sources:` and optional `applicability:` / `valid_until` in
//! frontmatter. Absence means a pure tool may still run but must stay
//! `evidence=unverified` — never invent a business SourceRef.

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest as _, Sha256};

use super::ir::{Digest, SourceRef};

#[derive(Debug, Clone, Default)]
pub struct ModuleProvenance {
    /// Relative corpus paths, optional `#anchor`.
    pub sources: Vec<String>,
    pub valid_until: Option<String>,
    pub region: Option<String>,
}

/// Scan YAML-ish frontmatter for sources / applicability (no full YAML parser).
pub fn parse_module_provenance(source: &str) -> ModuleProvenance {
    let mut lines = source.lines();
    if lines.next().map(|l| l.trim_end()) != Some("---") {
        return ModuleProvenance::default();
    }
    let mut out = ModuleProvenance::default();
    let mut in_sources = false;
    let mut in_applicability = false;
    for line in lines {
        let t = line.trim_end();
        if t.trim() == "---" {
            break;
        }
        let trimmed = t.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            in_sources = false;
            in_applicability = false;
        }
        if trimmed == "sources:" || trimmed.starts_with("sources:") {
            in_sources = true;
            in_applicability = false;
            if let Some(rest) = trimmed.strip_prefix("sources:") {
                let rest = rest.trim();
                if !rest.is_empty() && rest != "|" {
                    // inline list not used; ignore
                }
            }
            continue;
        }
        if trimmed == "applicability:" || trimmed.starts_with("applicability:") {
            in_applicability = true;
            in_sources = false;
            continue;
        }
        if in_sources {
            if let Some(item) = trimmed.strip_prefix("- ") {
                let path = item.trim().trim_matches('"').to_string();
                if !path.is_empty() {
                    out.sources.push(path);
                }
            }
            continue;
        }
        if in_applicability {
            if let Some(rest) = trimmed.strip_prefix("valid_until:") {
                out.valid_until = Some(rest.trim().trim_matches('"').to_string());
            } else if let Some(rest) = trimmed.strip_prefix("region:") {
                out.region = Some(rest.trim().trim_matches('"').to_string());
            }
        }
    }
    out
}

#[derive(Debug, Clone)]
pub enum ProvenanceDiag {
    Missing { path: String },
    Expired { valid_until: String },
}

impl ProvenanceDiag {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Missing { .. } => "source.missing",
            Self::Expired { .. } => "source.expired",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Missing { path } => format!("来源文件缺失：`{path}`"),
            Self::Expired { valid_until } => {
                format!("来源适用期已过（valid_until={valid_until}）")
            }
        }
    }
}

pub struct ProvenanceResolution {
    pub refs: Vec<SourceRef>,
    pub applicability: String,
    pub evidence_status: String,
    pub diagnostics: Vec<ProvenanceDiag>,
}

/// Resolve corpus paths relative to the module file; never mark verified here.
pub fn resolve_provenance(
    module_path: &Path,
    prov: &ModuleProvenance,
) -> ProvenanceResolution {
    let mut refs = Vec::new();
    let mut diagnostics = Vec::new();
    let base = module_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    for raw in &prov.sources {
        let (path_part, locator) = match raw.split_once('#') {
            Some((p, a)) => (p.trim(), Some(a.trim().to_string())),
            None => (raw.as_str(), None),
        };
        let resolved = base.join(path_part);
        if !resolved.is_file() {
            diagnostics.push(ProvenanceDiag::Missing {
                path: raw.clone(),
            });
            refs.push(SourceRef {
                uri: path_part.to_string(),
                locator,
                content_digest: None,
                relation: "supports".into(),
            });
            continue;
        }
        let content = fs::read(&resolved).unwrap_or_default();
        let mut hash = Sha256::new();
        hash.update(&content);
        refs.push(SourceRef {
            uri: path_part.to_string(),
            locator,
            content_digest: Some(Digest(format!("sha256:{:x}", hash.finalize()))),
            relation: "supports".into(),
        });
    }

    if let Some(until) = &prov.valid_until {
        if is_expired(until) {
            diagnostics.push(ProvenanceDiag::Expired {
                valid_until: until.clone(),
            });
        }
    }

    let applicability = match (&prov.region, &prov.valid_until) {
        (None, None) if prov.sources.is_empty() => {
            "no explicit applicability constraints".into()
        }
        (region, until) => {
            let mut parts = Vec::new();
            if let Some(r) = region {
                parts.push(format!("region={r}"));
            }
            if let Some(u) = until {
                parts.push(format!("valid_until={u}"));
            }
            if parts.is_empty() {
                "sources declared".into()
            } else {
                parts.join("; ")
            }
        }
    };

    let evidence_status = if diagnostics.iter().any(|d| matches!(d, ProvenanceDiag::Missing { .. }))
    {
        "missing_source".into()
    } else if diagnostics
        .iter()
        .any(|d| matches!(d, ProvenanceDiag::Expired { .. }))
    {
        "expired".into()
    } else if prov.sources.is_empty() {
        "unverified".into()
    } else {
        // Corpus present and in date — still not domain-verified by compile alone.
        "sourced".into()
    };

    ProvenanceResolution {
        refs,
        applicability,
        evidence_status,
        diagnostics,
    }
}

fn is_expired(valid_until: &str) -> bool {
    // Expect YYYY-MM-DD; compare lexicographically to today's UTC date.
    let today = chrono_today();
    !valid_until.is_empty() && valid_until < today.as_str()
}

fn chrono_today() -> String {
    // Avoid new dependency: use system local date via `date` is bad; use
    // SystemTime → rough days since epoch is fragile. Prefer `time` crate if
    // present, else format via `js_sys` — check Cargo.toml.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    // UTC civil date from days since epoch (proleptic Gregorian).
    let days = secs.div_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Algorithm from Howard Hinnant (public domain).
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sources_and_valid_until() {
        let src = "---\nsources:\n  - ../corpus/tax.md#rate\napplicability:\n  region: CN\n  valid_until: 2099-12-31\n---\n\n# main\n";
        let p = parse_module_provenance(src);
        assert_eq!(p.sources, vec!["../corpus/tax.md#rate"]);
        assert_eq!(p.region.as_deref(), Some("CN"));
        assert_eq!(p.valid_until.as_deref(), Some("2099-12-31"));
    }
}
