//! Real dense metrics: lstsq, norm, cond, rank (L6).

use serde_json::{json, Value};

use crate::dense::{self, Mat};
use crate::factor;

/// Least squares `min ||A x - b||_2`. Tall: thin QR; wide/square: SVD pseudoinverse.
pub fn lstsq(a: &Mat, b: &Mat) -> Result<Value, String> {
    let (m, n) = dense::shape(a)?;
    let (br, bc) = dense::shape(b)?;
    if br != m {
        return Err(format!("lstsq: A is {m}×{n} but b has {br} rows"));
    }
    factor::check_side_limit(m.max(n), "lstsq")?;
    let (x, residual_fro, method) = if m >= n {
        lstsq_qr(a, b)?
    } else {
        lstsq_svd(a, b)?
    };
    let residual_fro = if bc == 1 {
        residual_fro
    } else {
        // recompute full residual for multi-RHS
        let ax = dense::matmul(a, &x)?;
        frobenius_diff(&ax, b)
    };
    Ok(json!({
        "_type": "linalg_lstsq",
        "method": method,
        "x": dense::to_dense_value(&x)?,
        "residual_fro": residual_fro,
    }))
}

fn lstsq_qr(a: &Mat, b: &Mat) -> Result<(Mat, f64, &'static str), String> {
    let (q, r) = factor::qr_decomp(a)?;
    let qt = dense::transpose(&q);
    let qtb = dense::matmul(&qt, b)?;
    let x = dense::solve(&r, &qtb)?;
    let ax = dense::matmul(a, &x)?;
    Ok((x, frobenius_diff(&ax, b), "qr"))
}

fn lstsq_svd(a: &Mat, b: &Mat) -> Result<(Mat, f64, &'static str), String> {
    let (m, n) = dense::shape(a)?;
    let (_, bc) = dense::shape(b)?;
    let (u, s, vt) = factor::svd_decomp(a)?;
    let rank = n.min(m);
    let mut s_inv = vec![0.0; rank];
    let smax = s.first().copied().unwrap_or(0.0);
    let tol = smax * (m.max(n) as f64) * f64::EPSILON * 100.0;
    for i in 0..rank {
        if s[i] > tol {
            s_inv[i] = 1.0 / s[i];
        }
    }
    // x = V Σ⁺ Uᵀ b ; U is m×rank, Vt is rank×n
    let ut = dense::transpose(&u);
    let utb = dense::matmul(&ut, b)?; // rank×bc
    let mut mid = dense::zeros(rank, bc);
    for i in 0..rank {
        for j in 0..bc {
            mid[i][j] = s_inv[i] * utb[i][j];
        }
    }
    let v = dense::transpose(&vt); // n×rank
    let x = dense::matmul(&v, &mid)?;
    let ax = dense::matmul(a, &x)?;
    Ok((x, frobenius_diff(&ax, b), "svd"))
}

fn frobenius_diff(a: &Mat, b: &Mat) -> f64 {
    let mut s = 0.0;
    for (ra, rb) in a.iter().zip(b.iter()) {
        for (&x, &y) in ra.iter().zip(rb.iter()) {
            let d = x - y;
            s += d * d;
        }
    }
    s.sqrt()
}

/// Matrix norm: `fro` (default), `1`, `inf`, `2`.
pub fn norm(m: &Mat, ord: &str) -> Result<f64, String> {
    let (rows, cols) = dense::shape(m)?;
    match ord.trim().to_ascii_lowercase().as_str() {
        "fro" | "frobenius" | "f" | "" => {
            let mut s = 0.0;
            for row in m {
                for &v in row {
                    s += v * v;
                }
            }
            Ok(s.sqrt())
        }
        "1" | "one" => {
            let mut best: f64 = 0.0;
            for j in 0..cols {
                let mut col = 0.0;
                for i in 0..rows {
                    col += m[i][j].abs();
                }
                best = best.max(col);
            }
            Ok(best)
        }
        "inf" | "infty" | "∞" => {
            let mut best: f64 = 0.0;
            for row in m {
                let mut r = 0.0;
                for &v in row {
                    r += v.abs();
                }
                best = best.max(r);
            }
            Ok(best)
        }
        "2" | "spectral" => {
            let (_, s, _) = factor::svd_decomp(m)?;
            Ok(s.first().copied().unwrap_or(0.0))
        }
        other => Err(format!(
            "unknown norm `{other}` (want fro|1|inf|2)"
        )),
    }
}

/// 2-norm condition number σ_max / σ_min (∞ if rank-deficient).
pub fn cond(m: &Mat) -> Result<Value, String> {
    let (rows, cols) = dense::shape(m)?;
    let (_, s, _) = factor::svd_decomp(m)?;
    let smax = s.first().copied().unwrap_or(0.0);
    let smin = s
        .iter()
        .copied()
        .filter(|&x| x > 0.0)
        .last()
        .unwrap_or(0.0);
    let tol = smax * (rows.max(cols) as f64) * f64::EPSILON * 100.0;
    let (value, infinite) = if smin <= tol {
        (f64::INFINITY, true)
    } else {
        (smax / smin, false)
    };
    Ok(json!({
        "_type": "linalg_cond",
        "ord": 2,
        "value": if infinite { Value::Null } else { json!(value) },
        "infinite": infinite,
        "sigma_max": smax,
        "sigma_min": smin,
    }))
}

/// Numerical rank via SVD.
pub fn rank(m: &Mat) -> Result<usize, String> {
    let (rows, cols) = dense::shape(m)?;
    let (_, s, _) = factor::svd_decomp(m)?;
    let smax = s.first().copied().unwrap_or(0.0);
    let tol = smax * (rows.max(cols) as f64) * f64::EPSILON * 100.0;
    Ok(s.iter().filter(|&&x| x > tol).count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lstsq_line_fit() {
        // points (0,1), (1,2), (2,2) → line ≈ 1.1667 + 0.5 t
        let a = vec![vec![1.0, 0.0], vec![1.0, 1.0], vec![1.0, 2.0]];
        let b = vec![vec![1.0], vec![2.0], vec![2.0]];
        let out = lstsq(&a, &b).unwrap();
        let x = dense::from_value(out.get("x").unwrap()).unwrap();
        assert!((x[0][0] - 7.0 / 6.0).abs() < 1e-9);
        assert!((x[1][0] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn fro_identity() {
        let i = dense::eye(3);
        assert!((norm(&i, "fro").unwrap() - 3f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn cond_diag() {
        let m = vec![vec![2.0, 0.0], vec![0.0, 1.0]];
        let c = cond(&m).unwrap();
        assert_eq!(c["infinite"], false);
        assert!((c["value"].as_f64().unwrap() - 2.0).abs() < 1e-9);
        assert_eq!(rank(&m).unwrap(), 2);
    }
}
