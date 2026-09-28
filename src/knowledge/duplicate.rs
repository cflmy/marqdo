//! Duplicate capability / symbol detection via fingerprints.

use crate::knowledge::fingerprint::symbol_similarity;
use crate::knowledge::ir::{CapabilityInfo, Edge, EdgeKind, SymbolInfo};

#[derive(Debug, Clone)]
pub struct DuplicatePair {
    pub left: String,
    pub right: String,
    pub similarity: f64,
    pub left_resource: String,
    pub right_resource: String,
}

pub fn find_duplicates(symbols: &[SymbolInfo], threshold: f64) -> Vec<DuplicatePair> {
    let mut pairs = Vec::new();
    for i in 0..symbols.len() {
        if symbols[i].kind == "test" {
            continue;
        }
        for j in (i + 1)..symbols.len() {
            if symbols[j].kind == "test" {
                continue;
            }
            if symbols[i].module == symbols[j].module && symbols[i].name == symbols[j].name {
                continue;
            }
            let sim = symbol_similarity(&symbols[i], &symbols[j]);
            if sim >= threshold {
                pairs.push(DuplicatePair {
                    left: symbols[i].id.clone(),
                    right: symbols[j].id.clone(),
                    similarity: sim,
                    left_resource: symbols[i].resource.clone(),
                    right_resource: symbols[j].resource.clone(),
                });
            }
        }
    }
    pairs.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    pairs
}

/// During extract: add conflicts_with-style related edges for near-duplicates.
pub fn annotate_duplicates(
    symbols: &[SymbolInfo],
    edges: &mut Vec<Edge>,
    _capabilities: &[CapabilityInfo],
) {
    for pair in find_duplicates(symbols, 0.75) {
        edges.push(Edge {
            from: format!("symbol:{}", pair.left),
            to: format!("symbol:{}", pair.right),
            kind: EdgeKind::ConflictsWith,
            note: Some(format!("possible_duplicate:{:.2}", pair.similarity)),
        });
    }
}

pub fn apply_duplicate_flags(
    mut capabilities: Vec<CapabilityInfo>,
    symbols: &[SymbolInfo],
) -> Vec<CapabilityInfo> {
    let pairs = find_duplicates(symbols, 0.75);
    for pair in &pairs {
        for c in &mut capabilities {
            if c.implemented_by.iter().any(|s| s == &pair.left) {
                if !c.possible_duplicates.contains(&pair.right) {
                    c.possible_duplicates.push(pair.right.clone());
                }
            }
            if c.implemented_by.iter().any(|s| s == &pair.right) {
                if !c.possible_duplicates.contains(&pair.left) {
                    c.possible_duplicates.push(pair.left.clone());
                }
            }
        }
    }
    capabilities
}

pub fn format_duplicates_text(pairs: &[DuplicatePair]) -> String {
    let mut out = String::from("Possible duplicate capabilities\n\n");
    if pairs.is_empty() {
        out.push_str("_None above threshold._\n");
        return out;
    }
    for (i, p) in pairs.iter().enumerate() {
        out.push_str(&format!(
            "{}. {}::{}\n   {}::{}\n   similarity: {:.2}\n\n",
            i + 1,
            p.left_resource,
            p.left.rsplit("::").next().unwrap_or(&p.left),
            p.right_resource,
            p.right.rsplit("::").next().unwrap_or(&p.right),
            p.similarity
        ));
    }
    out
}
