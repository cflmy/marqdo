//! Postgres backend for `# db` (same author-facing API as SQLite).

use crate::driver::{db_kind, rewrite_placeholders_pg, DbKind};
use postgres::types::ToSql;
use postgres::{Client, NoTls, Row};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

static POOL: Mutex<Option<HashMap<String, Arc<Mutex<Client>>>>> = Mutex::new(None);
static TXNS: Mutex<Option<HashMap<String, (String, Arc<Mutex<Client>>)>>> = Mutex::new(None);

pub fn is_postgres(url: &str) -> bool {
    matches!(db_kind(url), DbKind::Postgres)
}

fn pooled(url: &str) -> Result<Arc<Mutex<Client>>, String> {
    let mut guard = POOL.lock().unwrap_or_else(|e| e.into_inner());
    let pool = guard.get_or_insert_with(HashMap::new);
    if let Some(c) = pool.get(url) {
        return Ok(c.clone());
    }
    let client = Client::connect(url, NoTls).map_err(|e| format!("postgres connect: {e}"))?;
    let arc = Arc::new(Mutex::new(client));
    pool.insert(url.to_string(), arc.clone());
    Ok(arc)
}

fn conn_for(url: &str, txn_id: Option<&str>) -> Result<Arc<Mutex<Client>>, String> {
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

fn next_txn_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("pg-txn-{}-{nanos}", std::process::id())
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

fn cell_str(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn pg_type(t: &str) -> &'static str {
    match t.trim().to_ascii_lowercase().as_str() {
        "int" | "integer" => "INTEGER",
        "real" | "float" | "double" => "DOUBLE PRECISION",
        "blob" | "bytea" => "BYTEA",
        "bool" | "boolean" => "BOOLEAN",
        "timestamp" | "datetime" | "timestamptz" | "审计" => "TIMESTAMPTZ",
        _ => "TEXT",
    }
}

fn to_pg_box(v: &Value) -> Box<dyn ToSql + Sync + Send> {
    match v {
        Value::Null => Box::new(None::<String>),
        Value::Bool(b) => Box::new(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Box::new(i)
            } else if let Some(f) = n.as_f64() {
                Box::new(f)
            } else {
                Box::new(n.to_string())
            }
        }
        Value::String(s) => Box::new(s.clone()),
        other => Box::new(other.to_string()),
    }
}

fn row_to_json(row: &Row) -> Value {
    let mut m = Map::new();
    for (i, col) in row.columns().iter().enumerate() {
        let name = col.name().to_string();
        let v: Value = if let Ok(x) = row.try_get::<_, Option<i64>>(i) {
            x.map(|n| json!(n)).unwrap_or(Value::Null)
        } else if let Ok(x) = row.try_get::<_, Option<f64>>(i) {
            x.map(|n| json!(n)).unwrap_or(Value::Null)
        } else if let Ok(x) = row.try_get::<_, Option<bool>>(i) {
            x.map(|b| json!(b)).unwrap_or(Value::Null)
        } else if let Ok(x) = row.try_get::<_, Option<String>>(i) {
            x.map(Value::String).unwrap_or(Value::Null)
        } else {
            Value::Null
        };
        m.insert(name, v);
    }
    Value::Object(m)
}

fn exec_sql(client: &mut Client, sql: &str, params: &[Value]) -> Result<u64, String> {
    let sql = rewrite_placeholders_pg(sql);
    let boxes: Vec<Box<dyn ToSql + Sync + Send>> = params.iter().map(to_pg_box).collect();
    let refs: Vec<&(dyn ToSql + Sync)> = boxes
        .iter()
        .map(|b| b.as_ref() as &(dyn ToSql + Sync))
        .collect();
    client
        .execute(sql.as_str(), refs.as_slice())
        .map_err(|e| e.to_string())
}

fn query_sql(client: &mut Client, sql: &str, params: &[Value]) -> Result<Vec<Value>, String> {
    let sql = rewrite_placeholders_pg(sql);
    let boxes: Vec<Box<dyn ToSql + Sync + Send>> = params.iter().map(to_pg_box).collect();
    let refs: Vec<&(dyn ToSql + Sync)> = boxes
        .iter()
        .map(|b| b.as_ref() as &(dyn ToSql + Sync))
        .collect();
    let rows = client
        .query(sql.as_str(), refs.as_slice())
        .map_err(|e| e.to_string())?;
    Ok(rows.iter().map(row_to_json).collect())
}

/// Equality-map / filter-table → `(AND exprs with ?, vals)`.
fn parse_where_simple(where_v: Option<&Value>) -> Result<(Vec<String>, Vec<Value>), String> {
    let Some(w) = where_v else {
        return Ok((vec![], vec![]));
    };
    let w = crate::table::as_rows(w);
    let mut exprs = Vec::new();
    let mut vals = Vec::new();
    match w {
        Value::Array(arr) => {
            for item in arr {
                let Value::Object(m) = item else {
                    continue;
                };
                let field = m
                    .get("field")
                    .or_else(|| m.get("字段"))
                    .or_else(|| m.get("列"))
                    .map(cell_str)
                    .unwrap_or_default();
                if !field.is_empty() {
                    let col = ident(&field)?;
                    let op = m
                        .get("op")
                        .or_else(|| m.get("操作"))
                        .map(cell_str)
                        .unwrap_or_else(|| "=".into());
                    let op = match op.trim().to_ascii_lowercase().as_str() {
                        "" | "=" | "eq" | "等于" => "=",
                        "!=" | "<>" | "ne" | "不等于" => "!=",
                        ">" | "gt" | "大于" => ">",
                        ">=" | "gte" => ">=",
                        "<" | "lt" | "小于" => "<",
                        "<=" | "lte" => "<=",
                        "like" | "包含" => "LIKE",
                        other => return Err(format!("unsupported where op `{other}`")),
                    };
                    let val = m
                        .get("value")
                        .or_else(|| m.get("值"))
                        .cloned()
                        .unwrap_or(Value::Null);
                    exprs.push(format!("\"{col}\" {op} ?"));
                    vals.push(val);
                } else {
                    for (k, v) in &m {
                        if k == "@" || k == "行" || k == "row" || k.starts_with('_') {
                            continue;
                        }
                        let col = ident(k)?;
                        exprs.push(format!("\"{col}\" = ?"));
                        vals.push(v.clone());
                    }
                }
            }
        }
        _ => {}
    }
    Ok((exprs, vals))
}

fn parse_fk(raw: &str) -> Result<(String, String), String> {
    let s = raw.trim();
    if s.is_empty() {
        return Err("empty foreign key".into());
    }
    if let Some((t, rest)) = s.split_once('(') {
        let col = rest.trim().trim_end_matches(')').trim();
        let t = ident(t)?.to_string();
        let col = if col.is_empty() {
            "id".into()
        } else {
            ident(col)?.to_string()
        };
        return Ok((t, col));
    }
    if let Some((t, c)) = s.split_once('.') {
        return Ok((ident(t)?.to_string(), ident(c)?.to_string()));
    }
    Ok((ident(s)?.to_string(), "id".into()))
}

fn utc_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let tod = secs % 86400;
    let h = tod / 3600;
    let m = (tod % 3600) / 60;
    let s = tod % 60;
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn column_names(url: &str, table: &str, txn_id: Option<&str>) -> Result<Vec<String>, String> {
    let table = ident(table)?;
    let conn = conn_for(url, txn_id)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let rows = query_sql(
        &mut c,
        "SELECT column_name FROM information_schema.columns WHERE table_name = ? ORDER BY ordinal_position",
        &[json!(table)],
    )?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            r.get("column_name")
                .or_else(|| r.get("COLUMN_NAME"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect())
}

pub fn init(url: &str, table: &str, fields: &Value) -> Result<Value, String> {
    let table = ident(table)?;
    let cols = crate::table::as_fields(fields);
    let arr = cols.as_array().ok_or("fields must be a list")?;
    if arr.is_empty() {
        return Err("empty fields".into());
    }
    let mut parts = Vec::new();
    let mut has_pk = false;
    let mut index_sql = Vec::new();
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
                Value::String(s) => {
                    Some(matches!(s.as_str(), "true" | "True" | "1" | "是" | "可"))
                }
                _ => None,
            })
            .unwrap_or(true);
        let unique = c
            .get("unique")
            .or_else(|| c.get("唯一"))
            .and_then(|v| match v {
                Value::Bool(b) => Some(*b),
                Value::String(s) => {
                    Some(matches!(s.as_str(), "true" | "True" | "1" | "yes" | "是" | "唯一"))
                }
                _ => None,
            })
            .unwrap_or(false);
        let index = c
            .get("index")
            .or_else(|| c.get("索引"))
            .and_then(|v| match v {
                Value::Bool(b) => Some(*b),
                Value::String(s) => {
                    Some(matches!(s.as_str(), "true" | "True" | "1" | "yes" | "是" | "索引"))
                }
                _ => None,
            })
            .unwrap_or(false);
        let part = if name == "id" && !has_pk && pg_type(&ty) == "INTEGER" {
            has_pk = true;
            format!("\"{name}\" SERIAL PRIMARY KEY")
        } else if name == "id" && !has_pk {
            has_pk = true;
            format!("\"{name}\" {} PRIMARY KEY", pg_type(&ty))
        } else {
            let mut col = format!("\"{name}\" {}", pg_type(&ty));
            if !nullable {
                col.push_str(" NOT NULL");
            }
            if unique {
                col.push_str(" UNIQUE");
            }
            let fk_raw = c
                .get("fk")
                .or_else(|| c.get("外键"))
                .map(cell_str)
                .unwrap_or_default();
            if !fk_raw.is_empty() {
                let (ref_table, ref_col) = parse_fk(&fk_raw)?;
                col.push_str(&format!(" REFERENCES \"{ref_table}\"(\"{ref_col}\")"));
            }
            col
        };
        parts.push(part);
        if (index || unique) && name != "id" {
            let kind = if unique { "UNIQUE " } else { "" };
            index_sql.push(format!(
                "CREATE {kind}INDEX IF NOT EXISTS \"idx_{table}_{name}\" ON \"{table}\" (\"{name}\")"
            ));
        }
    }
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS \"{table}\" ({})",
        parts.join(", ")
    );
    let conn = pooled(url)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    c.batch_execute(&sql).map_err(|e| e.to_string())?;
    for stmt in index_sql {
        let _ = c.batch_execute(&stmt);
    }
    Ok(json!({ "_type": "db_table", "name": table, "url": url }))
}

pub fn insert(url: &str, table: &str, rows: &Value, txn_id: Option<&str>) -> Result<Value, String> {
    let table = ident(table)?;
    let cols_present = column_names(url, table, txn_id).unwrap_or_default();
    let has_created = cols_present.iter().any(|c| c == "created_at");
    let has_updated = cols_present.iter().any(|c| c == "updated_at");
    let rows = crate::table::as_rows(rows);
    let arr = rows.as_array().ok_or("rows must be a list")?;
    let conn = conn_for(url, txn_id)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let mut n = 0i64;
    let now = utc_now_iso();
    for row in arr {
        let obj = row.as_object().ok_or("row must be a map")?;
        let mut cols = Vec::new();
        let mut ph = Vec::new();
        let mut vals = Vec::new();
        let mut seen_created = false;
        let mut seen_updated = false;
        for (k, v) in obj {
            if k == "@" || k == "行" || k == "row" {
                continue;
            }
            let _ = ident(k)?;
            if k == "created_at" {
                seen_created = true;
            }
            if k == "updated_at" {
                seen_updated = true;
            }
            cols.push(format!("\"{k}\""));
            ph.push("?");
            vals.push(v.clone());
        }
        if has_created && !seen_created {
            cols.push("\"created_at\"".into());
            ph.push("?");
            vals.push(json!(now.clone()));
        }
        if has_updated && !seen_updated {
            cols.push("\"updated_at\"".into());
            ph.push("?");
            vals.push(json!(now.clone()));
        }
        if cols.is_empty() {
            continue;
        }
        let sql = format!(
            "INSERT INTO \"{table}\" ({}) VALUES ({})",
            cols.join(", "),
            ph.join(", ")
        );
        exec_sql(&mut c, &sql, &vals)?;
        n += 1;
    }
    Ok(json!({ "ok": true, "inserted": n }))
}

pub fn select_order(
    url: &str,
    table: &str,
    limit: i64,
    where_v: Option<&Value>,
    order: Option<&str>,
    offset: Option<i64>,
    txn_id: Option<&str>,
) -> Result<Value, String> {
    let table = ident(table)?;
    let (exprs, mut vals) = parse_where_simple(where_v)?;
    let mut sql = format!("SELECT * FROM \"{table}\"");
    let where_sql = if exprs.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", exprs.join(" AND "))
    };
    sql.push_str(&where_sql);
    if let Some(order) = order.map(str::trim).filter(|s| !s.is_empty()) {
        let mut order_exprs = Vec::new();
        for part in order.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let (col, dir) = match part.strip_prefix('-') {
                Some(c) => (c, " DESC"),
                None => (part, ""),
            };
            order_exprs.push(format!("\"{}\"{}", ident(col)?, dir));
        }
        if !order_exprs.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&order_exprs.join(", "));
        }
    }
    sql.push_str(" LIMIT ?");
    vals.push(json!(limit));
    if let Some(off) = offset {
        if off > 0 {
            sql.push_str(" OFFSET ?");
            vals.push(json!(off));
        }
    }
    let conn = conn_for(url, txn_id)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let rows = query_sql(&mut c, &sql, &vals)?;
    if offset.is_some() {
        let mut count_sql = format!("SELECT COUNT(*) AS count FROM \"{table}\"");
        count_sql.push_str(&where_sql);
        let count_vals: Vec<Value> = vals
            .iter()
            .take(exprs.len())
            .cloned()
            .collect();
        let total = query_sql(&mut c, &count_sql, &count_vals)?
            .first()
            .and_then(|r| r.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(rows.len() as i64);
        Ok(json!({ "rows": rows, "total": total }))
    } else {
        Ok(json!({ "rows": rows }))
    }
}

pub fn get(url: &str, table: &str, id: &str, txn_id: Option<&str>) -> Result<Value, String> {
    let table = ident(table)?;
    let sql = format!("SELECT * FROM \"{table}\" WHERE \"id\" = ? LIMIT 1");
    let conn = conn_for(url, txn_id)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let rows = query_sql(&mut c, &sql, &[json!(id)])?;
    Ok(rows.into_iter().next().unwrap_or(Value::Null))
}

pub fn update(
    url: &str,
    table: &str,
    id: &str,
    row: &Value,
    txn_id: Option<&str>,
) -> Result<Value, String> {
    let table = ident(table)?;
    let cols_present = column_names(url, table, txn_id).unwrap_or_default();
    let has_updated = cols_present.iter().any(|c| c == "updated_at");
    let obj = row.as_object().ok_or("row must be a map")?;
    let mut sets = Vec::new();
    let mut vals = Vec::new();
    let mut seen_updated = false;
    for (k, v) in obj {
        if k == "id" || k == "@" || k == "行" || k == "row" {
            continue;
        }
        let _ = ident(k)?;
        if k == "updated_at" {
            seen_updated = true;
        }
        sets.push(format!("\"{k}\" = ?"));
        vals.push(v.clone());
    }
    if has_updated && !seen_updated {
        sets.push("\"updated_at\" = ?".into());
        vals.push(json!(utc_now_iso()));
    }
    if sets.is_empty() {
        return Ok(json!({ "ok": true, "updated": 0 }));
    }
    vals.push(json!(id));
    let sql = format!(
        "UPDATE \"{table}\" SET {} WHERE \"id\" = ?",
        sets.join(", ")
    );
    let conn = conn_for(url, txn_id)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let n = exec_sql(&mut c, &sql, &vals)?;
    Ok(json!({ "ok": true, "updated": n }))
}

pub fn delete(url: &str, table: &str, id: &str, txn_id: Option<&str>) -> Result<Value, String> {
    let table = ident(table)?;
    let sql = format!("DELETE FROM \"{table}\" WHERE \"id\" = ?");
    let conn = conn_for(url, txn_id)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let n = exec_sql(&mut c, &sql, &[json!(id)])?;
    Ok(json!({ "ok": true, "deleted": n }))
}

pub fn exec(url: &str, sql: &str, args: Option<&Value>, txn_id: Option<&str>) -> Result<Value, String> {
    let vals: Vec<Value> = match args {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let conn = conn_for(url, txn_id)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let n = exec_sql(&mut c, sql, &vals)?;
    Ok(json!({ "ok": true, "changes": n }))
}

pub fn query(url: &str, sql: &str, args: Option<&Value>, txn_id: Option<&str>) -> Result<Value, String> {
    let vals: Vec<Value> = match args {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let conn = conn_for(url, txn_id)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let rows = query_sql(&mut c, sql, &vals)?;
    Ok(json!({ "rows": rows, "count": rows.len() }))
}

pub fn count(
    url: &str,
    table: &str,
    where_v: Option<&Value>,
    txn_id: Option<&str>,
) -> Result<Value, String> {
    let table = ident(table)?;
    let (exprs, vals) = parse_where_simple(where_v)?;
    let mut sql = format!("SELECT COUNT(*) AS count FROM \"{table}\"");
    if !exprs.is_empty() {
        sql.push_str(&format!(" WHERE {}", exprs.join(" AND ")));
    }
    let conn = conn_for(url, txn_id)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let rows = query_sql(&mut c, &sql, &vals)?;
    let n = rows
        .first()
        .and_then(|r| r.get("count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    Ok(json!({ "count": n }))
}

pub fn begin(url: &str) -> Result<Value, String> {
    let conn = pooled(url)?;
    {
        let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
        c.batch_execute("BEGIN").map_err(|e| e.to_string())?;
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

pub fn commit(txn_id: &str) -> Result<Value, String> {
    let (_, conn) = take_txn(txn_id)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    c.batch_execute("COMMIT").map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

pub fn rollback(txn_id: &str) -> Result<Value, String> {
    let (_, conn) = take_txn(txn_id)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    c.batch_execute("ROLLBACK").map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

fn take_txn(txn_id: &str) -> Result<(String, Arc<Mutex<Client>>), String> {
    let mut guard = TXNS.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .as_mut()
        .and_then(|m| m.remove(txn_id))
        .ok_or_else(|| format!("unknown transaction `{txn_id}`"))
}

pub fn list_tables(url: &str) -> Result<Vec<String>, String> {
    let conn = pooled(url)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let rows = query_sql(
        &mut c,
        "SELECT tablename AS name FROM pg_catalog.pg_tables WHERE schemaname = 'public' ORDER BY tablename",
        &[],
    )?;
    Ok(rows
        .iter()
        .filter_map(|r| r.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect())
}

pub fn table_info(url: &str, table: &str) -> Result<Vec<crate::db::ColumnInfo>, String> {
    let table = ident(table)?;
    let conn = pooled(url)?;
    let mut c = conn.lock().unwrap_or_else(|e| e.into_inner());
    let rows = query_sql(
        &mut c,
        "SELECT column_name AS name, data_type AS sql_type, \
         CASE WHEN is_nullable = 'NO' THEN true ELSE false END AS notnull \
         FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = ? \
         ORDER BY ordinal_position",
        &[json!(table)],
    )?;
    Ok(rows
        .iter()
        .map(|r| crate::db::ColumnInfo {
            name: r.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
            sql_type: r
                .get("sql_type")
                .and_then(|v| v.as_str())
                .unwrap_or("text")
                .into(),
            notnull: r.get("notnull").and_then(|v| v.as_bool()).unwrap_or(false),
            pk: r.get("name").and_then(|v| v.as_str()) == Some("id"),
        })
        .collect())
}
