//! Dense real matrix kernels for L2 (`explicit` / solve / det / trace).

use serde_json::{json, Value};

use crate::matexpr::{Dim, Expr};

/// Max elements for `explicit` / dense ops (design default 10^4).
pub const MAX_ELEMENTS: usize = 10_000;

pub type Mat = Vec<Vec<f64>>;

pub fn check_elems(rows: usize, cols: usize) -> Result<(), String> {
    let n = rows.saturating_mul(cols);
    if n > MAX_ELEMENTS {
        return Err(format!(
            "dense matrix {rows}×{cols} has {n} elements > limit {MAX_ELEMENTS}; keep symbolic or reduce size"
        ));
    }
    Ok(())
}

pub fn eye(n: usize) -> Mat {
    let mut m = vec![vec![0.0; n]; n];
    for i in 0..n {
        m[i][i] = 1.0;
    }
    m
}

pub fn zeros(rows: usize, cols: usize) -> Mat {
    vec![vec![0.0; cols]; rows]
}

pub fn shape(m: &Mat) -> Result<(usize, usize), String> {
    if m.is_empty() {
        return Err("empty dense matrix".into());
    }
    let r = m.len();
    let c = m[0].len();
    if c == 0 || m.iter().any(|row| row.len() != c) {
        return Err("dense matrix rows must have equal non-zero length".into());
    }
    Ok((r, c))
}

pub fn to_dense_value(m: &Mat) -> Result<Value, String> {
    let (rows, cols) = shape(m)?;
    check_elems(rows, cols)?;
    Ok(json!({
        "_type": "linalg_dense",
        "rows": rows,
        "cols": cols,
        "data": m,
        "dtype": "real",
        "ascii": format_ascii(m),
        "latex": format_latex(m),
    }))
}

fn format_ascii(m: &Mat) -> String {
    let rows: Vec<String> = m
        .iter()
        .map(|row| {
            let cells: Vec<String> = row.iter().map(|x| format_num(*x)).collect();
            format!("[{}]", cells.join(","))
        })
        .collect();
    format!("[{}]", rows.join(","))
}

fn format_latex(m: &Mat) -> String {
    let body: Vec<String> = m
        .iter()
        .map(|row| {
            row.iter()
                .map(|x| format_num(*x))
                .collect::<Vec<_>>()
                .join(" & ")
        })
        .collect();
    format!(
        "\\begin{{bmatrix}}{}\\end{{bmatrix}}",
        body.join(" \\\\ ")
    )
}

fn format_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

pub fn from_value(v: &Value) -> Result<Mat, String> {
    if let Some(obj) = v.as_object() {
        if let Some(data) = obj.get("data") {
            return parse_data(data);
        }
        // linalg_expr kind=dense
        if obj.get("kind").and_then(|k| k.as_str()) == Some("dense") {
            if let Some(data) = obj.get("data") {
                return parse_data(data);
            }
        }
    }
    // Formula Matrix / raw nested list
    if v.is_array() {
        return parse_data(v);
    }
    Err("expected linalg_dense, dense expr, or nested list".into())
}

fn parse_data(v: &Value) -> Result<Mat, String> {
    let rows = v
        .as_array()
        .ok_or_else(|| "dense data must be list of rows".to_string())?;
    let mut out = Vec::new();
    let mut width = None;
    for row in rows {
        let cells = row
            .as_array()
            .ok_or_else(|| "dense row must be a list".to_string())?;
        let mut r = Vec::new();
        for c in cells {
            let n = c
                .as_f64()
                .or_else(|| c.as_i64().map(|i| i as f64))
                .or_else(|| c.as_u64().map(|u| u as f64))
                .or_else(|| c.as_str().and_then(|s| s.trim().parse().ok()))
                .ok_or_else(|| "dense cell must be a number".to_string())?;
            r.push(n);
        }
        if let Some(w) = width {
            if r.len() != w {
                return Err("dense matrix rows must have equal length".into());
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

fn dim_usize(d: &Dim) -> Result<usize, String> {
    match d {
        Dim::Int(n) => Ok(*n),
        Dim::Sym(s) => Err(format!(
            "cannot explicit symbolic dim `{s}`; bind concrete sizes or use dense leaves only"
        )),
    }
}

/// Evaluate MatExpr to dense; symbols are not allowed unless already dense leaves.
pub fn explicit(expr: &Expr) -> Result<Mat, String> {
    match expr {
        Expr::Dense { data } => {
            let (r, c) = shape(data)?;
            check_elems(r, c)?;
            Ok(data.clone())
        }
        Expr::Eye { n } => {
            let n = dim_usize(n)?;
            check_elems(n, n)?;
            Ok(eye(n))
        }
        Expr::Zero { rows, cols } => {
            let r = dim_usize(rows)?;
            let c = dim_usize(cols)?;
            check_elems(r, c)?;
            Ok(zeros(r, c))
        }
        Expr::Symbol { name, .. } => Err(format!(
            "cannot explicit unbound symbol `{name}`; substitute a dense matrix first"
        )),
        Expr::Mul { factors } => {
            if factors.is_empty() {
                return Err("empty MatMul".into());
            }
            let mut acc = explicit(&factors[0])?;
            for f in &factors[1..] {
                acc = matmul(&acc, &explicit(f)?)?;
            }
            Ok(acc)
        }
        Expr::Add { terms } => {
            if terms.is_empty() {
                return Err("empty MatAdd".into());
            }
            let mut acc = explicit(&terms[0])?;
            for t in &terms[1..] {
                acc = mat_add(&acc, &explicit(t)?)?;
            }
            Ok(acc)
        }
        Expr::Sub { left, right } => mat_sub(&explicit(left)?, &explicit(right)?),
        Expr::Transpose { arg } => Ok(transpose(&explicit(arg)?)),
        Expr::Inv { arg } => invert(&explicit(arg)?),
    }
}

pub fn matmul(a: &Mat, b: &Mat) -> Result<Mat, String> {
    let (ar, ac) = shape(a)?;
    let (br, bc) = shape(b)?;
    if ac != br {
        return Err(format!(
            "matmul shape mismatch: {ar}×{ac} · {br}×{bc}"
        ));
    }
    check_elems(ar, bc)?;
    let mut out = zeros(ar, bc);
    for i in 0..ar {
        for k in 0..ac {
            let aik = a[i][k];
            if aik == 0.0 {
                continue;
            }
            for j in 0..bc {
                out[i][j] += aik * b[k][j];
            }
        }
    }
    Ok(out)
}

pub fn mat_add(a: &Mat, b: &Mat) -> Result<Mat, String> {
    let (ar, ac) = shape(a)?;
    let (br, bc) = shape(b)?;
    if ar != br || ac != bc {
        return Err(format!("add shape mismatch: {ar}×{ac} vs {br}×{bc}"));
    }
    let mut out = zeros(ar, ac);
    for i in 0..ar {
        for j in 0..ac {
            out[i][j] = a[i][j] + b[i][j];
        }
    }
    Ok(out)
}

pub fn mat_sub(a: &Mat, b: &Mat) -> Result<Mat, String> {
    let (ar, ac) = shape(a)?;
    let (br, bc) = shape(b)?;
    if ar != br || ac != bc {
        return Err(format!("sub shape mismatch: {ar}×{ac} vs {br}×{bc}"));
    }
    let mut out = zeros(ar, ac);
    for i in 0..ar {
        for j in 0..ac {
            out[i][j] = a[i][j] - b[i][j];
        }
    }
    Ok(out)
}

pub fn transpose(m: &Mat) -> Mat {
    let (r, c) = shape(m).unwrap_or((0, 0));
    let mut out = zeros(c, r);
    for i in 0..r {
        for j in 0..c {
            out[j][i] = m[i][j];
        }
    }
    out
}

pub fn trace(m: &Mat) -> Result<f64, String> {
    let (r, c) = shape(m)?;
    if r != c {
        return Err(format!("trace needs square matrix, got {r}×{c}"));
    }
    Ok((0..r).map(|i| m[i][i]).sum())
}

pub fn det(m: &Mat) -> Result<f64, String> {
    let (n, c) = shape(m)?;
    if n != c {
        return Err(format!("det needs square matrix, got {n}×{c}"));
    }
    if n == 0 {
        return Ok(1.0);
    }
    if n == 1 {
        return Ok(m[0][0]);
    }
    if n == 2 {
        return Ok(m[0][0] * m[1][1] - m[0][1] * m[1][0]);
    }
    // Gaussian elimination with partial pivoting
    let mut a = m.clone();
    let mut det = 1.0;
    for k in 0..n {
        let mut piv = k;
        let mut best = a[k][k].abs();
        for i in (k + 1)..n {
            let v = a[i][k].abs();
            if v > best {
                best = v;
                piv = i;
            }
        }
        if best < 1e-15 {
            return Ok(0.0);
        }
        if piv != k {
            a.swap(k, piv);
            det = -det;
        }
        let akk = a[k][k];
        det *= akk;
        for i in (k + 1)..n {
            let f = a[i][k] / akk;
            for j in k..n {
                a[i][j] -= f * a[k][j];
            }
        }
    }
    Ok(det)
}

pub fn invert(m: &Mat) -> Result<Mat, String> {
    let (n, c) = shape(m)?;
    if n != c {
        return Err(format!("inverse needs square matrix, got {n}×{c}"));
    }
    check_elems(n, n)?;
    // Augment [A|I]
    let mut a = zeros(n, 2 * n);
    for i in 0..n {
        for j in 0..n {
            a[i][j] = m[i][j];
        }
        a[i][n + i] = 1.0;
    }
    for k in 0..n {
        let mut piv = k;
        let mut best = a[k][k].abs();
        for i in (k + 1)..n {
            let v = a[i][k].abs();
            if v > best {
                best = v;
                piv = i;
            }
        }
        if best < 1e-15 {
            return Err("matrix is singular (not invertible)".into());
        }
        if piv != k {
            a.swap(k, piv);
        }
        let akk = a[k][k];
        for j in 0..(2 * n) {
            a[k][j] /= akk;
        }
        for i in 0..n {
            if i == k {
                continue;
            }
            let f = a[i][k];
            for j in 0..(2 * n) {
                a[i][j] -= f * a[k][j];
            }
        }
    }
    let mut out = zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            out[i][j] = a[i][n + j];
        }
    }
    Ok(out)
}

/// Solve A x = b. `b` may be n×1 or n×k; returns n×k.
pub fn solve(a: &Mat, b: &Mat) -> Result<Mat, String> {
    let (n, ac) = shape(a)?;
    if n != ac {
        return Err(format!("solve needs square A, got {n}×{ac}"));
    }
    let (br, bc) = shape(b)?;
    if br != n {
        return Err(format!("solve rhs rows {br} != A rows {n}"));
    }
    // Augment [A|B]
    let mut m = zeros(n, n + bc);
    for i in 0..n {
        for j in 0..n {
            m[i][j] = a[i][j];
        }
        for j in 0..bc {
            m[i][n + j] = b[i][j];
        }
    }
    for k in 0..n {
        let mut piv = k;
        let mut best = m[k][k].abs();
        for i in (k + 1)..n {
            let v = m[i][k].abs();
            if v > best {
                best = v;
                piv = i;
            }
        }
        if best < 1e-15 {
            return Err("singular matrix in solve".into());
        }
        if piv != k {
            m.swap(k, piv);
        }
        let akk = m[k][k];
        for j in k..(n + bc) {
            m[k][j] /= akk;
        }
        for i in 0..n {
            if i == k {
                continue;
            }
            let f = m[i][k];
            for j in k..(n + bc) {
                m[i][j] -= f * m[k][j];
            }
        }
    }
    let mut x = zeros(n, bc);
    for i in 0..n {
        for j in 0..bc {
            x[i][j] = m[i][n + j];
        }
    }
    Ok(x)
}

pub fn solve_result(a: &Mat, b: &Mat) -> Result<Value, String> {
    let x = solve(a, b)?;
    let ax = matmul(a, &x)?;
    let resid = mat_sub(&ax, b)?;
    let mut max_r = 0.0_f64;
    for row in &resid {
        for &v in row {
            max_r = max_r.max(v.abs());
        }
    }
    Ok(json!({
        "_type": "linalg_solve",
        "method": "gauss",
        "x": to_dense_value(&x)?,
        "residual_max": max_r,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_2x2() {
        let a = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let b = vec![vec![5.0], vec![5.0]];
        let x = solve(&a, &b).unwrap();
        assert!((x[0][0] - 2.0).abs() < 1e-9);
        assert!((x[1][0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn det_2x2() {
        let m = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        assert!((det(&m).unwrap() + 2.0).abs() < 1e-9);
    }

    #[test]
    fn explicit_mul() {
        let a = Expr::Dense {
            data: vec![vec![1.0, 2.0], vec![0.0, 1.0]],
        };
        let b = Expr::Dense {
            data: vec![vec![1.0, 0.0], vec![1.0, 1.0]],
        };
        let p = Expr::Mul {
            factors: vec![a, b],
        };
        let m = explicit(&p).unwrap();
        assert_eq!(m, vec![vec![3.0, 2.0], vec![1.0, 1.0]]);
    }

    #[test]
    fn oversize_guard() {
        assert!(check_elems(101, 100).is_err());
    }
}
