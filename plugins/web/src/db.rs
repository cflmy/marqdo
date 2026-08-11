//! SQLite helpers: open, migrate, define, CRUD.

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params_from_iter, Connection, ToSql};
use serde_json::{json, Map, Value};

fn resolve_sqlite_path(url: &str) -> Result<PathBuf, String> {
    let u = url.trim();
    let path = if let Some(rest) = u.strip_prefix("sqlite:") {
        let rest = rest.trim();
        if let Some(r) = rest.strip_prefix("///") {
            // sqlite:///abs/path
            format!("/{r}")
        } else if rest.starts_with('/') {
            // sqlite:/abs/path
            rest.to_string()
        } else {
            rest.trim_start_matches("./").to_string()
        }
    } else if u.starts_with("postgres://") || u.starts_with("postgresql://") {
        return Err("postgres not enabled in this build (W4); use sqlite: URL".into());
    } else {
        u.to_string()
    };
    let p = PathBuf::from(if path.is_empty() {
        "./data/app.db"
    } else {
        &path
    });
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("create db dir: {e}"))?;
        }
    }
    Ok(p)
}

pub fn open_db(url: &str) -> Result<Connection, String> {
    let path = resolve_sqlite_path(url)?;
    Connection::open(&path).map_err(|e| format!("open db {}: {e}", path.display()))
}

fn ident_ok(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn quote_ident(s: &str) -> Result<String, String> {
    if !ident_ok(s) {
        return Err(format!("invalid identifier `{s}`"));
    }
    Ok(format!("\"{s}\""))
}

fn sql_type(t: &str) -> &'static str {
    match t.trim().to_ascii_lowercase().as_str() {
        "int" | "integer" | "bigint" => "INTEGER",
        "bool" | "boolean" => "INTEGER",
        "real" | "float" | "double" => "REAL",
        "timestamp" | "datetime" => "TEXT",
        _ => "TEXT",
    }
}

/// Ensure migrations table and apply `*.sql` files in `dir` (sorted).
pub fn migrate(url: &str, dir: &str) -> Result<Value, String> {
    let conn = open_db(url)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _marqdo_migrations (
            id TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|e| format!("ensure migrations table: {e}"))?;

    let dir = Path::new(dir);
    if !dir.is_dir() {
        return Ok(json!({ "ok": true, "applied": [], "note": "no migrations dir" }));
    }
    let mut files: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("read migrations: {e}"))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("sql"))
        .collect();
    files.sort();

    let mut applied = Vec::new();
    for path in files {
        let id = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let already: bool = conn
            .query_row(
                "SELECT 1 FROM _marqdo_migrations WHERE id = ?1",
                [&id],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if already {
            continue;
        }
        let sql = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        conn.execute_batch(&sql)
            .map_err(|e| format!("migrate {id}: {e}"))?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO _marqdo_migrations (id, applied_at) VALUES (?1, ?2)",
            [&id, &now],
        )
        .map_err(|e| format!("record migration: {e}"))?;
        applied.push(Value::String(id));
    }
    Ok(json!({ "ok": true, "applied": applied }))
}

/// Create table from field list (list of maps: name, type, null, default, unique).
pub fn define_table(url: &str, table: &str, fields: &Value, primary: &str) -> Result<Value, String> {
    let table_q = quote_ident(table)?;
    let primary = if primary.is_empty() { "id" } else { primary };
    let rows = match fields {
        Value::Array(a) => a.clone(),
        _ => return Err("fields must be a list of maps".into()),
    };
    let mut cols = Vec::new();
    for row in &rows {
        let name = row
            .get("name")
            .or_else(|| row.get("名"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "field missing name".to_string())?;
        let ty = row
            .get("type")
            .or_else(|| row.get("类型"))
            .and_then(|v| v.as_str())
            .unwrap_or("text");
        let nullable = row
            .get("null")
            .or_else(|| row.get("可空"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let unique = row
            .get("unique")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let name_q = quote_ident(name)?;
        let mut col = format!("{name_q} {}", sql_type(ty));
        if name == primary {
            col.push_str(" PRIMARY KEY");
            if sql_type(ty) == "INTEGER" {
                col.push_str(" AUTOINCREMENT");
            }
        } else if !nullable {
            col.push_str(" NOT NULL");
        }
        if unique && name != primary {
            col.push_str(" UNIQUE");
        }
        if let Some(def) = row.get("default").or_else(|| row.get("默认")) {
            if let Some(s) = def.as_str() {
                if s.eq_ignore_ascii_case("now") {
                    col.push_str(" DEFAULT (datetime('now'))");
                } else if s == "false" {
                    col.push_str(" DEFAULT 0");
                } else if s == "true" {
                    col.push_str(" DEFAULT 1");
                } else {
                    col.push_str(&format!(" DEFAULT '{s}'"));
                }
            } else if let Some(b) = def.as_bool() {
                col.push_str(if b { " DEFAULT 1" } else { " DEFAULT 0" });
            } else if let Some(n) = def.as_i64() {
                col.push_str(&format!(" DEFAULT {n}"));
            }
        }
        cols.push(col);
    }
    if cols.is_empty() {
        return Err("no fields".into());
    }
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS {table_q} ({});",
        cols.join(", ")
    );
    let conn = open_db(url)?;
    conn.execute_batch(&sql)
        .map_err(|e| format!("define table: {e}"))?;
    Ok(json!({ "ok": true, "table": table, "sql": sql }))
}

fn row_values(
    row: &rusqlite::Row<'_>,
    names: &[String],
) -> Result<Value, String> {
    let mut map = Map::new();
    for (i, name) in names.iter().enumerate() {
        let v: Value = match row.get_ref(i).map_err(|e| e.to_string())? {
            rusqlite::types::ValueRef::Null => Value::Null,
            rusqlite::types::ValueRef::Integer(n) => json!(n),
            rusqlite::types::ValueRef::Real(f) => json!(f),
            rusqlite::types::ValueRef::Text(t) => {
                Value::String(String::from_utf8_lossy(t).into_owned())
            }
            rusqlite::types::ValueRef::Blob(b) => {
                Value::String(format!("blob:{}bytes", b.len()))
            }
        };
        map.insert(name.clone(), v);
    }
    Ok(Value::Object(map))
}

pub fn query_all(url: &str, table: &str, limit: i64) -> Result<Value, String> {
    let table_q = quote_ident(table)?;
    let lim = if limit <= 0 { 200 } else { limit.min(2000) };
    let conn = open_db(url)?;
    let sql = format!("SELECT * FROM {table_q} LIMIT {lim}");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let names: Vec<String> = (0..stmt.column_count())
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        out.push(row_values(row, &names)?);
    }
    Ok(json!({ "rows": out, "count": out.len() }))
}

pub fn exec_sql(url: &str, sql: &str, args: Option<&Value>) -> Result<Value, String> {
    let conn = open_db(url)?;
    let bind = json_args(args)?;
    let refs: Vec<&dyn ToSql> = bind.iter().map(|v| v as &dyn ToSql).collect();
    let n = conn
        .execute(sql, params_from_iter(refs))
        .map_err(|e| format!("exec: {e}"))?;
    Ok(json!({ "ok": true, "changes": n }))
}

pub fn query_sql(url: &str, sql: &str, args: Option<&Value>) -> Result<Value, String> {
    let conn = open_db(url)?;
    let bind = json_args(args)?;
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let names: Vec<String> = (0..stmt.column_count())
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();
    let refs: Vec<&dyn ToSql> = bind.iter().map(|v| v as &dyn ToSql).collect();
    let mut rows = stmt
        .query(params_from_iter(refs))
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let mut map = Map::new();
        for (i, name) in names.iter().enumerate() {
            let v: Value = match row.get_ref(i).map_err(|e| e.to_string())? {
                rusqlite::types::ValueRef::Null => Value::Null,
                rusqlite::types::ValueRef::Integer(n) => json!(n),
                rusqlite::types::ValueRef::Real(f) => json!(f),
                rusqlite::types::ValueRef::Text(t) => {
                    Value::String(String::from_utf8_lossy(t).into_owned())
                }
                rusqlite::types::ValueRef::Blob(b) => {
                    Value::String(format!("blob:{}bytes", b.len()))
                }
            };
            map.insert(name.clone(), v);
        }
        out.push(Value::Object(map));
    }
    Ok(json!({ "rows": out, "count": out.len() }))
}

enum SqlVal {
    Null,
    Int(i64),
    Real(f64),
    Text(String),
}

impl ToSql for SqlVal {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(match self {
            SqlVal::Null => rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Null),
            SqlVal::Int(n) => rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer(*n)),
            SqlVal::Real(f) => rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Real(*f)),
            SqlVal::Text(s) => {
                rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Text(s.clone()))
            }
        })
    }
}

fn json_args(args: Option<&Value>) -> Result<Vec<SqlVal>, String> {
    let Some(v) = args else {
        return Ok(Vec::new());
    };
    let arr = v
        .as_array()
        .ok_or_else(|| "args must be a JSON array".to_string())?;
    let mut out = Vec::new();
    for a in arr {
        out.push(match a {
            Value::Null => SqlVal::Null,
            Value::Bool(b) => SqlVal::Int(if *b { 1 } else { 0 }),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    SqlVal::Int(i)
                } else if let Some(f) = n.as_f64() {
                    SqlVal::Real(f)
                } else {
                    SqlVal::Text(n.to_string())
                }
            }
            Value::String(s) => SqlVal::Text(s.clone()),
            other => SqlVal::Text(other.to_string()),
        });
    }
    Ok(out)
}

pub fn insert_row(url: &str, table: &str, row: &Value) -> Result<Value, String> {
    let obj = row
        .as_object()
        .ok_or_else(|| "row must be a map".to_string())?;
    let table_q = quote_ident(table)?;
    let mut cols = Vec::new();
    let mut placeholders = Vec::new();
    let mut vals = Vec::new();
    for (k, v) in obj {
        if !ident_ok(k) {
            return Err(format!("bad column `{k}`"));
        }
        cols.push(format!("\"{k}\""));
        placeholders.push("?");
        vals.push(v.clone());
    }
    if cols.is_empty() {
        return Err("empty row".into());
    }
    let sql = format!(
        "INSERT INTO {table_q} ({}) VALUES ({})",
        cols.join(", "),
        placeholders.join(", ")
    );
    let r = exec_sql(url, &sql, Some(&Value::Array(vals)))?;
    let _ = admin_log(url, "create", table, None, Some(row));
    Ok(r)
}

/// Insert many rows from `as_rows` output.
pub fn insert_rows(url: &str, table: &str, rows: &Value) -> Result<Value, String> {
    let arr = rows
        .as_array()
        .ok_or_else(|| "rows must be an array".to_string())?;
    let mut n = 0;
    for row in arr {
        insert_row(url, table, row)?;
        n += 1;
    }
    Ok(json!({ "ok": true, "inserted": n }))
}

pub fn get_row(url: &str, table: &str, id: &str) -> Result<Value, String> {
    let table_q = quote_ident(table)?;
    let conn = open_db(url)?;
    let sql = format!("SELECT * FROM {table_q} WHERE \"id\" = ?1 LIMIT 1");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let names: Vec<String> = (0..stmt.column_count())
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();
    let mut rows = stmt.query(rusqlite::params![id]).map_err(|e| e.to_string())?;
    let Some(row) = rows.next().map_err(|e| e.to_string())? else {
        return Err(format!("row id={id} not found"));
    };
    row_values(row, &names)
}

pub fn update_row(url: &str, table: &str, id: &str, row: &Value) -> Result<Value, String> {
    let obj = row
        .as_object()
        .ok_or_else(|| "row must be a map".to_string())?;
    let table_q = quote_ident(table)?;
    let mut sets = Vec::new();
    let mut vals = Vec::new();
    for (k, v) in obj {
        if k == "id" || !ident_ok(k) {
            continue;
        }
        sets.push(format!("\"{k}\" = ?"));
        vals.push(v.clone());
    }
    if sets.is_empty() {
        return Err("nothing to update".into());
    }
    vals.push(json!(id));
    let sql = format!(
        "UPDATE {table_q} SET {} WHERE \"id\" = ?",
        sets.join(", ")
    );
    let r = exec_sql(url, &sql, Some(&Value::Array(vals)))?;
    let _ = admin_log(url, "update", table, Some(id), Some(row));
    Ok(r)
}

pub fn delete_row(url: &str, table: &str, id: &str) -> Result<Value, String> {
    let table_q = quote_ident(table)?;
    let sql = format!("DELETE FROM {table_q} WHERE \"id\" = ?");
    let r = exec_sql(url, &sql, Some(&json!([id])))?;
    let _ = admin_log(url, "delete", table, Some(id), None);
    Ok(r)
}

pub fn ensure_admin_log(url: &str) -> Result<(), String> {
    let conn = open_db(url)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _marqdo_admin_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            at TEXT NOT NULL DEFAULT (datetime('now')),
            action TEXT NOT NULL,
            table_name TEXT NOT NULL,
            row_id TEXT,
            detail TEXT
        );",
    )
    .map_err(|e| format!("ensure admin log: {e}"))?;
    Ok(())
}

pub fn admin_log(
    url: &str,
    action: &str,
    table: &str,
    row_id: Option<&str>,
    detail: Option<&Value>,
) -> Result<(), String> {
    ensure_admin_log(url)?;
    let detail_s = detail
        .map(|v| v.to_string())
        .unwrap_or_default();
    exec_sql(
        url,
        "INSERT INTO _marqdo_admin_log (action, table_name, row_id, detail) VALUES (?, ?, ?, ?)",
        Some(&json!([action, table, row_id.unwrap_or(""), detail_s])),
    )?;
    Ok(())
}

/// User tables for admin (skip sqlite_* and _marqdo_*).
pub fn list_user_tables(url: &str) -> Result<Vec<String>, String> {
    let conn = open_db(url)?;
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master
             WHERE type='table'
               AND name NOT LIKE 'sqlite_%'
               AND name NOT LIKE '_marqdo_%'
             ORDER BY name",
        )
        .map_err(|e| format!("list tables: {e}"))?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| format!("list tables query: {e}"))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("list tables row: {e}"))?);
    }
    Ok(out)
}

/// Column names via PRAGMA table_info.
pub fn table_columns(url: &str, table: &str) -> Result<Vec<String>, String> {
    if !ident_ok(table) {
        return Err(format!("bad table `{table}`"));
    }
    let conn = open_db(url)?;
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info(\"{table}\")"))
        .map_err(|e| format!("pragma: {e}"))?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .map_err(|e| format!("pragma map: {e}"))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("pragma row: {e}"))?);
    }
    Ok(out)
}

fn admin_shell(title: &str, body: &str, tables: &[String], current: Option<&str>) -> String {
    let mut side = Vec::new();
    side.push(json!({"label": "Dashboard", "href": "/admin"}));
    for t in tables {
        let label = if current == Some(t.as_str()) {
            format!("· {t}")
        } else {
            t.clone()
        };
        side.push(json!({"label": label, "href": format!("/admin/{t}")}));
    }
    side.push(json!({"label": "Log", "href": "/admin/log"}));
    crate::page::render_page(&json!({
        "title": title,
        "nav": [
            {"label": "Site", "href": "/"},
            {"label": "Admin", "href": "/admin"},
            {"label": "Log", "href": "/admin/log"}
        ],
        "sidebar": side,
        "intro": body,
    }))
}

fn admin_bar(title: &str, crumbs: &str, actions: &str) -> String {
    format!(
        "<div class=\"admin-page\">\
         <header class=\"admin-bar\">\
           <div class=\"admin-bar-text\">\
             <p class=\"admin-crumbs\">{crumbs}</p>\
             <h1>{title}</h1>\
           </div>\
           <div class=\"admin-actions\">{actions}</div>\
         </header>"
    )
}

fn cell_display(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

pub fn admin_home_html(url: Option<&str>, tables: &[String]) -> String {
    let count = tables.len();
    let actions = "<a class=\"btn\" href=\"/admin/log\">View log</a>";
    let mut html = admin_bar("Dashboard", "Admin", actions);
    html.push_str(&format!(
        "<p class=\"admin-lead\">{count} table(s) in the database.</p>"
    ));
    if tables.is_empty() {
        html.push_str(
            "<p class=\"admin-empty\">No user tables yet. Define one with <code>db.init</code>.</p>",
        );
    } else {
        html.push_str("<div class=\"admin-cards\">");
        for t in tables {
            let t_e = crate::page::html_escape(t);
            let n = url
                .and_then(|u| query_all(u, t, 200).ok())
                .and_then(|d| d.get("count").and_then(|v| v.as_u64()))
                .unwrap_or(0);
            html.push_str(&format!(
                "<article class=\"admin-card\">\
                   <h2>{t_e}</h2>\
                   <p class=\"admin-meta\">{n} rows</p>\
                   <div class=\"admin-card-actions\">\
                     <a class=\"btn\" href=\"/admin/{t_e}\">Open</a>\
                     <a class=\"btn btn-ghost\" href=\"/admin/{t_e}/new\">New</a>\
                   </div>\
                 </article>"
            ));
        }
        html.push_str("</div>");
    }
    html.push_str("</div>");
    admin_shell("Admin", &html, tables, None)
}

pub fn admin_list_html(url: &str, table: &str) -> Result<String, String> {
    let tables = list_user_tables(url).unwrap_or_default();
    let cols = table_columns(url, table)?;
    let data = query_all(url, table, 200)?;
    let rows = data
        .get("rows")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let t = crate::page::html_escape(table);
    let actions = format!("<a class=\"btn\" href=\"/admin/{t}/new\">New row</a>");
    let mut html = admin_bar(
        table,
        &format!("<a href=\"/admin\">Admin</a> / {t}"),
        &actions,
    );
    let headers = if cols.is_empty() {
        rows.first()
            .and_then(|r| r.as_object())
            .map(|m| m.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    } else {
        cols.clone()
    };
    html.push_str("<div class=\"content table-wrap admin-table\"><table><thead><tr>");
    for k in &headers {
        html.push_str(&format!("<th>{}</th>", crate::page::html_escape(k)));
    }
    html.push_str("<th class=\"col-actions\">Actions</th></tr></thead><tbody>");
    if rows.is_empty() {
        html.push_str(&format!(
            "<tr><td colspan=\"{}\">No rows yet.</td></tr>",
            headers.len() + 1
        ));
    } else {
        for r in &rows {
            html.push_str("<tr>");
            let obj = r.as_object().cloned().unwrap_or_default();
            for k in &headers {
                let cell = obj.get(k).map(cell_display).unwrap_or_default();
                let truncated = if cell.chars().count() > 120 {
                    let s: String = cell.chars().take(117).collect();
                    format!("{s}…")
                } else {
                    cell
                };
                html.push_str(&format!(
                    "<td>{}</td>",
                    crate::page::html_escape(&truncated)
                ));
            }
            let id = obj.get("id").map(cell_display).unwrap_or_default();
            let id_e = crate::page::html_escape(&id);
            html.push_str(&format!(
                "<td class=\"col-actions\">\
                   <a class=\"btn btn-small\" href=\"/admin/{t}/{id_e}/edit\">Edit</a>\
                   <form class=\"inline-form\" method=\"post\" action=\"/admin/{t}/{id_e}/delete\" \
                   onsubmit=\"return confirm('Delete row {id_e}?');\">\
                   <button type=\"submit\" class=\"btn btn-small btn-danger\">Delete</button></form>\
                 </td></tr>"
            ));
        }
    }
    html.push_str("</tbody></table></div></div>");
    Ok(admin_shell(
        &format!("Admin · {table}"),
        &html,
        &tables,
        Some(table),
    ))
}

fn form_fields(cols: &[String], values: Option<&Map<String, Value>>) -> String {
    let mut fields = String::from("<div class=\"admin-form-fields\">");
    let mut any = false;
    for c in cols {
        if c == "id" {
            continue;
        }
        any = true;
        let name = crate::page::html_escape(c);
        let val = values
            .and_then(|m| m.get(c))
            .map(cell_display)
            .unwrap_or_default();
        let val_e = crate::page::html_escape(&val);
        if c == "body" || c == "content" || c == "正文" || c == "detail" {
            fields.push_str(&format!(
                "<label class=\"admin-field\"><span>{name}</span>\
                 <textarea name=\"{name}\" rows=\"6\">{val_e}</textarea></label>"
            ));
        } else {
            fields.push_str(&format!(
                "<label class=\"admin-field\"><span>{name}</span>\
                 <input name=\"{name}\" value=\"{val_e}\"/></label>"
            ));
        }
    }
    if !any {
        fields.push_str("<p class=\"admin-empty\">No editable columns.</p>");
    }
    fields.push_str("</div>");
    fields
}

pub fn admin_new_form_html(url: &str, table: &str) -> Result<String, String> {
    let tables = list_user_tables(url).unwrap_or_default();
    let cols = table_columns(url, table)?;
    let fields = form_fields(&cols, None);
    let t = crate::page::html_escape(table);
    let actions = format!("<a class=\"btn btn-ghost\" href=\"/admin/{t}\">Cancel</a>");
    let mut html = admin_bar(
        &format!("New · {table}"),
        &format!("<a href=\"/admin\">Admin</a> / <a href=\"/admin/{t}\">{t}</a> / new"),
        &actions,
    );
    html.push_str(&format!(
        "<form class=\"admin-form\" method=\"post\" action=\"/admin/{t}/new\">{fields}\
         <div class=\"admin-form-footer\">\
           <button type=\"submit\" class=\"btn\">Create</button>\
           <a class=\"btn btn-ghost\" href=\"/admin/{t}\">Back</a>\
         </div></form></div>"
    ));
    Ok(admin_shell(
        &format!("New {table}"),
        &html,
        &tables,
        Some(table),
    ))
}

pub fn admin_edit_form_html(url: &str, table: &str, id: &str) -> Result<String, String> {
    let tables = list_user_tables(url).unwrap_or_default();
    let cols = table_columns(url, table)?;
    let row = get_row(url, table, id)?;
    let obj = row.as_object();
    let fields = form_fields(&cols, obj);
    let t = crate::page::html_escape(table);
    let id_e = crate::page::html_escape(id);
    let actions = format!("<a class=\"btn btn-ghost\" href=\"/admin/{t}\">Cancel</a>");
    let mut html = admin_bar(
        &format!("Edit · {table} #{id}"),
        &format!("<a href=\"/admin\">Admin</a> / <a href=\"/admin/{t}\">{t}</a> / {id_e}"),
        &actions,
    );
    html.push_str(&format!(
        "<form class=\"admin-form\" method=\"post\" action=\"/admin/{t}/{id_e}/edit\">{fields}\
         <div class=\"admin-form-footer\">\
           <button type=\"submit\" class=\"btn\">Save</button>\
           <a class=\"btn btn-ghost\" href=\"/admin/{t}\">Back</a>\
         </div></form></div>"
    ));
    Ok(admin_shell(
        &format!("Edit {table}"),
        &html,
        &tables,
        Some(table),
    ))
}

pub fn admin_log_html(url: &str) -> Result<String, String> {
    ensure_admin_log(url)?;
    let tables = list_user_tables(url).unwrap_or_default();
    let data = query_sql(
        url,
        "SELECT id, at, action, table_name, row_id, detail FROM _marqdo_admin_log ORDER BY id DESC LIMIT 200",
        None,
    )?;
    let rows = data
        .get("rows")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut html = admin_bar("Activity log", "<a href=\"/admin\">Admin</a> / log", "");
    html.push_str(
        "<div class=\"content table-wrap admin-table\"><table><thead><tr>\
         <th>id</th><th>at</th><th>action</th><th>table</th><th>row</th><th>detail</th>\
         </tr></thead><tbody>",
    );
    if rows.is_empty() {
        html.push_str("<tr><td colspan=\"6\">No log entries yet.</td></tr>");
    } else {
        for r in &rows {
            let o = r.as_object().cloned().unwrap_or_default();
            html.push_str("<tr>");
            for k in ["id", "at", "action", "table_name", "row_id", "detail"] {
                let mut cell = o.get(k).map(cell_display).unwrap_or_default();
                if k == "detail" && cell.chars().count() > 80 {
                    cell = format!("{}…", cell.chars().take(77).collect::<String>());
                }
                let class = if k == "action" {
                    format!(
                        " class=\"log-action log-{}\"",
                        crate::page::html_escape(&cell)
                    )
                } else {
                    String::new()
                };
                html.push_str(&format!(
                    "<td{class}>{}</td>",
                    crate::page::html_escape(&cell)
                ));
            }
            html.push_str("</tr>");
        }
    }
    html.push_str("</tbody></table></div></div>");
    Ok(admin_shell("Admin log", &html, &tables, None))
}
