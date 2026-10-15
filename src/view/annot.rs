//! View annotations: bind values, returns, writebacks, branch/loop trace → Structure callouts.

use std::collections::HashMap;

use crate::ast::{Function, Module, Stmt};
use crate::capture::{RunCapture, StmtEventKind};
use crate::host::writeback;
use crate::parse::{scan_prose_line, ProseBit};
use crate::value::Value;
use crate::view::html::escape;

/// One result callout anchored to a source span.
#[derive(Debug, Clone)]
pub struct ViewAnnot {
    pub id: String,
    pub kind: AnnotKind,
    pub anchor_line: u32,
    pub anchor_byte_start: Option<usize>,
    pub anchor_byte_end: Option<usize>,
    pub label: String,
    pub value: String,
    pub detail: String,
    pub arm_index: Option<usize>,
    pub call_id: Option<u32>,
    pub loop_id: Option<u32>,
    /// 0-based iteration index when inside a loop frame.
    pub iter: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotKind {
    Bind,
    Return,
    #[allow(dead_code)]
    Call,
    #[allow(dead_code)] // reserved for non-call BoldCode expr chips
    Expr,
    Branch,
    Loop,
    Writeback,
    /// Synthetic: one per CallFrame for function picker (label = fn name, value = args_brief).
    CallFrame,
    /// Synthetic: one per loop iteration for loop picker.
    LoopFrame,
}

impl AnnotKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AnnotKind::Bind => "bind",
            AnnotKind::Return => "return",
            AnnotKind::Call => "call",
            AnnotKind::Expr => "expr",
            AnnotKind::Branch => "branch",
            AnnotKind::Loop => "loop",
            AnnotKind::Writeback => "writeback",
            AnnotKind::CallFrame => "callframe",
            AnnotKind::LoopFrame => "loopframe",
        }
    }
}

/// Truncate display text for callout chips.
pub fn brief_value(v: &Value) -> String {
    let s = v.as_display();
    if s.chars().count() > 48 {
        format!("{}…", s.chars().take(48).collect::<String>())
    } else {
        s
    }
}

fn brief_str(s: &str) -> String {
    if s.chars().count() > 48 {
        format!("{}…", s.chars().take(48).collect::<String>())
    } else {
        s.to_string()
    }
}

/// Build annotations from capture + source (post-stmt events preferred).
pub fn build_view_annots(module: &Module, source: &str, cap: &RunCapture) -> Vec<ViewAnnot> {
    let mut out = Vec::new();
    let mut id_seq = 0u32;
    let mut next_id = |prefix: &str| -> String {
        id_seq += 1;
        format!("{prefix}{id_seq}")
    };

    let name_sites = collect_name_sites(module, source);
    let lines: Vec<&str> = source.lines().collect();

    // Prefer StmtEvent assign/return (post-stmt). Keep one chip per (line,name,call,loop,iter).
    type AssignKey = (u32, String, u32, Option<u32>, Option<u32>);
    let mut last_assign: HashMap<AssignKey, &crate::capture::StmtEvent> = HashMap::new();
    let mut assign_counts: HashMap<AssignKey, u32> = HashMap::new();
    for ev in &cap.view_trace.events {
        if ev.kind == StmtEventKind::Assign {
            if let Some(ref name) = ev.name {
                let key = (ev.line, name.clone(), ev.call_id, ev.loop_id, ev.iter);
                *assign_counts.entry(key.clone()).or_insert(0) += 1;
                last_assign.insert(key, ev);
            }
        }
    }

    if !last_assign.is_empty() {
        for ((line, name, _cid, _lid, _iter), ev) in &last_assign {
            let (start, end) = name_sites
                .get(name.as_str())
                .and_then(|(l, s, e)| if l == line { Some((*s, *e)) } else { None })
                .unwrap_or_else(|| {
                    find_name_in_line(
                        lines
                            .get((*line as usize).saturating_sub(1))
                            .copied()
                            .unwrap_or(""),
                        name,
                    )
                });
            let key = (*line, name.clone(), ev.call_id, ev.loop_id, ev.iter);
            let raw = ev.value_after.clone().unwrap_or_else(|| "?".into());
            let (value, detail) = abbreviate_bind_value(&raw);
            let mut value = value;
            let count = assign_counts.get(&key).copied().unwrap_or(1);
            if count > 1 {
                value = format!("{value} · ×{count}");
            }
            out.push(ViewAnnot {
                id: next_id("b"),
                kind: AnnotKind::Bind,
                anchor_line: *line,
                anchor_byte_start: start,
                anchor_byte_end: end,
                label: name.clone(),
                value: brief_str(&value),
                detail,
                arm_index: None,
                call_id: Some(ev.call_id),
                loop_id: ev.loop_id,
                iter: ev.iter,
            });
        }
    } else {
        // Fallback (bytecode / empty events): final bindings — only when no events.
        for (name, val) in &cap.bindings {
            let (line, start, end) = name_sites
                .get(name.as_str())
                .copied()
                .unwrap_or((0, None, None));
            if line == 0 {
                continue;
            }
            out.push(ViewAnnot {
                id: next_id("b"),
                kind: AnnotKind::Bind,
                anchor_line: line,
                anchor_byte_start: start,
                anchor_byte_end: end,
                label: name.clone(),
                value: brief_value(val),
                detail: val.as_display(),
                arm_index: None,
                call_id: None,
                loop_id: None,
                iter: None,
            });
        }
    }

    // BindRef: foreach item = collection[i] (one per iteration).
    for ev in &cap.view_trace.events {
        if ev.kind != StmtEventKind::BindRef {
            continue;
        }
        let Some(ref name) = ev.name else { continue };
        let value = ev.value_after.clone().unwrap_or_default();
        // Label is `分`; value is `分数表[3] · 88` (not `分 = …` again).
        let display = if let Some((left, right)) = value.split_once('=') {
            format!("{left} · {right}")
        } else {
            value.clone()
        };
        out.push(ViewAnnot {
            id: next_id("brf"),
            kind: AnnotKind::Bind,
            anchor_line: ev.line,
            anchor_byte_start: None,
            anchor_byte_end: None,
            label: name.clone(),
            value: brief_str(&display),
            detail: format!("{name} = {display}"),
            arm_index: None,
            call_id: Some(ev.call_id),
            loop_id: ev.loop_id,
            iter: ev.iter,
        });
    }

    // Return events (last per line+call+iter).
    type RetKey = (u32, u32, Option<u32>, Option<u32>);
    let mut last_ret: HashMap<RetKey, &crate::capture::StmtEvent> = HashMap::new();
    for ev in &cap.view_trace.events {
        if ev.kind == StmtEventKind::Return {
            last_ret.insert((ev.line, ev.call_id, ev.loop_id, ev.iter), ev);
        }
    }
    for ((line, _, _, _), ev) in last_ret {
        let text = lines
            .get(line.saturating_sub(1) as usize)
            .copied()
            .unwrap_or("");
        let mut start = None;
        let mut end = None;
        for bit in scan_prose_line(text) {
            if let ProseBit::ItalicReturn { at, .. } = &bit {
                start = Some(*at);
                end = Some(prose_bit_end(text, &bit));
            }
        }
        let value = ev.value_after.clone().unwrap_or_default();
        out.push(ViewAnnot {
            id: next_id("r"),
            kind: AnnotKind::Return,
            anchor_line: line,
            anchor_byte_start: start,
            anchor_byte_end: end,
            label: "return".into(),
            value: brief_str(&value),
            detail: value,
            arm_index: None,
            call_id: Some(ev.call_id),
            loop_id: ev.loop_id,
            iter: ev.iter,
        });
    }

    // Writebacks.
    let wb = writeback::writeback_map(source);
    for (line, body) in wb {
        let brief = body.lines().next().unwrap_or(body.as_str());
        out.push(ViewAnnot {
            id: next_id("w"),
            kind: AnnotKind::Writeback,
            anchor_line: line,
            anchor_byte_start: None,
            anchor_byte_end: None,
            label: "output".into(),
            value: brief_str(brief),
            detail: body,
            arm_index: None,
            call_id: None,
            loop_id: None,
            iter: None,
        });
    }

    // Branch hits — one annot per (line, arm, call) with call_id for frame filtering.
    for hit in &cap.view_trace.branches {
        out.push(ViewAnnot {
            id: next_id("br"),
            kind: AnnotKind::Branch,
            anchor_line: hit.line,
            anchor_byte_start: None,
            anchor_byte_end: None,
            label: format!("arm {}", hit.arm_index + 1),
            value: if hit.count > 1 {
                format!("taken ×{}", hit.count)
            } else {
                "taken".into()
            },
            detail: format!(
                "arm {} hit {} time(s) call {}",
                hit.arm_index + 1,
                hit.count,
                hit.call_id
            ),
            arm_index: Some(hit.arm_index),
            call_id: Some(hit.call_id),
            loop_id: None,
            iter: None,
        });
    }

    // Per-iteration frames for the picker. Foreach item binding is already a BindRef chip
    // (`分` → `分数表[i] · v`); do not emit a second Loop summary on the same line.
    for fr in &cap.view_trace.loop_frames {
        let item = fr.item_name.as_deref().unwrap_or("item");
        let coll = fr.collection.as_deref().unwrap_or("?");
        let has_item_bind = fr.item_name.is_some() && !fr.iterations.is_empty();
        if !has_item_bind {
            // while / empty: keep a single summary chip (no BindRef duplicate).
            let last = fr.iterations.last();
            let summary = if let Some(it) = last {
                format!("{} iters · last={}", fr.iterations.len(), it.item_value)
            } else {
                format!("{} iters", fr.iterations.len())
            };
            out.push(ViewAnnot {
                id: next_id("lp"),
                kind: AnnotKind::Loop,
                anchor_line: fr.line,
                anchor_byte_start: None,
                anchor_byte_end: None,
                label: "loop".into(),
                value: brief_str(&summary),
                detail: summary,
                arm_index: None,
                call_id: last.map(|it| it.call_id),
                loop_id: Some(fr.loop_id),
                iter: None,
            });
        }
        for it in &fr.iterations {
            let value = format!("{} = {}[{}] · {}", item, coll, it.index, it.item_value);
            out.push(ViewAnnot {
                id: format!("lf{}-{}", fr.loop_id, it.index),
                kind: AnnotKind::LoopFrame,
                anchor_line: fr.line,
                anchor_byte_start: None,
                anchor_byte_end: None,
                label: format!("{}", it.index + 1),
                value: brief_str(&value),
                detail: value,
                arm_index: None,
                call_id: Some(it.call_id),
                loop_id: Some(fr.loop_id),
                iter: Some(it.index),
            });
        }
    }
    // Fallback LoopHit when no LoopFrame (legacy / while without frames).
    if cap.view_trace.loop_frames.is_empty() {
        for hit in &cap.view_trace.loops {
            let mut value = format!("{} iters", hit.iterations);
            if let Some(ref last) = hit.last_item {
                value.push_str(" · last=");
                value.push_str(last);
            }
            out.push(ViewAnnot {
                id: next_id("lp"),
                kind: AnnotKind::Loop,
                anchor_line: hit.line,
                anchor_byte_start: None,
                anchor_byte_end: None,
                label: "loop".into(),
                value,
                detail: format!("iterations={}", hit.iterations),
                arm_index: None,
                call_id: None,
                loop_id: None,
                iter: None,
            });
        }
    }

    // Call / expr events → chips; zip with BoldCode spans when several share a line.
    let mut calls_by_line: HashMap<u32, Vec<&crate::capture::StmtEvent>> = HashMap::new();
    for ev in &cap.view_trace.events {
        if ev.kind == StmtEventKind::Call {
            calls_by_line.entry(ev.line).or_default().push(ev);
        }
    }
    for (line, evs) in calls_by_line {
        let text = lines
            .get(line.saturating_sub(1) as usize)
            .copied()
            .unwrap_or("");
        let bold_spans: Vec<(usize, usize)> = scan_prose_line(text)
            .into_iter()
            .filter_map(|bit| match bit {
                ProseBit::BoldCode { .. } => {
                    let start = bit_at(&bit);
                    Some((start, prose_bit_end(text, &bit)))
                }
                _ => None,
            })
            .collect();
        for (i, ev) in evs.iter().enumerate() {
            // One BoldCode on the line (e.g. print across call frames): share it.
            // Several BoldCodes (e.g. two greets in main): zip by index.
            let (start, end) = if bold_spans.is_empty() {
                (None, None)
            } else if bold_spans.len() == 1 {
                let (s, e) = bold_spans[0];
                (Some(s), Some(e))
            } else {
                bold_spans
                    .get(i)
                    .copied()
                    .map(|(s, e)| (Some(s), Some(e)))
                    .unwrap_or((None, None))
            };
            let value = ev.value_after.clone().unwrap_or_default();
            let label = ev.name.clone().unwrap_or_else(|| "call".into());
            out.push(ViewAnnot {
                id: next_id("c"),
                kind: AnnotKind::Call,
                anchor_line: line,
                anchor_byte_start: start,
                anchor_byte_end: end,
                label,
                value: brief_str(&value),
                detail: value,
                arm_index: None,
                call_id: Some(ev.call_id),
                loop_id: ev.loop_id,
                iter: ev.iter,
            });
        }
    }

    // Call frames for picker (label = fn name).
    for fr in &cap.view_trace.calls {
        out.push(ViewAnnot {
            id: format!("cf{}", fr.call_id),
            kind: AnnotKind::CallFrame,
            anchor_line: fr.fn_line,
            anchor_byte_start: None,
            anchor_byte_end: None,
            label: fr.fn_name.clone(),
            value: fr.args_brief.clone(),
            detail: format!("call #{} {}", fr.ordinal, fr.args_brief),
            arm_index: None,
            call_id: Some(fr.call_id),
            loop_id: None,
            iter: None,
        });
    }

    // Param Decl sites in function body: bind chips from CallFrame args.
    for fr in &cap.view_trace.calls {
        let Some(fun) = find_fun_by_line(module, fr.fn_line) else {
            continue;
        };
        let body_lines = fun_stmt_lines(fun);
        for (pname, pval) in parse_args_brief_pairs(&fr.args_brief) {
            for (i, text) in lines.iter().enumerate() {
                let line_no = (i + 1) as u32;
                if !body_lines.contains(&line_no) {
                    continue;
                }
                for bit in scan_prose_line(text) {
                    if let ProseBit::Decl { name, at, .. } = &bit {
                        if name != &pname {
                            continue;
                        }
                        let end = prose_bit_end(text, &bit);
                        out.push(ViewAnnot {
                            id: next_id("p"),
                            kind: AnnotKind::Bind,
                            anchor_line: line_no,
                            anchor_byte_start: Some(*at),
                            anchor_byte_end: Some(end),
                            label: pname.clone(),
                            value: brief_str(&pval),
                            detail: format!("{pname}={pval}"),
                            arm_index: None,
                            call_id: Some(fr.call_id),
                            loop_id: None,
                            iter: None,
                        });
                    }
                }
            }
        }
    }

    // Same prose span across call frames shares one id so <mark data-annot-id>
    // matches the visible chip after frame filter (else L-leaders find no anchor).
    unify_annot_ids_by_span(&mut out);

    out
}

fn unify_annot_ids_by_span(annots: &mut [ViewAnnot]) {
    let mut first: HashMap<(u32, Option<usize>, Option<usize>, u8), String> = HashMap::new();
    for a in annots.iter_mut() {
        if matches!(a.kind, AnnotKind::CallFrame | AnnotKind::LoopFrame) {
            continue;
        }
        if a.anchor_byte_start.is_none() {
            continue;
        }
        let tag = match a.kind {
            AnnotKind::Bind => 1u8,
            AnnotKind::Return => 2,
            AnnotKind::Call | AnnotKind::Expr => 3,
            AnnotKind::Branch => 4,
            AnnotKind::Loop => 5,
            AnnotKind::Writeback => 6,
            AnnotKind::CallFrame | AnnotKind::LoopFrame => 0,
        };
        let key = (
            a.anchor_line,
            a.anchor_byte_start,
            a.anchor_byte_end,
            tag,
        );
        if let Some(id) = first.get(&key) {
            a.id = id.clone();
        } else {
            first.insert(key, a.id.clone());
        }
    }
}

fn abbreviate_bind_value(raw: &str) -> (String, String) {
    if raw.contains("<svg") {
        return ("svg".into(), "svg".into());
    }
    if raw.contains("```") {
        return ("code".into(), "code".into());
    }
    if raw.chars().count() > 96 {
        let short: String = raw.chars().take(48).collect();
        return (format!("{short}…"), format!("{short}…"));
    }
    (raw.to_string(), raw.to_string())
}

fn parse_args_brief_pairs(brief: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for part in brief.split(", ") {
        if let Some((k, v)) = part.split_once('=') {
            if !k.is_empty() {
                out.push((k.to_string(), v.to_string()));
            }
        }
    }
    out
}

fn find_fun_by_line(module: &Module, line: u32) -> Option<&Function> {
    fn walk<'a>(fun: &'a Function, line: u32) -> Option<&'a Function> {
        if fun.span.line == line {
            return Some(fun);
        }
        for c in &fun.children {
            if let Some(f) = walk(c, line) {
                return Some(f);
            }
        }
        None
    }
    for f in &module.functions {
        if let Some(found) = walk(f, line) {
            return Some(found);
        }
    }
    None
}

fn fun_stmt_lines(fun: &Function) -> std::collections::HashSet<u32> {
    let mut s = std::collections::HashSet::new();
    fn walk_stmt(st: &Stmt, s: &mut std::collections::HashSet<u32>) {
        let line = match st {
            Stmt::Assign { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::Call { span, .. }
            | Stmt::Expr { span, .. }
            | Stmt::Branch { span, .. }
            | Stmt::While { span, .. }
            | Stmt::ForEach { span, .. } => span.line,
        };
        s.insert(line);
        match st {
            Stmt::Branch { arms, .. } => {
                for arm in arms {
                    for b in &arm.body {
                        walk_stmt(b, s);
                    }
                }
            }
            Stmt::While { body, .. } | Stmt::ForEach { body, .. } => {
                for b in body {
                    walk_stmt(b, s);
                }
            }
            _ => {}
        }
    }
    for st in &fun.body {
        walk_stmt(st, &mut s);
    }
    for c in &fun.children {
        s.extend(fun_stmt_lines(c));
    }
    s
}

/// Serialize ViewExecutionReport JSON for AI / page embed.
pub fn view_execution_report_json(path: &str, ok: bool, cap: &RunCapture) -> String {
    use serde_json::{json, Map};
    let mut bindings = Map::new();
    for (k, v) in &cap.bindings {
        bindings.insert(k.clone(), json!(v.as_display()));
    }
    let calls: Vec<_> = cap
        .view_trace
        .calls
        .iter()
        .map(|c| {
            json!({
                "call_id": c.call_id,
                "fn": c.fn_name,
                "fn_line": c.fn_line,
                "ordinal": c.ordinal,
                "args_brief": c.args_brief,
            })
        })
        .collect();
    let events: Vec<_> = cap
        .view_trace
        .events
        .iter()
        .map(|e| {
            json!({
                "line": e.line,
                "kind": e.kind.as_str(),
                "name": e.name,
                "value_after": e.value_after,
                "arm_index": e.arm_index,
                "iter": e.iter,
                "call_id": e.call_id,
                "loop_id": e.loop_id,
            })
        })
        .collect();
    let loops: Vec<_> = cap
        .view_trace
        .loop_frames
        .iter()
        .map(|fr| {
            json!({
                "loop_id": fr.loop_id,
                "line": fr.line,
                "item": fr.item_name,
                "collection": fr.collection,
                "iters": fr.iterations.iter().map(|it| json!({
                    "index": it.index,
                    "value": it.item_value,
                    "call_id": it.call_id,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "version": 1,
        "path": path,
        "ok": ok,
        "calls": calls,
        "loops": loops,
        "events": events,
        "bindings_final": bindings,
    })
    .to_string()
}

fn collect_name_sites(
    module: &Module,
    source: &str,
) -> HashMap<String, (u32, Option<usize>, Option<usize>)> {
    let mut map = HashMap::new();
    let lines: Vec<&str> = source.lines().collect();
    for fun in &module.functions {
        collect_name_sites_fun(fun, &lines, &mut map);
    }
    for (i, text) in lines.iter().enumerate() {
        let line_no = (i + 1) as u32;
        for bit in scan_prose_line(text) {
            if let ProseBit::Decl { name, at, .. } = &bit {
                let end = prose_bit_end(text, &bit);
                map.entry(name.clone())
                    .or_insert((line_no, Some(*at), Some(end)));
            }
        }
    }
    map
}

fn collect_name_sites_fun(
    fun: &Function,
    lines: &[&str],
    map: &mut HashMap<String, (u32, Option<usize>, Option<usize>)>,
) {
    for stmt in &fun.body {
        if let Stmt::Assign { name, span, .. } = stmt {
            let text = lines
                .get(span.line.saturating_sub(1) as usize)
                .copied()
                .unwrap_or("");
            let (start, end) = find_name_in_line(text, name);
            // Prefer first site for fallback map; post-stmt uses line from event.
            map.entry(name.clone())
                .or_insert((span.line, start, end));
        }
        walk_stmt_nested(stmt, lines, map);
    }
    for child in &fun.children {
        collect_name_sites_fun(child, lines, map);
    }
}

fn walk_stmt_nested(
    stmt: &Stmt,
    lines: &[&str],
    map: &mut HashMap<String, (u32, Option<usize>, Option<usize>)>,
) {
    match stmt {
        Stmt::Branch { arms, .. } => {
            for arm in arms {
                for s in &arm.body {
                    if let Stmt::Assign { name, span, .. } = s {
                        let text = lines
                            .get(span.line.saturating_sub(1) as usize)
                            .copied()
                            .unwrap_or("");
                        let (start, end) = find_name_in_line(text, name);
                        map.entry(name.clone())
                            .or_insert((span.line, start, end));
                    }
                    walk_stmt_nested(s, lines, map);
                }
            }
        }
        Stmt::While { body, .. } | Stmt::ForEach { body, .. } => {
            for s in body {
                if let Stmt::Assign { name, span, .. } = s {
                    let text = lines
                        .get(span.line.saturating_sub(1) as usize)
                        .copied()
                        .unwrap_or("");
                    let (start, end) = find_name_in_line(text, name);
                    map.entry(name.clone())
                        .or_insert((span.line, start, end));
                }
                walk_stmt_nested(s, lines, map);
            }
        }
        _ => {}
    }
}

fn find_name_in_line(text: &str, name: &str) -> (Option<usize>, Option<usize>) {
    for bit in scan_prose_line(text) {
        match &bit {
            ProseBit::Decl { name: n, at, .. } if n == name => {
                return (Some(*at), Some(prose_bit_end(text, &bit)));
            }
            ProseBit::BoldCode { inner, at } if inner.contains(name) => {
                return (Some(*at), Some(prose_bit_end(text, &bit)));
            }
            _ => {}
        }
    }
    if let Some(at) = text.find(name) {
        return (Some(at), Some(at + name.len()));
    }
    (None, None)
}

/// Byte end (exclusive) of a prose bit in `line`.
pub fn prose_bit_end(line: &str, bit: &ProseBit) -> usize {
    match bit {
        ProseBit::Decl { name, default, at } => {
            let after_ticks = *at + 1 + name.len() + 1;
            match default {
                None => after_ticks,
                Some(d) => {
                    let mut end = after_ticks;
                    if line.get(end..end + 1) == Some("=") {
                        end += 1 + d.len();
                    }
                    end.min(line.len())
                }
            }
        }
        ProseBit::BoldCode { inner, at } => (*at + 2 + inner.len() + 2).min(line.len()),
        ProseBit::ItalicReturn { inner, at } => (*at + 1 + inner.len() + 1).min(line.len()),
    }
}

pub fn bit_at(bit: &ProseBit) -> usize {
    match bit {
        ProseBit::Decl { at, .. }
        | ProseBit::BoldCode { at, .. }
        | ProseBit::ItalicReturn { at, .. } => *at,
    }
}

pub fn bit_kind_class(bit: &ProseBit) -> &'static str {
    match bit {
        ProseBit::Decl { .. } => "decl",
        ProseBit::BoldCode { .. } => "code",
        ProseBit::ItalicReturn { .. } => "return",
    }
}

/// Render annotation band HTML for a set of annots (already filtered to one block).
///
/// SSR defaults to the **last** call_id and **last** iteration per loop_id (matches
/// pickers). Overflow (`max_visible`) is counted only among chips visible in that
/// default frame — so `who` on the last greet call is not buried behind +N.
/// Client `applyCallFrame` recomputes overflow after switching.
pub fn render_annot_band(annots: &[ViewAnnot], max_visible: usize) -> String {
    let annots: Vec<&ViewAnnot> = annots
        .iter()
        .filter(|a| a.kind != AnnotKind::CallFrame && a.kind != AnnotKind::LoopFrame)
        .collect();
    if annots.is_empty() {
        return String::new();
    }
    // Default visible iteration = max index per loop_id (matches picker default = last).
    let mut default_iter: HashMap<Option<u32>, u32> = HashMap::new();
    for a in &annots {
        if let Some(i) = a.iter {
            let e = default_iter.entry(a.loop_id).or_insert(i);
            if i > *e {
                *e = i;
            }
        }
    }
    // Default call frame = max call_id among chips that carry one (last invocation).
    let default_call_id: Option<u32> = annots.iter().filter_map(|a| a.call_id).max();
    // Left-to-right by anchor so chips sit under their underlines (avoids
    // crossed L-leaders that look like “missing” lines when crowded).
    let mut ordered: Vec<&ViewAnnot> = annots;
    ordered.sort_by(|a, b| {
        let sa = a.anchor_byte_start.unwrap_or(usize::MAX);
        let sb = b.anchor_byte_start.unwrap_or(usize::MAX);
        sa.cmp(&sb).then_with(|| a.id.cmp(&b.id))
    });

    let mut s = String::from(r#"<div class="annot-band" data-annot-band>"#);
    // Leaders live on the parent .doc-block (see initAnnotLeaders); keep a hook
    // for keyed mode / a11y but geometry is computed against the block.
    s.push_str(r#"<div class="annot-leaders" aria-hidden="true"></div>"#);
    s.push_str(r#"<div class="annot-chips">"#);
    let mut shown = 0usize;
    let mut overflow_count = 0usize;
    for a in &ordered {
        let iter_miss = a
            .iter
            .zip(default_iter.get(&a.loop_id))
            .map(|(i, d)| i != *d)
            .unwrap_or(false);
        let call_miss = match (a.call_id, default_call_id) {
            (Some(cid), Some(def)) if cid != def => true,
            _ => false,
        };
        let frame_miss = iter_miss || call_miss;
        let overflow = if frame_miss {
            false
        } else if shown >= max_visible {
            overflow_count += 1;
            true
        } else {
            shown += 1;
            false
        };
        let mut classes = format!("annot-chip annot-{}", a.kind.as_str());
        if overflow {
            classes.push_str(" annot-chip-overflow");
        }
        if frame_miss {
            classes.push_str(" is-frame-hidden");
        }
        let call_attr = a
            .call_id
            .map(|c| format!(r#" data-call-id="{c}""#))
            .unwrap_or_default();
        let loop_attr = a
            .loop_id
            .map(|c| format!(r#" data-loop-id="{c}""#))
            .unwrap_or_default();
        let iter_attr = a
            .iter
            .map(|c| format!(r#" data-iter="{c}""#))
            .unwrap_or_default();
        let arm_attr = a
            .arm_index
            .map(|c| format!(r#" data-arm-index="{c}""#))
            .unwrap_or_default();
        let hidden_attr = if frame_miss { " hidden" } else { "" };
        s.push_str(&format!(
            r#"<button type="button" class="{classes}" data-annot-id="{id}" data-annot-kind="{kind}"{call}{loop_id}{iter}{arm}{hidden} title="{detail}">
<span class="annot-label">{label}</span>
<span class="annot-value">{value}</span>
</button>"#,
            classes = classes,
            kind = a.kind.as_str(),
            id = escape(&a.id),
            call = call_attr,
            loop_id = loop_attr,
            iter = iter_attr,
            arm = arm_attr,
            hidden = hidden_attr,
            detail = escape(&a.detail),
            label = escape(&a.label),
            value = escape(&a.value),
        ));
    }
    if overflow_count > 0 {
        s.push_str(&format!(
            r#"<button type="button" class="annot-more" data-annot-more>+{}</button>"#,
            overflow_count
        ));
    }
    s.push_str("</div></div>");
    s
}

/// Group annots by anchor line.
pub fn annots_by_line(annots: &[ViewAnnot]) -> HashMap<u32, Vec<&ViewAnnot>> {
    let mut m: HashMap<u32, Vec<&ViewAnnot>> = HashMap::new();
    for a in annots {
        if a.kind == AnnotKind::CallFrame || a.kind == AnnotKind::LoopFrame {
            continue;
        }
        m.entry(a.anchor_line).or_default().push(a);
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::{RunCapture, ViewTrace};

    #[test]
    fn prose_bit_end_bold_and_return() {
        let line = "操作**n=n+1**接着返回*n*。";
        let bits = scan_prose_line(line);
        assert!(bits.len() >= 2, "{bits:?}");
        for bit in &bits {
            let end = prose_bit_end(line, bit);
            assert!(end <= line.len());
            assert!(end > bit_at(bit));
        }
    }

    #[test]
    fn post_stmt_init_not_final() {
        let src = include_str!("../../tests/view/document-annot-demo.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        let annots = build_view_annots(&module, src, &cap);
        // Find bind for 总分 on the init line (**总分 = 0**).
        let init = annots.iter().find(|a| {
            a.kind == AnnotKind::Bind && a.label == "总分" && a.value.starts_with('0')
        });
        assert!(
            init.is_some(),
            "expected 总分=0 post-stmt annot, got: {:?}",
            annots
                .iter()
                .filter(|a| a.label == "总分")
                .map(|a| (&a.value, a.anchor_line))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn build_annots_branch_and_loop() {
        let src = include_str!("../../tests/view/document-annot-demo.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        let annots = build_view_annots(&module, src, &cap);
        assert!(annots.iter().any(|a| a.kind == AnnotKind::Branch));
        assert!(!cap.view_trace.events.is_empty(), "expected stmt events");
        // Branch annots must carry call_id so call picker can filter.
        assert!(
            annots
                .iter()
                .filter(|a| a.kind == AnnotKind::Branch)
                .all(|a| a.call_id.is_some()),
            "branch annots missing call_id"
        );
        assert!(!cap.view_trace.loop_frames.is_empty(), "expected loop frames");
        assert!(annots.iter().any(|a| a.kind == AnnotKind::LoopFrame));
        // Foreach: BindRef chips only — no duplicate Loop summary on the same line.
        let loop_line = cap.view_trace.loop_frames[0].line;
        let on_loop: Vec<_> = annots
            .iter()
            .filter(|a| a.anchor_line == loop_line)
            .collect();
        assert!(
            on_loop.iter().any(|a| a.kind == AnnotKind::Bind && a.label == "分"),
            "expected BindRef for 分"
        );
        assert!(
            !on_loop.iter().any(|a| a.kind == AnnotKind::Loop),
            "foreach must not emit duplicate Loop summary chip: {:?}",
            on_loop.iter().map(|a| a.kind.as_str()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn call_frames_filter_branch_arms() {
        let src = include_str!("../../tests/view/document-annot-demo.mq.md");
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        let grade_calls: Vec<_> = cap
            .view_trace
            .calls
            .iter()
            .filter(|c| c.fn_name == "等级说明")
            .collect();
        assert!(grade_calls.len() >= 2, "expected ≥2 calls to 等级说明");
        let c1 = grade_calls[0].call_id;
        let c2 = grade_calls[1].call_id;
        let arm_for = |cid: u32| {
            cap.view_trace
                .events
                .iter()
                .find(|e| e.kind == StmtEventKind::Branch && e.call_id == cid)
                .and_then(|e| e.arm_index)
        };
        let a1 = arm_for(c1).expect("branch for call 1");
        let a2 = arm_for(c2).expect("branch for call 2");
        assert_ne!(a1, a2, "two calls should take different arms");
    }

    #[test]
    fn loop_frame_item_binding() {
        let src = include_str!("../../tests/view/document-annot-demo.mq.md");
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        let fr = cap.view_trace.loop_frames.first().expect("loop frame");
        assert_eq!(fr.item_name.as_deref(), Some("分"));
        assert_eq!(fr.collection.as_deref(), Some("分数表"));
        assert_eq!(fr.iterations.len(), 4);
        assert_eq!(fr.iterations.last().unwrap().item_value, "88");
        assert_eq!(fr.iterations.last().unwrap().index, 3);
    }

    #[test]
    fn loop_band_ssr_hides_non_default_iters() {
        let src = include_str!("../../tests/view/document-annot-demo.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        let annots = build_view_annots(&module, src, &cap);
        // Collect bind annots on the 总分 assign line inside the loop.
        let total_line = annots
            .iter()
            .find(|a| a.kind == AnnotKind::Bind && a.label == "总分" && a.iter == Some(3))
            .map(|a| a.anchor_line)
            .expect("last-iter 总分");
        let on_line: Vec<_> = annots
            .iter()
            .filter(|a| a.anchor_line == total_line && a.kind == AnnotKind::Bind)
            .cloned()
            .collect();
        assert!(on_line.len() >= 4, "expected per-iter 总分 chips");
        let html = render_annot_band(&on_line, 8);
        // Non-last iters are SSR-hidden; last remains visible.
        assert!(html.contains(r#"data-iter="3""#), "{html}");
        assert!(
            html.contains(r#"data-iter="0""#) && html.contains("is-frame-hidden"),
            "expected non-default iters hidden: {html}"
        );
        let visible_totals = html.matches(r#"data-iter="3""#).count();
        assert_eq!(visible_totals, 1);
        // Hidden attribute on iter 0 chip.
        assert!(
            html.contains(r#"data-iter="0""#)
                && html
                    .split("<button")
                    .filter(|b| b.contains(r#"data-iter="0""#))
                    .all(|b| b.contains(" hidden") || b.contains("is-frame-hidden")),
            "{html}"
        );
    }


    /// `# main` is the entry object; `##` children are methods. A `>` call under
    /// `## 安装` belongs to that method — it does **not** run until something
    /// calls `安装`. This is object semantics, not a parser failure.
    #[test]
    fn call_under_child_heading_does_not_auto_run() {
        let src = r#"---
title: t
---

# main

## 安装

> 问候 谁=a

## 问候
    + `谁`

> print text=hi `谁`
"#;
        let module = crate::parse::parse_source(src).unwrap();
        let main = &module.functions[0];
        assert_eq!(main.name, "main");
        let install = main.children.iter().find(|c| c.name == "安装").expect("安装");
        let install_calls: Vec<_> = install
            .body
            .iter()
            .filter_map(|s| match s {
                crate::ast::Stmt::Call { call, .. } => Some(call.callee.as_str()),
                _ => None,
            })
            .collect();
        let main_calls: Vec<_> = main
            .body
            .iter()
            .filter_map(|s| match s {
                crate::ast::Stmt::Call { call, .. } => Some(call.callee.as_str()),
                _ => None,
            })
            .collect();
        assert!(main_calls.is_empty());
        assert_eq!(install_calls, ["问候"]);
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        assert!(
            cap.stdout.trim().is_empty(),
            "entry does not auto-run ## 安装; stdout={:?}",
            cap.stdout
        );
        assert!(
            !cap.view_trace.calls.iter().any(|c| c.fn_name == "问候"),
            "no CallFrame for 问候 when only nested under uncalled ## 安装"
        );
    }

    #[test]
    fn call_in_main_before_child_def_runs_like_nested_call() {
        let src = include_str!("../../tests/structure/nested-call.mq.md");
        let cap = crate::run_source(src, &crate::RunOptions::default()).unwrap();
        assert!(cap.stdout.contains("Hello"), "{:?}", cap.stdout);
        assert!(
            cap.view_trace.calls.iter().any(|c| c.fn_name == "问候"),
            "expected CallFrame for 问候: {:?}",
            cap.view_trace.calls
        );
    }

    #[test]
    fn report_json_has_version() {
        let cap = RunCapture {
            stdout: String::new(),
            value: Value::None,
            plots: Vec::new(),
            bindings: HashMap::new(),
            view_trace: ViewTrace::default(),
        };
        let j = view_execution_report_json("x.mq.md", true, &cap);
        assert!(j.contains("\"version\":1"), "{j}");
        assert!(j.contains("events"), "{j}");
        assert!(j.contains("loops"), "{j}");
    }
}
