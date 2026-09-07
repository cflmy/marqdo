//! Dense factorizations for L4: LU, QR, eig (symmetric Jacobi), SVD, Cholesky.

use serde_json::{json, Value};

use crate::dense::{self, Mat, MAX_ELEMENTS};

pub const FACTOR_MAX_SIDE: usize = 64;

fn check_side(n: usize, label: &str) -> Result<(), String> {
    if n > FACTOR_MAX_SIDE {
        return Err(format!(
            "{label} limited to side ≤ {FACTOR_MAX_SIDE} (got {n})"
        ));
    }
    Ok(())
}

fn check_mat(m: &Mat) -> Result<(usize, usize), String> {
    let (r, c) = dense::shape(m)?;
    dense::check_elems(r, c)?;
    Ok((r, c))
}

/// Factorize `kind` ∈ lu|qr|svd|eig|chol.
pub fn factorize(m: &Mat, kind: &str) -> Result<Value, String> {
    let kind = kind.trim().to_ascii_lowercase();
    match kind.as_str() {
        "lu" => lu_value(m),
        "qr" => qr_value(m),
        "eig" => eig_value(m),
        "svd" => svd_value(m),
        "chol" | "cholesky" => chol_value(m),
        other => Err(format!(
            "unknown factorize kind `{other}` (want lu|qr|svd|eig|chol)"
        )),
    }
}

fn lu_value(m: &Mat) -> Result<Value, String> {
    let (n, c) = check_mat(m)?;
    if n != c {
        return Err(format!("LU needs square matrix, got {n}×{c}"));
    }
    check_side(n, "LU")?;
    let (l, u, piv) = lu_decomp(m)?;
    Ok(json!({
        "_type": "linalg_factor",
        "kind": "lu",
        "L": dense::to_dense_value(&l)?,
        "U": dense::to_dense_value(&u)?,
        "pivots": piv,
        "latex": "A = LU",
        "ascii": "LU",
    }))
}

fn qr_value(m: &Mat) -> Result<Value, String> {
    let (rows, cols) = check_mat(m)?;
    check_side(rows.max(cols), "QR")?;
    let (q, r) = qr_decomp(m)?;
    Ok(json!({
        "_type": "linalg_factor",
        "kind": "qr",
        "Q": dense::to_dense_value(&q)?,
        "R": dense::to_dense_value(&r)?,
        "latex": "A = QR",
        "ascii": "QR",
    }))
}

fn eig_value(m: &Mat) -> Result<Value, String> {
    let (n, c) = check_mat(m)?;
    if n != c {
        return Err(format!("eig needs square matrix, got {n}×{c}"));
    }
    check_side(n, "eig")?;
    // Jacobi assumes symmetric; if nearly symmetric, symmetrize.
    let (evals, evecs) = jacobi_symm(m)?;
    let d = dense::zeros(n, n);
    let mut d = d;
    for i in 0..n {
        d[i][i] = evals[i];
    }
    Ok(json!({
        "_type": "linalg_factor",
        "kind": "eig",
        "eigenvalues": evals,
        "eigenvectors": dense::to_dense_value(&evecs)?,
        "D": dense::to_dense_value(&d)?,
        "latex": "A = P D P^{-1}",
        "ascii": "eig",
    }))
}

fn svd_value(m: &Mat) -> Result<Value, String> {
    let (rows, cols) = check_mat(m)?;
    check_side(rows.max(cols), "SVD")?;
    let (u, s, vt) = svd_decomp(m)?;
    Ok(json!({
        "_type": "linalg_factor",
        "kind": "svd",
        "U": dense::to_dense_value(&u)?,
        "S": s,
        "Vt": dense::to_dense_value(&vt)?,
        "latex": "A = U \\Sigma V^{\\top}",
        "ascii": "SVD",
    }))
}

fn chol_value(m: &Mat) -> Result<Value, String> {
    let (n, c) = check_mat(m)?;
    if n != c {
        return Err(format!("Cholesky needs square matrix, got {n}×{c}"));
    }
    check_side(n, "Cholesky")?;
    let l = cholesky(m)?;
    Ok(json!({
        "_type": "linalg_factor",
        "kind": "chol",
        "L": dense::to_dense_value(&l)?,
        "latex": "A = L L^{\\top}",
        "ascii": "chol",
    }))
}

/// PA = LU with partial pivoting; returns (L, U, pivot row order).
pub fn lu_decomp(a: &Mat) -> Result<(Mat, Mat, Vec<usize>), String> {
    let (n, _) = dense::shape(a)?;
    let mut m = a.clone();
    let mut piv: Vec<usize> = (0..n).collect();
    let mut l = dense::eye(n);
    for k in 0..n {
        let mut piv_row = k;
        let mut best = m[k][k].abs();
        for i in (k + 1)..n {
            let v = m[i][k].abs();
            if v > best {
                best = v;
                piv_row = i;
            }
        }
        if best < 1e-15 {
            return Err("LU failed: singular or near-singular matrix".into());
        }
        if piv_row != k {
            m.swap(k, piv_row);
            piv.swap(k, piv_row);
            for j in 0..k {
                let t = l[k][j];
                l[k][j] = l[piv_row][j];
                l[piv_row][j] = t;
            }
        }
        for i in (k + 1)..n {
            let f = m[i][k] / m[k][k];
            l[i][k] = f;
            m[i][k] = 0.0;
            for j in (k + 1)..n {
                m[i][j] -= f * m[k][j];
            }
        }
    }
    Ok((l, m, piv))
}

/// Thin QR via modified Gram-Schmidt. A is m×n with m≥n preferred.
pub fn qr_decomp(a: &Mat) -> Result<(Mat, Mat), String> {
    let (m, n) = dense::shape(a)?;
    let mut v = a.clone();
    let mut q = dense::zeros(m, n);
    let mut r = dense::zeros(n, n);
    for j in 0..n {
        // orthogonalize against previous
        for i in 0..j {
            let mut dot = 0.0;
            for k in 0..m {
                dot += q[k][i] * v[k][j];
            }
            r[i][j] = dot;
            for k in 0..m {
                v[k][j] -= dot * q[k][i];
            }
        }
        let mut norm = 0.0;
        for k in 0..m {
            norm += v[k][j] * v[k][j];
        }
        norm = norm.sqrt();
        if norm < 1e-15 {
            return Err("QR failed: linearly dependent columns".into());
        }
        r[j][j] = norm;
        for k in 0..m {
            q[k][j] = v[k][j] / norm;
        }
    }
    Ok((q, r))
}

/// Jacobi eigenvalue for symmetric (or symmetrized) real matrix.
/// Returns eigenvalues (descending) and eigenvector matrix (columns).
pub fn jacobi_symm(a: &Mat) -> Result<(Vec<f64>, Mat), String> {
    let (n, _) = dense::shape(a)?;
    // Symmetrize
    let mut s = dense::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            s[i][j] = 0.5 * (a[i][j] + a[j][i]);
        }
    }
    let mut v = dense::eye(n);
    for _ in 0..(n * n * 40).max(40) {
        // Find largest off-diagonal
        let mut p = 0usize;
        let mut q = 1usize;
        let mut best = 0.0;
        for i in 0..n {
            for j in (i + 1)..n {
                let v = s[i][j].abs();
                if v > best {
                    best = v;
                    p = i;
                    q = j;
                }
            }
        }
        if best < 1e-12 {
            break;
        }
        let app = s[p][p];
        let aqq = s[q][q];
        let apq = s[p][q];
        let tau = (aqq - app) / (2.0 * apq);
        let t = if tau >= 0.0 {
            1.0 / (tau + (1.0 + tau * tau).sqrt())
        } else {
            -1.0 / (-tau + (1.0 + tau * tau).sqrt())
        };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let sn = t * c;
        // Rotate S
        s[p][p] = app - t * apq;
        s[q][q] = aqq + t * apq;
        s[p][q] = 0.0;
        s[q][p] = 0.0;
        for i in 0..n {
            if i == p || i == q {
                continue;
            }
            let sip = s[i][p];
            let siq = s[i][q];
            s[i][p] = c * sip - sn * siq;
            s[p][i] = s[i][p];
            s[i][q] = c * siq + sn * sip;
            s[q][i] = s[i][q];
        }
        // Accumulate V
        for i in 0..n {
            let vip = v[i][p];
            let viq = v[i][q];
            v[i][p] = c * vip - sn * viq;
            v[i][q] = c * viq + sn * vip;
        }
    }
    let mut pairs: Vec<(f64, usize)> = (0..n).map(|i| (s[i][i], i)).collect();
    pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut evals = Vec::with_capacity(n);
    let mut evecs = dense::zeros(n, n);
    for (col, &(ev, idx)) in pairs.iter().enumerate() {
        evals.push(ev);
        for row in 0..n {
            evecs[row][col] = v[row][idx];
        }
    }
    Ok((evals, evecs))
}

/// Economy SVD via eig of AᵀA (n small).
pub fn svd_decomp(a: &Mat) -> Result<(Mat, Vec<f64>, Mat), String> {
    let (m, n) = dense::shape(a)?;
    // Gram = Aᵀ A (n×n)
    let at = dense::transpose(a);
    let gram = dense::matmul(&at, a)?;
    let (evals, v) = jacobi_symm(&gram)?;
    let mut svals = Vec::with_capacity(n.min(m));
    let rank = n.min(m);
    for i in 0..rank {
        let s = if evals[i] > 0.0 { evals[i].sqrt() } else { 0.0 };
        svals.push(s);
    }
    // Vt = Vᵀ
    let vt = dense::transpose(&v);
    // U = A V S^{+}
    let mut u = dense::zeros(m, rank);
    for j in 0..rank {
        if svals[j] < 1e-15 {
            continue;
        }
        // u_j = A v_j / s_j
        for i in 0..m {
            let mut sum = 0.0;
            for k in 0..n {
                sum += a[i][k] * v[k][j];
            }
            u[i][j] = sum / svals[j];
        }
    }
    // Truncate Vt to rank rows
    let mut vt_r = dense::zeros(rank, n);
    for i in 0..rank {
        for j in 0..n {
            vt_r[i][j] = vt[i][j];
        }
    }
    let _ = MAX_ELEMENTS;
    Ok((u, svals, vt_r))
}

pub fn cholesky(a: &Mat) -> Result<Mat, String> {
    let (n, _) = dense::shape(a)?;
    let mut l = dense::zeros(n, n);
    for i in 0..n {
        for j in 0..=i {
            let mut sum = 0.0;
            for k in 0..j {
                sum += l[i][k] * l[j][k];
            }
            if i == j {
                let v = a[i][i] - sum;
                if v <= 1e-15 {
                    return Err("Cholesky failed: matrix not positive definite".into());
                }
                l[i][j] = v.sqrt();
            } else {
                if l[j][j].abs() < 1e-15 {
                    return Err("Cholesky failed: zero pivot".into());
                }
                l[i][j] = (a[i][j] - sum) / l[j][j];
            }
        }
    }
    Ok(l)
}

/// Forward/back-sub with LU from `factorize kind=lu` (pivots applied to rhs).
pub fn solve_lu_factor(factor: &Value, b: &Mat) -> Result<Mat, String> {
    if factor.get("kind").and_then(|k| k.as_str()) != Some("lu") {
        return Err("solve(factor=…) needs kind=lu factor".into());
    }
    let l = dense::from_value(factor.get("L").ok_or_else(|| "factor missing L".to_string())?)?;
    let u = dense::from_value(factor.get("U").ok_or_else(|| "factor missing U".to_string())?)?;
    let piv = factor
        .get("pivots")
        .and_then(|p| p.as_array())
        .ok_or_else(|| "factor missing pivots".to_string())?;
    let (n, _) = dense::shape(&l)?;
    let (br, bc) = dense::shape(b)?;
    if br != n {
        return Err(format!("rhs rows {br} != factor size {n}"));
    }
    let mut pb = dense::zeros(n, bc);
    for (i, p) in piv.iter().enumerate() {
        let src = p
            .as_u64()
            .or_else(|| p.as_i64().map(|x| x as u64))
            .ok_or_else(|| "bad pivot".to_string())? as usize;
        if src >= n {
            return Err("pivot out of range".into());
        }
        for j in 0..bc {
            pb[i][j] = b[src][j];
        }
    }
    // Ly = Pb
    let mut y = dense::zeros(n, bc);
    for i in 0..n {
        for j in 0..bc {
            let mut s = pb[i][j];
            for k in 0..i {
                s -= l[i][k] * y[k][j];
            }
            y[i][j] = s / l[i][i];
        }
    }
    // Ux = y
    let mut x = dense::zeros(n, bc);
    for i in (0..n).rev() {
        for j in 0..bc {
            let mut s = y[i][j];
            for k in (i + 1)..n {
                s -= u[i][k] * x[k][j];
            }
            if u[i][i].abs() < 1e-15 {
                return Err("singular U in LU solve".into());
            }
            x[i][j] = s / u[i][i];
        }
    }
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eig_diag() {
        let m = vec![vec![2.0, 0.0], vec![0.0, 3.0]];
        let (ev, _) = jacobi_symm(&m).unwrap();
        assert!((ev[0] - 3.0).abs() < 1e-9);
        assert!((ev[1] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn qr_roundtrip() {
        let a = vec![vec![1.0, 1.0], vec![0.0, 1.0], vec![1.0, 0.0]];
        let (q, r) = qr_decomp(&a).unwrap();
        let recon = dense::matmul(&q, &r).unwrap();
        for i in 0..3 {
            for j in 0..2 {
                assert!((recon[i][j] - a[i][j]).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn lu_roundtrip_no_pivot_needed() {
        let a = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let (l, u, _) = lu_decomp(&a).unwrap();
        let recon = dense::matmul(&l, &u).unwrap();
        for i in 0..2 {
            for j in 0..2 {
                assert!((recon[i][j] - a[i][j]).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn lu_solve_reuse() {
        let a = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let b = vec![vec![5.0], vec![5.0]];
        let f = factorize(&a, "lu").unwrap();
        let x = solve_lu_factor(&f, &b).unwrap();
        assert!((x[0][0] - 2.0).abs() < 1e-9);
        assert!((x[1][0] - 1.0).abs() < 1e-9);
    }
}
