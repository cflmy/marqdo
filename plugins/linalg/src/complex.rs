//! Complex dense baseline (L6): parse/store + Frobenius norm.
//! Factorize / solve stay real-only for now.

use serde_json::{json, Value};

use crate::dense::MAX_ELEMENTS;

pub type CMat = Vec<Vec<(f64, f64)>>;

pub fn shape(m: &CMat) -> Result<(usize, usize), String> {
    if m.is_empty() {
        return Err("empty complex matrix".into());
    }
    let r = m.len();
    let c = m[0].len();
    if c == 0 || m.iter().any(|row| row.len() != c) {
        return Err("complex matrix rows must have equal non-zero length".into());
    }
    Ok((r, c))
}

fn check_elems(rows: usize, cols: usize) -> Result<(), String> {
    let n = rows.saturating_mul(cols);
    if n > MAX_ELEMENTS {
        return Err(format!(
            "complex matrix {rows}×{cols} has {n} elements > limit {MAX_ELEMENTS}"
        ));
    }
    Ok(())
}

fn parse_cell(c: &Value) -> Result<(f64, f64), String> {
    if let Some(n) = c
        .as_f64()
        .or_else(|| c.as_i64().map(|i| i as f64))
        .or_else(|| c.as_u64().map(|u| u as f64))
    {
        return Ok((n, 0.0));
    }
    if let Some(arr) = c.as_array() {
        if arr.len() == 2 {
            let re = arr[0]
                .as_f64()
                .or_else(|| arr[0].as_i64().map(|i| i as f64))
                .ok_or_else(|| "complex re must be number".to_string())?;
            let im = arr[1]
                .as_f64()
                .or_else(|| arr[1].as_i64().map(|i| i as f64))
                .ok_or_else(|| "complex im must be number".to_string())?;
            return Ok((re, im));
        }
    }
    if let Some(obj) = c.as_object() {
        let re = obj
            .get("re")
            .or_else(|| obj.get("real"))
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)))
            .ok_or_else(|| "complex cell needs re/real".to_string())?;
        let im = obj
            .get("im")
            .or_else(|| obj.get("imag"))
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)))
            .unwrap_or(0.0);
        return Ok((re, im));
    }
    Err("complex cell must be number, [re,im], or {re,im}".into())
}

fn cell_looks_complex(c: &Value) -> bool {
    if let Some(arr) = c.as_array() {
        return arr.len() == 2
            && arr.iter().all(|x| {
                x.is_number() || x.as_i64().is_some() || x.as_u64().is_some()
            });
    }
    c.as_object()
        .map(|o| o.contains_key("re") || o.contains_key("real") || o.contains_key("im"))
        .unwrap_or(false)
}

/// True if nested list contains any complex-looking cell.
pub fn data_has_complex(v: &Value) -> bool {
    let Some(rows) = v.as_array() else {
        return false;
    };
    rows.iter().any(|row| {
        row.as_array()
            .map(|cells| cells.iter().any(cell_looks_complex))
            .unwrap_or(false)
    })
}

pub fn from_data(v: &Value) -> Result<CMat, String> {
    let rows = v
        .as_array()
        .ok_or_else(|| "complex data must be list of rows".to_string())?;
    let mut out = Vec::new();
    let mut width = None;
    for row in rows {
        let cells = row
            .as_array()
            .ok_or_else(|| "complex row must be a list".to_string())?;
        let mut r = Vec::new();
        for c in cells {
            r.push(parse_cell(c)?);
        }
        if let Some(w) = width {
            if r.len() != w {
                return Err("complex matrix rows must have equal length".into());
            }
        } else {
            width = Some(r.len());
        }
        out.push(r);
    }
    let (rows, cols) = shape(&out)?;
    check_elems(rows, cols)?;
    Ok(out)
}

pub fn from_value(v: &Value) -> Result<CMat, String> {
    if let Some(obj) = v.as_object() {
        if obj.get("dtype").and_then(|d| d.as_str()) == Some("complex") {
            if let Some(data) = obj.get("data") {
                return from_data(data);
            }
        }
        if let Some(data) = obj.get("data") {
            if data_has_complex(data) {
                return from_data(data);
            }
        }
    }
    if v.is_array() && data_has_complex(v) {
        return from_data(v);
    }
    Err("expected linalg_dense dtype=complex or nested complex list".into())
}

fn format_c(z: (f64, f64)) -> String {
    let (re, im) = z;
    if im.abs() < 1e-15 {
        if re.fract() == 0.0 && re.abs() < 1e15 {
            format!("{}", re as i64)
        } else {
            format!("{re}")
        }
    } else if re.abs() < 1e-15 {
        format!("{im}i")
    } else {
        format!("{re}{im:+}i")
    }
}

pub fn to_dense_value(m: &CMat) -> Result<Value, String> {
    let (rows, cols) = shape(m)?;
    check_elems(rows, cols)?;
    let data: Vec<Vec<Value>> = m
        .iter()
        .map(|row| {
            row.iter()
                .map(|&(re, im)| json!([re, im]))
                .collect()
        })
        .collect();
    let ascii_rows: Vec<String> = m
        .iter()
        .map(|row| {
            let cells: Vec<String> = row.iter().map(|&z| format_c(z)).collect();
            format!("[{}]", cells.join(","))
        })
        .collect();
    Ok(json!({
        "_type": "matrix",
        "kind": "dense",
        "rows": rows,
        "cols": cols,
        "data": data,
        "dtype": "complex",
        "ascii": format!("[{}]", ascii_rows.join(",")),
        "latex": format!(
            "\\begin{{bmatrix}}{}\\end{{bmatrix}}",
            m.iter()
                .map(|row| {
                    row.iter()
                        .map(|&z| format_c(z))
                        .collect::<Vec<_>>()
                        .join(" & ")
                })
                .collect::<Vec<_>>()
                .join(" \\\\ ")
        ),
    }))
}

pub fn frobenius_norm(m: &CMat) -> Result<f64, String> {
    let _ = shape(m)?;
    let mut s = 0.0;
    for row in m {
        for &(re, im) in row {
            s += re * re + im * im;
        }
    }
    Ok(s.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fro_i() {
        let m = from_data(&json!([[[0.0, 1.0], [1.0, 0.0]], [[1.0, 0.0], [0.0, -1.0]]])).unwrap();
        assert!((frobenius_norm(&m).unwrap() - 2.0).abs() < 1e-12);
    }
}
