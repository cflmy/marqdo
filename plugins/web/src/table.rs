//! GFM table helpers + site path parsing (cells stay string literals).

use serde_json::{json, Map, Value};

const FIELD_KEYS: &[&str] = &["字段", "name", "列", "column"];
const TYPE_KEYS: &[&str] = &["类型", "type"];
const NULL_KEYS: &[&str] = &["可空", "null", "nullable"];
const FRONT_KEYS: &[&str] = &["属性", "前端变量", "front", "field"];
const BACK_KEYS: &[&str] = &["值", "后端数据库", "back", "db"];
const CSS_KEYS: &[&str] = &["样式", "绑定css样式", "css", "class", "style"];
const SRC_KEYS: &[&str] = &["组件", "导入的页面", "src", "page"];
const STYLE_KEYS: &[&str] = &["样式", "style", "class"];
const PROP_KEYS: &[&str] = &["属性", "property", "prop", "名", "name"];
const VAL_KEYS: &[&str] = &["值", "value"];
const SEL_KEYS: &[&str] = &["选择器", "selector", "sel"];
const MEDIA_KEYS: &[&str] = &["媒体", "media", "mq", "@media"];

pub fn normalize_ref(s: &str) -> String {
    let s = s.trim();
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find('`') {
        out.push_str(&rest[..start]);
        rest = &rest[start + 1..];
        if let Some(end) = rest.find('`') {
            out.push_str(&rest[..end]);
            rest = &rest[end + 1..];
        } else {
            out.push('`');
            out.push_str(rest);
            return out;
        }
    }
    out.push_str(rest);
    out
}

fn pick<'a>(m: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|k| m.get(*k))
}

fn cell_str(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn as_str_list(v: &Value) -> Vec<String> {
    match v {
        Value::Array(a) => a.iter().map(cell_str).collect(),
        Value::Null => Vec::new(),
        other => vec![cell_str(other)],
    }
}

fn boolish(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::String(s) => matches!(s.trim(), "1" | "true" | "True" | "yes" | "是" | "可"),
        Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
        _ => false,
    }
}

const UNIQUE_KEYS: &[&str] = &["唯一", "unique", "uniq"];
const INDEX_KEYS: &[&str] = &["索引", "index", "idx"];
const FK_KEYS: &[&str] = &["外键", "fk", "references", "ref", "引用"];

/// Schema table → `[{name,type,nullable,unique,index,fk}, …]`
pub fn as_fields(table: &Value) -> Value {
    match table {
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                if let Some(m) = row.as_object() {
                    let name = pick(m, FIELD_KEYS).map(cell_str).unwrap_or_default();
                    if name.is_empty() {
                        continue;
                    }
                    let ty = pick(m, TYPE_KEYS)
                        .map(cell_str)
                        .unwrap_or_else(|| "text".into());
                    let nullable = pick(m, NULL_KEYS).map(boolish).unwrap_or(true);
                    let unique = pick(m, UNIQUE_KEYS).map(boolish).unwrap_or(false);
                    let index = pick(m, INDEX_KEYS).map(boolish).unwrap_or(false);
                    let fk = pick(m, FK_KEYS)
                        .map(cell_str)
                        .filter(|s| !s.is_empty())
                        .unwrap_or_default();
                    out.push(json!({
                        "name": name,
                        "type": ty,
                        "nullable": nullable,
                        "unique": unique,
                        "index": index,
                        "fk": fk,
                    }));
                }
            }
            Value::Array(out)
        }
        Value::Object(m) => {
            let names = pick(m, FIELD_KEYS).map(as_str_list).unwrap_or_default();
            let types = pick(m, TYPE_KEYS).map(as_str_list).unwrap_or_default();
            let nulls = pick(m, NULL_KEYS).map(as_str_list).unwrap_or_default();
            let uniques = pick(m, UNIQUE_KEYS).map(as_str_list).unwrap_or_default();
            let indexes = pick(m, INDEX_KEYS).map(as_str_list).unwrap_or_default();
            let fks = pick(m, FK_KEYS).map(as_str_list).unwrap_or_default();
            let mut out = Vec::new();
            for (i, name) in names.iter().enumerate() {
                if name.is_empty() {
                    continue;
                }
                let ty = types.get(i).cloned().unwrap_or_else(|| "text".into());
                let nullable = nulls
                    .get(i)
                    .map(|s| boolish(&Value::String(s.clone())))
                    .unwrap_or(true);
                let unique = uniques
                    .get(i)
                    .map(|s| boolish(&Value::String(s.clone())))
                    .unwrap_or(false);
                let index = indexes
                    .get(i)
                    .map(|s| boolish(&Value::String(s.clone())))
                    .unwrap_or(false);
                let fk = fks.get(i).cloned().unwrap_or_default();
                out.push(json!({
                    "name": name,
                    "type": ty,
                    "nullable": nullable,
                    "unique": unique,
                    "index": index,
                    "fk": fk,
                }));
            }
            Value::Array(out)
        }
        _ => json!([]),
    }
}

/// Seed / row table → list of maps.
pub fn as_rows(table: &Value) -> Value {
    match table {
        Value::Array(a) => Value::Array(a.clone()),
        Value::Object(m) => {
            // @ / 行 / row column → list of maps
            let keys: Vec<String> = m.keys().cloned().collect();
            let row_key = ["@", "行", "row"]
                .iter()
                .find(|k| m.contains_key(**k))
                .map(|s| s.to_string());
            if let Some(rk) = row_key {
                let n = as_str_list(m.get(&rk).unwrap()).len();
                let mut out = Vec::new();
                for i in 0..n {
                    let mut row = Map::new();
                    for k in &keys {
                        if k == &rk {
                            continue;
                        }
                        let list = as_str_list(m.get(k).unwrap_or(&Value::Null));
                        row.insert(k.clone(), json!(list.get(i).cloned().unwrap_or_default()));
                    }
                    if let Some(id) = as_str_list(m.get(&rk).unwrap()).get(i) {
                        if !id.is_empty() {
                            row.insert("id".into(), json!(id));
                        }
                    }
                    out.push(Value::Object(row));
                }
                return Value::Array(out);
            }
            // Columnar GFM table: each value is a same-length array → zip into rows.
            let mut col_lens: Vec<usize> = Vec::new();
            let mut all_arrays = !m.is_empty();
            for v in m.values() {
                match v {
                    Value::Array(a) => col_lens.push(a.len()),
                    _ => {
                        all_arrays = false;
                        break;
                    }
                }
            }
            if all_arrays {
                let n = col_lens.iter().copied().max().unwrap_or(0);
                if n > 0 && col_lens.iter().all(|&l| l == n || l == 0) {
                    let mut out = Vec::new();
                    for i in 0..n {
                        let mut row = Map::new();
                        for (k, v) in m {
                            if let Value::Array(a) = v {
                                row.insert(
                                    k.clone(),
                                    a.get(i).cloned().unwrap_or(Value::Null),
                                );
                            }
                        }
                        out.push(Value::Object(row));
                    }
                    return Value::Array(out);
                }
            }
            // single map
            Value::Array(vec![Value::Object(m.clone())])
        }
        _ => json!([]),
    }
}

/// Bind table → `[{front,back,css}, …]`
pub fn as_bind(table: &Value) -> Value {
    match table {
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                if let Some(m) = row.as_object() {
                    let front = normalize_ref(&pick(m, FRONT_KEYS).map(cell_str).unwrap_or_default());
                    let back = normalize_ref(&pick(m, BACK_KEYS).map(cell_str).unwrap_or_default());
                    let css = normalize_ref(&pick(m, CSS_KEYS).map(cell_str).unwrap_or_default());
                    if !front.is_empty() || !back.is_empty() {
                        out.push(json!({ "front": front, "back": back, "css": css }));
                    }
                }
            }
            Value::Array(out)
        }
        Value::Object(m) => {
            let fronts = pick(m, FRONT_KEYS).map(as_str_list).unwrap_or_default();
            let backs = pick(m, BACK_KEYS).map(as_str_list).unwrap_or_default();
            let csses = pick(m, CSS_KEYS).map(as_str_list).unwrap_or_default();
            let n = fronts.len().max(backs.len());
            let mut out = Vec::new();
            for i in 0..n {
                let front = normalize_ref(&fronts.get(i).cloned().unwrap_or_default());
                let back = normalize_ref(&backs.get(i).cloned().unwrap_or_default());
                let css = normalize_ref(&csses.get(i).cloned().unwrap_or_default());
                if !front.is_empty() || !back.is_empty() {
                    out.push(json!({ "front": front, "back": back, "css": css }));
                }
            }
            Value::Array(out)
        }
        _ => json!([]),
    }
}

/// Page table → `[{src,style,slot}, …]`
pub fn as_compose(table: &Value) -> Value {
    match table {
        Value::Object(m) => {
            let srcs = pick(m, SRC_KEYS).map(as_str_list).unwrap_or_default();
            let styles = pick(m, STYLE_KEYS).map(as_str_list).unwrap_or_default();
            let mut out = Vec::new();
            for (i, src_raw) in srcs.iter().enumerate() {
                let src = normalize_ref(src_raw);
                if src.is_empty() {
                    continue;
                }
                let style = normalize_ref(&styles.get(i).cloned().unwrap_or_default());
                let slot = infer_slot(&src);
                out.push(json!({ "src": src, "style": style, "slot": slot }));
            }
            Value::Array(out)
        }
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                if let Some(m) = row.as_object() {
                    let src = normalize_ref(&pick(m, SRC_KEYS).map(cell_str).unwrap_or_default());
                    if src.is_empty() {
                        continue;
                    }
                    let style = normalize_ref(&pick(m, STYLE_KEYS).map(cell_str).unwrap_or_default());
                    let slot = infer_slot(&src);
                    out.push(json!({ "src": src, "style": style, "slot": slot }));
                }
            }
            Value::Array(out)
        }
        _ => json!([]),
    }
}

fn infer_slot(src: &str) -> String {
    let base = src.rsplit('.').next().unwrap_or(src);
    match base {
        "nav" | "links" | "导航" => "nav".into(),
        "side" | "sidebar" | "侧栏" => "sidebar".into(),
        "foot" | "footer" | "页脚" => "footer".into(),
        other => other.to_string(),
    }
}

pub fn normalize_slot(name: &str) -> String {
    match name {
        "nav" | "links" | "导航" => "nav".into(),
        "side" | "sidebar" | "侧栏" => "sidebar".into(),
        "foot" | "footer" | "页脚" => "footer".into(),
        "main" | "index" | "" | "主体" => "main".into(),
        other => other.to_string(),
    }
}

/// Style table → CSS text.
///
/// Table shapes:
/// 1. Rule rows `|媒体|选择器|属性|值|` — rows grouped by `媒体` emit
///    `@media { … }` blocks; rows sharing a selector are merged into one rule.
/// 2. Rule rows `|选择器|属性|值|` — each row emits `selector { prop: value; }`,
///    rows sharing a selector are merged into one rule. `selector` may be any
///    CSS selector (`aside.side`, `ul.side-nav a:hover`).
/// 3. Plain property rows `|属性|值|` — emitted as `.name { prop: value; }`.
/// 4. **`@keyframes`** — selector `@keyframes name` (optional stop in the
///    selector: `@keyframes name from`) or stop in the `属性` column:
///    `|@keyframes fade|from|opacity: 0|` → nested keyframe rules.
///
/// Examples (rule shape):
/// ```text
/// | 媒体 | 选择器 | 属性 | 值 |
/// |------|--------|------|-----|
/// | (max-width: 860px) | aside.side | padding | 1rem 2rem |
///
/// | 选择器 | 属性 | 值 |
/// |--------|------|-----|
/// | aside.side | padding | 2rem 1.25rem |
/// | @keyframes pulse | 0% | opacity: 0 |
/// | @keyframes pulse | 100% | opacity: 1 |
/// | @keyframes pulse from | opacity | 0 |
/// ```
pub fn as_css_named(name: &str, table: &Value) -> String {
    as_css_named_checked(name, table, false).unwrap_or_default()
}

/// Like [`as_css_named`], but when `strict` is true, reject cells that look like
/// T5 evaluated a bare `/` (numeric/bool CSS values). Default mode warns on stderr.
pub fn as_css_named_checked(name: &str, table: &Value, strict: bool) -> Result<String, String> {
    let name = normalize_ref(name);
    if name.is_empty() {
        return Ok(String::new());
    }
    let mut out = String::new();
    match table {
        Value::Object(m) => {
            let sels = pick(m, SEL_KEYS).map(as_str_list).unwrap_or_default();
            let props = pick(m, PROP_KEYS).map(as_str_list).unwrap_or_default();
            let vals_raw = pick(m, VAL_KEYS);
            let vals = vals_raw.map(as_str_list).unwrap_or_default();
            let medias = pick(m, MEDIA_KEYS).map(as_str_list).unwrap_or_default();
            let n = props.len().max(sels.len()).max(vals.len());
            if n == 0 {
                return Ok(String::new());
            }
            if let Some(raw) = vals_raw {
                check_css_value_cells(raw, &props, strict)?;
            }
            let rows: Vec<(String, String, String, String)> = (0..n)
                .map(|i| {
                    (
                        medias.get(i).cloned().unwrap_or_default(),
                        sels.get(i).cloned().unwrap_or_default(),
                        props.get(i).cloned().unwrap_or_default(),
                        vals.get(i).cloned().unwrap_or_default(),
                    )
                })
                .collect();
            emit_css_rows(&rows, &name, &mut out);
        }
        Value::Array(rows) => {
            let mut collected = Vec::new();
            for row in rows {
                let Some(m) = row.as_object() else {
                    continue;
                };
                let prop = pick(m, PROP_KEYS).map(cell_str).unwrap_or_default();
                if let Some(v) = pick(m, VAL_KEYS) {
                    note_css_value_cell(&prop, v, strict)?;
                }
                collected.push((
                    pick(m, MEDIA_KEYS).map(cell_str).unwrap_or_default(),
                    pick(m, SEL_KEYS).map(cell_str).unwrap_or_default(),
                    prop,
                    pick(m, VAL_KEYS).map(cell_str).unwrap_or_default(),
                ));
            }
            emit_css_rows(&collected, &name, &mut out);
        }
        _ => {}
    }
    Ok(out)
}

fn css_value_suspicious(v: &Value) -> Option<&'static str> {
    match v {
        Value::Number(_) => Some(
            "numeric CSS value (likely bare `/` division such as `1 / 5`; quote the cell, e.g. `\"1 / 5\"`)",
        ),
        Value::Bool(_) => Some("boolean CSS value (quote the cell if intentional)"),
        _ => None,
    }
}

fn note_css_value_cell(prop: &str, val: &Value, strict: bool) -> Result<(), String> {
    let Some(why) = css_value_suspicious(val) else {
        return Ok(());
    };
    let prop = if prop.trim().is_empty() {
        "(value)"
    } else {
        prop.trim()
    };
    let msg = format!("ext/web style: suspicious cell for `{prop}`: {why}");
    if strict {
        Err(msg)
    } else {
        eprintln!("warning: {msg}");
        Ok(())
    }
}

fn check_css_value_cells(vals: &Value, props: &[String], strict: bool) -> Result<(), String> {
    match vals {
        Value::Array(a) => {
            for (i, v) in a.iter().enumerate() {
                let prop = props.get(i).map(|s| s.as_str()).unwrap_or("");
                note_css_value_cell(prop, v, strict)?;
            }
        }
        other => note_css_value_cell(props.first().map(|s| s.as_str()).unwrap_or(""), other, strict)?,
    }
    Ok(())
}

/// Parse `@keyframes name` or `@keyframes name stop` from a selector cell.
fn parse_keyframes_sel(sel: &str) -> Option<(String, Option<String>)> {
    let t = sel.trim();
    if t.len() < 10 || !t[..10].eq_ignore_ascii_case("@keyframes") {
        return None;
    }
    let rest = t[10..].trim();
    if rest.is_empty() {
        return None;
    }
    // First token = animation name; optional second = keyframe stop.
    let mut parts = rest.split_whitespace();
    let anim = parts.next()?.to_string();
    let stop = parts.next().map(|s| s.to_string());
    Some((anim, stop))
}

fn is_keyframe_stop(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    if t.eq_ignore_ascii_case("from") || t.eq_ignore_ascii_case("to") {
        return true;
    }
    // 0%, 50%, 100.5%, etc.
    let stripped = t.trim_end_matches('%');
    !stripped.is_empty()
        && stripped
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.')
        && t.ends_with('%')
}

fn keyframe_decl_lines(css_prop: &str, value: &str) -> Vec<String> {
    // Value may be a full declaration (`opacity: 0`) or bare value when the
    // CSS property lived in the selector form `@keyframes name from` + 属性.
    let v = value.trim().trim_end_matches(';').trim();
    if v.is_empty() {
        return Vec::new();
    }
    if v.contains(':') {
        v.split(';')
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .map(|p| format!("    {p};"))
            .collect()
    } else if !css_prop.is_empty() && !is_keyframe_stop(css_prop) {
        // `@keyframes name from` + 属性=opacity + 值=0
        vec![format!("    {css_prop}: {v};")]
    } else {
        // Bare value — prefer `opacity: 0` in the 值 column.
        vec![format!("    {v};")]
    }
}

fn push_keyframe(
    groups: &mut Vec<(String, Vec<(String, Vec<String>)>)>,
    anim: &str,
    stop: &str,
    decls: Vec<String>,
) {
    if anim.is_empty() || stop.is_empty() || decls.is_empty() {
        return;
    }
    let anim_group = match groups.iter_mut().find(|(n, _)| n == anim) {
        Some((_, g)) => g,
        None => {
            groups.push((anim.to_string(), Vec::new()));
            &mut groups.last_mut().unwrap().1
        }
    };
    match anim_group.iter_mut().find(|(s, _)| s == stop) {
        Some((_, d)) => d.extend(decls),
        None => anim_group.push((stop.to_string(), decls)),
    }
}

fn emit_css_rows(rows: &[(String, String, String, String)], class_name: &str, out: &mut String) {
    let mut by_media: Vec<(String, Vec<(String, Vec<String>)>)> = Vec::new();
    let mut by_keyframes: Vec<(String, Vec<(String, Vec<String>)>)> = Vec::new();
    let mut plain: Vec<String> = Vec::new();

    for (media, sel, p, v) in rows {
        let media = media.trim();
        let sel = sel.trim();
        let p = p.trim();
        let v = v.trim();

        if let Some((anim, stop_in_sel)) = parse_keyframes_sel(sel) {
            let stop_from_sel = stop_in_sel.is_some();
            let (stop, prop_for_decl) = match stop_in_sel {
                Some(s) => (s, p.to_string()),
                None => {
                    // 属性 = keyframe stop (from / to / N%); 值 = declaration(s)
                    if !is_keyframe_stop(p) {
                        continue;
                    }
                    (p.to_string(), String::new())
                }
            };
            let decls = if stop_from_sel {
                keyframe_decl_lines(&prop_for_decl, v)
            } else {
                keyframe_decl_lines("", v)
            };
            let decls = if decls.is_empty() && !v.is_empty() && !prop_for_decl.is_empty() {
                keyframe_decl_lines(&prop_for_decl, v)
            } else {
                decls
            };
            push_keyframe(&mut by_keyframes, &anim, &stop, decls);
            continue;
        }

        if p.is_empty() {
            continue;
        }
        let decl = format!("  {p}: {v};");
        if !media.is_empty() {
            let group = match by_media.iter_mut().find(|(mq, _)| *mq == media) {
                Some((_, g)) => g,
                None => {
                    by_media.push((media.to_string(), Vec::new()));
                    let (_, g) = by_media.last_mut().unwrap();
                    g
                }
            };
            push_rule(group, sel, decl);
        } else if !sel.is_empty() {
            let group = match by_media.iter_mut().find(|(mq, _)| mq.is_empty()) {
                Some((_, g)) => g,
                None => {
                    by_media.push((String::new(), Vec::new()));
                    let (_, g) = by_media.last_mut().unwrap();
                    g
                }
            };
            push_rule(group, sel, decl);
        } else {
            plain.push(decl);
        }
    }

    for (media, rules) in by_media {
        if media.is_empty() {
            for (sel, decls) in rules {
                out.push_str(&format!("{} {{\n{}\n}}\n", sel, decls.join("\n")));
            }
        } else {
            let inner = rules
                .iter()
                .map(|(sel, decls)| format!("{} {{\n{}\n}}", sel, decls.join("\n")))
                .collect::<Vec<_>>()
                .join("\n");
            out.push_str(&format!("@media {} {{\n{}\n}}\n", media, inner));
        }
    }
    for (anim, stops) in by_keyframes {
        out.push_str(&format!("@keyframes {} {{\n", anim));
        for (stop, decls) in stops {
            out.push_str(&format!("  {} {{\n", stop));
            out.push_str(&decls.join("\n"));
            out.push('\n');
            out.push_str("  }\n");
        }
        out.push_str("}\n");
    }
    if !plain.is_empty() {
        out.push_str(&format!(".{} {{\n{}\n}}\n", class_name, plain.join("\n")));
    }
}

/// Push `decl` under `sel` into a `(selector, declarations)` group list,
/// merging declarations when the selector already exists.
fn push_rule(groups: &mut Vec<(String, Vec<String>)>, sel: &str, decl: String) {
    if sel.is_empty() {
        return;
    }
    match groups.iter_mut().find(|(s, _)| s == sel) {
        Some((_, decls)) => decls.push(decl),
        None => groups.push((sel.to_string(), vec![decl])),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SitePath {
    LibMember { lib: String, member: String },
    DbField { table: String, field: String },
    Plain(String),
}

pub fn parse_site_path(raw: &str) -> SitePath {
    let s = normalize_ref(raw);
    if s.is_empty() {
        return SitePath::Plain(String::new());
    }
    let parts: Vec<&str> = s.split('.').filter(|p| !p.is_empty()).collect();
    match parts.as_slice() {
        [lib, member] => SitePath::LibMember {
            lib: (*lib).into(),
            member: (*member).into(),
        },
        [_, table, field] => SitePath::DbField {
            table: (*table).into(),
            field: (*field).into(),
        },
        _ => SitePath::Plain(s),
    }
}

pub fn bind_table_name(binds: &[Value]) -> Option<String> {
    let mut found: Option<String> = None;
    for b in binds {
        let back = b.get("back").and_then(|v| v.as_str()).unwrap_or("");
        if let SitePath::DbField { table, .. } = parse_site_path(back) {
            match &found {
                None => found = Some(table),
                Some(prev) if prev != &table => return None,
                _ => {}
            }
        } else if back.contains('.') {
            let mut p = back.splitn(2, '.');
            if let (Some(t), Some(_)) = (p.next(), p.next()) {
                match &found {
                    None => found = Some(t.into()),
                    Some(prev) if prev != t => return None,
                    _ => {}
                }
            }
        }
    }
    found
}

pub fn project_rows(binds: &[Value], rows: &[Value]) -> Value {
    let mut out = Vec::new();
    for row in rows {
        let obj = row.as_object().cloned().unwrap_or_default();
        let mut m = Map::new();
        let mut css = Map::new();
        for b in binds {
            let front = b.get("front").and_then(|v| v.as_str()).unwrap_or("");
            let back = b.get("back").and_then(|v| v.as_str()).unwrap_or("");
            let class = b.get("css").and_then(|v| v.as_str()).unwrap_or("");
            let col = match parse_site_path(back) {
                SitePath::DbField { field, .. } => field,
                SitePath::LibMember { member, .. } => member,
                SitePath::Plain(s) if s.contains('.') => {
                    s.split('.').next_back().unwrap_or(&s).to_string()
                }
                SitePath::Plain(s) => s,
            };
            if let Some(v) = obj.get(&col).or_else(|| obj.get(front)) {
                m.insert(front.to_string(), v.clone());
            }
            if !class.is_empty() {
                css.insert(front.to_string(), json!(class));
            }
        }
        if let Some(id) = obj.get("id") {
            m.insert("id".into(), id.clone());
        }
        if !css.is_empty() {
            m.insert("_css".into(), Value::Object(css));
        }
        out.push(Value::Object(m));
    }
    Value::Array(out)
}

const META_KEY_KEYS: &[&str] = &["键", "key", "name"];
const META_VAL_KEYS: &[&str] = &["值", "value"];

/// SEO / OpenGraph meta table → `{title, description, og:…, canonical, …}`.
pub fn as_meta_map(table: &Value) -> Map<String, Value> {
    let mut out = Map::new();
    match table {
        Value::Array(rows) => {
            for row in rows {
                let Some(obj) = row.as_object() else {
                    continue;
                };
                let key = META_KEY_KEYS
                    .iter()
                    .find_map(|k| obj.get(*k))
                    .map(|v| normalize_ref(&meta_text(v)))
                    .unwrap_or_default();
                let val = META_VAL_KEYS
                    .iter()
                    .find_map(|k| obj.get(*k))
                    .map(|v| meta_text(v))
                    .unwrap_or_default();
                if !key.is_empty() {
                    out.insert(key, json!(val));
                }
            }
        }
        Value::Object(m) => {
            for (k, v) in m {
                out.insert(k.clone(), json!(meta_text(v)));
            }
        }
        _ => {}
    }
    out
}

fn meta_text(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod css_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keyframes_stop_in_prop() {
        let t = json!([
            {"选择器": "@keyframes pulse", "属性": "0%", "值": "opacity: 0"},
            {"选择器": "@keyframes pulse", "属性": "100%", "值": "opacity: 1"},
        ]);
        let css = as_css_named("anim", &t);
        assert!(css.contains("@keyframes pulse {"), "{css}");
        assert!(css.contains("0% {"), "{css}");
        assert!(css.contains("opacity: 0;"), "{css}");
        assert!(css.contains("100% {"), "{css}");
        assert!(!css.contains("0%: opacity"), "{css}");
    }

    #[test]
    fn keyframes_stop_in_selector() {
        let t = json!([
            {"选择器": "@keyframes fade from", "属性": "opacity", "值": "0"},
            {"选择器": "@keyframes fade to", "属性": "opacity", "值": "1"},
        ]);
        let css = as_css_named("anim", &t);
        assert!(css.contains("@keyframes fade {"), "{css}");
        assert!(css.contains("from {"), "{css}");
        assert!(css.contains("opacity: 0;"), "{css}");
        assert!(css.contains("to {"), "{css}");
    }

    #[test]
    fn media_still_works() {
        let t = json!([
            {"媒体": "(max-width: 800px)", "选择器": ".x", "属性": "color", "值": "red"},
        ]);
        let css = as_css_named("x", &t);
        assert!(css.contains("@media (max-width: 800px)"), "{css}");
        assert!(css.contains(".x {"), "{css}");
        assert!(css.contains("color: red;"), "{css}");
    }

    #[test]
    fn strict_rejects_numeric_css_value() {
        let t = json!([
            {"选择器": ".x", "属性": "grid-column", "值": 0},
        ]);
        let err = as_css_named_checked("x", &t, true).unwrap_err();
        assert!(err.contains("suspicious"), "{err}");
        // Non-strict still emits (with warning on stderr).
        let css = as_css_named_checked("x", &t, false).unwrap();
        assert!(css.contains("grid-column: 0;"), "{css}");
    }

    #[test]
    fn quoted_string_ok_in_strict() {
        let t = json!([
            {"选择器": ".x", "属性": "grid-column", "值": "1 / 5"},
        ]);
        let css = as_css_named_checked("x", &t, true).unwrap();
        assert!(css.contains("grid-column: 1 / 5;"), "{css}");
    }
}
