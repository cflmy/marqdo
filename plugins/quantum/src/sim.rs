//! Pure-state state-vector simulator (design §6).

use serde_json::{json, Map, Value};

pub const DEFAULT_MAX_QUBITS: usize = 12;
pub const HARD_MAX_QUBITS: usize = 16;

#[derive(Clone, Copy, Debug, Default)]
pub struct C {
    pub re: f64,
    pub im: f64,
}

impl C {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    pub fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }
    pub fn one() -> Self {
        Self { re: 1.0, im: 0.0 }
    }
    pub fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    pub fn mul(self, o: C) -> C {
        C {
            re: self.re * o.re - self.im * o.im,
            im: self.re * o.im + self.im * o.re,
        }
    }
    pub fn add(self, o: C) -> C {
        C {
            re: self.re + o.re,
            im: self.im + o.im,
        }
    }
    pub fn scale(self, s: f64) -> C {
        C {
            re: self.re * s,
            im: self.im * s,
        }
    }
    pub fn to_json(self) -> Value {
        json!({ "re": self.re, "im": self.im })
    }
}

pub fn max_qubits() -> usize {
    match std::env::var("MARQDO_QUANTUM_MAX_QUBITS") {
        Ok(s) => s
            .trim()
            .parse::<usize>()
            .ok()
            .map(|n| n.clamp(1, HARD_MAX_QUBITS))
            .unwrap_or(DEFAULT_MAX_QUBITS),
        Err(_) => DEFAULT_MAX_QUBITS,
    }
}

pub fn check_qubits(n: usize) -> Result<(), String> {
    let max = max_qubits();
    if n == 0 {
        return Err("qubits must be >= 1".into());
    }
    if n > max {
        return Err(format!(
            "qubits={n} exceeds max {max} (set MARQDO_QUANTUM_MAX_QUBITS to lower, hard cap {HARD_MAX_QUBITS})"
        ));
    }
    Ok(())
}

fn bit(i: usize, q: usize) -> bool {
    ((i >> q) & 1) == 1
}

fn flip(i: usize, q: usize) -> usize {
    i ^ (1 << q)
}

/// Apply 2×2 gate `[[a,b],[c,d]]` on qubit `q`.
pub fn apply_u2(amps: &mut [C], q: usize, a: C, b: C, c: C, d: C) {
    let n = amps.len();
    let step = 1 << q;
    let mut i = 0;
    while i < n {
        for j in 0..step {
            let i0 = i + j;
            let i1 = i0 + step;
            let v0 = amps[i0];
            let v1 = amps[i1];
            amps[i0] = a.mul(v0).add(b.mul(v1));
            amps[i1] = c.mul(v0).add(d.mul(v1));
        }
        i += 2 * step;
    }
}

pub fn apply_x(amps: &mut [C], q: usize) {
    apply_u2(
        amps,
        q,
        C::zero(),
        C::one(),
        C::one(),
        C::zero(),
    );
}

pub fn apply_h(amps: &mut [C], q: usize) {
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let t = C::new(s, 0.0);
    apply_u2(amps, q, t, t, t, t.scale(-1.0));
}

pub fn apply_i(_amps: &mut [C], _q: usize) {}

pub fn apply_cx(amps: &mut [C], control: usize, target: usize) {
    if control == target {
        return;
    }
    let n = amps.len();
    for i in 0..n {
        if bit(i, control) && !bit(i, target) {
            let j = flip(i, target);
            if i < j {
                amps.swap(i, j);
            }
        }
    }
}

pub fn zero_state(qubits: usize) -> Vec<C> {
    let dim = 1usize << qubits;
    let mut amps = vec![C::zero(); dim];
    amps[0] = C::one();
    amps
}

pub fn renormalize(amps: &mut [C]) {
    let n2: f64 = amps.iter().map(|c| c.norm_sq()).sum();
    if n2 <= 0.0 {
        return;
    }
    let inv = 1.0 / n2.sqrt();
    for a in amps.iter_mut() {
        *a = a.scale(inv);
    }
}

pub fn basis_label(i: usize, qubits: usize) -> String {
    // LSB = qubit 0 on the right
    format!("{i:0qubits$b}")
}

pub fn probabilities(amps: &[C], qubits: usize) -> Map<String, Value> {
    let mut m = Map::new();
    for (i, a) in amps.iter().enumerate() {
        let p = a.norm_sq();
        if p > 1e-15 {
            m.insert(basis_label(i, qubits), json!(p));
        }
    }
    m
}

pub fn amps_to_json(amps: &[C]) -> Value {
    Value::Array(amps.iter().map(|c| c.to_json()).collect())
}

#[derive(Clone, Debug)]
pub struct Op {
    pub gate: String,
    pub qubits: Vec<usize>,
    #[allow(dead_code)]
    pub theta: Option<f64>, // reserved for Rx/Ry/Rz (Q2)
}

pub fn parse_ops(circuit: &Value) -> Result<Vec<Op>, String> {
    let arr = circuit
        .get("ops")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for row in arr {
        let gate = row
            .get("gate")
            .or_else(|| row.get("门"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "op missing gate".to_string())?
            .to_ascii_uppercase();
        let qubits = match row.get("qubits").or_else(|| row.get("比特")) {
            Some(Value::Array(a)) => a
                .iter()
                .map(|v| {
                    v.as_u64()
                        .or_else(|| v.as_i64().map(|i| i as u64))
                        .ok_or_else(|| "bad qubit index".to_string())
                        .map(|u| u as usize)
                })
                .collect::<Result<Vec<_>, _>>()?,
            Some(Value::Number(n)) => {
                let u = n.as_u64().or_else(|| n.as_i64().map(|i| i as u64)).unwrap() as usize;
                vec![u]
            }
            Some(Value::String(s)) => parse_qubit_list(s)?,
            _ => {
                if let Some(q) = row
                    .get("qubit")
                    .or_else(|| row.get("比特"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
                {
                    vec![q as usize]
                } else if let (Some(c), Some(t)) = (
                    row.get("control")
                        .or_else(|| row.get("控制"))
                        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64))),
                    row.get("target")
                        .or_else(|| row.get("目标"))
                        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64))),
                ) {
                    vec![c as usize, t as usize]
                } else {
                    return Err(format!("op `{gate}` missing qubits"));
                }
            }
        };
        let theta = row
            .get("theta")
            .or_else(|| row.get("参数"))
            .and_then(|v| v.as_f64());
        out.push(Op { gate, qubits, theta });
    }
    Ok(out)
}

fn parse_qubit_list(s: &str) -> Result<Vec<usize>, String> {
    s.split(|c| c == ',' || c == ' ' || c == '，')
        .filter(|t| !t.is_empty())
        .map(|t| {
            t.trim()
                .parse::<usize>()
                .map_err(|_| format!("bad qubit `{t}`"))
        })
        .collect()
}

pub fn apply_op(amps: &mut [C], qubits: usize, op: &Op) -> Result<(), String> {
    for &q in &op.qubits {
        if q >= qubits {
            return Err(format!("qubit {q} out of range for n={qubits}"));
        }
    }
    match op.gate.as_str() {
        "I" | "ID" | "IDENTITY" => {
            let q = *op.qubits.first().ok_or("I needs qubit")?;
            apply_i(amps, q);
        }
        "X" | "NOT" => {
            let q = *op.qubits.first().ok_or("X needs qubit")?;
            apply_x(amps, q);
        }
        "H" | "HADAMARD" => {
            let q = *op.qubits.first().ok_or("H needs qubit")?;
            apply_h(amps, q);
        }
        "CX" | "CNOT" | "CN" => {
            if op.qubits.len() < 2 {
                return Err("CX needs control and target".into());
            }
            apply_cx(amps, op.qubits[0], op.qubits[1]);
        }
        other => return Err(format!("unsupported gate `{other}` in Q1 (use I/X/H/CX)")),
    }
    Ok(())
}

pub fn simulate_circuit(circuit: &Value) -> Result<(usize, Vec<C>), String> {
    let qubits = circuit
        .get("qubits")
        .or_else(|| circuit.get("比特数"))
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .ok_or_else(|| "circuit missing qubits".to_string())? as usize;
    check_qubits(qubits)?;
    let ops = parse_ops(circuit)?;
    let mut amps = zero_state(qubits);
    for op in &ops {
        apply_op(&mut amps, qubits, op)?;
    }
    renormalize(&mut amps);
    Ok((qubits, amps))
}

pub fn push_op(circuit: &Value, gate: &str, qubits: Vec<usize>, theta: Option<f64>) -> Result<Value, String> {
    let n = circuit
        .get("qubits")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .ok_or_else(|| "circuit missing qubits".to_string())? as usize;
    check_qubits(n)?;
    for &q in &qubits {
        if q >= n {
            return Err(format!("qubit {q} out of range for n={n}"));
        }
    }
    let mut obj = circuit.as_object().cloned().unwrap_or_default();
    let mut ops = obj
        .get("ops")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut op = json!({
        "gate": gate.to_ascii_uppercase(),
        "qubits": qubits,
    });
    if let Some(t) = theta {
        op.as_object_mut().unwrap().insert("theta".into(), json!(t));
    }
    ops.push(op);
    obj.insert("ops".into(), Value::Array(ops));
    Ok(Value::Object(obj))
}
