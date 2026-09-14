//! Pipeline dump helpers + interactive debug (see doc/design/pipeline-debug.md, view-debug).

mod ctrl;
mod trace;

pub use crate::lex::format_lines_dump;
pub use ctrl::{
    snapshot_json, DebugAction, DebugController, DebugPause, DebugSnapshot,
};
pub use trace::emit_trace;

use crate::lex::{ClassifiedLine, LineKind};
use crate::parse::{scan_prose_line, ProseBit};

/// Line dump with v0.3 prose bits (decl / bold / italic) on each line.
pub fn format_lines_dump_v03(path: &str, lines: &[ClassifiedLine]) -> String {
    let mut out = String::new();
    out.push_str(&format!("=== marqdo: lines ({path}) ===\n"));
    for line in lines {
        let preview = if line.text.len() > 80 {
            format!("{}…", &line.text.chars().take(80).collect::<String>())
        } else {
            line.text.clone()
        };
        match line.kind {
            LineKind::Blank => {
                out.push_str(&format!("{:4}  {:7}\n", line.line_no, line.kind));
            }
            _ => {
                let tags = prose_tags(&preview);
                if tags.is_empty() {
                    out.push_str(&format!(
                        "{:4}  {:7}  | {}\n",
                        line.line_no, line.kind, preview
                    ));
                } else {
                    out.push_str(&format!(
                        "{:4}  {:7}  | {}  {}\n",
                        line.line_no, line.kind, preview, tags
                    ));
                }
            }
        }
    }
    out.push_str("=== marqdo: end lines ===\n");
    out
}

fn prose_tags(text: &str) -> String {
    let bits = scan_prose_line(text.trim_end_matches(['\r', '\n']));
    if bits.is_empty() {
        return String::new();
    }
    bits.iter()
        .map(|b| match b {
            ProseBit::Decl { name, default, .. } => match default {
                Some(d) => format!("[decl {name}={d}]"),
                None => format!("[decl {name}]"),
            },
            ProseBit::BoldCode { inner, .. } => format!("[bold {}]", compact(inner)),
            ProseBit::ItalicReturn { inner, .. } => format!("[italic {}]", compact(inner)),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn compact(s: &str) -> String {
    let t = s.trim();
    if t.chars().count() <= 40 {
        t.to_string()
    } else {
        format!("{}…", t.chars().take(40).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::classify_source;

    #[test]
    fn dump_lines_tags_decl_bold_italic() {
        let src = "对于输入变量`n`,我们执行操作**n=n+1**接着返回*n*。\n";
        let lines = classify_source(src);
        let dump = format_lines_dump_v03("t.mq.md", &lines);
        assert!(dump.contains("[decl n]"), "{dump}");
        assert!(dump.contains("[bold n=n+1]"), "{dump}");
        assert!(dump.contains("[italic n]"), "{dump}");
    }
}
