//! SQLite helpers for `# db` CRUD and W2 data-layer capabilities.
//!
//! - **连接池**：进程级 `Pool` 复用连接（WAL + busy_timeout + foreign_keys），
//!   避免每请求重连与并发写 `database is locked`。
//! - **事务**：`begin` 把连接从池中借出并独占，`commit` / `rollback` 后归还。
//! - **查询表达力**：where 支持 OR 组、`in` / `between` / `is null`。
//! - **分页**：`select` 支持 `offset`，返回 `{rows, total}`。

use rusqlite::{types::ValueRef, Connection, OptionalExtension, params_from_iter};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Process-wide pool: url → shared connection. Connections are reused across
/// calls instead of reopening per request.
static POOL: Mutex<Option<HashMap<String, Arc<Mutex<Connection>>>>> = Mutex::new(None);

/// Process-wide active transactions: txn id → (url, borrowed connection).
static TXNS: Mutex<Option<HashMap<String, (String, Arc<Mutex<Connection>>)>>> =
    Mutex::new(None);

/// Reset pools (used between listens / tests).
#[allow(dead_code)]
pub fn reset_pool() {
    let mut p = POOL.lock().unwrap_or_else(|e| e.into_inner());
    *p = Some(HashMap::new());
    let mut t = TXNS.lock().unwrap_or_else(|e| e.into_inner());
    *t = Some(HashMap::new());
}

fn next_txn_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    static COUNTER: Mutex<u64> = Mutex::new(0x243f_6a88_85a3_08d3);
    let mut c = COUNTER.lock().unwrap_or_else(|e| e.into_inner());
    let mut x = *c;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *c = x;
    format!("txn-{pid:x}{nanos:x}{x:x}")
}

/// Open (or reuse) a pooled connection. WAL + busy_timeout + foreign_keys are
/// applied once per physical connection.
fn pooled(url: &str) -> Result<Arc<Mutex<Connection>>, String> {
    let mut guard = POOL.lock().unwrap_or_else(|e| e.into_inner());
    let pool = guard.get_or_insert_with(HashMap::new);
    if let Some(c) = pool.get(url) {
        return Ok(c.clone());
    }
    let conn = open_conn(url)?;
    let arc = Arc::new(Mutex::new(conn));
    pool.insert(url.to_string(), arc.clone());
    Ok(arc)
}

/// Open a fresh physical connection to `url`, creating parent dirs as needed.
fn open_conn(url: &str) -> Result<Connection, String> {
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
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA busy_timeout = 5000;",
    )
    .map_err(|e| e.to_string())?;
    Ok(conn)
}

/// Borrow a connection for a single statement. If `txn_id` is set, the txn's
/// borrowed connection is used (transactional reads/writes); otherwise the pool.
fn conn_for<'a>(
    url: &str,
    txn_id: Option<&'a str>,
) -> Result<Arc<Mutex<Connection>>, String> {
    if let Some(tid) = txn_id {
        let guard = TXNS.lock().unwrap_or_else(|e| e.into_inner());
        match guard.as_ref().and_then(|m| m.get(tid)) {
            Some((_, c)) => Ok(c.clone()),
            None => Err(format!("unknown transaction `{tid}`")),
        }
    } else {
        pooled(url)
    }
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
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::init(url, table, fields);
    }
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
    let conn = pooled(url)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    c.execute_batch(&sql).map_err(|e| e.to_string())?;
    Ok(json!({ "_type": "db_table", "name": table, "url": url }))
}

pub fn insert(url: &str, table: &str, rows: &Value, txn_id: Option<&str>) -> Result<Value, String> {
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::insert(url, table, rows, txn_id);
    }
    let table = ident(table)?;
    let rows = crate::table::as_rows(rows);
    let arr = rows.as_array().ok_or("rows must be a list")?;
    let conn = conn_for(url, txn_id)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
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
        c.execute(&sql, params_from_iter(vals.iter().cloned()))
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
        "in" | "在" => Ok("IN"),
        "between" | "介于" | "在之间" => Ok("BETWEEN"),
        "is null" | "isnull" | "为空" | "空" => Ok("IS NULL"),
        other => Err(format!("unsupported where op `{other}`")),
    }
}

/// One parsed filter clause: column, op, bound values, and whether the clause
/// joins the previous one with `OR` (false ⇒ AND).
struct Clause {
    expr: String,
    vals: Vec<rusqlite::types::Value>,
    or: bool,
}

fn in_values(v: &Value) -> Vec<rusqlite::types::Value> {
    match v {
        Value::Array(a) => a.iter().map(to_sql).collect(),
        Value::String(s) => s
            .split(',')
            .map(|p| to_sql(&Value::String(p.trim().to_string())))
            .collect(),
        other => vec![to_sql(other)],
    }
}

fn between_values(v: &Value) -> Result<Vec<rusqlite::types::Value>, String> {
    match v {
        Value::Array(a) if a.len() == 2 => Ok(vec![to_sql(&a[0]), to_sql(&a[1])]),
        Value::String(s) => {
            let mut it = s.split(',');
            let lo = it.next().unwrap_or("").trim().to_string();
            let hi = it.next().unwrap_or("").trim().to_string();
            Ok(vec![
                to_sql(&Value::String(lo)),
                to_sql(&Value::String(hi)),
            ])
        }
        _ => Err("between expects two bounds (\"lo,hi\" or [lo,hi])".into()),
    }
}

/// Parse a filter table row into a `Clause`.
fn clause_from_row(m: &Map<String, Value>) -> Result<Option<Clause>, String> {
    let field = m
        .get("field")
        .or_else(|| m.get("字段"))
        .or_else(|| m.get("列"))
        .map(cell_str)
        .unwrap_or_default();
    if field.is_empty() {
        return Ok(None);
    }
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
    let or = m
        .get("or")
        .or_else(|| m.get("或"))
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            Value::String(s) => Some(matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true" | "是" | "or" | "或者")),
            _ => None,
        })
        .unwrap_or(false);

    let (expr, vals) = match op {
        "IS NULL" => (format!("\"{col}\" IS NULL"), vec![]),
        "IN" => {
            let vs = in_values(&val);
            let ph: Vec<&str> = vs.iter().map(|_| "?").collect();
            (
                format!("\"{col}\" IN ({})", ph.join(", ")),
                vs,
            )
        }
        "BETWEEN" => {
            let vs = between_values(&val)?;
            (format!("\"{col}\" BETWEEN ? AND ?"), vs)
        }
        _ => (format!("\"{col}\" {op} ?"), vec![to_sql(&val)]),
    };
    Ok(Some(Clause { expr, vals, or }))
}

/// Parse simple filters: map of column→value (AND `=`), or rows
/// `{field|字段, op|操作|=, value|值, or|或?}` / `{列, 值}`.
/// Supports `in` / `between` / `is null` and OR groups via the `或` column.
///
/// GFM tables reach the plugin in columnar form (`{字段:[…], 值:[…]}`), so the
/// value is first normalized via `as_rows` into a row list.
fn parse_where(where_v: Option<&Value>) -> Result<(Vec<String>, Vec<rusqlite::types::Value>), String> {
    let mut clauses: Vec<Clause> = Vec::new();
    let Some(w) = where_v else {
        return Ok((vec![], vec![]));
    };
    // Normalize GFM tables (columnar object) into row arrays; a single map
    // (column→value filter) becomes a one-element row list.
    let w = crate::table::as_rows(w);
    match &w {
        Value::Null => {}
        Value::Array(arr) => {
            for item in arr {
                match item {
                    Value::Object(m) => {
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
                            if let Some(cl) = clause_from_row(m)? {
                                clauses.push(cl);
                            }
                        } else {
                            for (k, v) in m {
                                if skip_where_key(k) {
                                    continue;
                                }
                                let col = ident(k)?;
                                clauses.push(Clause {
                                    expr: format!("\"{col}\" = ?"),
                                    vals: vec![to_sql(v)],
                                    or: false,
                                });
                            }
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
    // Assemble: AND by default, OR where the clause is marked.
    let mut exprs = Vec::new();
    let mut vals = Vec::new();
    let mut group: Vec<String> = Vec::new();
    for cl in clauses {
        if cl.or {
            group.push(cl.expr);
        } else {
            if !group.is_empty() {
                exprs.push(format!("({})", group.join(" OR ")));
                group.clear();
            }
            group.push(cl.expr);
        }
        vals.extend(cl.vals);
    }
    if !group.is_empty() {
        exprs.push(format!("({})", group.join(" OR ")));
    }
    Ok((exprs, vals))
}

pub fn select(
    url: &str,
    table: &str,
    limit: i64,
    where_v: Option<&Value>,
) -> Result<Value, String> {
    select_order(url, table, limit, where_v, None, None, None)
}

/// Like `select`, with an optional `ORDER BY`, `OFFSET` and a transaction.
///
/// - `order` is a column name, optionally prefixed with `-` for descending
///   (`"created_at"`, `"-created_at"`). Comma-separated lists are allowed.
/// - `offset` (页偏移/跳过) enables pagination: `LIMIT … OFFSET …`.
/// - When `offset` is provided, the result carries `total` (row count without
///   limit/offset) so pages can render `上一页 / 下一页`.
/// - `txn_id` routes the read through an open transaction (事务内一致性读).
pub fn select_order(
    url: &str,
    table: &str,
    limit: i64,
    where_v: Option<&Value>,
    order: Option<&str>,
    offset: Option<i64>,
    txn_id: Option<&str>,
) -> Result<Value, String> {
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::select_order(url, table, limit, where_v, order, offset, txn_id);
    }
    let table = ident(table)?;
    let (exprs, mut vals) = parse_where(where_v)?;
    let conn = conn_for(url, txn_id)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let mut sql = format!("SELECT * FROM \"{table}\"");
    let where_sql = if exprs.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", exprs.join(" AND "))
    };
    sql.push_str(&where_sql);
    if let Some(order) = order {
        let order = order.trim();
        if !order.is_empty() {
            let parts: Vec<&str> = order
                .split(',')
                .map(|p| p.trim())
                .filter(|p| !p.is_empty())
                .collect();
            let mut order_exprs = Vec::new();
            for part in parts {
                let (col, dir) = match part.strip_prefix('-') {
                    Some(col) => (col, " DESC"),
                    None => (part, ""),
                };
                order_exprs.push(format!("\"{}\"{}", ident(col)?, dir));
            }
            if !order_exprs.is_empty() {
                sql.push_str(" ORDER BY ");
                sql.push_str(&order_exprs.join(", "));
            }
        }
    }
    sql.push_str(&format!(" LIMIT ?{}", vals.len() + 1));
    vals.push(rusqlite::types::Value::Integer(limit));
    if let Some(off) = offset {
        if off > 0 {
            sql.push_str(&format!(" OFFSET ?{}", vals.len() + 1));
            vals.push(rusqlite::types::Value::Integer(off));
        }
    }
    let rows = query_rows(&c, &sql, &vals)?;

    // Total count (ignoring limit/offset) for pagination.
    let total = if offset.is_some() {
        let (_exprs2, where_vals) = parse_where(where_v)?;
        let n_where = where_vals.len();
        let count_sql = format!("SELECT COUNT(*) FROM \"{table}\"{where_sql}");
        count_rows(&c, &count_sql, &vals[..n_where]).unwrap_or(rows.len() as i64)
    } else {
        rows.len() as i64
    };

    if offset.is_some() {
        Ok(json!({ "rows": rows, "total": total }))
    } else {
        Ok(json!({ "rows": rows }))
    }
}

/// Run `SELECT …` and collect rows as a list of column→value maps.
fn query_rows(
    c: &Connection,
    sql: &str,
    vals: &[rusqlite::types::Value],
) -> Result<Vec<Value>, String> {
    let mut stmt = c.prepare(sql).map_err(|e| e.to_string())?;
    let names: Vec<String> = (0..stmt.column_count())
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();
    let mut rows_iter = stmt
        .query(params_from_iter(vals.iter().cloned()))
        .map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    while let Some(r) = rows_iter.next().map_err(|e| e.to_string())? {
        let mut m = Map::new();
        for (i, name) in names.iter().enumerate() {
            m.insert(name.clone(), from_sql(r.get_ref(i).map_err(|e| e.to_string())?));
        }
        rows.push(Value::Object(m));
    }
    Ok(rows)
}

/// Run `SELECT COUNT(*) …` and return the scalar.
fn count_rows(
    c: &Connection,
    sql: &str,
    vals: &[rusqlite::types::Value],
) -> Result<i64, String> {
    c.query_row(sql, params_from_iter(vals.iter().cloned()), |r| r.get(0))
        .map_err(|e| e.to_string())
}

pub fn get(url: &str, table: &str, id: &str, txn_id: Option<&str>) -> Result<Value, String> {
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::get(url, table, id, txn_id);
    }

    let table = ident(table)?;
    let conn = conn_for(url, txn_id)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let sql = format!("SELECT * FROM \"{table}\" WHERE \"id\" = ?1 LIMIT 1");
    let mut stmt = c.prepare(&sql).map_err(|e| e.to_string())?;
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

pub fn update(
    url: &str,
    table: &str,
    id: &str,
    row: &Value,
    txn_id: Option<&str>,
) -> Result<Value, String> {
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::update(url, table, id, row, txn_id);
    }

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
    let conn = conn_for(url, txn_id)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let n = c
        .execute(&sql, params_from_iter(vals.iter().cloned()))
        .map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "updated": n }))
}

pub fn delete(url: &str, table: &str, id: &str, txn_id: Option<&str>) -> Result<Value, String> {
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::delete(url, table, id, txn_id);
    }

    let table = ident(table)?;
    let conn = conn_for(url, txn_id)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let sql = format!("DELETE FROM \"{table}\" WHERE \"id\" = ?1");
    let n = c
        .execute(&sql, rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "deleted": n }))
}

pub fn exec(url: &str, sql: &str, args: Option<&Value>, txn_id: Option<&str>) -> Result<Value, String> {
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::exec(url, sql, args, txn_id);
    }

    let conn = conn_for(url, txn_id)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let vals: Vec<rusqlite::types::Value> = match args {
        Some(Value::Array(a)) => a.iter().map(to_sql).collect(),
        _ => Vec::new(),
    };
    let n = c
        .execute(sql, params_from_iter(vals.iter().cloned()))
        .map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "changes": n }))
}

/// Run a `SELECT …` and return the result set (count / join / group / subquery).
/// Bare SQL — the result carries `{ rows, count }`.
pub fn query(url: &str, sql: &str, args: Option<&Value>, txn_id: Option<&str>) -> Result<Value, String> {
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::query(url, sql, args, txn_id);
    }

    let conn = conn_for(url, txn_id)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let vals: Vec<rusqlite::types::Value> = match args {
        Some(Value::Array(a)) => a.iter().map(to_sql).collect(),
        _ => Vec::new(),
    };
    let rows = query_rows(&c, sql, &vals)?;
    Ok(json!({ "rows": rows, "count": rows.len() }))
}

/// Count rows matching a `where` filter (aggregation helper).
pub fn count(url: &str, table: &str, where_v: Option<&Value>, txn_id: Option<&str>) -> Result<Value, String> {
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::count(url, table, where_v, txn_id);
    }

    let table = ident(table)?;
    let (exprs, vals) = parse_where(where_v)?;
    let conn = conn_for(url, txn_id)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let mut sql = format!("SELECT COUNT(*) FROM \"{table}\"");
    if !exprs.is_empty() {
        sql.push_str(&format!(" WHERE {}", exprs.join(" AND ")));
    }
    let n = count_rows(&c, &sql, &vals)?;
    Ok(json!({ "count": n }))
}

/// Begin a transaction. Borrows the pooled connection exclusively and records
/// it under a fresh txn id; returns `{ txn, url }` for `commit` / `rollback`.
pub fn begin(url: &str) -> Result<Value, String> {
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::begin(url);
    }

    let conn = pooled(url)?;
    {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        c.execute_batch("BEGIN IMMEDIATE").map_err(|e| e.to_string())?;
    }
    let id = next_txn_id();
    {
        let mut guard = TXNS.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .get_or_insert_with(HashMap::new)
            .insert(id.clone(), (url.to_string(), conn));
    }
    Ok(json!({ "_type": "txn", "txn": id, "url": url, "事务": id, "地址": url }))
}

/// Commit a transaction and return its connection to the pool.
pub fn commit(txn_id: &str) -> Result<Value, String> {
    if txn_id.starts_with("pg-txn-") {
        return crate::db_pg::commit(txn_id);
    }

    let (_, conn) = take_txn(txn_id)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    c.execute_batch("COMMIT").map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

/// Roll back a transaction and return its connection to the pool.
pub fn rollback(txn_id: &str) -> Result<Value, String> {
    if txn_id.starts_with("pg-txn-") {
        return crate::db_pg::rollback(txn_id);
    }

    let (_, conn) = take_txn(txn_id)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    c.execute_batch("ROLLBACK").map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

/// Remove a txn from the registry, returning its connection for reuse.
fn take_txn(txn_id: &str) -> Result<(String, Arc<Mutex<Connection>>), String> {
    let mut guard = TXNS.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .as_mut()
        .and_then(|m| m.remove(txn_id))
        .ok_or_else(|| format!("unknown transaction `{txn_id}`"))
}

pub fn list_tables(url: &str) -> Result<Vec<String>, String> {
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::list_tables(url);
    }

    let conn = pooled(url)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = c
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
    if crate::db_pg::is_postgres(url) {
        return crate::db_pg::table_info(url, table);
    }

    let table = ident(table)?;
    let conn = pooled(url)?;
    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let sql = format!("PRAGMA table_info(\"{table}\")");
    let mut stmt = c.prepare(&sql).map_err(|e| e.to_string())?;
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
