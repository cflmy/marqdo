//! L3 Capability derivation from symbols + modules.

use crate::knowledge::fingerprint::tokenize_ident;
use crate::knowledge::ir::{CapabilityInfo, Evidence, ModuleInfo, SymbolInfo};
use crate::knowledge::util::slugify;

pub fn derive_capabilities(
    symbols: &[SymbolInfo],
    modules: &[ModuleInfo],
) -> Vec<CapabilityInfo> {
    let mut caps: Vec<CapabilityInfo> = Vec::new();

    for s in symbols {
        if s.kind == "test" {
            continue;
        }
        let (id, name, desc) = if let Some(ref c) = s.capability {
            let id = slugify(c);
            (id.clone(), c.clone(), s.use_when.clone().unwrap_or_else(|| c.clone()))
        } else {
            let tokens = tokenize_ident(&s.name);
            let name = if tokens.is_empty() {
                s.name.clone()
            } else {
                // module.verb_noun style
                let mod_short = s
                    .module
                    .rsplit('/')
                    .next()
                    .unwrap_or(&s.module)
                    .replace(".mq.md", "");
                format!("{}.{}", slugify(&mod_short), tokens.join("_"))
            };
            let id = slugify(&name);
            let desc = s
                .use_when
                .clone()
                .or_else(|| {
                    modules
                        .iter()
                        .find(|m| m.id == s.module)
                        .and_then(|m| m.responsibility.clone())
                })
                .unwrap_or_else(|| format!("Capability implemented by `{}`", s.name));
            (id, name, desc)
        };

        if let Some(existing) = caps.iter_mut().find(|c| c.id == id) {
            if !existing.implemented_by.contains(&s.id) {
                existing.implemented_by.push(s.id.clone());
            }
            if !existing.modules.contains(&s.module) {
                existing.modules.push(s.module.clone());
            }
            continue;
        }

        caps.push(CapabilityInfo {
            id,
            name,
            description: desc,
            implemented_by: vec![s.id.clone()],
            modules: vec![s.module.clone()],
            status: s.status.clone(),
            use_when: s.use_when.clone(),
            do_not_use_when: s.do_not_use_when.clone(),
            related: s.related.clone(),
            evidence: Evidence {
                sources: vec![s.resource.clone()],
                tests: s.tests.clone(),
                ..Evidence::default()
            },
            confidence: s.confidence.clone(),
            introduced: None,
            verified: None,
            last_used: None,
            last_tested: None,
            possible_duplicates: Vec::new(),
        });
    }

    // related edges are authored; graph builder adds RelatedTo edges
    caps.sort_by(|a, b| a.id.cmp(&b.id));
    caps
}
