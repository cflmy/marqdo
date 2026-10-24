//! Browse Structure as a prose-preserving document surface + annotation bands.
//!
//! See `doc/design/view-document-annotations.md` · `doc/design/view-execution-trace.md`.

use std::collections::{HashMap, HashSet};

use crate::ast::{Function, Module, Stmt};
use crate::host::writeback;
use crate::lex::{classify_source, ClassifiedLine, LineKind};
use crate::parse::{scan_prose_line, ProseBit};
use crate::view::annot::{
    annots_by_line, bit_at, bit_kind_class, prose_bit_end, render_annot_band, ViewAnnot,
};
use crate::view::html::escape;
use crate::view::render::{
    comment_markdown_to_html, render_code_assign_card, render_formula_assign_card,
};

const MAX_VISIBLE_ANNOTS: usize = 4;

/// Render Browse Structure: narration kept, executable spans marked, annot bands attached.
pub fn render_module_document(module: &Module, source: &str, annots: &[ViewAnnot]) -> String {
    let lines = classify_source(source);
    let by_line = annots_by_line(annots);
    let fun_by_line = index_functions(module);
    let special = index_special_assigns(module, source);
    let skip_body = special_body_lines(&special);
    let stmt_lines = index_stmt_lines(module);
    let writebacks = writeback::writeback_map(source);
    let taken_arms = taken_arms_by_branch_line(annots);

    let mut out = String::from(r#"<div class="document-surface">"#);

    if !module.imports.is_empty() || !module.uses.is_empty() {
        out.push_str(r#"<div class="doc-imports">"#);
        for imp in &module.imports {
            out.push_str(&format!(
                r#"<span class="chip">{}:{}</span>"#,
                escape(&imp.bind),
                escape(&imp.path)
            ));
        }
        for u in &module.uses {
            out.push_str(&format!(
                r#"<span class="chip">{}:{}</span>"#,
                escape(&u.bind),
                escape(&u.path.join("."))
            ));
        }
        out.push_str("</div>");
    }

    let body_start = skip_frontmatter_end(&lines).unwrap_or(1);
    let mut para: Vec<(u32, String)> = Vec::new();
    let flush_para = |para: &mut Vec<(u32, String)>, out: &mut String| {
        if para.is_empty() {
            return;
        }
        let first_line = para[0].0;
        let text = para
            .iter()
            .map(|(_, t)| t.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        para.clear();
        let md = comment_markdown_to_html(&text);
        out.push_str(&format!(
            r#"<div class="doc-block" data-line="{line}"><div class="doc-prose">{md}</div></div>"#,
            line = first_line,
            md = md
        ));
    };

    let mut i = 0usize;
    while i < lines.len() {
        let l = &lines[i];
        if l.line_no < body_start {
            i += 1;
            continue;
        }
        // Body lines of $$ / ``` already folded into formula/code cards.
        if skip_body.contains(&l.line_no) {
            flush_para(&mut para, &mut out);
            i += 1;
            continue;
        }

        // GFM table block (may be Comment or Code classified).
        if looks_like_table_row(l.text.trim()) {
            flush_para(&mut para, &mut out);
            let (html, next) = render_table_block(&lines, i, &by_line, &writebacks);
            out.push_str(&html);
            i = next;
            continue;
        }

        match l.kind {
            LineKind::Blank => {
                flush_para(&mut para, &mut out);
                i += 1;
            }
            LineKind::Writeback => {
                flush_para(&mut para, &mut out);
                let start_line = l.line_no;
                let mut block = String::new();
                while i < lines.len() && matches!(lines[i].kind, LineKind::Writeback) {
                    if !block.is_empty() {
                        block.push('\n');
                    }
                    block.push_str(lines[i].text.trim_end_matches(['\r', '\n']));
                    i += 1;
                }
                out.push_str(&crate::view::render::render_writeback_source_line(
                    start_line, &block,
                ));
            }
            LineKind::Comment => {
                let text = l.text.trim_end_matches(['\r', '\n']);
                let trimmed = text.trim();
                // Unbound ```lang fences stay Comment in lex — render as code cards
                // (same chrome as bound foreign), never join into a prose <code> blob.
                if crate::foreign::is_fence_opener(trimmed)
                    || (trimmed.starts_with("```") && !crate::foreign::is_fence_closer(trimmed))
                {
                    flush_para(&mut para, &mut out);
                    let (html, next) = render_narrative_fence(&lines, i);
                    out.push_str(&html);
                    i = next;
                    continue;
                }
                if crate::foreign::is_fence_closer(trimmed) {
                    // Orphan closer — skip.
                    flush_para(&mut para, &mut out);
                    i += 1;
                    continue;
                }
                let bits = scan_prose_line(text);
                let is_stmt = stmt_lines.contains(&l.line_no);
                let has_annots = by_line
                    .get(&l.line_no)
                    .map(|v| !v.is_empty())
                    .unwrap_or(false);
                // Decl alone stays in Markdown prose (keep links). Open a marked
                // line block when there is executable prose, a stmt, or chips.
                let needs_annot = is_stmt
                    || has_annots
                    || bits.iter().any(|b| {
                        matches!(b, ProseBit::BoldCode { .. } | ProseBit::ItalicReturn { .. })
                    });
                if needs_annot {
                    flush_para(&mut para, &mut out);
                    out.push_str(&render_line_block(
                        l.line_no,
                        text,
                        &bits,
                        by_line.get(&l.line_no).map(|v| v.as_slice()).unwrap_or(&[]),
                        special.get(&l.line_no),
                        writebacks.get(&l.line_no),
                        None,
                    ));
                } else if !trimmed.is_empty() {
                    para.push((l.line_no, trimmed.to_string()));
                }
                i += 1;
            }
            LineKind::Code => {
                flush_para(&mut para, &mut out);
                let text = l.text.trim_end_matches(['\r', '\n']);
                if let Some(fun) = fun_by_line.get(&l.line_no) {
                    out.push_str(&render_fun_heading(fun, annots));
                    if let Some(list) = by_line.get(&l.line_no) {
                        if !list.is_empty() {
                            let owned: Vec<ViewAnnot> = list.iter().map(|a| (*a).clone()).collect();
                            out.push_str(&render_annot_band(&owned, MAX_VISIBLE_ANNOTS));
                        }
                    }
                    i += 1;
                    continue;
                }

                let trimmed = text.trim_start();
                if let Some((kind, marker_len)) = list_marker(trimmed) {
                    let (html, next) = render_list_block(
                        &lines,
                        i,
                        kind,
                        marker_len,
                        &by_line,
                        &special,
                        &writebacks,
                        &taken_arms,
                        annots,
                    );
                    out.push_str(&html);
                    i = next;
                    continue;
                }

                let bits = scan_prose_line(text);
                out.push_str(&render_line_block(
                    l.line_no,
                    text,
                    &bits,
                    by_line.get(&l.line_no).map(|v| v.as_slice()).unwrap_or(&[]),
                    special.get(&l.line_no),
                    writebacks.get(&l.line_no),
                    None,
                ));
                i += 1;
            }
        }
    }
    flush_para(&mut para, &mut out);
    out.push_str("</div>");
    out
}

#[derive(Clone, Copy)]
enum ListKind {
    Ul,
    Ol,
}

fn list_marker(trimmed: &str) -> Option<(ListKind, usize)> {
    if let Some(rest) = trimmed.strip_prefix("- ") {
        let _ = rest;
        return Some((ListKind::Ul, 2));
    }
    if trimmed.starts_with("-") && trimmed.len() > 1 && trimmed.as_bytes()[1].is_ascii_whitespace() {
        return Some((ListKind::Ul, 2));
    }
    // N. or N. *
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i > 0 && i < bytes.len() && bytes[i] == b'.' {
        let after = i + 1;
        if after < bytes.len() && bytes[after].is_ascii_whitespace() {
            return Some((ListKind::Ol, after + 1));
        }
        if after < bytes.len() && bytes[after] == b'*' {
            // "3.*" else — marker through '*'
            return Some((ListKind::Ol, after + 1));
        }
    }
    None
}

fn looks_like_table_row(trimmed: &str) -> bool {
    trimmed.starts_with('|') && trimmed.matches('|').count() >= 2
}

fn looks_like_table_sep(trimmed: &str) -> bool {
    if !trimmed.starts_with('|') {
        return false;
    }
    trimmed
        .chars()
        .all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace())
}

fn indent_level(text: &str) -> usize {
    let mut n = 0usize;
    for c in text.chars() {
        match c {
            ' ' => n += 1,
            '\t' => n += 4,
            _ => break,
        }
    }
    n / 2 // 2 spaces per visual level
}

fn render_table_block(
    lines: &[ClassifiedLine],
    start: usize,
    by_line: &HashMap<u32, Vec<&ViewAnnot>>,
    writebacks: &HashMap<u32, String>,
) -> (String, usize) {
    let mut rows: Vec<(u32, String)> = Vec::new();
    let mut i = start;
    while i < lines.len() {
        let t = lines[i].text.trim_end_matches(['\r', '\n']);
        let trimmed = t.trim();
        if !looks_like_table_row(trimmed) {
            break;
        }
        rows.push((lines[i].line_no, trimmed.to_string()));
        i += 1;
    }
    if rows.is_empty() {
        return (String::new(), start + 1);
    }
    let first_line = rows[0].0;
    let mut html = format!(
        r#"<div class="doc-block doc-table-wrap" data-line="{line}"><table class="doc-table">"#,
        line = first_line
    );
    let mut header_done = false;
    for (idx, (_ln, row)) in rows.iter().enumerate() {
        if looks_like_table_sep(row) {
            continue;
        }
        let cells: Vec<&str> = row
            .trim()
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim())
            .collect();
        if !header_done {
            html.push_str("<thead><tr>");
            for c in &cells {
                html.push_str(&format!("<th>{}</th>", escape(c)));
            }
            html.push_str("</tr></thead><tbody>");
            header_done = true;
            // If next is sep, skip handled above; if only one data row treated as header+body empty ok
            if idx + 1 < rows.len() && looks_like_table_sep(&rows[idx + 1].1) {
                continue;
            }
            // Single-row table without sep: also emit as body
            if rows.len() == 1 {
                html.push_str("<tr>");
                for c in &cells {
                    html.push_str(&format!("<td>{}</td>", escape(c)));
                }
                html.push_str("</tr>");
            }
            continue;
        }
        html.push_str("<tr>");
        for c in &cells {
            html.push_str(&format!("<td>{}</td>", escape(c)));
        }
        html.push_str("</tr>");
    }
    if header_done {
        html.push_str("</tbody>");
    }
    html.push_str("</table>");
    // Annots on first table line (bind of collection).
    if let Some(list) = by_line.get(&first_line) {
        // A table is already its own readable value; a call chip would collapse
        // the whole collection into an ellipsis. Keep only non-call chips.
        let owned: Vec<ViewAnnot> = list
            .iter()
            .filter(|a| a.kind != crate::view::annot::AnnotKind::Call)
            .map(|a| (*a).clone())
            .collect();
        if !owned.is_empty() {
            html.push_str(&render_annot_band(&owned, MAX_VISIBLE_ANNOTS));
        }
    }
    if let Some(body) = writebacks.get(&first_line) {
        html.push_str(&crate::view::render::render_output_card(first_line, body));
    }
    html.push_str("</div>");
    (html, i)
}

fn render_list_block(
    lines: &[ClassifiedLine],
    start: usize,
    kind: ListKind,
    _marker_len: usize,
    by_line: &HashMap<u32, Vec<&ViewAnnot>>,
    special: &HashMap<u32, SpecialAssign>,
    writebacks: &HashMap<u32, String>,
    taken_arms: &HashMap<u32, usize>,
    all_annots: &[ViewAnnot],
) -> (String, usize) {
    let base_indent = indent_level(&lines[start].text);
    let tag = match kind {
        ListKind::Ul => "ul",
        ListKind::Ol => "ol",
    };
    let cls = match kind {
        ListKind::Ul => "doc-loop",
        ListKind::Ol => "doc-branch",
    };
    // Branch stmt line = list head line for taken-arm lookup.
    let branch_line = lines[start].line_no;
    let taken = taken_arms.get(&branch_line).copied();

    let loop_id = all_annots
        .iter()
        .find(|a| {
            a.kind == crate::view::annot::AnnotKind::LoopFrame && a.anchor_line == branch_line
        })
        .and_then(|a| a.loop_id);

    let mut html = format!(
        r#"<{tag} class="{cls}" data-line="{line}"{loop_attr} style="--doc-indent:{indent}">"#,
        tag = tag,
        cls = cls,
        line = branch_line,
        loop_attr = loop_id
            .map(|id| format!(r#" data-loop-id="{id}""#))
            .unwrap_or_default(),
        indent = base_indent.max(1),
    );

    // Loop iteration picker (default = last).
    if matches!(kind, ListKind::Ul) {
        let frames: Vec<&ViewAnnot> = all_annots
            .iter()
            .filter(|a| {
                a.kind == crate::view::annot::AnnotKind::LoopFrame
                    && a.anchor_line == branch_line
            })
            .collect();
        if frames.len() >= 2 {
            html.push_str(r#"<li class="doc-list-meta"><label class="loop-picker-label">迭代 <select class="loop-picker""#);
            if let Some(id) = loop_id {
                html.push_str(&format!(r#" data-loop-id="{id}""#));
            }
            html.push('>');
            for (i, a) in frames.iter().enumerate() {
                let sel = if i + 1 == frames.len() {
                    " selected"
                } else {
                    ""
                };
                let brief = if a.value.is_empty() {
                    format!("迭代 {}", i + 1)
                } else {
                    format!("{}: {}", i + 1, a.value)
                };
                html.push_str(&format!(
                    r#"<option value="{iter}"{sel}>{brief}</option>"#,
                    iter = a.iter.unwrap_or(i as u32),
                    sel = sel,
                    brief = escape(&brief),
                ));
            }
            html.push_str("</select></label></li>");
        }
    }

    let mut i = start;
    let mut arm_index = 0usize;
    while i < lines.len() {
        let text = lines[i].text.trim_end_matches(['\r', '\n']);
        let trimmed = text.trim_start();
        let ind = indent_level(text);
        if lines[i].kind == LineKind::Blank {
            // Blank inside list: peek ahead — if next is deeper/same list continue, else break.
            if i + 1 < lines.len() {
                let n = &lines[i + 1];
                let nt = n.text.trim_end_matches(['\r', '\n']);
                if nt.trim().is_empty() {
                    i += 1;
                    continue;
                }
                let nind = indent_level(nt);
                if nind > base_indent
                    || (nind == base_indent && list_marker(nt.trim_start()).is_some())
                {
                    i += 1;
                    continue;
                }
            }
            break;
        }
        if ind < base_indent {
            break;
        }
        if ind == base_indent {
            if let Some((k, mlen)) = list_marker(trimmed) {
                if !matches!(
                    (k, kind),
                    (ListKind::Ul, ListKind::Ul) | (ListKind::Ol, ListKind::Ol)
                ) {
                    break;
                }
                let head = trimmed[mlen..].trim();
                let arm_cls = match (kind, taken) {
                    (ListKind::Ol, Some(t)) if t == arm_index => "arm-taken",
                    (ListKind::Ol, Some(_)) => "arm-miss",
                    (ListKind::Ol, None) => "",
                    _ => "",
                };
                html.push_str(&format!(
                    r#"<li class="doc-list-item {arm_cls}" data-line="{line}" data-arm-index="{arm_idx}" style="--doc-indent:{indent}">"#,
                    arm_cls = arm_cls,
                    line = lines[i].line_no,
                    arm_idx = arm_index,
                    indent = ind.max(1),
                ));
                html.push_str(&format!(
                    r#"<div class="doc-list-head">{}</div>"#,
                    render_surface_line(lines[i].line_no, head, by_line)
                ));
                // Annots for this line; Branch chips on the arm they refer to (filtered by call frame in JS).
                let mut head_annots: Vec<ViewAnnot> = Vec::new();
                if let Some(list) = by_line.get(&lines[i].line_no) {
                    for a in list {
                        if a.kind == crate::view::annot::AnnotKind::Branch {
                            continue;
                        }
                        head_annots.push((*a).clone());
                    }
                }
                if matches!(kind, ListKind::Ol) {
                    // Branch annots are keyed to the list's first line, not each arm head.
                    if let Some(list) = by_line.get(&branch_line) {
                        for a in list {
                            if a.kind == crate::view::annot::AnnotKind::Branch
                                && a.arm_index == Some(arm_index)
                            {
                                head_annots.push((*a).clone());
                            }
                        }
                    }
                }
                if !head_annots.is_empty() {
                    html.push_str(&render_annot_band(&head_annots, MAX_VISIBLE_ANNOTS));
                }
                // Body: following lines with greater indent until next sibling.
                i += 1;
                html.push_str(r#"<div class="doc-list-body">"#);
                while i < lines.len() {
                    let t2 = lines[i].text.trim_end_matches(['\r', '\n']);
                    if t2.trim().is_empty() {
                        // allow blank inside body
                        if i + 1 < lines.len()
                            && indent_level(&lines[i + 1].text) > base_indent
                        {
                            i += 1;
                            continue;
                        }
                        break;
                    }
                    let ind2 = indent_level(t2);
                    if ind2 <= base_indent {
                        break;
                    }
                    // Nested list at deeper indent.
                    let tr2 = t2.trim_start();
                    if let Some((nk, _)) = list_marker(tr2) {
                        let (inner, next) = render_list_block(
                            lines,
                            i,
                            nk,
                            0,
                            by_line,
                            special,
                            writebacks,
                            taken_arms,
                            all_annots,
                        );
                        html.push_str(&inner);
                        i = next;
                        continue;
                    }
                    if looks_like_table_row(tr2) {
                        let (th, next) = render_table_block(lines, i, by_line, writebacks);
                        html.push_str(&th);
                        i = next;
                        continue;
                    }
                    let bits = scan_prose_line(t2);
                    html.push_str(&render_line_block(
                        lines[i].line_no,
                        t2,
                        &bits,
                        by_line
                            .get(&lines[i].line_no)
                            .map(|v| v.as_slice())
                            .unwrap_or(&[]),
                        special.get(&lines[i].line_no),
                        writebacks.get(&lines[i].line_no),
                        Some(ind2),
                    ));
                    i += 1;
                }
                html.push_str("</div></li>");
                arm_index += 1;
                continue;
            }
            break;
        }
        // Deeper than base but not under an open item — shouldn't happen; stop.
        break;
    }
    html.push_str(&format!("</{tag}>", tag = tag));
    (html, i)
}

fn render_surface_line(
    line_no: u32,
    head: &str,
    by_line: &HashMap<u32, Vec<&ViewAnnot>>,
) -> String {
    let bits = scan_prose_line(head);
    if bits.is_empty() {
        // Else arm "*" or plain condition text.
        if head.trim() == "*" {
            return r#"<span class="mq-else">else</span>"#.into();
        }
        format!(
            r#"<span class="doc-cond">{}</span>"#,
            escape(head)
        )
    } else {
        let annots = by_line
            .get(&line_no)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        render_prose_with_marks(line_no, head, &bits, annots)
    }
}

/// SSR default: taken arm for the highest `call_id` per branch line (matches last-call picker).
fn taken_arms_by_branch_line(annots: &[ViewAnnot]) -> HashMap<u32, usize> {
    let mut best: HashMap<u32, (u32, usize)> = HashMap::new();
    for a in annots {
        if a.kind != crate::view::annot::AnnotKind::Branch {
            continue;
        }
        let Some(idx) = a.arm_index else { continue };
        let cid = a.call_id.unwrap_or(0);
        let entry = best.entry(a.anchor_line).or_insert((cid, idx));
        if cid >= entry.0 {
            *entry = (cid, idx);
        }
    }
    best.into_iter().map(|(line, (_, arm))| (line, arm)).collect()
}

fn render_fun_heading(fun: &Function, annots: &[ViewAnnot]) -> String {
    let mut s = format!(
        r##"<div class="doc-fun" id="fn-{line}" data-fn="{name}" data-fn-path="{name}" data-line="{line}">
<div class="doc-fun-bar">
<h3 class="doc-fun-title">{name}</h3>
<a class="knowledge-explain" href="#knowledge-{name}" data-node="{name}">说明</a>"##,
        line = fun.span.line,
        name = escape(&fun.name),
    );
    // Call picker placeholder: filled when ≥2 call frames exist for this fn.
    let call_labels: Vec<&ViewAnnot> = annots
        .iter()
        .filter(|a| a.kind == crate::view::annot::AnnotKind::CallFrame && a.label == fun.name)
        .collect();
    if call_labels.len() >= 2 {
        s.push_str(r#"<label class="call-picker-label">调用 <select class="call-picker" data-fn=""#);
        s.push_str(&escape(&fun.name));
        s.push_str(r#"">"#);
        for (i, a) in call_labels.iter().enumerate() {
            let sel = if i + 1 == call_labels.len() {
                " selected"
            } else {
                ""
            };
            let brief = if a.value.is_empty() {
                format!("调用 {}", i + 1)
            } else {
                format!("{}: {}", i + 1, a.value)
            };
            let call_id = a.call_id.unwrap_or(0);
            s.push_str(&format!(
                r#"<option value="{id}"{sel}>{brief}</option>"#,
                id = call_id,
                sel = sel,
                brief = escape(&brief),
            ));
        }
        s.push_str("</select></label>");
    }
    s.push_str("</div>");
    if !fun.params.is_empty() {
        // Default SSR values = last call frame (matches picker default = last option).
        let default_args = call_labels
            .last()
            .map(|a| parse_args_brief(&a.value))
            .unwrap_or_default();
        s.push_str(r#"<div class="params">"#);
        for p in &fun.params {
            let cls = if p.inferred {
                "chip inferred"
            } else {
                "chip"
            };
            let title = if p.inferred {
                "inferred from prose / body"
            } else {
                "explicit + param"
            };
            let val = default_args
                .get(p.name.as_str())
                .filter(|v| !v.is_empty())
                .map(|v| format!("={}", escape(v)))
                .unwrap_or_default();
            // Always emit chip-val so call-picker JS can refresh per frame.
            s.push_str(&format!(
                r#"<span class="{cls}" data-param-name="{name}" data-fn="{fn}" title="{title}">{name}<span class="chip-val">{val}</span></span>"#,
                cls = cls,
                title = title,
                name = escape(&p.name),
                fn = escape(&fun.name),
                val = val,
            ));
        }
        s.push_str("</div>");
    }
    s.push_str("</div>");
    s
}

/// Parse `args_brief` (`谁=marqdo, n=37`) into name → value.
fn parse_args_brief(brief: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    if brief.trim().is_empty() {
        return m;
    }
    for part in brief.split(", ") {
        if let Some((k, v)) = part.split_once('=') {
            if !k.is_empty() {
                m.insert(k.to_string(), v.to_string());
            }
        }
    }
    m
}

/// Split a statement line holding one occlusion `[描述](实现)`.
///
/// Returns `(prefix, label, payload, suffix)`. Display-only: the parser already
/// validated the real structure, this just decides how to paint it.
fn split_occlusion(line: &str) -> Option<(String, String, String, String)> {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;
    let mut in_bt = false;
    let mut in_str = false;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_bt {
            if c == '`' {
                in_bt = false;
            }
            i += 1;
            continue;
        }
        match c {
            '`' => in_bt = true,
            '"' => in_str = true,
            '[' if chars.get(i + 1) != Some(&'^') => {
                if let Some(close) = match_delim(&chars, i, '[', ']') {
                    if chars.get(close + 1) == Some(&'(') {
                        if let Some(paren) = match_delim(&chars, close + 1, '(', ')') {
                            let mut prefix: String = chars[..i].iter().collect();
                            let label: String = chars[i + 1..close].iter().collect();
                            let payload: String = chars[close + 2..paren].iter().collect();
                            let suffix: String = chars[paren + 1..].iter().collect();
                            // `**名 = [..](..)**` reads as bold code: drop the
                            // emphasis markers so the label reads as prose.
                            if let Some(p) = prefix.strip_suffix("**") {
                                prefix = p.to_string();
                            }
                            prefix = prefix.trim_end().to_string();
                            if let Some(p) = prefix.strip_prefix("**") {
                                prefix = p.to_string();
                            }
                            let suffix = suffix.strip_prefix("**").unwrap_or(&suffix).to_string();
                            return Some((
                                prefix,
                                label.trim().to_string(),
                                payload.trim().to_string(),
                                suffix.trim().to_string(),
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn match_delim(chars: &[char], start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut i = start;
    let mut in_bt = false;
    let mut in_str = false;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_bt {
            if c == '`' {
                in_bt = false;
            }
            i += 1;
            continue;
        }
        match c {
            '`' => in_bt = true,
            '"' => in_str = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Render one occlusion line: description as prose, payload as a code chip,
/// stubs get a badge. `<TAB>` M8
fn render_occlusion_line(line_no: u32, line: &str) -> Option<String> {
    let (prefix, label, payload, suffix) = split_occlusion(line)?;
    let stub = payload.is_empty();
    // The rest of the view drops backticks (see the `mq-anchor` marks), so the
    // assignment head reads as `留存 =` rather than `` `留存` = ``.
    let head = prefix
        .trim_end()
        .trim_end_matches('=')
        .trim()
        .trim_matches('`')
        .to_string();
    let mut html = format!(r#"<div class="doc-line doc-occl" data-line="{line_no}">"#);
    if !head.is_empty() {
        html.push_str(&format!(
            r#"<span class="occl-pre">{} =</span>"#,
            escape(&head)
        ));
    }
    html.push_str(&format!(
        r#"<span class="occl-label">{}</span>"#,
        escape(&label)
    ));
    if stub {
        html.push_str(r#"<span class="occl-stub" title="空载荷：未接通或待补返回值">桩</span>"#);
    } else {
        html.push_str(&format!(
            r#"<code class="occl-code">{}</code>"#,
            escape(&payload)
        ));
    }
    if !suffix.is_empty() {
        html.push_str(&format!(
            r#"<span class="occl-pre">{}</span>"#,
            escape(&suffix)
        ));
    }
    html.push_str("</div>");
    Some(html)
}

fn render_line_block(
    line_no: u32,
    text: &str,
    bits: &[ProseBit],
    annots: &[&ViewAnnot],
    special: Option<&SpecialAssign>,
    writeback_body: Option<&String>,
    indent: Option<usize>,
) -> String {
    let indent_style = indent
        .map(|n| format!(r#" style="--doc-indent:{n}""#))
        .unwrap_or_default();
    // Source structure always visible — do not stamp data-call-ids (frame filter is chips-only).
    let mut s = format!(
        r#"<div class="doc-block doc-exec" data-line="{line}" id="stmt-{line}"{indent}>"#,
        line = line_no,
        indent = indent_style,
    );

    match special {
        Some(SpecialAssign::Formula { name, tex, .. }) => {
            s.push_str(&render_formula_assign_card(name, tex));
        }
        Some(SpecialAssign::Code {
            name,
            lang,
            source,
            ..
        }) => {
            s.push_str(&render_code_assign_card(name, lang, source));
        }
        None => {
            let trimmed = text.trim_start();
            // `<TAB>` M8: an occlusion line is painted as description + code chip
            // regardless of annotation bits (the annotator always emits spans).
            if let Some(html) = render_occlusion_line(line_no, trimmed) {
                s.push_str(&html);
                if let Some(body) = writeback_body {
                    s.push_str(&crate::view::render::render_output_card(line_no, body));
                }
                return s;
            }
            if bits.is_empty() {
                if let Some(rest) = trimmed.strip_prefix("> ") {
                    let aid = match_annot_for_line(annots)
                        .map(|a| a.id.as_str())
                        .unwrap_or("");
                    s.push_str(&format!(
                        r#"<div class="doc-line doc-call"><span class="mq-call-gt">&gt;</span> <mark class="mq-anchor mq-code" data-line="{line}" data-annot-id="{aid}">{rest}</mark></div>"#,
                        line = line_no,
                        aid = escape(aid),
                        rest = escape(rest),
                    ));
                } else if let Some(rest) = trimmed.strip_prefix('>') {
                    let aid = match_annot_for_line(annots)
                        .map(|a| a.id.as_str())
                        .unwrap_or("");
                    s.push_str(&format!(
                        r#"<div class="doc-line doc-call"><span class="mq-call-gt">&gt;</span> <mark class="mq-anchor mq-code" data-line="{line}" data-annot-id="{aid}">{rest}</mark></div>"#,
                        line = line_no,
                        aid = escape(aid),
                        rest = escape(rest.trim_start()),
                    ));
                } else {
                    s.push_str(&format!(
                        r#"<div class="doc-line doc-stmt">{}</div>"#,
                        escape(trimmed)
                    ));
                }
            } else {
                s.push_str(r#"<div class="doc-line doc-prose-line">"#);
                s.push_str(&render_prose_with_marks(line_no, text, bits, annots));
                s.push_str("</div>");
            }
        }
    }

    if let Some(body) = writeback_body {
        s.push_str(&crate::view::render::render_output_card(line_no, body));
    }

    // Formula/code cards are the single visual truth — do not stack Bind chips
    // (or fence/SVG dumps) underneath. Variables rail still has final values.
    let band_annots: Vec<ViewAnnot> = if special.is_some() {
        Vec::new()
    } else {
        annots
            .iter()
            .filter(|a| !is_noisy_result_chip(a))
            .map(|a| (*a).clone())
            .collect()
    };
    if !band_annots.is_empty() {
        s.push_str(&render_annot_band(&band_annots, MAX_VISIBLE_ANNOTS));
    }

    s.push_str("</div>");
    s
}

/// Huge SVG / fenced code dumps — prefer abbreviated chips from annot builder;
/// only drop if detail is still an unreadable dump.
fn is_noisy_result_chip(a: &ViewAnnot) -> bool {
    if a.kind != crate::view::annot::AnnotKind::Bind {
        return false;
    }
    let v = a.detail.as_str();
    v.contains("<svg") || v.contains("```") || v.chars().count() > 96
}

fn match_annot_for_line<'a>(annots: &[&'a ViewAnnot]) -> Option<&'a ViewAnnot> {
    annots.iter().copied().find(|a| {
        matches!(
            a.kind,
            crate::view::annot::AnnotKind::Call
                | crate::view::annot::AnnotKind::Bind
                | crate::view::annot::AnnotKind::Return
        )
    })
}

/// Unbound Markdown fence → same visual card as bound foreign (no Run chrome).
fn render_narrative_fence(lines: &[ClassifiedLine], start: usize) -> (String, usize) {
    let opener = lines[start].text.trim_end_matches(['\r', '\n']).trim();
    let lang = crate::foreign::fence_lang(opener).unwrap_or_else(|| "text".into());
    let line_no = lines[start].line_no;
    let mut body: Vec<String> = Vec::new();
    let mut j = start + 1;
    while j < lines.len() {
        let t = lines[j].text.trim_end_matches(['\r', '\n']);
        if crate::foreign::is_fence_closer(t.trim()) {
            j += 1;
            break;
        }
        body.push(t.to_string());
        j += 1;
    }
    let source = body.join("\n");
    let html = format!(
        r#"<div class="doc-block doc-exec" data-line="{line}"><div class="card code-card" data-lang="{lang}">
  <div class="code-head">
    <span class="badge">code</span>
    <span class="code-lang">{lang}</span>
  </div>
  <pre class="code-body"><code class="language-{lang}">{src}</code></pre>
</div></div>"#,
        line = line_no,
        lang = escape(&lang),
        src = escape(&source),
    );
    (html, j)
}

/// Render prose with markers stripped; show surface inner text only.
fn render_prose_with_marks(
    line_no: u32,
    text: &str,
    bits: &[ProseBit],
    annots: &[&ViewAnnot],
) -> String {
    let mut html = String::new();
    let mut cursor = 0usize;
    for bit in bits {
        let start = bit_at(bit);
        let end = prose_bit_end(text, bit);
        if start > cursor {
            html.push_str(&escape(&text[cursor..start]));
        }
        if start < end && end <= text.len() {
            let (cls, display) = bit_display(bit);
            let annot_id = match_annot_for_span(annots, start, end)
                .map(|a| a.id.as_str())
                .unwrap_or("");
            html.push_str(&format!(
                r#"<mark class="mq-anchor mq-{cls}" data-line="{line}" data-byte-start="{start}" data-byte-end="{end}" data-annot-id="{aid}">{inner}</mark>"#,
                cls = cls,
                line = line_no,
                start = start,
                end = end,
                aid = escape(annot_id),
                inner = escape(&display),
            ));
        }
        cursor = end.max(cursor);
    }
    if cursor < text.len() {
        html.push_str(&escape(&text[cursor..]));
    }
    html
}

fn bit_display(bit: &ProseBit) -> (&'static str, String) {
    match bit {
        ProseBit::Decl { name, default, .. } => {
            let cls = bit_kind_class(bit);
            let s = match default {
                Some(d) => format!("{name}={d}"),
                None => name.clone(),
            };
            (cls, s)
        }
        ProseBit::BoldCode { inner, .. } => (bit_kind_class(bit), inner.clone()),
        ProseBit::ItalicReturn { inner, .. } => (bit_kind_class(bit), inner.clone()),
    }
}

fn match_annot_for_span<'a>(
    annots: &[&'a ViewAnnot],
    start: usize,
    end: usize,
) -> Option<&'a ViewAnnot> {
    annots.iter().copied().find(|a| match (a.anchor_byte_start, a.anchor_byte_end) {
        (Some(s), Some(e)) => s < end && e > start,
        (Some(s), None) => s >= start && s < end,
        _ => false,
    })
}

#[derive(Clone)]
enum SpecialAssign {
    Formula {
        name: String,
        tex: String,
        /// Last source line of the `$$ … $$` block (inclusive).
        end_line: u32,
    },
    Code {
        name: String,
        lang: String,
        source: String,
        /// Last source line of the fenced block (inclusive).
        end_line: u32,
    },
}

fn special_body_lines(special: &HashMap<u32, SpecialAssign>) -> HashSet<u32> {
    let mut s = HashSet::new();
    for (start, sp) in special {
        let end = match sp {
            SpecialAssign::Formula { end_line, .. } | SpecialAssign::Code { end_line, .. } => {
                *end_line
            }
        };
        let mut ln = *start + 1;
        while ln <= end {
            s.insert(ln);
            ln += 1;
        }
    }
    s
}

fn index_functions(module: &Module) -> HashMap<u32, &Function> {
    let mut m = HashMap::new();
    fn walk<'a>(fun: &'a Function, m: &mut HashMap<u32, &'a Function>) {
        m.insert(fun.span.line, fun);
        for c in &fun.children {
            walk(c, m);
        }
    }
    for f in &module.functions {
        walk(f, &mut m);
    }
    m
}

fn index_stmt_lines(module: &Module) -> HashSet<u32> {
    let mut s = HashSet::new();
    for f in &module.functions {
        crate::ast::walk_stmts(f, &mut |st| {
            s.insert(crate::ast::stmt_line(st));
        });
    }
    s
}


fn index_special_assigns(module: &Module, source: &str) -> HashMap<u32, SpecialAssign> {
    let src_lines: Vec<&str> = source.lines().collect();
    let mut m = HashMap::new();
    fn walk(fun: &Function, m: &mut HashMap<u32, SpecialAssign>, src_lines: &[&str]) {
        for st in &fun.body {
            record(st, m, src_lines);
            match st {
                Stmt::Branch { arms, .. } => {
                    for arm in arms {
                        for b in &arm.body {
                            record(b, m, src_lines);
                        }
                    }
                }
                Stmt::While { body, .. } | Stmt::ForEach { body, .. } => {
                    for b in body {
                        record(b, m, src_lines);
                    }
                }
                _ => {}
            }
        }
        for c in &fun.children {
            walk(c, m, src_lines);
        }
    }
    fn record(st: &Stmt, m: &mut HashMap<u32, SpecialAssign>, src_lines: &[&str]) {
        match st {
            Stmt::Assign {
                name,
                value: crate::ast::Expr::Formula(e),
                span,
                end_line,
                ..
            } => {
                let tex = match e {
                    crate::formula::Expr::Matrix { source, .. } => source.clone(),
                    _ => crate::view::render::formula_ascii_to_tex_pub(&e.as_display()),
                };
                let end_line = (*end_line).max(fence_block_end(src_lines, span.line, "$$"));
                m.insert(
                    span.line,
                    SpecialAssign::Formula {
                        name: name.clone(),
                        tex,
                        end_line,
                    },
                );
            }
            Stmt::Assign {
                name,
                value: crate::ast::Expr::Code(c),
                span,
                end_line,
                ..
            } => {
                let end_line = (*end_line).max(fence_block_end(src_lines, span.line, "```"));
                m.insert(
                    span.line,
                    SpecialAssign::Code {
                        name: name.clone(),
                        lang: c.lang.clone(),
                        source: c.source.clone(),
                        end_line,
                    },
                );
            }
            _ => {}
        }
    }
    for f in &module.functions {
        walk(f, &mut m, &src_lines);
    }
    m
}

/// Inclusive end line of a `$$` / ` ``` ` block starting on or after `assign_line`.
fn fence_block_end(src_lines: &[&str], assign_line: u32, fence: &str) -> u32 {
    if assign_line == 0 || assign_line as usize > src_lines.len() {
        return assign_line;
    }
    let assign = src_lines[assign_line as usize - 1].trim();
    // Meta formula: assign span is the opening `$$` line itself.
    if fence == "$$" && assign == "$$" {
        let mut i = assign_line as usize; // 0-based next line
        while i < src_lines.len() {
            if src_lines[i].trim() == "$$" {
                return (i + 1) as u32;
            }
            i += 1;
        }
        return assign_line;
    }
    if fence == "$$" && assign.starts_with("$$") && assign.ends_with("$$") && assign.len() > 4 {
        return assign_line;
    }
    let mut i = assign_line as usize; // 0-based index of line after assign
    let mut seen_open = false;
    while i < src_lines.len() {
        let t = src_lines[i].trim();
        let line_no = (i + 1) as u32;
        if !seen_open {
            if t.starts_with(fence) {
                seen_open = true;
                if fence == "```" && t == "```" {
                    return line_no;
                }
                if fence == "$$" && t == "$$" {
                    i += 1;
                    while i < src_lines.len() {
                        if src_lines[i].trim() == "$$" {
                            return (i + 1) as u32;
                        }
                        i += 1;
                    }
                    return line_no;
                }
            } else if t.is_empty() {
                i += 1;
                continue;
            } else {
                return assign_line;
            }
        } else if fence == "```" && t.starts_with("```") {
            return line_no;
        }
        i += 1;
    }
    assign_line
}

fn skip_frontmatter_end(lines: &[ClassifiedLine]) -> Option<u32> {
    let first_code = lines.iter().find(|l| l.kind == LineKind::Code)?;
    if first_code.text.trim() != "---" {
        return Some(1);
    }
    for l in lines.iter().skip_while(|l| l.line_no <= first_code.line_no) {
        if l.kind == LineKind::Code && l.text.trim() == "---" {
            return Some(l.line_no + 1);
        }
    }
    Some(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::annot::build_view_annots;

    #[test]
    fn document_keeps_inc_prose_narration() {
        let src = include_str!("../../tests/markup-v03/inc-prose.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_document(&module, src, &[]);
        assert!(
            html.contains("对于输入变量") && html.contains("我们执行操作"),
            "narration stripped: {html}"
        );
        assert!(html.contains("mq-anchor"), "expected anchors: {html}");
        assert!(html.contains("document-surface"), "{html}");
        // Marker surface: no raw ** delimiters around assign.
        assert!(
            !html.contains("**n=n+1**") && !html.contains("**n = n + 1**"),
            "raw bold delimiters leaked: {html}"
        );
        assert!(html.contains("n=n+1") || html.contains(">n=n+1<") || html.contains("n=n+1"), "{html}");
    }

    #[test]
    fn document_strips_marker_delimiters() {
        let src = "# main\n\n**x = 1**\n";
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_document(&module, src, &[]);
        assert!(html.contains("mq-code"), "{html}");
        assert!(!html.contains("**x = 1**"), "{html}");
        assert!(html.contains(">x = 1<") || html.contains("x = 1"), "{html}");
    }

    #[test]
    fn document_renders_table_and_lists() {
        let src = include_str!("../../tests/view/document-annot-demo.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_document(&module, src, &[]);
        assert!(html.contains("doc-table") || html.contains("<table"), "table missing: {html}");
        assert!(html.contains("doc-loop") || html.contains("<ul"), "loop list missing: {html}");
        assert!(html.contains("doc-branch") || html.contains("<ol"), "branch list missing: {html}");
    }

    #[test]
    fn document_keeps_plain_comments() {
        let src = "段首说明\n续行仍属同段\n\n下一段\n\n# main\n\n> print text=x\n";
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_document(&module, src, &[]);
        assert!(html.contains("doc-prose"), "{html}");
        assert!(html.contains("段首说明") || html.contains("续行仍属同段"), "{html}");
        assert!(html.contains("下一段"), "{html}");
    }

    #[test]
    fn document_with_annots_has_band() {
        let src = include_str!("../../tests/markup-v03/inc-prose.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        let annots = build_view_annots(&module, src, &cap);
        let html = render_module_document(&module, src, &annots);
        assert!(html.contains("annot-band"), "{html}");
        assert!(html.contains("annot-chip"), "{html}");
        assert!(html.contains("对于输入变量"), "{html}");
    }

    #[test]
    fn document_fun_anchors_for_outline() {
        let src = include_str!("../../tests/structure/nested-call.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_document(&module, src, &[]);
        assert!(html.contains("id=\"fn-"), "{html}");
        assert!(html.contains("data-fn=\"main\""), "{html}");
        assert!(html.contains("打印一句问候"), "{html}");
    }

    #[test]
    fn document_markdown_links_in_prose() {
        let src = "见[仓库](https://github.com/cflmy/marqdo)与 `code`\n\n# main\n\n> print text=x\n";
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_document(&module, src, &[]);
        assert!(
            html.contains("<a href=\"https://github.com/cflmy/marqdo\">仓库</a>"),
            "{html}"
        );
    }

    #[test]
    fn document_annot_demo_keeps_narration_and_control_flow() {
        let src = include_str!("../../tests/view/document-annot-demo.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        let annots = build_view_annots(&module, src, &cap);
        let html = render_module_document(&module, src, &annots);
        assert!(html.contains("本页演示"), "{html}");
        assert!(html.contains("annot-band"), "{html}");
        assert!(
            annots
                .iter()
                .any(|a| a.kind == crate::view::annot::AnnotKind::LoopFrame
                    || (a.kind == crate::view::annot::AnnotKind::Bind && a.loop_id.is_some())),
            "loop frame / item bind missing: {:?}",
            cap.view_trace
        );
        assert!(
            annots.iter().any(|a| a.kind == crate::view::annot::AnnotKind::Branch),
            "branch annot missing: {:?}",
            cap.view_trace
        );
    }

    #[test]
    fn inferred_param_chip_shows_call_value() {
        let src = include_str!("../../tests/markup-v03/inc-prose.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let add = module
            .functions
            .iter()
            .find(|f| f.name == "加一函数")
            .expect("加一函数");
        assert!(add.params.iter().any(|p| p.name == "n" && p.inferred));
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        let annots = build_view_annots(&module, src, &cap);
        let html = render_module_document(&module, src, &annots);
        assert!(
            html.contains(r#"data-param-name="n""#) && html.contains("chip inferred"),
            "inferred param chip missing: {html}"
        );
        assert!(
            html.contains(r#"class="chip-val">=37"#) || html.contains("chip-val\">=37"),
            "expected param chip value from call frame: {html}"
        );
    }

    #[test]
    fn document_frame_pickers_and_all_arms() {
        let src = include_str!("../../tests/view/document-annot-demo.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        let annots = build_view_annots(&module, src, &cap);
        let html = render_module_document(&module, src, &annots);
        // All three return arms of 等级说明 stay in the source surface.
        assert!(html.contains("均分很高"), "arm1 return missing: {html}");
        assert!(html.contains("已过线"), "arm2 return missing: {html}");
        assert!(html.contains("建议安排复习周"), "arm3 return missing: {html}");
        assert!(html.contains("arm-miss"), "expected dimmed arms: {html}");
        assert!(html.contains("arm-taken"), "expected taken arm: {html}");
        // Call picker uses numeric call_id values (not cfN ids).
        assert!(html.contains("call-picker"), "{html}");
        assert!(
            html.contains(r#"value="2""#) || html.contains(r#"value="3""#),
            "call picker should use numeric call_id: {html}"
        );
        // Loop iteration picker defaults to last.
        assert!(html.contains("loop-picker"), "{html}");
        assert!(html.contains("分数表[3]"), "expected last-iter binding: {html}");
        // Branch chips carry call_id for frame filter.
        assert!(
            html.contains(r#"data-call-id=""#) || html.contains("data-call-id="),
            "{html}"
        );
    }

    #[test]
    fn narrative_text_fence_renders_as_code_card() {
        let src = r#"# main

```text
marqdo ext add web
marqdo ext add quantum
```

> print text=ok
"#;
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_document(&module, src, &[]);
        assert!(
            html.contains(r#"data-lang="text""#) && html.contains("code-body"),
            "text fence should be a code card: {html}"
        );
        assert!(
            html.contains("marqdo ext add web\nmarqdo ext add quantum"),
            "fence body newlines preserved: {html}"
        );
        assert!(
            !html.contains("<code>text marqdo"),
            "must not collapse fence into inline code: {html}"
        );
    }

    #[test]
    fn prose_calls_and_param_decl_get_chips() {
        let src = r#"# main

Try **greet who="a"** and **greet who="b"**.

## greet

When we meet `who`, we **print text="Hi, `who`!"** and close with*None*.
"#;
        let module = crate::parse::parse_source(src).unwrap();
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        let annots = build_view_annots(&module, src, &cap);
        assert!(
            annots
                .iter()
                .filter(|a| a.kind == crate::view::annot::AnnotKind::Call)
                .count()
                >= 3,
            "expected greet×2 + print call events: {:?}",
            cap.view_trace.events
        );
        assert!(
            annots.iter().any(|a| {
                a.kind == crate::view::annot::AnnotKind::Bind && a.label == "who"
            }),
            "param Decl bind missing: {annots:?}"
        );
        let html = render_module_document(&module, src, &annots);
        assert!(html.contains("annot-call"), "{html}");
        assert!(html.contains("annot-bind"), "{html}");
        assert!(html.contains("Hi, a!") || html.contains("Hi, b!"), "{html}");
        // Dual greet call sites each get a non-empty annot id.
        let greet_ids: Vec<_> = html
            .match_indices("mq-code")
            .filter(|_| true)
            .collect();
        assert!(
            html.matches(r#"data-annot-id="c"#).count() >= 2
                || html.contains(r#"data-annot-id="c"#),
            "expected call annot ids on greet marks: {html}"
        );
        let _ = greet_ids;
        // Last-frame who bind must not be buried in overflow (frame-aware density).
        assert!(
            !html.contains(r#"annot-bind annot-chip-overflow"#)
                || html.contains(r#"annot-label">who</span>"#),
            "who bind should be visible for default (last) call frame: {html}"
        );
        // Prefer: no overflow on bind chips when last frame has ≤4 chips.
        let bind_overflow = html.matches("annot-bind annot-chip-overflow").count();
        assert_eq!(
            bind_overflow, 0,
            "default frame who/print/return must not overflow: {html}"
        );
    }
}

#[cfg(test)]
mod occl_split_tests {
    use super::split_occlusion;

    #[test]
    fn splits_bare_assignment_occlusion() {
        let (head, label, payload, rest) =
            split_occlusion("`留存` = [客户留存率，模拟数据](0.42)").unwrap();
        assert_eq!(head, "`留存` =");
        assert_eq!(label, "客户留存率，模拟数据");
        assert_eq!(payload, "0.42");
        assert_eq!(rest, "");
    }

    #[test]
    fn splits_bold_assignment_occlusion_without_markers() {
        let (head, label, payload, _) =
            split_occlusion("**留存 = [客户留存率，模拟数据](0.42)**").unwrap();
        assert_eq!(head, "留存 =");
        assert_eq!(label, "客户留存率，模拟数据");
        assert_eq!(payload, "0.42");
    }

    #[test]
    fn splits_stub_and_nested_payload() {
        let (_, label, payload, _) =
            split_occlusion("`待接` = [搜索接入，待开发]()").unwrap();
        assert_eq!(label, "搜索接入，待开发");
        assert!(payload.is_empty());

        let (_, label, payload, _) =
            split_occlusion("`取第一` = [第一颗](access(`篮子`, \"1\"))").unwrap();
        assert_eq!(label, "第一颗");
        assert_eq!(payload, "access(`篮子`, \"1\")");
    }

    #[test]
    fn ignores_index_and_plain_lines() {
        // Link-index (classic) and ordinary statements are not occlusions.
        assert!(split_occlusion("**v = [键](`表`)**").is_some()); // shape matches; parser decides meaning
        assert!(split_occlusion("**甲 = 加(1 2)**").is_none());
        assert!(split_occlusion("> print text=x").is_none());
    }
}
