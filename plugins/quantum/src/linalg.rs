//! Q7a: density matrix, Kronecker, partial trace, Hermitian eig, Schmidt, Pauli expect.

use serde_json::{json, Map, Value};

use crate::sim::{
    amps_to_json, matrix_to_json, parse_matrix, simulate_circuit, C,
};

pub const DENSITY_MAX_QUBITS: usize = 6;

fn check_density_qubits(n: usize) -> Result<(), String> {
    if n > DENSITY_MAX_QUBITS {
        return Err(format!(
            "density-matrix ops limited to {DENSITY_MAX_QUBITS} qubits (got {n}); use state-vector APIs for larger n"
        ));
    }
    Ok(())
}

fn dim_of(n: usize) -> usize {
    1usize << n
}

/// Flat row-major dim×dim complex matrix.
pub type Mat = Vec<C>;

pub fn mat_get(m: &Mat, dim: usize, i: usize, j: usize) -> C {
    m[i * dim + j]
}

pub fn mat_set(m: &mut Mat, dim: usize, i: usize, j: usize, v: C) {
    m[i * dim + j] = v;
}

pub fn mat_zeros(dim: usize) -> Mat {
    vec![C::zero(); dim * dim]
}

pub fn nested_to_flat(m: &[Vec<C>]) -> Result<(usize, Mat), String> {
    if m.is_empty() {
        return Err("matrix is empty".into());
    }
    let dim = m.len();
    for (i, row) in m.iter().enumerate() {
        if row.len() != dim {
            return Err(format!("matrix row {i} length {} != {dim}", row.len()));
        }
    }
    let mut flat = Mat::with_capacity(dim * dim);
    for row in m {
        flat.extend_from_slice(row);
    }
    Ok((dim, flat))
}

pub fn flat_to_nested(m: &Mat, dim: usize) -> Vec<Vec<C>> {
    let mut out = Vec::with_capacity(dim);
    for i in 0..dim {
        out.push(m[i * dim..(i + 1) * dim].to_vec());
    }
    out
}

pub fn density_from_amps(amps: &[C], qubits: usize) -> Result<(usize, Mat), String> {
    check_density_qubits(qubits)?;
    let dim = dim_of(qubits);
    if amps.len() != dim {
        return Err(format!(
            "state dim {} != 2^{qubits}={dim}",
            amps.len()
        ));
    }
    let mut rho = mat_zeros(dim);
    for i in 0..dim {
        for j in 0..dim {
            // ρ_ij = ψ_i * conj(ψ_j)
            mat_set(&mut rho, dim, i, j, amps[i].mul(amps[j].conj()));
        }
    }
    Ok((dim, rho))
}

pub fn amps_from_state_or_circuit(v: &Value) -> Result<(usize, Vec<C>), String> {
    if v.get("_type").and_then(|t| t.as_str()) == Some("quantum_circuit")
        || (v.get("qubits").is_some() && v.get("ops").is_some())
    {
        let (n, amps) = simulate_circuit(v)?;
        return Ok((n, amps));
    }
    if v.get("_type").and_then(|t| t.as_str()) == Some("quantum_state")
        || v.get("amplitudes").is_some()
    {
        let qubits = v
            .get("qubits")
            .or_else(|| v.get("比特数"))
            .and_then(|x| x.as_u64().or_else(|| x.as_i64().map(|i| i as u64)))
            .ok_or_else(|| "state missing qubits".to_string())? as usize;
        let amps_v = v
            .get("amplitudes")
            .or_else(|| v.get("振幅"))
            .ok_or_else(|| "state missing amplitudes".to_string())?;
        let nested = match amps_v {
            Value::Array(a) => a
                .iter()
                .map(|c| match c {
                    Value::Object(o) => Ok(C::new(
                        o.get("re").and_then(|x| x.as_f64()).unwrap_or(0.0),
                        o.get("im").and_then(|x| x.as_f64()).unwrap_or(0.0),
                    )),
                    _ => Err("amplitude cell must be {re,im}".into()),
                })
                .collect::<Result<Vec<_>, String>>()?,
            _ => return Err("amplitudes must be a list".into()),
        };
        return Ok((qubits, nested));
    }
    Err("expected quantum_state or quantum_circuit".into())
}

pub fn density_handle(qubits: usize, rho: &Mat) -> Value {
    let dim = dim_of(qubits);
    json!({
        "_type": "density",
        "qubits": qubits,
        "dim": dim,
        "matrix": matrix_to_json(&flat_to_nested(rho, dim)),
    })
}

pub fn parse_density(v: &Value) -> Result<(usize, Mat), String> {
    if let Some(mat_v) = v.get("matrix").or_else(|| v.get("矩阵")) {
        let nested = parse_matrix(mat_v)?;
        let (dim, flat) = nested_to_flat(&nested)?;
        let qubits = (dim as f64).log2() as usize;
        if dim_of(qubits) != dim {
            return Err(format!("density dim {dim} is not a power of two"));
        }
        check_density_qubits(qubits)?;
        check_hermitian(&flat, dim)?;
        return Ok((qubits, flat));
    }
    let (n, amps) = amps_from_state_or_circuit(v)?;
    density_from_amps(&amps, n)
}

pub fn check_hermitian(m: &Mat, dim: usize) -> Result<(), String> {
    let mut err = 0.0;
    for i in 0..dim {
        for j in i..dim {
            let a = mat_get(m, dim, i, j);
            let b = mat_get(m, dim, j, i).conj();
            let d = a.sub(b);
            err += d.norm_sq();
        }
    }
    if err.sqrt() > 1e-8 {
        return Err(format!(
            "matrix is not Hermitian (‖ρ−ρ†‖_F ≈ {:.2e})",
            err.sqrt()
        ));
    }
    Ok(())
}

pub fn purity(m: &Mat, dim: usize) -> f64 {
    // Tr(ρ²) = Σ_ij |ρ_ij|² for Hermitian ρ? Actually Tr(ρ²)=Σ_ij ρ_ij ρ_ji = Σ_ij |ρ_ij|² when Hermitian.
    let mut s = 0.0;
    for i in 0..dim {
        for j in 0..dim {
            s += mat_get(m, dim, i, j).norm_sq();
        }
    }
    s
}

pub fn kronecker(a: &Mat, da: usize, b: &Mat, db: usize) -> Mat {
    let dim = da * db;
    let mut out = mat_zeros(dim);
    for i in 0..da {
        for j in 0..da {
            let aij = mat_get(a, da, i, j);
            for k in 0..db {
                for l in 0..db {
                    let bij = mat_get(b, db, k, l);
                    mat_set(
                        &mut out,
                        dim,
                        i * db + k,
                        j * db + l,
                        aij.mul(bij),
                    );
                }
            }
        }
    }
    out
}

pub fn kronecker_amps(a: &[C], b: &[C]) -> Vec<C> {
    let mut out = Vec::with_capacity(a.len() * b.len());
    for &ai in a {
        for &bj in b {
            out.push(ai.mul(bj));
        }
    }
    out
}

/// Keep listed qubits (LSB numbering). Trace out the rest.
pub fn partial_trace(rho: &Mat, qubits: usize, keep: &[usize]) -> Result<(usize, Mat), String> {
    check_density_qubits(qubits)?;
    let mut keep_sorted = keep.to_vec();
    keep_sorted.sort_unstable();
    keep_sorted.dedup();
    for &q in &keep_sorted {
        if q >= qubits {
            return Err(format!("keep qubit {q} out of range for n={qubits}"));
        }
    }
    let k = keep_sorted.len();
    check_density_qubits(k)?;
    let dim = dim_of(qubits);
    let kdim = dim_of(k);
    let mut out = mat_zeros(kdim);

    // Map full basis index -> kept subspace index
    let map_keep = |full: usize| -> usize {
        let mut idx = 0usize;
        for (bit_pos, &q) in keep_sorted.iter().enumerate() {
            if (full >> q) & 1 == 1 {
                idx |= 1 << bit_pos;
            }
        }
        idx
    };

    let traced: Vec<usize> = (0..qubits).filter(|q| !keep_sorted.contains(q)).collect();

    for i in 0..dim {
        for j in 0..dim {
            // Only contribute when traced bits of i and j match
            let mut same = true;
            for &q in &traced {
                if ((i >> q) & 1) != ((j >> q) & 1) {
                    same = false;
                    break;
                }
            }
            if !same {
                continue;
            }
            let ik = map_keep(i);
            let jk = map_keep(j);
            let v = mat_get(rho, dim, i, j);
            let cur = mat_get(&out, kdim, ik, jk);
            mat_set(&mut out, kdim, ik, jk, cur.add(v));
        }
    }
    Ok((k, out))
}

/// Jacobi eigenvalue algorithm for Hermitian matrices. Returns eigenvalues (desc) and eigenvectors as columns.
pub fn hermite_eig(m: &Mat, dim: usize) -> Result<(Vec<f64>, Vec<Vec<C>>), String> {
    check_hermitian(m, dim)?;
    let mut a = m.clone();
    // Force diagonal imaginary parts to 0 (Hermitian)
    for i in 0..dim {
        let d = mat_get(&a, dim, i, i);
        mat_set(&mut a, dim, i, i, C::new(d.re, 0.0));
    }
    let mut v = mat_zeros(dim);
    for i in 0..dim {
        mat_set(&mut v, dim, i, i, C::one());
    }

    let max_iter = 200 * dim * dim;
    for _ in 0..max_iter {
        let mut p = 0usize;
        let mut q = 1usize;
        let mut max_abs = 0.0;
        for i in 0..dim {
            for j in (i + 1)..dim {
                let mag = mat_get(&a, dim, i, j).norm_sq().sqrt();
                if mag > max_abs {
                    max_abs = mag;
                    p = i;
                    q = j;
                }
            }
        }
        if max_abs < 1e-14 {
            break;
        }

        let app = mat_get(&a, dim, p, p).re;
        let aqq = mat_get(&a, dim, q, q).re;
        let apq = mat_get(&a, dim, p, q);
        let abs_apq = apq.norm_sq().sqrt();
        let phase = apq.scale(1.0 / abs_apq); // e^{iα}

        // tan(2φ) = 2|apq| / (app - aqq)
        let diff = app - aqq;
        let (c, s_mag) = if abs_apq < 1e-30 {
            (1.0, 0.0)
        } else {
            let tau = diff / (2.0 * abs_apq);
            let t = if tau >= 0.0 {
                1.0 / (tau + (1.0 + tau * tau).sqrt())
            } else {
                -1.0 / (-tau + (1.0 + tau * tau).sqrt())
            };
            let cc = 1.0 / (1.0 + t * t).sqrt();
            (cc, t * cc)
        };
        let s = phase.scale(s_mag); // complex sine

        // Apply Jacobi rotation to A (similarity J† A J)
        let mut row_p = vec![C::zero(); dim];
        let mut row_q = vec![C::zero(); dim];
        for i in 0..dim {
            let aip = mat_get(&a, dim, i, p);
            let aiq = mat_get(&a, dim, i, q);
            row_p[i] = aip.scale(c).add(aiq.mul(s));
            row_q[i] = aiq.scale(c).sub(aip.mul(s.conj()));
        }
        // Diagonals: phase = apq/|apq| ⇒ Re(conj(phase)*apq) = |apq|
        let new_app = c * c * app + s_mag * s_mag * aqq + 2.0 * c * s_mag * abs_apq;
        let new_aqq = c * c * aqq + s_mag * s_mag * app - 2.0 * c * s_mag * abs_apq;

        for i in 0..dim {
            if i == p || i == q {
                continue;
            }
            mat_set(&mut a, dim, i, p, row_p[i]);
            mat_set(&mut a, dim, p, i, row_p[i].conj());
            mat_set(&mut a, dim, i, q, row_q[i]);
            mat_set(&mut a, dim, q, i, row_q[i].conj());
        }
        mat_set(&mut a, dim, p, p, C::new(new_app, 0.0));
        mat_set(&mut a, dim, q, q, C::new(new_aqq, 0.0));
        mat_set(&mut a, dim, p, q, C::zero());
        mat_set(&mut a, dim, q, p, C::zero());

        // V ← V J
        for i in 0..dim {
            let vip = mat_get(&v, dim, i, p);
            let viq = mat_get(&v, dim, i, q);
            mat_set(&mut v, dim, i, p, vip.scale(c).add(viq.mul(s)));
            mat_set(
                &mut v,
                dim,
                i,
                q,
                viq.scale(c).sub(vip.mul(s.conj())),
            );
        }
    }

    let mut evals: Vec<(f64, usize)> = (0..dim)
        .map(|i| (mat_get(&a, dim, i, i).re, i))
        .collect();
    evals.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut eigenvalues = Vec::with_capacity(dim);
    let mut eigenvectors = Vec::with_capacity(dim);
    for &(ev, idx) in &evals {
        eigenvalues.push(ev);
        let mut col = Vec::with_capacity(dim);
        for row in 0..dim {
            col.push(mat_get(&v, dim, row, idx));
        }
        let nrm = col.iter().map(|c| c.norm_sq()).sum::<f64>().sqrt();
        if nrm > 1e-15 {
            for c in &mut col {
                *c = c.scale(1.0 / nrm);
            }
        }
        eigenvectors.push(col);
    }
    Ok((eigenvalues, eigenvectors))
}

/// Thin SVD via eig of A†A for complex matrix rows×cols (rows = 2^cut, cols = 2^(n-cut)).
pub fn schmidt_decompose(amps: &[C], qubits: usize, cut: usize) -> Result<Value, String> {
    if cut == 0 || cut >= qubits {
        return Err(format!("schmidt cut must be in 1..{} (got {cut})", qubits - 1));
    }
    check_density_qubits(qubits)?;
    let da = dim_of(cut);
    let db = dim_of(qubits - cut);
    if amps.len() != da * db {
        return Err("state dimension mismatch for schmidt".into());
    }
    // Build Gram G = M M† (da×da) where M[ia, ib] = ψ[(ib << cut) | ia] (A = low bits)
    let mut gram = mat_zeros(da);
    for ia in 0..da {
        for ja in 0..da {
            let mut s = C::zero();
            for ib in 0..db {
                let i = (ib << cut) | ia;
                let j = (ib << cut) | ja;
                s = s.add(amps[i].mul(amps[j].conj()));
            }
            mat_set(&mut gram, da, ia, ja, s);
        }
    }
    let (evals, evecs) = hermite_eig(&gram, da)?;
    let coeffs: Vec<f64> = evals
        .iter()
        .map(|e| e.max(0.0).sqrt())
        .collect();
    // entropy
    let mut entropy = 0.0;
    for &c in &coeffs {
        let p = c * c;
        if p > 1e-15 {
            entropy -= p * p.ln();
        }
    }
    // Keep significant coeffs
    let ua: Vec<Value> = evecs
        .iter()
        .map(|col| Value::Array(col.iter().map(|c| c.to_json()).collect()))
        .collect();

    // For each singular vector u_k, v_k = M† u_k / σ_k
    let mut ub = Vec::new();
    for (k, sigma) in coeffs.iter().enumerate() {
        if *sigma < 1e-12 {
            ub.push(Value::Array(
                (0..db).map(|_| C::zero().to_json()).collect(),
            ));
            continue;
        }
        let u = &evecs[k];
        let mut vk = vec![C::zero(); db];
        for ib in 0..db {
            let mut s = C::zero();
            for ia in 0..da {
                let idx = (ib << cut) | ia;
                // (M† u)_ib = Σ_ia conj(M_ia,ib) u_ia = Σ conj(ψ) u
                s = s.add(amps[idx].conj().mul(u[ia]));
            }
            vk[ib] = s.scale(1.0 / sigma);
        }
        ub.push(Value::Array(vk.iter().map(|c| c.to_json()).collect()));
    }

    // Drop near-zero trailing for readability but keep all for determinism
    let _ = coeffs
        .iter()
        .position(|&c| c < 1e-10)
        .unwrap_or(coeffs.len());

    Ok(json!({
        "_type": "quantum_schmidt",
        "cut": cut,
        "qubits": qubits,
        "coeffs": coeffs,
        "ua": ua,
        "ub": ub,
        "entropy": entropy,
    }))
}

fn pauli_matrix(p: char) -> Result<[[C; 2]; 2], String> {
    match p.to_ascii_uppercase() {
        'I' => Ok([
            [C::one(), C::zero()],
            [C::zero(), C::one()],
        ]),
        'X' => Ok([
            [C::zero(), C::one()],
            [C::one(), C::zero()],
        ]),
        'Y' => Ok([
            [C::zero(), C::new(0.0, -1.0)],
            [C::new(0.0, 1.0), C::zero()],
        ]),
        'Z' => Ok([
            [C::one(), C::zero()],
            [C::zero(), C::new(-1.0, 0.0)],
        ]),
        other => Err(format!("unknown Pauli `{other}` (I/X/Y/Z)")),
    }
}

/// Pauli string: left = qubit n-1 (MSB), right = qubit 0 (LSB). Returns dim×dim.
pub fn pauli_string_matrix(s: &str, qubits: usize) -> Result<(usize, Mat), String> {
    let t = s.trim();
    if t.len() != qubits {
        return Err(format!(
            "Pauli string length {} != qubits {qubits} (left=high bit)",
            t.len()
        ));
    }
    let chars: Vec<char> = t.chars().collect();
    // chars[0] = qubit n-1
    let mut mat = {
        let p = pauli_matrix(chars[qubits - 1])?; // qubit 0 first in kron chain
        let mut m = mat_zeros(2);
        for i in 0..2 {
            for j in 0..2 {
                mat_set(&mut m, 2, i, j, p[i][j]);
            }
        }
        (2usize, m)
    };
    for q in 1..qubits {
        let ch = chars[qubits - 1 - q];
        let p = pauli_matrix(ch)?;
        let mut b = mat_zeros(2);
        for i in 0..2 {
            for j in 0..2 {
                mat_set(&mut b, 2, i, j, p[i][j]);
            }
        }
        // New = b ⊗ old  so that higher qubit is left in kron... 
        // Full basis: index bit q corresponds to qubit q.
        // Kronecker A⊗B means A on high subspace if we use (i<<logB)|j with j low.
        // We build from qubit0 (LSB) up: mat = P_{q} ⊗ mat
        let (da, a) = (2usize, b);
        let (db, ref bm) = mat;
        let out = kronecker(&a, da, bm, db);
        mat = (da * db, out);
    }
    Ok(mat)
}

pub fn expect_pauli(rho: &Mat, dim: usize, qubits: usize, obs: &str) -> Result<f64, String> {
    let (od, op) = pauli_string_matrix(obs, qubits)?;
    if od != dim {
        return Err("observable dimension mismatch".into());
    }
    // ⟨P⟩ = Tr(ρ P) = Σ_ij ρ_ij P_ji
    let mut s = C::zero();
    for i in 0..dim {
        for j in 0..dim {
            s = s.add(mat_get(rho, dim, i, j).mul(mat_get(&op, dim, j, i)));
        }
    }
    if s.im.abs() > 1e-8 {
        return Err(format!("expectation has imaginary part {}", s.im));
    }
    Ok(s.re)
}

pub fn expect_matrix(rho: &Mat, dim: usize, obs: &Mat, odim: usize) -> Result<f64, String> {
    if odim != dim {
        return Err("observable dimension mismatch".into());
    }
    let mut s = C::zero();
    for i in 0..dim {
        for j in 0..dim {
            s = s.add(mat_get(rho, dim, i, j).mul(mat_get(obs, dim, j, i)));
        }
    }
    Ok(s.re)
}

pub fn fidelity_pure(a: &[C], b: &[C]) -> Result<f64, String> {
    if a.len() != b.len() {
        return Err("fidelity: state dimensions differ".into());
    }
    let mut s = C::zero();
    for i in 0..a.len() {
        s = s.add(a[i].conj().mul(b[i]));
    }
    Ok(s.norm_sq())
}

pub fn all_pauli_labels(qubits: usize) -> Result<Vec<String>, String> {
    if qubits > 3 {
        return Err("paulivec default basis only for n≤3; pass explicit ops".into());
    }
    let letters = ['I', 'X', 'Y', 'Z'];
    let n = qubits;
    let total = 4usize.pow(n as u32);
    let mut out = Vec::with_capacity(total);
    for code in 0..total {
        let mut s = String::with_capacity(n);
        let mut c = code;
        // left = high qubit
        let mut chars = vec!['I'; n];
        for q in 0..n {
            chars[n - 1 - q] = letters[c % 4];
            c /= 4;
        }
        for ch in chars {
            s.push(ch);
        }
        out.push(s);
    }
    Ok(out)
}

pub fn spectrum_handle(evals: &[f64], evecs: &[Vec<C>]) -> Value {
    let eigenvectors: Vec<Value> = evecs
        .iter()
        .map(|col| Value::Array(col.iter().map(|c| c.to_json()).collect()))
        .collect();
    json!({
        "_type": "quantum_spectrum",
        "eigenvalues": evals,
        "eigenvectors": eigenvectors,
    })
}

pub fn value_as_matrix(v: &Value) -> Result<(usize, Mat), String> {
    if v.get("_type").and_then(|t| t.as_str()) == Some("quantum_density")
        || v.get("_type").and_then(|t| t.as_str()) == Some("density")
        || v.get("_type").and_then(|t| t.as_str()) == Some("密度")
        || v.get("matrix").is_some()
    {
        return parse_density(v);
    }
    if v.get("_type").and_then(|t| t.as_str()) == Some("quantum_gate") {
        let nested = crate::sim::gate_matrix_of(v)?;
        let (dim, flat) = nested_to_flat(&nested)?;
        return Ok(((dim as f64).log2() as usize, flat));
    }
    // nested list
    let nested = parse_matrix(v)?;
    let (dim, flat) = nested_to_flat(&nested)?;
    Ok(((dim as f64).log2() as usize, flat))
}

/// Helper for JSON amps list export (re-export style).
pub fn amps_json(amps: &[C]) -> Value {
    amps_to_json(amps)
}

pub fn density_map_insert_matrix(obj: &mut Map<String, Value>, rho: &Mat, dim: usize) {
    obj.insert(
        "matrix".into(),
        matrix_to_json(&flat_to_nested(rho, dim)),
    );
}
