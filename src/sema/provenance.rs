//! Optional source / applicability facts from module frontmatter (G05 / M02).
//!
//! Authors declare `sources:` and optional `applicability:` / `valid_until` in
//! frontmatter. Absence means a pure tool may still run but must stay
//! `evidence=unverified` — never invent a business SourceRef.

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest as _, Sha256};

use super::ir::{Digest, SourceRef};

/// Provenance extractor / verifier identity hashed into evidence digests.
pub const PROVENANCE_EXTRACTOR_VERSION: &str = "marqdo-provenance/1";

#[derive(Debug, Clone, Default)]
pub struct ModuleProvenance {
    /// Relative corpus paths, optional `#anchor`.
    pub sources: Vec<String>,
    pub valid_until: Option<String>,
    pub region: Option<String>,
    /// True when author wrote `sources: [...]` inline (ignored by parser — M07).
    pub inline_sources_ignored: bool,
    /// Optional per-function source overrides (M07): `function_sources: { name: [uri…] }`.
    pub function_sources: std::collections::BTreeMap<String, Vec<String>>,
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
    let mut in_function_sources = false;
    let mut current_fn: Option<String> = None;
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
            in_function_sources = false;
            current_fn = None;
        }
        if trimmed == "sources:" || trimmed.starts_with("sources:") {
            in_sources = true;
            in_applicability = false;
            in_function_sources = false;
            if let Some(rest) = trimmed.strip_prefix("sources:") {
                let rest = rest.trim();
                if !rest.is_empty() && rest != "|" {
                    // inline list is not parsed — flag for diagnosis (M07).
                    out.inline_sources_ignored = true;
                }
            }
            continue;
        }
        if trimmed == "function_sources:" || trimmed.starts_with("function_sources:") {
            in_function_sources = true;
            in_sources = false;
            in_applicability = false;
            current_fn = None;
            continue;
        }
        if trimmed == "applicability:" || trimmed.starts_with("applicability:") {
            in_applicability = true;
            in_sources = false;
            in_function_sources = false;
            continue;
        }
        if in_function_sources {
            // Indent levels: `  name:` then `    - uri`
            let indent = line.len() - line.trim_start().len();
            if trimmed.ends_with(':') && !trimmed.starts_with('-') {
                let name = trimmed.trim_end_matches(':').trim().to_string();
                if !name.is_empty() {
                    current_fn = Some(name.clone());
                    out.function_sources.entry(name).or_default();
                }
                continue;
            }
            if let Some(item) = trimmed.strip_prefix("- ") {
                let path = item.trim().trim_matches('"').to_string();
                if let Some(name) = &current_fn {
                    if !path.is_empty() {
                        out.function_sources
                            .entry(name.clone())
                            .or_default()
                            .push(path);
                    }
                } else if indent >= 2 && !path.is_empty() {
                    // ignore orphan list items
                }
            }
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

/// Sources applicable to a named function: function_sources override, else module sources.
pub fn sources_for_function(prov: &ModuleProvenance, function_name: &str) -> Vec<String> {
    if let Some(list) = prov.function_sources.get(function_name) {
        if !list.is_empty() {
            return list.clone();
        }
    }
    prov.sources.clone()
}

#[derive(Debug, Clone)]
pub enum ProvenanceDiag {
    Missing { path: String },
    AnchorMissing { path: String, locator: String },
    ReadError { path: String, message: String },
    Expired { valid_until: String },
    InvalidDate { valid_until: String },
    InlineSourcesIgnored,
}

impl ProvenanceDiag {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Missing { .. } => "source.missing",
            Self::AnchorMissing { .. } => "source.anchor_missing",
            Self::ReadError { .. } => "source.read_error",
            Self::Expired { .. } => "source.expired",
            Self::InvalidDate { .. } => "source.invalid_date",
            Self::InlineSourcesIgnored => "source.inline_ignored",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Missing { path } => format!("来源文件缺失：`{path}`"),
            Self::AnchorMissing { path, locator } => {
                format!("来源锚点不存在：`{path}` 中找不到 `#{}`", locator)
            }
            Self::ReadError { path, message } => {
                format!("来源文件无法读取：`{path}`（{message}）")
            }
            Self::Expired { valid_until } => {
                format!("来源适用期已过（valid_until={valid_until}）")
            }
            Self::InvalidDate { valid_until } => {
                format!("适用期日期格式无效（期望 YYYY-MM-DD）：`{valid_until}`")
            }
            Self::InlineSourcesIgnored => {
                "sources 行内列表被忽略；请改用 YAML block list（`- path#anchor`）".into()
            }
        }
    }
}

pub struct ProvenanceResolution {
    pub refs: Vec<SourceRef>,
    pub applicability: String,
    pub evidence_status: String,
    pub diagnostics: Vec<ProvenanceDiag>,
    /// Canonical record hashed into `hashes.evidence` (M02).
    pub evidence_record: String,
}

/// Resolve corpus paths relative to the module file; never mark verified here.
pub fn resolve_provenance(
    module_path: &Path,
    prov: &ModuleProvenance,
) -> ProvenanceResolution {
    resolve_provenance_sources(module_path, prov, &prov.sources)
}

/// Resolve an explicit source list (module or per-function).
pub fn resolve_provenance_sources(
    module_path: &Path,
    prov: &ModuleProvenance,
    sources: &[String],
) -> ProvenanceResolution {
    let mut refs = Vec::new();
    let mut diagnostics = Vec::new();
    if prov.inline_sources_ignored {
        diagnostics.push(ProvenanceDiag::InlineSourcesIgnored);
    }
    let base = module_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    for raw in sources {
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
        let content = match fs::read(&resolved) {
            Ok(bytes) => bytes,
            Err(e) => {
                diagnostics.push(ProvenanceDiag::ReadError {
                    path: path_part.to_string(),
                    message: e.to_string(),
                });
                refs.push(SourceRef {
                    uri: path_part.to_string(),
                    locator,
                    content_digest: None,
                    relation: "supports".into(),
                });
                continue;
            }
        };
        let mut hash = Sha256::new();
        hash.update(&content);
        let content_digest = Digest(format!("sha256:{:x}", hash.finalize()));
        if let Some(loc) = locator.as_ref() {
            let text = String::from_utf8_lossy(&content);
            if !anchor_exists(&text, loc) {
                diagnostics.push(ProvenanceDiag::AnchorMissing {
                    path: path_part.to_string(),
                    locator: loc.clone(),
                });
            }
        }
        refs.push(SourceRef {
            uri: path_part.to_string(),
            locator,
            content_digest: Some(content_digest),
            relation: "supports".into(),
        });
    }

    if let Some(until) = &prov.valid_until {
        match validate_date(until) {
            DateCheck::Ok => {
                if is_expired(until) {
                    diagnostics.push(ProvenanceDiag::Expired {
                        valid_until: until.clone(),
                    });
                }
            }
            DateCheck::Invalid => {
                diagnostics.push(ProvenanceDiag::InvalidDate {
                    valid_until: until.clone(),
                });
            }
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

    let evidence_status = if diagnostics
        .iter()
        .any(|d| matches!(d, ProvenanceDiag::Missing { .. } | ProvenanceDiag::ReadError { .. }))
    {
        "missing_source".into()
    } else if diagnostics
        .iter()
        .any(|d| matches!(d, ProvenanceDiag::AnchorMissing { .. }))
    {
        "anchor_missing".into()
    } else if diagnostics
        .iter()
        .any(|d| matches!(d, ProvenanceDiag::Expired { .. }))
    {
        "expired".into()
    } else if diagnostics
        .iter()
        .any(|d| matches!(d, ProvenanceDiag::InvalidDate { .. }))
    {
        "invalid_date".into()
    } else if sources.is_empty() {
        "unverified".into()
    } else {
        // Corpus present and in date — still not domain-verified by compile alone.
        "sourced".into()
    };

    let prov_for_record = ModuleProvenance {
        sources: sources.to_vec(),
        valid_until: prov.valid_until.clone(),
        region: prov.region.clone(),
        inline_sources_ignored: prov.inline_sources_ignored,
        function_sources: Default::default(),
    };
    let evidence_record = evidence_canonical_record(&prov_for_record, &refs);

    ProvenanceResolution {
        refs,
        applicability,
        evidence_status,
        diagnostics,
        evidence_record,
    }
}

/// Canonical evidence inputs: extractor version, applicability, sorted source facts.
pub fn evidence_canonical_record(prov: &ModuleProvenance, refs: &[SourceRef]) -> String {
    let mut parts = vec![
        format!("extractor={PROVENANCE_EXTRACTOR_VERSION}"),
        format!("region={}", prov.region.as_deref().unwrap_or("")),
        format!("valid_until={}", prov.valid_until.as_deref().unwrap_or("")),
    ];
    let mut source_parts: Vec<String> = refs
        .iter()
        .filter(|r| r.relation == "supports")
        .map(|r| {
            format!(
                "{}#{}@{}",
                r.uri,
                r.locator.as_deref().unwrap_or(""),
                r.content_digest
                    .as_ref()
                    .map(|d| d.0.as_str())
                    .unwrap_or("none")
            )
        })
        .collect();
    source_parts.sort();
    parts.extend(source_parts);
    parts.join("|")
}

/// Whether a Markdown heading / id matches the `#locator` fragment.
pub fn anchor_exists(content: &str, locator: &str) -> bool {
    let loc = locator.trim().trim_start_matches('#').trim();
    if loc.is_empty() {
        return true;
    }
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix('#') {
            let heading = rest.trim_start_matches('#').trim();
            let heading = heading
                .split('{')
                .next()
                .unwrap_or(heading)
                .trim();
            if heading == loc || slugify_heading(heading) == loc {
                return true;
            }
        }
        if t.contains(&format!("id=\"{loc}\""))
            || t.contains(&format!("id='{loc}'"))
            || t.contains(&format!("{{#{loc}}}"))
        {
            return true;
        }
    }
    false
}

fn slugify_heading(heading: &str) -> String {
    let mut out = String::new();
    for ch in heading.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == ' ' || ch == '-' || ch == '_' {
            if !out.ends_with('-') {
                out.push('-');
            }
        } else if !ch.is_ascii() {
            // Keep non-ASCII letters (common in ZH docs) without lowercasing rules.
            out.push(ch);
        }
    }
    out.trim_matches('-').to_string()
}

enum DateCheck {
    Ok,
    Invalid,
}

fn validate_date(valid_until: &str) -> DateCheck {
    let parts: Vec<&str> = valid_until.split('-').collect();
    if parts.len() != 3 {
        return DateCheck::Invalid;
    }
    let Ok(y) = parts[0].parse::<i32>() else {
        return DateCheck::Invalid;
    };
    let Ok(m) = parts[1].parse::<u32>() else {
        return DateCheck::Invalid;
    };
    let Ok(d) = parts[2].parse::<u32>() else {
        return DateCheck::Invalid;
    };
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || y < 1970 {
        return DateCheck::Invalid;
    }
    if parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return DateCheck::Invalid;
    }
    DateCheck::Ok
}

fn is_expired(valid_until: &str) -> bool {
    let today = chrono_today();
    !valid_until.is_empty() && valid_until < today.as_str()
}

/// Public recheck for handle.execute (M02).
pub fn valid_until_expired(valid_until: &str) -> bool {
    matches!(validate_date(valid_until), DateCheck::Ok) && is_expired(valid_until)
}

fn chrono_today() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
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

    #[test]
    fn anchor_matches_heading() {
        assert!(anchor_exists("# rate\n\nrate = 0.13\n", "rate"));
        assert!(!anchor_exists("# rate\n\nrate = 0.13\n", "missing-anchor"));
    }

    #[test]
    fn invalid_date_rejected() {
        assert!(matches!(validate_date("2099-12-31"), DateCheck::Ok));
        assert!(matches!(validate_date("not-a-date"), DateCheck::Invalid));
        assert!(matches!(validate_date("2099-13-01"), DateCheck::Invalid));
    }
}
