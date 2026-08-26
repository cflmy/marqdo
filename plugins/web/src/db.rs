//! SQLite helpers for `# db` CRUD (`init` / `insert` / `select` / `get` / `update` / `delete` / `exec`).

use rusqlite::{types::ValueRef, Connection, OptionalExtension, params_from_iter};
use serde_json::{json, Map, Value};
use std::path::PathBuf;

fn open(url: &str) -> Result<Connection, String> {
    let path = url
        .strip_prefix("sqlite:")
        .or_else(|| url.strip_prefix("SQLITE:"))
        .unwrap_or(url);
    if let Some(parent) = PathBuf::from(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| e.to_string())?;
    Ok(conn)
}

fn ident(s: &str) -> Result<&str, String> {
    let s = s.trim();
    if s.is_empty()
        || !s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(format!("bad identifier `{s}`"));
    }
    Ok(s)
}

fn sql_type(t: &str) -> &'static str {
    match t.trim().to_ascii_lowercase().as_str() {
        "int" | "integer" => "INTEGER",
        "real" | "float" | "double" => "REAL",
        "blob" => "BLOB",
        _ => "TEXT",
    }
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

fn to_sql(v: &Value) -> rusqlite::types::Value {
    match v {
        Value::Null => rusqlite::types::Value::Null,
        Value::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rusqlite::types::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                rusqlite::types::Value::Real(f)
            } else {
                rusqlite::types::Value::Text(n.to_string())
            }
        }
        Value::String(s) => {
            let t = s.trim();
            if let Ok(i) = t.parse::<i64>() {
                rusqlite::types::Value::Integer(i)
            } else if let Ok(f) = t.parse::<f64>() {
                if !t.contains('e') && !t.contains('E') && t.contains('.') {
                    rusqlite::types::Value::Real(f)
                } else {
                    rusqlite::types::Value::Text(s.clone())
                }
            } else {
                rusqlite::types::Value::Text(s.clone())
            }
        }
        other => rusqlite::types::Value::Text(other.to_string()),
    }
}

fn from_sql(v: ValueRef<'_>) -> Value {
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => json!(i),
        ValueRef::Real(f) => json!(f),
        ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => Value::String(format!("blob:{}b", b.len())),
    }
}

/// `fields` = list of {name,type,nullable?} or columnar schema table already normalized.
pub fn init(url: &str, table: &str, fields: &Value) -> Result<Value, String> {
    let table = ident(table)?;
    let cols = crate::table::as_fields(fields);
    let arr = cols.as_array().ok_or("fields must be a list")?;
    if arr.is_empty() {
        return Err("empty fields".into());
    }
    let mut parts = Vec::new();
    let mut has_pk = false;
    for c in arr {
        let name = c
            .get("name")
            .or_else(|| c.get("字段"))
            .map(cell_str)
            .unwrap_or_default();
        let name = ident(&name)?;
        let ty = c
            .get("type")
            .or_else(|| c.get("类型"))
            .map(cell_str)
            .unwrap_or_else(|| "text".into());
        let nullable = c
            .get("nullable")
            .or_else(|| c.get("可空"))
            .and_then(|v| match v {
                Value::Bool(b) => Some(*b),
                Value::String(s) => Some(matches!(s.as_str(), "true" | "True" | "1" | "是" | "可")),
                _ => None,
            })
            .unwrap_or(true);
        let mut col = format!("\"{name}\" {}", sql_type(&ty));
        if name == "id" && !has_pk {
            col.push_str(" PRIMARY KEY");
            if sql_type(&ty) == "INTEGER" {
                col.push_str(" AUTOINCREMENT");
            }
            has_pk = true;
        } else if !nullable {
            col.push_str(" NOT NULL");
        }
        parts.push(col);
    }
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS \"{table}\" ({})",
        parts.join(", ")
    );
    let conn = open(url)?;
    conn.execute_batch(&sql).map_err(|e| e.to_string())?;
    Ok(json!({ "_type": "db_table", "name": table, "url": url }))
}

pub fn insert(url: &str, table: &str, rows: &Value) -> Result<Value, String> {
    let table = ident(table)?;
    let rows = crate::table::as_rows(rows);
    let arr = rows.as_array().ok_or("rows must be a list")?;
    let conn = open(url)?;
    let mut n = 0i64;
    for row in arr {
        let obj = row.as_object().ok_or("row must be a map")?;
        let mut cols = Vec::new();
        let mut placeholders = Vec::new();
        let mut vals = Vec::new();
        for (k, v) in obj {
            if k == "@" || k == "行" || k == "row" {
                continue;
            }
            let _ = ident(k)?;
            cols.push(format!("\"{k}\""));
            placeholders.push("?");
            vals.push(to_sql(v));
        }
        if cols.is_empty() {
            continue;
        }
        let sql = format!(
            "INSERT INTO \"{table}\" ({}) VALUES ({})",
            cols.join(", "),
            placeholders.join(", ")
        );
        conn.execute(&sql, params_from_iter(vals.iter().cloned()))
            .map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok(json!({ "ok": true, "inserted": n }))
}

fn skip_where_key(k: &str) -> bool {
    matches!(k, "@" | "行" | "row") || k.starts_with('_')
}

fn where_op(raw: &str) -> Result<&'static str, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "=" | "eq" | "==" | "等于" => Ok("="),
        "!=" | "<>" | "ne" | "不等于" => Ok("!="),
        ">" | "gt" | "大于" => Ok(">"),
        ">=" | "gte" | "大于等于" => Ok(">="),
        "<" | "lt" | "小于" => Ok("<"),
        "<=" | "lte" | "小于等于" => Ok("<="),
        "like" | "contains" | "包含" | "匹配" => Ok("LIKE"),
        other => Err(format!("unsupported where op `{other}`")),
    }
}

/// Parse simple filters: map of column→value (AND `=`), or rows
/// `{field|字段, op|操作|=, value|值}` / `{列, 值}`.
fn parse_where(where_v: Option<&Value>) -> Result<(Vec<String>, Vec<rusqlite::types::Value>), String> {
    let mut clauses = Vec::new();
    let mut vals = Vec::new();
    let Some(w) = where_v else {
        return Ok((clauses, vals));
    };
    match w {
        Value::Null => {}
        Value::Object(m) => {
            // Filter-row shape vs column→value map.
            let field = m
                .get("field")
                .or_else(|| m.get("字段"))
                .or_else(|| m.get("列"))
                .map(cell_str)
                .unwrap_or_default();
            if !field.is_empty()
                && (m.contains_key("value")
                    || m.contains_key("值")
                    || m.contains_key("op")
                    || m.contains_key("操作"))
            {
                let col = ident(&field)?;
                let op = where_op(
                    &m.get("op")
                        .or_else(|| m.get("操作"))
                        .map(cell_str)
                        .unwrap_or_default(),
                )?;
                let val = m
                    .get("value")
                    .or_else(|| m.get("值"))
                    .cloned()
                    .unwrap_or(Value::Null);
                clauses.push(format!("\"{col}\" {op} ?"));
                vals.push(to_sql(&val));
            } else {
                for (k, v) in m {
                    if skip_where_key(k) {
                        continue;
                    }
                    let col = ident(k)?;
                    clauses.push(format!("\"{col}\" = ?"));
                    vals.push(to_sql(v));
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                let (c, v) = parse_where(Some(item))?;
                clauses.extend(c);
                vals.extend(v);
            }
        }
        other => {
            return Err(format!(
                "where must be a map or filter table, got {}",
                match other {
                    Value::String(_) => "string",
                    Value::Number(_) => "number",
                    Value::Bool(_) => "bool",
                    _ => "value",
                }
            ));
        }
    }
    Ok((clauses, vals))
}

pub fn select(
    url: &str,
    table: &str,
    limit: i64,
    where_v: Option<&Value>,
) -> Result<Value, String> {
    select_order(url, table, limit, where_v, None)
}

/// Like `select`, with an optional `ORDER BY`.
///
/// `order` is a column name, optionally prefixed with `-` for descending
/// (`"created_at"`, `"-created_at"`). Comma-separated lists are allowed.
pub fn select_order(
    url: &str,
    table: &str,
    limit: i64,
    where_v: Option<&Value>,
    order: Option<&str>,
) -> Result<Value, String> {
    let table = ident(table)?;
    let (clauses, mut vals) = parse_where(where_v)?;
    let conn = open(url)?;
    let mut sql = format!("SELECT * FROM \"{table}\"");
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }
    if let Some(order) = order {
        let order = order.trim();
        if !order.is_empty() {
            let parts: Vec<&str> = order
                .split(',')
                .map(|p| p.trim())
                .filter(|p| !p.is_empty())
                .collect();
            let mut exprs = Vec::new();
            for part in parts {
                let (col, dir) = match part.strip_prefix('-') {
                    Some(col) => (col, " DESC"),
                    None => (part, ""),
                };
                exprs.push(format!("\"{}\"{}", ident(col)?, dir));
            }
            if !exprs.is_empty() {
                sql.push_str(" ORDER BY ");
                sql.push_str(&exprs.join(", "));
            }
        }
    }
    sql.push_str(&format!(" LIMIT ?{}", vals.len() + 1));
    vals.push(rusqlite::types::Value::Integer(limit));
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let names: Vec<String> = (0..stmt.column_count())
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();
    let mut rows = Vec::new();
    let mut rows_iter = stmt
        .query(params_from_iter(vals.iter().cloned()))
        .map_err(|e| e.to_string())?;
    while let Some(r) = rows_iter.next().map_err(|e| e.to_string())? {
        let mut m = Map::new();
        for (i, name) in names.iter().enumerate() {
            m.insert(name.clone(), from_sql(r.get_ref(i).map_err(|e| e.to_string())?));
        }
        rows.push(Value::Object(m));
    }
    Ok(json!({ "rows": rows }))
}

pub fn get(url: &str, table: &str, id: &str) -> Result<Value, String> {
    let table = ident(table)?;
    let conn = open(url)?;
    let sql = format!("SELECT * FROM \"{table}\" WHERE \"id\" = ?1 LIMIT 1");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let names: Vec<String> = (0..stmt.column_count())
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();
    let row = stmt
        .query_row(rusqlite::params![id], |r| {
            let mut m = Map::new();
            for (i, name) in names.iter().enumerate() {
                m.insert(name.clone(), from_sql(r.get_ref(i)?));
            }
            Ok(Value::Object(m))
        })
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(row.unwrap_or(Value::Null))
}

pub fn update(url: &str, table: &str, id: &str, row: &Value) -> Result<Value, String> {
    let table = ident(table)?;
    let obj = row.as_object().ok_or("row must be a map")?;
    let mut sets = Vec::new();
    let mut vals = Vec::new();
    for (k, v) in obj {
        if k == "id" {
            continue;
        }
        let _ = ident(k)?;
        sets.push(format!("\"{k}\" = ?"));
        vals.push(to_sql(v));
    }
    if sets.is_empty() {
        return Err("nothing to update".into());
    }
    vals.push(rusqlite::types::Value::Text(id.to_string()));
    let sql = format!(
        "UPDATE \"{table}\" SET {} WHERE \"id\" = ?",
        sets.join(", ")
    );
    let conn = open(url)?;
    let n = conn
        .execute(&sql, params_from_iter(vals.iter().cloned()))
        .map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "updated": n }))
}

pub fn delete(url: &str, table: &str, id: &str) -> Result<Value, String> {
    let table = ident(table)?;
    let conn = open(url)?;
    let sql = format!("DELETE FROM \"{table}\" WHERE \"id\" = ?1");
    let n = conn
        .execute(&sql, rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "deleted": n }))
}

pub fn exec(url: &str, sql: &str, args: Option<&Value>) -> Result<Value, String> {
    let conn = open(url)?;
    let vals: Vec<rusqlite::types::Value> = match args {
        Some(Value::Array(a)) => a.iter().map(to_sql).collect(),
        _ => Vec::new(),
    };
    let n = conn
        .execute(sql, params_from_iter(vals.iter().cloned()))
        .map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "changes": n }))
}

pub fn list_tables(url: &str) -> Result<Vec<String>, String> {
    let conn = open(url)?;
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_marqdo%' ORDER BY name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub sql_type: String,
    pub notnull: bool,
    pub pk: bool,
}

/// `PRAGMA table_info` → column metadata for admin forms.
pub fn table_info(url: &str, table: &str) -> Result<Vec<ColumnInfo>, String> {
    let table = ident(table)?;
    let conn = open(url)?;
    let sql = format!("PRAGMA table_info(\"{table}\")");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ColumnInfo {
                name: r.get::<_, String>(1)?,
                sql_type: r.get::<_, String>(2).unwrap_or_else(|_| "TEXT".into()),
                notnull: r.get::<_, i64>(3).unwrap_or(0) != 0,
                pk: r.get::<_, i64>(5).unwrap_or(0) != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}
