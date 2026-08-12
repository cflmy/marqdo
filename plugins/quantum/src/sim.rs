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

pub fn apply_y(amps: &mut [C], q: usize) {
    // [[0, -i], [i, 0]]
    apply_u2(
        amps,
        q,
        C::zero(),
        C::new(0.0, -1.0),
        C::new(0.0, 1.0),
        C::zero(),
    );
}

pub fn apply_z(amps: &mut [C], q: usize) {
    apply_u2(
        amps,
        q,
        C::one(),
        C::zero(),
        C::zero(),
        C::new(-1.0, 0.0),
    );
}

pub fn apply_s(amps: &mut [C], q: usize) {
    apply_u2(
        amps,
        q,
        C::one(),
        C::zero(),
        C::zero(),
        C::new(0.0, 1.0),
    );
}

pub fn apply_t(amps: &mut [C], q: usize) {
    let s = std::f64::consts::FRAC_1_SQRT_2;
    apply_u2(
        amps,
        q,
        C::one(),
        C::zero(),
        C::zero(),
        C::new(s, s),
    );
}

pub fn apply_rx(amps: &mut [C], q: usize, theta: f64) {
    let half = theta / 2.0;
    let c = half.cos();
    let s = half.sin();
    apply_u2(
        amps,
        q,
        C::new(c, 0.0),
        C::new(0.0, -s),
        C::new(0.0, -s),
        C::new(c, 0.0),
    );
}

pub fn apply_ry(amps: &mut [C], q: usize, theta: f64) {
    let half = theta / 2.0;
    let c = half.cos();
    let s = half.sin();
    apply_u2(
        amps,
        q,
        C::new(c, 0.0),
        C::new(-s, 0.0),
        C::new(s, 0.0),
        C::new(c, 0.0),
    );
}

pub fn apply_rz(amps: &mut [C], q: usize, theta: f64) {
    let half = theta / 2.0;
    apply_u2(
        amps,
        q,
        C::new((-half).cos(), (-half).sin()),
        C::zero(),
        C::zero(),
        C::new(half.cos(), half.sin()),
    );
}

pub fn apply_cz(amps: &mut [C], control: usize, target: usize) {
    if control == target {
        return;
    }
    for (i, a) in amps.iter_mut().enumerate() {
        if bit(i, control) && bit(i, target) {
            *a = a.scale(-1.0);
        }
    }
}

pub fn apply_swap(amps: &mut [C], a: usize, b: usize) {
    if a == b {
        return;
    }
    let n = amps.len();
    for i in 0..n {
        let j = if bit(i, a) != bit(i, b) {
            flip(flip(i, a), b)
        } else {
            i
        };
        if i < j {
            amps.swap(i, j);
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
    pub theta: Option<f64>,
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
            Some(Value::String(s)) => {
                let t = s.trim();
                if t.is_empty() {
                    vec![]
                } else {
                    parse_qubit_list(s)?
                }
            }
            Some(Value::Null) => vec![],
            None => {
                if is_meta_gate(&gate) {
                    vec![]
                } else if let Some(q) = row
                    .get("qubit")
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
            _ => return Err(format!("op `{gate}` bad qubits")),
        };
        let theta = row
            .get("theta")
            .or_else(|| row.get("参数"))
            .or_else(|| row.get("params"))
            .and_then(|v| parse_theta_value(v).ok().flatten());
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
        "BARRIER" | "MEASURE" | "M" => Ok(()),
        "I" | "ID" | "IDENTITY" => {
            let q = *op.qubits.first().ok_or("I needs qubit")?;
            apply_i(amps, q);
            Ok(())
        }
        "X" | "NOT" => {
            let q = *op.qubits.first().ok_or("X needs qubit")?;
            apply_x(amps, q);
            Ok(())
        }
        "Y" => {
            let q = *op.qubits.first().ok_or("Y needs qubit")?;
            apply_y(amps, q);
            Ok(())
        }
        "Z" => {
            let q = *op.qubits.first().ok_or("Z needs qubit")?;
            apply_z(amps, q);
            Ok(())
        }
        "S" => {
            let q = *op.qubits.first().ok_or("S needs qubit")?;
            apply_s(amps, q);
            Ok(())
        }
        "T" => {
            let q = *op.qubits.first().ok_or("T needs qubit")?;
            apply_t(amps, q);
            Ok(())
        }
        "H" | "HADAMARD" => {
            let q = *op.qubits.first().ok_or("H needs qubit")?;
            apply_h(amps, q);
            Ok(())
        }
        "RX" => {
            let q = *op.qubits.first().ok_or("Rx needs qubit")?;
            let th = op.theta.ok_or("Rx needs theta")?;
            apply_rx(amps, q, th);
            Ok(())
        }
        "RY" => {
            let q = *op.qubits.first().ok_or("Ry needs qubit")?;
            let th = op.theta.ok_or("Ry needs theta")?;
            apply_ry(amps, q, th);
            Ok(())
        }
        "RZ" => {
            let q = *op.qubits.first().ok_or("Rz needs qubit")?;
            let th = op.theta.ok_or("Rz needs theta")?;
            apply_rz(amps, q, th);
            Ok(())
        }
        "CX" | "CNOT" | "CN" => {
            if op.qubits.len() < 2 {
                return Err("CX needs control and target".into());
            }
            apply_cx(amps, op.qubits[0], op.qubits[1]);
            Ok(())
        }
        "CZ" => {
            if op.qubits.len() < 2 {
                return Err("CZ needs control and target".into());
            }
            apply_cz(amps, op.qubits[0], op.qubits[1]);
            Ok(())
        }
        "SWAP" => {
            if op.qubits.len() < 2 {
                return Err("SWAP needs two qubits".into());
            }
            apply_swap(amps, op.qubits[0], op.qubits[1]);
            Ok(())
        }
        other => Err(format!(
            "unsupported gate `{other}` (I/X/Y/Z/H/S/T/Rx/Ry/Rz/CX/CZ/SWAP/BARRIER/MEASURE)"
        )),
    }
}

pub fn is_meta_gate(gate: &str) -> bool {
    matches!(
        gate.to_ascii_uppercase().as_str(),
        "BARRIER" | "MEASURE" | "M"
    )
}

pub fn is_unitary_op(op: &Op) -> bool {
    !is_meta_gate(&op.gate)
}

#[derive(Clone, Debug)]
pub struct NoiseSpec {
    pub kind: String, // bitflip | depolarizing
    pub p: f64,
}

pub fn parse_noise(circuit: &Value) -> Result<Option<NoiseSpec>, String> {
    let kind = circuit
        .get("noise_kind")
        .or_else(|| circuit.get("噪声种类"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_ascii_lowercase());
    let p = circuit
        .get("noise_p")
        .or_else(|| circuit.get("噪声概率"))
        .and_then(|v| v.as_f64());
    match (kind, p) {
        (None, None) => Ok(None),
        (Some(k), Some(p)) => {
            if !(0.0..=1.0).contains(&p) {
                return Err("noise p must be in [0,1]".into());
            }
            let kind = match k.as_str() {
                "bitflip" | "bit_flip" | "比特翻转" => "bitflip".into(),
                "depolarizing" | "depolarise" | "退极化" => "depolarizing".into(),
                other => {
                    return Err(format!(
                        "unknown noise kind `{other}` (bitflip|depolarizing)"
                    ));
                }
            };
            Ok(Some(NoiseSpec { kind, p }))
        }
        _ => Err("noise needs both kind and p".into()),
    }
}

pub fn set_noise(circuit: &Value, kind: &str, p: f64) -> Result<Value, String> {
    if !(0.0..=1.0).contains(&p) {
        return Err("noise p must be in [0,1]".into());
    }
    let k = match kind.trim().to_ascii_lowercase().as_str() {
        "bitflip" | "bit_flip" | "比特翻转" => "bitflip",
        "depolarizing" | "depolarise" | "退极化" => "depolarizing",
        other => {
            return Err(format!(
                "unknown noise kind `{other}` (bitflip|depolarizing)"
            ));
        }
    };
    let mut obj = circuit.as_object().cloned().unwrap_or_default();
    obj.insert("noise_kind".into(), json!(k));
    obj.insert("noise_p".into(), json!(p));
    Ok(Value::Object(obj))
}

/// Qubits to read out: union of MEASURE ops; empty MEASURE list = all qubits; no MEASURE = all.
pub fn measure_targets(ops: &[Op], n: usize) -> Vec<usize> {
    let mut measured = Vec::new();
    let mut any = false;
    for op in ops {
        let g = op.gate.to_ascii_uppercase();
        if g == "MEASURE" || g == "M" {
            any = true;
            if op.qubits.is_empty() {
                return (0..n).collect();
            }
            for &q in &op.qubits {
                if !measured.contains(&q) {
                    measured.push(q);
                }
            }
        }
    }
    if !any {
        return (0..n).collect();
    }
    measured.sort_unstable();
    measured
}

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9e37_79b9_7f4a_7c15
            } else {
                seed
            },
        }
    }
    fn next_u64(&mut self) -> u64 {
        // SplitMix64
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }
}

fn apply_noise_on_qubits(
    amps: &mut [C],
    qs: &[usize],
    noise: &NoiseSpec,
    rng: &mut Rng,
) {
    if noise.p <= 0.0 {
        return;
    }
    for &q in qs {
        if rng.next_f64() >= noise.p {
            continue;
        }
        match noise.kind.as_str() {
            "bitflip" => apply_x(amps, q),
            "depolarizing" => match rng.next_u64() % 3 {
                0 => apply_x(amps, q),
                1 => apply_y(amps, q),
                _ => apply_z(amps, q),
            },
            _ => {}
        }
    }
}

fn sample_label(amps: &[C], n: usize, targets: &[usize], rng: &mut Rng) -> String {
    // Build CDF over full basis, then project bits onto targets (ascending, LSB = targets[0]).
    let mut cdf = Vec::with_capacity(amps.len());
    let mut acc = 0.0;
    for a in amps {
        acc += a.norm_sq();
        cdf.push(acc);
    }
    if acc <= 0.0 {
        return "0".repeat(targets.len().max(1));
    }
    let inv = 1.0 / acc;
    for x in &mut cdf {
        *x *= inv;
    }
    let r = rng.next_f64();
    let mut idx = 0usize;
    for (i, &c) in cdf.iter().enumerate() {
        if r <= c {
            idx = i;
            break;
        }
        idx = i;
    }
    if targets.len() == n {
        return basis_label(idx, n);
    }
    let mut bits = String::with_capacity(targets.len());
    // label: targets[0] is rightmost (LSB of the label string)
    for &q in targets.iter().rev() {
        bits.push(if bit(idx, q) { '1' } else { '0' });
    }
    bits
}

/// Sample computational-basis shots; returns counts map.
#[allow(dead_code)]
pub fn sample_counts(amps: &[C], qubits: usize, shots: usize, seed: u64) -> Map<String, Value> {
    let targets: Vec<usize> = (0..qubits).collect();
    let mut counts: Map<String, Value> = Map::new();
    let mut rng = Rng::new(seed);
    for _ in 0..shots {
        let label = sample_label(amps, qubits, &targets, &mut rng);
        let n = counts.get(&label).and_then(|v| v.as_u64()).unwrap_or(0) + 1;
        counts.insert(label, json!(n));
    }
    counts
}

pub fn run_circuit(circuit: &Value, shots: usize, seed: u64) -> Result<Value, String> {
    if shots == 0 {
        return Err("shots must be >= 1".into());
    }
    let qubits = circuit
        .get("qubits")
        .or_else(|| circuit.get("比特数"))
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .ok_or_else(|| "circuit missing qubits".to_string())? as usize;
    check_qubits(qubits)?;
    let ops = parse_ops(circuit)?;
    let noise = parse_noise(circuit)?;
    let targets = measure_targets(&ops, qubits);
    let mut counts: Map<String, Value> = Map::new();
    let mut rng = Rng::new(seed);

    for _ in 0..shots {
        let mut amps = zero_state(qubits);
        for op in &ops {
            if !is_unitary_op(op) {
                continue;
            }
            apply_op(&mut amps, qubits, op)?;
            if let Some(ref nspec) = noise {
                if nspec.p > 0.0 {
                    apply_noise_on_qubits(&mut amps, &op.qubits, nspec, &mut rng);
                }
            }
        }
        renormalize(&mut amps);
        let label = sample_label(&amps, qubits, &targets, &mut rng);
        let c = counts.get(&label).and_then(|v| v.as_u64()).unwrap_or(0) + 1;
        counts.insert(label, json!(c));
    }

    let mut out = json!({
        "qubits": qubits,
        "shots": shots,
        "seed": seed,
        "counts": counts,
        "measure": targets,
    });
    if let Some(nspec) = noise {
        out.as_object_mut().unwrap().insert(
            "noise".into(),
            json!({ "kind": nspec.kind, "p": nspec.p }),
        );
    }
    Ok(out)
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

/// Build a circuit from `qubits` + optional `steps=` table (column map or row list).
pub fn circuit_new(qubits: usize, steps: Option<&Value>) -> Result<Value, String> {
    check_qubits(qubits)?;
    let ops = match steps {
        None | Some(Value::Null) => Vec::new(),
        Some(v) => ops_from_steps(v)?,
    };
    for op in &ops {
        for &q in &op.qubits {
            if q >= qubits {
                return Err(format!("qubit {q} out of range for n={qubits}"));
            }
        }
    }
    let ops_json: Vec<Value> = ops
        .into_iter()
        .map(|op| {
            let mut m = json!({
                "gate": op.gate,
                "qubits": op.qubits,
            });
            if let Some(t) = op.theta {
                m.as_object_mut()
                    .unwrap()
                    .insert("theta".into(), json!(t));
            }
            m
        })
        .collect();
    Ok(json!({
        "qubits": qubits,
        "ops": ops_json,
    }))
}

/// Parse GFM `steps=` into ops.
///
/// Accepts:
/// - list of row maps (`@` / `行` / `row` tables)
/// - column-oriented map (`gate`/`门` + `qubits`/`比特` lists or scalars)
pub fn ops_from_steps(steps: &Value) -> Result<Vec<Op>, String> {
    match steps {
        Value::Array(rows) => {
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                out.push(op_from_step_row(row)?);
            }
            Ok(out)
        }
        Value::Object(map) => {
            let gate_col = map
                .get("gate")
                .or_else(|| map.get("门"))
                .ok_or_else(|| "steps table needs `gate` / `门` column".to_string())?;
            let qubit_col = map
                .get("qubits")
                .or_else(|| map.get("qubit"))
                .or_else(|| map.get("比特"))
                .ok_or_else(|| "steps table needs `qubits` / `比特` column".to_string())?;
            let theta_col = map
                .get("theta")
                .or_else(|| map.get("参数"))
                .or_else(|| map.get("params"));

            let gates = as_col_list(gate_col)?;
            let qubits = as_col_list(qubit_col)?;
            let thetas = match theta_col {
                Some(v) => as_col_list(v)?,
                None => vec![Value::Null; gates.len()],
            };
            if qubits.len() != gates.len() {
                return Err(format!(
                    "steps column length mismatch: gate={} qubits={}",
                    gates.len(),
                    qubits.len()
                ));
            }
            if thetas.len() != gates.len() && theta_col.is_some() {
                return Err("steps `theta`/`参数` column length mismatch".into());
            }
            let mut out = Vec::with_capacity(gates.len());
            for i in 0..gates.len() {
                let gate = gates[i]
                    .as_str()
                    .ok_or_else(|| format!("steps gate row {i} must be text"))?
                    .to_ascii_uppercase();
                let qs = qubits_from_cell(&qubits[i])?;
                let theta = thetas
                    .get(i)
                    .map(parse_theta_value)
                    .transpose()?
                    .flatten();
                out.push(Op {
                    gate,
                    qubits: qs,
                    theta,
                });
            }
            Ok(out)
        }
        _ => Err("steps must be a table (map or list of rows)".into()),
    }
}

fn as_col_list(v: &Value) -> Result<Vec<Value>, String> {
    match v {
        Value::Array(a) => Ok(a.clone()),
        other => Ok(vec![other.clone()]),
    }
}

fn op_from_step_row(row: &Value) -> Result<Op, String> {
    let gate = row
        .get("gate")
        .or_else(|| row.get("门"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "step row missing `gate` / `门`".to_string())?
        .to_ascii_uppercase();
    let qubits = match row
        .get("qubits")
        .or_else(|| row.get("qubit"))
        .or_else(|| row.get("比特"))
    {
        None if is_meta_gate(&gate) => vec![],
        None => {
            return Err("step row missing `qubits` / `比特`".into());
        }
        Some(v) => qubits_from_cell(v)?,
    };
    let theta = row
        .get("theta")
        .or_else(|| row.get("参数"))
        .or_else(|| row.get("params"))
        .map(parse_theta_value)
        .transpose()?
        .flatten();
    Ok(Op {
        gate,
        qubits,
        theta,
    })
}

fn qubits_from_cell(v: &Value) -> Result<Vec<usize>, String> {
    match v {
        Value::Null => Ok(vec![]),
        Value::Array(a) => a
            .iter()
            .map(|x| {
                x.as_u64()
                    .or_else(|| x.as_i64().map(|i| i as u64))
                    .ok_or_else(|| "bad qubit index".to_string())
                    .map(|u| u as usize)
            })
            .collect(),
        Value::Number(n) => {
            let u = n
                .as_u64()
                .or_else(|| n.as_i64().map(|i| i as u64))
                .ok_or_else(|| "bad qubit index".to_string())? as usize;
            Ok(vec![u])
        }
        Value::String(s) => {
            if s.trim().is_empty() {
                Ok(vec![])
            } else {
                parse_qubit_list(s)
            }
        }
        _ => Err("bad qubits cell".into()),
    }
}

fn parse_theta_value(v: &Value) -> Result<Option<f64>, String> {
    match v {
        Value::Null => Ok(None),
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| "bad theta".to_string())
            .map(Some),
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() {
                return Ok(None);
            }
            parse_theta_str(t).map(Some)
        }
        _ => Err("bad theta".into()),
    }
}

fn parse_theta_str(s: &str) -> Result<f64, String> {
    let t = s.trim().to_ascii_lowercase().replace('π', "pi");
    if t == "pi" {
        return Ok(std::f64::consts::PI);
    }
    if let Some(rest) = t.strip_prefix("pi/") {
        let d: f64 = rest
            .trim()
            .parse()
            .map_err(|_| format!("bad theta `{s}`"))?;
        if d == 0.0 {
            return Err(format!("bad theta `{s}`"));
        }
        return Ok(std::f64::consts::PI / d);
    }
    if let Some(rest) = t.strip_suffix("*pi") {
        let n: f64 = rest
            .trim()
            .parse()
            .map_err(|_| format!("bad theta `{s}`"))?;
        return Ok(n * std::f64::consts::PI);
    }
    t.parse::<f64>()
        .map_err(|_| format!("bad theta `{s}`"))
}

pub fn push_op(circuit: &Value, gate: &str, qubits: Vec<usize>, theta: Option<f64>) -> Result<Value, String> {
    let n = circuit
        .get("qubits")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .ok_or_else(|| "circuit missing qubits".to_string())? as usize;
    check_qubits(n)?;
    let g = gate.to_ascii_uppercase();
    if !is_meta_gate(&g) || !qubits.is_empty() {
        for &q in &qubits {
            if q >= n {
                return Err(format!("qubit {q} out of range for n={n}"));
            }
        }
    }
    if is_meta_gate(&g) {
        // ok with empty qubits (barrier / measure-all)
    } else if qubits.is_empty() {
        return Err(format!("gate `{g}` needs qubits"));
    }
    let mut obj = circuit.as_object().cloned().unwrap_or_default();
    let mut ops = obj
        .get("ops")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut op = json!({
        "gate": g,
        "qubits": qubits,
    });
    if let Some(t) = theta {
        op.as_object_mut().unwrap().insert("theta".into(), json!(t));
    }
    ops.push(op);
    obj.insert("ops".into(), Value::Array(ops));
    Ok(Value::Object(obj))
}

/// Append another circuit's ops, a single op map, or reject bare gate handles without qubits.
pub fn append(circuit: &Value, op: &Value) -> Result<Value, String> {
    let n = circuit
        .get("qubits")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .ok_or_else(|| "circuit missing qubits".to_string())? as usize;
    let mut obj = circuit.as_object().cloned().unwrap_or_default();
    let mut ops = obj
        .get("ops")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if let Some(arr) = op.get("ops").and_then(|v| v.as_array()) {
        let other_n = op
            .get("qubits")
            .and_then(|v| v.as_u64())
            .unwrap_or(n as u64) as usize;
        if other_n > n {
            return Err(format!(
                "append: other circuit qubits={other_n} exceeds this n={n}"
            ));
        }
        for row in arr {
            ops.push(row.clone());
        }
    } else if op.get("gate").or_else(|| op.get("门")).is_some() {
        let parsed = parse_ops(&json!({ "ops": [op] }))?;
        for p in parsed {
            for &q in &p.qubits {
                if q >= n {
                    return Err(format!("append qubit {q} out of range for n={n}"));
                }
            }
            let mut row = json!({ "gate": p.gate, "qubits": p.qubits });
            if let Some(t) = p.theta {
                row.as_object_mut()
                    .unwrap()
                    .insert("theta".into(), json!(t));
            }
            ops.push(row);
        }
    } else if op.get("name").or_else(|| op.get("名")).is_some() {
        return Err("append gate handle needs an op with qubits; pass a circuit or {{gate,qubits}}".into());
    } else {
        return Err("append needs a circuit (ops) or op {{gate,qubits}}".into());
    }

    obj.insert("ops".into(), Value::Array(ops));
    Ok(Value::Object(obj))
}

/// Bloch vector (x,y,z) for reduced density of `qubit` (LSB = qubit 0).
pub fn bloch_vector(amps: &[C], qubits: usize, qubit: usize) -> Result<(f64, f64, f64), String> {
    if qubit >= qubits {
        return Err(format!("bloch qubit {qubit} out of range for n={qubits}"));
    }
    let mut rho00 = C::zero();
    let mut rho01 = C::zero();
    let mut rho11 = C::zero();
    let dim = amps.len();
    let mask = 1usize << qubit;
    for i in 0..dim {
        if i & mask != 0 {
            continue;
        }
        let i0 = i;
        let i1 = i | mask;
        let a0 = amps[i0];
        let a1 = amps[i1];
        // ρ00 += |a0|², ρ11 += |a1|², ρ01 += a0 conj(a1)
        rho00 = rho00.add(C::new(a0.norm_sq(), 0.0));
        rho11 = rho11.add(C::new(a1.norm_sq(), 0.0));
        rho01 = rho01.add(a0.mul(C::new(a1.re, -a1.im)));
    }
    let x = 2.0 * rho01.re;
    let y = 2.0 * rho01.im;
    let z = rho00.re - rho11.re;
    Ok((x, y, z))
}

/// Named single-/two-qubit unitary as row-major nested list of `{re,im}`.
pub fn named_gate_matrix(name: &str, theta: Option<f64>) -> Result<Vec<Vec<C>>, String> {
    let g = name.trim().to_ascii_uppercase();
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let m = match g.as_str() {
        "I" | "ID" | "IDENTITY" => vec![
            vec![C::one(), C::zero()],
            vec![C::zero(), C::one()],
        ],
        "X" | "NOT" => vec![
            vec![C::zero(), C::one()],
            vec![C::one(), C::zero()],
        ],
        "Y" => vec![
            vec![C::zero(), C::new(0.0, -1.0)],
            vec![C::new(0.0, 1.0), C::zero()],
        ],
        "Z" => vec![
            vec![C::one(), C::zero()],
            vec![C::zero(), C::new(-1.0, 0.0)],
        ],
        "S" => vec![
            vec![C::one(), C::zero()],
            vec![C::zero(), C::new(0.0, 1.0)],
        ],
        "T" => vec![
            vec![C::one(), C::zero()],
            vec![C::zero(), C::new(s, s)],
        ],
        "H" | "HADAMARD" => vec![
            vec![C::new(s, 0.0), C::new(s, 0.0)],
            vec![C::new(s, 0.0), C::new(-s, 0.0)],
        ],
        "RX" => {
            let th = theta.ok_or("Rx needs theta")?;
            let half = th / 2.0;
            let c = half.cos();
            let sn = half.sin();
            vec![
                vec![C::new(c, 0.0), C::new(0.0, -sn)],
                vec![C::new(0.0, -sn), C::new(c, 0.0)],
            ]
        }
        "RY" => {
            let th = theta.ok_or("Ry needs theta")?;
            let half = th / 2.0;
            let c = half.cos();
            let sn = half.sin();
            vec![
                vec![C::new(c, 0.0), C::new(-sn, 0.0)],
                vec![C::new(sn, 0.0), C::new(c, 0.0)],
            ]
        }
        "RZ" => {
            let th = theta.ok_or("Rz needs theta")?;
            let half = th / 2.0;
            vec![
                vec![C::new((-half).cos(), (-half).sin()), C::zero()],
                vec![C::zero(), C::new(half.cos(), half.sin())],
            ]
        }
        "CX" | "CNOT" | "CN" => vec![
            vec![C::one(), C::zero(), C::zero(), C::zero()],
            vec![C::zero(), C::one(), C::zero(), C::zero()],
            vec![C::zero(), C::zero(), C::zero(), C::one()],
            vec![C::zero(), C::zero(), C::one(), C::zero()],
        ],
        "CZ" => vec![
            vec![C::one(), C::zero(), C::zero(), C::zero()],
            vec![C::zero(), C::one(), C::zero(), C::zero()],
            vec![C::zero(), C::zero(), C::one(), C::zero()],
            vec![C::zero(), C::zero(), C::zero(), C::new(-1.0, 0.0)],
        ],
        "SWAP" => vec![
            vec![C::one(), C::zero(), C::zero(), C::zero()],
            vec![C::zero(), C::zero(), C::one(), C::zero()],
            vec![C::zero(), C::one(), C::zero(), C::zero()],
            vec![C::zero(), C::zero(), C::zero(), C::one()],
        ],
        other => {
            return Err(format!(
                "unknown gate `{other}` (I/X/Y/Z/H/S/T/Rx/Ry/Rz/CX/CZ/SWAP)"
            ));
        }
    };
    Ok(m)
}

pub fn matrix_to_json(m: &[Vec<C>]) -> Value {
    Value::Array(
        m.iter()
            .map(|row| Value::Array(row.iter().map(|c| c.to_json()).collect()))
            .collect(),
    )
}

fn parse_complex_cell(v: &Value) -> Result<C, String> {
    match v {
        Value::Number(n) => Ok(C::new(
            n.as_f64().ok_or_else(|| "bad matrix cell".to_string())?,
            0.0,
        )),
        Value::Object(map) => {
            let re = map
                .get("re")
                .and_then(|x| x.as_f64())
                .ok_or_else(|| "complex cell needs `re`".to_string())?;
            let im = map.get("im").and_then(|x| x.as_f64()).unwrap_or(0.0);
            Ok(C::new(re, im))
        }
        Value::Array(a) if a.len() == 2 => {
            let re = a[0]
                .as_f64()
                .ok_or_else(|| "complex pair needs numbers".to_string())?;
            let im = a[1]
                .as_f64()
                .ok_or_else(|| "complex pair needs numbers".to_string())?;
            Ok(C::new(re, im))
        }
        _ => Err("matrix cell must be number or {re,im}".into()),
    }
}

/// Parse nested list / column-map table into matrix rows.
pub fn parse_matrix(v: &Value) -> Result<Vec<Vec<C>>, String> {
    match v {
        Value::Array(rows) => {
            let mut out = Vec::with_capacity(rows.len());
            for (i, row) in rows.iter().enumerate() {
                match row {
                    Value::Array(cells) => {
                        let mut r = Vec::with_capacity(cells.len());
                        for c in cells {
                            r.push(parse_complex_cell(c)?);
                        }
                        out.push(r);
                    }
                    other => {
                        // Flat list of complexes → treat as single row only if nested fails.
                        // Prefer: list of row lists.
                        return Err(format!(
                            "matrix row {i}: expected list of cells, got {other}"
                        ));
                    }
                }
            }
            if out.is_empty() {
                return Err("matrix is empty".into());
            }
            let w = out[0].len();
            if w == 0 || out.iter().any(|r| r.len() != w) {
                return Err("matrix rows must be non-empty and equal length".into());
            }
            Ok(out)
        }
        Value::Object(map) => {
            // Column-oriented: keys are column names with list values → transpose to rows.
            // Or single key holding nested arrays.
            if let Some(Value::Array(rows)) = map.values().next() {
                if rows.iter().all(|r| r.is_array()) && map.len() == 1 {
                    return parse_matrix(&Value::Array(rows.clone()));
                }
            }
            // Horizontal 2-col map like | a | b | with list cells → one row per index
            let cols: Vec<&Vec<Value>> = map
                .values()
                .filter_map(|v| v.as_array())
                .collect();
            if cols.is_empty() {
                return Err("matrix map needs list columns".into());
            }
            let n = cols[0].len();
            if cols.iter().any(|c| c.len() != n) {
                return Err("matrix columns length mismatch".into());
            }
            let mut out = Vec::with_capacity(n);
            for i in 0..n {
                let mut row = Vec::with_capacity(cols.len());
                for c in &cols {
                    row.push(parse_complex_cell(&c[i])?);
                }
                out.push(row);
            }
            Ok(out)
        }
        _ => Err("matrix must be nested list or table".into()),
    }
}

pub fn matrices_close(a: &[Vec<C>], b: &[Vec<C>], tol: f64) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (ra, rb) in a.iter().zip(b.iter()) {
        if ra.len() != rb.len() {
            return false;
        }
        for (ca, cb) in ra.iter().zip(rb.iter()) {
            if (ca.re - cb.re).abs() > tol || (ca.im - cb.im).abs() > tol {
                return false;
            }
        }
    }
    true
}

pub fn gate_new(name: &str, theta: Option<f64>) -> Result<Value, String> {
    let g = name.trim().to_ascii_uppercase();
    // Validate known gate.
    let _ = named_gate_matrix(&g, theta)?;
    let mut out = json!({ "name": g });
    if let Some(t) = theta {
        out.as_object_mut()
            .unwrap()
            .insert("theta".into(), json!(t));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h_bloch_on_plus() {
        let mut amps = zero_state(1);
        apply_h(&mut amps, 0);
        let (x, y, z) = bloch_vector(&amps, 1, 0).unwrap();
        assert!((x - 1.0).abs() < 1e-9, "x={x}");
        assert!(y.abs() < 1e-9, "y={y}");
        assert!(z.abs() < 1e-9, "z={z}");
    }

    #[test]
    fn h_matrix_matches() {
        let m = named_gate_matrix("H", None).unwrap();
        let s = std::f64::consts::FRAC_1_SQRT_2;
        let expect = vec![
            vec![C::new(s, 0.0), C::new(s, 0.0)],
            vec![C::new(s, 0.0), C::new(-s, 0.0)],
        ];
        assert!(matrices_close(&m, &expect, 1e-12));
    }
}
