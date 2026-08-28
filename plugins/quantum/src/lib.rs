//! Marqdo quantum plugin (C ABI v2): state-vector circuit simulation.

mod draw;
mod linalg;
mod sim;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use serde_json::{json, Value};

const ABI_VERSION: u32 = 2;

type PluginFn = unsafe extern "C" fn(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int;

type HostQueryFn = unsafe extern "C" fn(
    userdata: *mut c_void,
    name: *const c_char,
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int;

#[repr(C)]
pub struct MarqdoHostApi {
    pub userdata: *mut c_void,
    pub register_fn: Option<
        unsafe extern "C" fn(
            userdata: *mut c_void,
            name: *const c_char,
            params: *const c_char,
            fn_ptr: PluginFn,
        ) -> c_int,
    >,
    pub alloc: Option<unsafe extern "C" fn(n: usize) -> *mut c_void>,
    pub free: Option<unsafe extern "C" fn(p: *mut c_void)>,
    pub host_query: Option<HostQueryFn>,
}

static mut HOST_FREE: Option<unsafe extern "C" fn(*mut c_void)> = None;
static mut HOST_ALLOC: Option<unsafe extern "C" fn(usize) -> *mut c_void> = None;
static mut HOST_QUERY: Option<HostQueryFn> = None;
static mut HOST_USERDATA: *mut c_void = ptr::null_mut();

unsafe fn host_strdup(s: &str) -> *mut c_char {
    let alloc = HOST_ALLOC.expect("host alloc");
    let bytes = s.as_bytes();
    let p = alloc(bytes.len() + 1) as *mut u8;
    if p.is_null() {
        return ptr::null_mut();
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
    *p.add(bytes.len()) = 0;
    p as *mut c_char
}

fn set_out(out: *mut *mut c_char, s: &str) {
    if out.is_null() {
        return;
    }
    unsafe {
        *out = host_strdup(s);
    }
}

fn set_err(err: *mut *mut c_char, s: &str) {
    if err.is_null() {
        return;
    }
    unsafe {
        *err = host_strdup(s);
    }
}

fn parse_args(args_json: *const c_char) -> Result<Value, String> {
    if args_json.is_null() {
        return Ok(json!({}));
    }
    let s = unsafe { CStr::from_ptr(args_json) }
        .to_str()
        .map_err(|_| "args not utf-8".to_string())?;
    if s.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(s).map_err(|e| e.to_string())
}

fn reply(out_json: *mut *mut c_char, err_msg: *mut *mut c_char, r: Result<Value, String>) -> c_int {
    match r {
        Ok(v) => {
            set_out(out_json, &v.to_string());
            0
        }
        Err(e) => {
            set_err(err_msg, &e);
            1
        }
    }
}

macro_rules! q_ffi {
    ($name:ident, $body:expr) => {
        unsafe extern "C" fn $name(
            args_json: *const c_char,
            out_json: *mut *mut c_char,
            err_msg: *mut *mut c_char,
        ) -> c_int {
            let args = match parse_args(args_json) {
                Ok(v) => v,
                Err(e) => {
                    set_err(err_msg, &format!("{}: {e}", stringify!($name)));
                    return 1;
                }
            };
            let r: Result<Value, String> = (|| $body(&args))();
            reply(
                out_json,
                err_msg,
                r.map_err(|e| format!("{}: {e}", stringify!($name))),
            )
        }
    };
}

fn arg_u(v: &Value, key: &str) -> Result<usize, String> {
    match v.get(key) {
        Some(Value::Number(n)) => n
            .as_u64()
            .or_else(|| n.as_i64().map(|i| i as u64))
            .map(|u| u as usize)
            .ok_or_else(|| format!("bad `{key}`")),
        Some(Value::String(s)) => s
            .trim()
            .parse()
            .map_err(|_| format!("bad `{key}`")),
        _ => Err(format!("missing `{key}`")),
    }
}

fn arg_f(v: &Value, key: &str) -> Result<f64, String> {
    match v.get(key) {
        Some(Value::Number(n)) => n
            .as_f64()
            .ok_or_else(|| format!("bad `{key}`")),
        Some(Value::String(s)) => s
            .trim()
            .parse()
            .map_err(|_| format!("bad `{key}`")),
        _ => Err(format!("missing `{key}`")),
    }
}

fn circuit_of(args: &Value) -> Result<&Value, String> {
    args.get("circuit")
        .or_else(|| args.get("电路"))
        .ok_or_else(|| "missing `circuit`".to_string())
}

fn take_host_string(p: *mut c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(p) }
        .to_str()
        .ok()
        .map(|s| s.to_string());
    unsafe {
        if let Some(free) = HOST_FREE {
            free(p as *mut c_void);
        }
    }
    s
}

fn host_query_json(name: &str, args: &Value) -> Result<Value, String> {
    let query = unsafe { HOST_QUERY }.ok_or_else(|| "host_query not available".to_string())?;
    let userdata = unsafe { HOST_USERDATA };
    let c_name = CString::new(name).map_err(|e| e.to_string())?;
    let c_args = CString::new(args.to_string()).map_err(|e| e.to_string())?;
    let mut out_ptr: *mut c_char = ptr::null_mut();
    let mut err_ptr: *mut c_char = ptr::null_mut();
    let rc = unsafe {
        query(
            userdata,
            c_name.as_ptr(),
            c_args.as_ptr(),
            &mut out_ptr,
            &mut err_ptr,
        )
    };
    let err = take_host_string(err_ptr);
    let out = take_host_string(out_ptr);
    if rc != 0 {
        return Err(err.unwrap_or_else(|| format!("host_query `{name}` failed")));
    }
    let out = out.unwrap_or_else(|| "null".into());
    serde_json::from_str(&out).map_err(|e| format!("host_query `{name}` bad JSON: {e}"))
}

fn record_plot(svg: &str, path: Option<&str>) -> Result<(), String> {
    let mut args = json!({ "svg": svg });
    if let Some(p) = path {
        if !p.is_empty() {
            args.as_object_mut()
                .unwrap()
                .insert("path".into(), json!(p));
        }
    }
    host_query_json("record_plot", &args)?;
    Ok(())
}

q_ffi!(quantum_ping, |_args: &Value| {
    Ok(json!({
        "ok": true,
        "name": "quantum",
        "abi": ABI_VERSION,
        "max_qubits": sim::max_qubits(),
    }))
});

q_ffi!(quantum_circuit_new, |args: &Value| {
    let qubits = arg_u(args, "qubits").or_else(|_| arg_u(args, "比特数"))?;
    let steps = args
        .get("steps")
        .or_else(|| args.get("步骤"))
        .filter(|v| !v.is_null());
    sim::circuit_new(qubits, steps)
});

q_ffi!(quantum_h, |args: &Value| {
    let c = circuit_of(args)?;
    let q = arg_u(args, "qubit").or_else(|_| arg_u(args, "比特"))?;
    sim::push_op(c, "H", vec![q], None)
});

q_ffi!(quantum_x, |args: &Value| {
    let c = circuit_of(args)?;
    let q = arg_u(args, "qubit").or_else(|_| arg_u(args, "比特"))?;
    sim::push_op(c, "X", vec![q], None)
});

q_ffi!(quantum_y, |args: &Value| {
    let c = circuit_of(args)?;
    let q = arg_u(args, "qubit").or_else(|_| arg_u(args, "比特"))?;
    sim::push_op(c, "Y", vec![q], None)
});

q_ffi!(quantum_z, |args: &Value| {
    let c = circuit_of(args)?;
    let q = arg_u(args, "qubit").or_else(|_| arg_u(args, "比特"))?;
    sim::push_op(c, "Z", vec![q], None)
});

q_ffi!(quantum_s, |args: &Value| {
    let c = circuit_of(args)?;
    let q = arg_u(args, "qubit").or_else(|_| arg_u(args, "比特"))?;
    sim::push_op(c, "S", vec![q], None)
});

q_ffi!(quantum_t, |args: &Value| {
    let c = circuit_of(args)?;
    let q = arg_u(args, "qubit").or_else(|_| arg_u(args, "比特"))?;
    sim::push_op(c, "T", vec![q], None)
});

q_ffi!(quantum_i, |args: &Value| {
    let c = circuit_of(args)?;
    let q = arg_u(args, "qubit").or_else(|_| arg_u(args, "比特"))?;
    sim::push_op(c, "I", vec![q], None)
});

q_ffi!(quantum_rx, |args: &Value| {
    let c = circuit_of(args)?;
    let q = arg_u(args, "qubit").or_else(|_| arg_u(args, "比特"))?;
    let th = arg_f(args, "theta").or_else(|_| arg_f(args, "参数"))?;
    sim::push_op(c, "RX", vec![q], Some(th))
});

q_ffi!(quantum_ry, |args: &Value| {
    let c = circuit_of(args)?;
    let q = arg_u(args, "qubit").or_else(|_| arg_u(args, "比特"))?;
    let th = arg_f(args, "theta").or_else(|_| arg_f(args, "参数"))?;
    sim::push_op(c, "RY", vec![q], Some(th))
});

q_ffi!(quantum_rz, |args: &Value| {
    let c = circuit_of(args)?;
    let q = arg_u(args, "qubit").or_else(|_| arg_u(args, "比特"))?;
    let th = arg_f(args, "theta").or_else(|_| arg_f(args, "参数"))?;
    sim::push_op(c, "RZ", vec![q], Some(th))
});

q_ffi!(quantum_cx, |args: &Value| {
    let c = circuit_of(args)?;
    let control = arg_u(args, "control").or_else(|_| arg_u(args, "控制"))?;
    let target = arg_u(args, "target").or_else(|_| arg_u(args, "目标"))?;
    sim::push_op(c, "CX", vec![control, target], None)
});

q_ffi!(quantum_cz, |args: &Value| {
    let c = circuit_of(args)?;
    let control = arg_u(args, "control").or_else(|_| arg_u(args, "控制"))?;
    let target = arg_u(args, "target").or_else(|_| arg_u(args, "目标"))?;
    sim::push_op(c, "CZ", vec![control, target], None)
});

q_ffi!(quantum_swap, |args: &Value| {
    let c = circuit_of(args)?;
    let a = arg_u(args, "a").or_else(|_| arg_u(args, "比特甲"))?;
    let b = arg_u(args, "b").or_else(|_| arg_u(args, "比特乙"))?;
    sim::push_op(c, "SWAP", vec![a, b], None)
});

q_ffi!(quantum_simulate, |args: &Value| {
    let c = circuit_of(args)?;
    let (qubits, amps) = sim::simulate_circuit(c)?;
    Ok(json!({
        "qubits": qubits,
        "dim": amps.len(),
        "amplitudes": sim::amps_to_json(&amps),
    }))
});

q_ffi!(quantum_probabilities, |args: &Value| {
    let c = circuit_of(args)?;
    let (qubits, amps) = sim::simulate_circuit(c)?;
    Ok(Value::Object(sim::probabilities(&amps, qubits)))
});

q_ffi!(quantum_run, |args: &Value| {
    let c = circuit_of(args)?;
    let shots = arg_u(args, "shots")
        .or_else(|_| arg_u(args, "次数"))
        .unwrap_or(1024);
    let seed = arg_u(args, "seed")
        .or_else(|_| arg_u(args, "种子"))
        .unwrap_or(1) as u64;
    sim::run_circuit(c, shots, seed)
});

q_ffi!(quantum_barrier, |args: &Value| {
    let c = circuit_of(args)?;
    sim::push_op(c, "BARRIER", vec![], None)
});

q_ffi!(quantum_measure, |args: &Value| {
    let c = circuit_of(args)?;
    let qs = match args.get("qubits").or_else(|| args.get("比特")) {
        None | Some(Value::Null) => vec![],
        Some(Value::Array(a)) => a
            .iter()
            .map(|v| {
                v.as_u64()
                    .or_else(|| v.as_i64().map(|i| i as u64))
                    .ok_or_else(|| "bad qubit".to_string())
                    .map(|u| u as usize)
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(Value::Number(n)) => {
            vec![n.as_u64().or_else(|| n.as_i64().map(|i| i as u64)).unwrap() as usize]
        }
        Some(Value::String(s)) => {
            if s.trim().is_empty() {
                vec![]
            } else {
                s.split(|c| c == ',' || c == ' ' || c == '，')
                    .filter(|t| !t.is_empty())
                    .map(|t| {
                        t.trim()
                            .parse::<usize>()
                            .map_err(|_| format!("bad qubit `{t}`"))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }
        }
        _ => return Err("bad qubits for measure".into()),
    };
    sim::push_op(c, "MEASURE", qs, None)
});

q_ffi!(quantum_append, |args: &Value| {
    let c = circuit_of(args)?;
    let op = args
        .get("op")
        .or_else(|| args.get("操作"))
        .ok_or_else(|| "append needs `op`".to_string())?;
    sim::append(c, op)
});

q_ffi!(quantum_noise, |args: &Value| {
    let c = circuit_of(args)?;
    let kind = args
        .get("kind")
        .or_else(|| args.get("种类"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "noise needs `kind`".to_string())?;
    let p = arg_f(args, "p").or_else(|_| arg_f(args, "概率"))?;
    sim::set_noise(c, kind, p)
});

q_ffi!(quantum_state, |args: &Value| {
    let c = circuit_of(args)?;
    let (qubits, amps) = sim::simulate_circuit(c)?;
    Ok(json!({
        "_type": "quantum_state",
        "qubits": qubits,
        "dim": amps.len(),
        "amplitudes": sim::amps_to_json(&amps),
    }))
});

q_ffi!(quantum_draw_circuit, |args: &Value| {
    let c = circuit_of(args)?;
    let kind = args
        .get("kind")
        .or_else(|| args.get("种类"))
        .and_then(|v| v.as_str())
        .unwrap_or("circuit")
        .to_ascii_lowercase();
    let qubit = arg_u(args, "qubit")
        .or_else(|_| arg_u(args, "比特"))
        .unwrap_or(0);
    let path = args
        .get("path")
        .or_else(|| args.get("路径"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());

    let (kind_out, svg) = match kind.as_str() {
        "circuit" | "" => ("circuit", draw::circuit_svg(c)?),
        "probs" | "probabilities" | "概率" => {
            let (qubits, amps) = sim::simulate_circuit(c)?;
            let probs = sim::probabilities(&amps, qubits);
            ("probs", draw::probs_svg(&probs))
        }
        "bloch" | "布洛赫" => {
            let (qubits, amps) = sim::simulate_circuit(c)?;
            let (x, y, z) = sim::bloch_vector(&amps, qubits, qubit)?;
            ("bloch", draw::bloch_svg(x, y, z))
        }
        "hinton" | "欣顿" => {
            let (n, amps) = sim::simulate_circuit(c)?;
            let (_, rho) = linalg::density_from_amps(&amps, n)?;
            let nested = linalg::flat_to_nested(&rho, 1 << n);
            ("hinton", draw::hinton_svg(&nested, "ρ hinton"))
        }
        "city" | "城市" => {
            let (n, amps) = sim::simulate_circuit(c)?;
            let (_, rho) = linalg::density_from_amps(&amps, n)?;
            let nested = linalg::flat_to_nested(&rho, 1 << n);
            ("city", draw::city_svg(&nested, "ρ"))
        }
        "density" | "密度图" => {
            let (n, amps) = sim::simulate_circuit(c)?;
            let (_, rho) = linalg::density_from_amps(&amps, n)?;
            let nested = linalg::flat_to_nested(&rho, 1 << n);
            ("density", draw::density_cells_svg(&nested, "ρ"))
        }
        "paulivec" | "泡利向量" => {
            let (n, amps) = sim::simulate_circuit(c)?;
            let (_, rho) = linalg::density_from_amps(&amps, n)?;
            let dim = 1 << n;
            let labels = linalg::all_pauli_labels(n)?;
            let mut vals = Vec::with_capacity(labels.len());
            for lab in &labels {
                vals.push(linalg::expect_pauli(&rho, dim, n, lab)?);
            }
            ("paulivec", draw::paulivec_svg(&labels, &vals, "Pauli"))
        }
        "qsphere" | "球" => {
            let (n, amps) = sim::simulate_circuit(c)?;
            ("qsphere", draw::qsphere_svg(&amps, n))
        }
        "multibloch" | "多布洛赫" => {
            let (n, amps) = sim::simulate_circuit(c)?;
            ("multibloch", draw::multibloch_svg(&amps, n)?)
        }
        other => {
            return Err(format!(
                "unknown draw kind `{other}` (circuit|probs|bloch|hinton|city|density|paulivec|qsphere|multibloch)"
            ));
        }
    };

    record_plot(&svg, path)?;
    let ops_n = c
        .get("ops")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let qubits = c
        .get("qubits")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    Ok(json!({
        "_type": "quantum_svg",
        "kind": kind_out,
        "qubits": qubits,
        "ops": ops_n,
        "svg": svg,
    }))
});

q_ffi!(quantum_gate_new, |args: &Value| {
    let matrix = args.get("matrix").or_else(|| args.get("矩阵"));
    if let Some(m) = matrix {
        if !m.is_null() {
            let name = args
                .get("name")
                .or_else(|| args.get("名"))
                .and_then(|v| v.as_str());
            return sim::gate_from_matrix(m, name);
        }
    }
    let name = args
        .get("name")
        .or_else(|| args.get("名"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "gate needs `name` or `matrix`".to_string())?;
    let theta = match args.get("theta").or_else(|| args.get("参数")) {
        None | Some(Value::Null) => None,
        Some(v) => Some(
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
                .ok_or_else(|| "bad theta".to_string())?,
        ),
    };
    sim::gate_new(name, theta)
});

q_ffi!(quantum_gate_from_matrix, |args: &Value| {
    let matrix = args
        .get("matrix")
        .or_else(|| args.get("矩阵"))
        .ok_or_else(|| "missing `matrix`".to_string())?;
    let name = args
        .get("name")
        .or_else(|| args.get("名"))
        .and_then(|v| v.as_str());
    sim::gate_from_matrix(matrix, name)
});

q_ffi!(quantum_gate_matrix, |args: &Value| {
    let gate = args
        .get("gate")
        .or_else(|| args.get("门"))
        .ok_or_else(|| "missing `gate`".to_string())?;
    let m = sim::gate_matrix_of(gate)?;
    Ok(sim::matrix_to_json(&m))
});

q_ffi!(quantum_gate_matches_matrix, |args: &Value| {
    let gate = args
        .get("gate")
        .or_else(|| args.get("门"))
        .ok_or_else(|| "missing `gate`".to_string())?;
    let matrix = args
        .get("matrix")
        .or_else(|| args.get("矩阵"))
        .ok_or_else(|| "missing `matrix`".to_string())?;
    let tol = args
        .get("tol")
        .or_else(|| args.get("容差"))
        .and_then(|v| v.as_f64())
        .unwrap_or(1e-9);
    let expect = sim::gate_matrix_of(gate)?;
    let got = sim::parse_matrix(matrix)?;
    Ok(json!(sim::matrices_close(&expect, &got, tol)))
});

q_ffi!(quantum_gate_draw, |args: &Value| {
    let gate = args
        .get("gate")
        .or_else(|| args.get("门"))
        .ok_or_else(|| "missing `gate`".to_string())?;
    let name = gate
        .get("name")
        .or_else(|| gate.get("名"))
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let kind = args
        .get("kind")
        .or_else(|| args.get("种类"))
        .or_else(|| args.get("类型"))
        .and_then(|v| v.as_str())
        .unwrap_or("gate")
        .to_ascii_lowercase();
    let path = args
        .get("path")
        .or_else(|| args.get("路径"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let (kind_out, svg) = if matches!(kind.as_str(), "matrix" | "heatmap" | "矩阵") {
        let m = sim::gate_matrix_of(gate)?;
        (
            "matrix",
            draw::matrix_heatmap_svg(&m, &format!("{name} matrix")),
        )
    } else if matches!(kind.as_str(), "gate" | "glyph" | "门") {
        ("gate", draw::gate_svg(name))
    } else {
        return Err(format!("unknown gate draw kind `{kind}` (gate|matrix)"));
    };
    record_plot(&svg, path)?;
    Ok(json!({
        "_type": "quantum_svg",
        "kind": kind_out,
        "svg": svg,
    }))
});

q_ffi!(quantum_apply, |args: &Value| {
    let c = circuit_of(args)?;
    let gate = args
        .get("gate")
        .or_else(|| args.get("门"))
        .ok_or_else(|| "missing `gate`".to_string())?;
    let qubits = args.get("qubits").or_else(|| args.get("比特"));
    sim::apply_gate(c, gate, qubits)
});

q_ffi!(quantum_density_from_state, |args: &Value| {
    let src = args
        .get("state")
        .or_else(|| args.get("态"))
        .or_else(|| args.get("circuit"))
        .or_else(|| args.get("电路"))
        .ok_or_else(|| "density needs `state` or `circuit`".to_string())?;
    let (n, amps) = linalg::amps_from_state_or_circuit(src)?;
    let (_, rho) = linalg::density_from_amps(&amps, n)?;
    Ok(linalg::density_handle(n, &rho))
});

q_ffi!(quantum_density_from_matrix, |args: &Value| {
    let matrix = args
        .get("matrix")
        .or_else(|| args.get("矩阵"))
        .ok_or_else(|| "missing `matrix`".to_string())?;
    let bag = json!({ "matrix": matrix });
    let (n, rho) = linalg::parse_density(&bag)?;
    Ok(linalg::density_handle(n, &rho))
});

q_ffi!(quantum_density_matrix, |args: &Value| {
    let d = args
        .get("density")
        .or_else(|| args.get("密度"))
        .ok_or_else(|| "missing `density`".to_string())?;
    let (n, rho) = linalg::parse_density(d)?;
    let dim = 1 << n;
    Ok(sim::matrix_to_json(&linalg::flat_to_nested(&rho, dim)))
});

q_ffi!(quantum_density_purity, |args: &Value| {
    let d = args
        .get("density")
        .or_else(|| args.get("密度"))
        .ok_or_else(|| "missing `density`".to_string())?;
    let (n, rho) = linalg::parse_density(d)?;
    let dim = 1 << n;
    Ok(json!(linalg::purity(&rho, dim)))
});

q_ffi!(quantum_density_partial_trace, |args: &Value| {
    let d = args
        .get("density")
        .or_else(|| args.get("密度"))
        .ok_or_else(|| "missing `density`".to_string())?;
    let keep_v = args
        .get("keep")
        .or_else(|| args.get("保留"))
        .ok_or_else(|| "missing `keep`".to_string())?;
    let keep = parse_usize_list(keep_v)?;
    let (n, rho) = linalg::parse_density(d)?;
    let (k, out) = linalg::partial_trace(&rho, n, &keep)?;
    Ok(linalg::density_handle(k, &out))
});

q_ffi!(quantum_density_eig, |args: &Value| {
    let d = args
        .get("density")
        .or_else(|| args.get("密度"))
        .ok_or_else(|| "missing `density`".to_string())?;
    let (n, rho) = linalg::parse_density(d)?;
    let dim = 1 << n;
    let (evals, evecs) = linalg::hermite_eig(&rho, dim)?;
    Ok(linalg::spectrum_handle(&evals, &evecs))
});

q_ffi!(quantum_density_expect, |args: &Value| {
    let d = args
        .get("density")
        .or_else(|| args.get("密度"))
        .or_else(|| args.get("state"))
        .or_else(|| args.get("态"))
        .or_else(|| args.get("circuit"))
        .or_else(|| args.get("电路"))
        .ok_or_else(|| "expect needs density/state/circuit".to_string())?;
    let (n, rho) = if d.get("amplitudes").is_some()
        || d.get("_type").and_then(|t| t.as_str()) == Some("quantum_state")
        || d.get("ops").is_some()
    {
        let (qn, amps) = linalg::amps_from_state_or_circuit(d)?;
        let (_, r) = linalg::density_from_amps(&amps, qn)?;
        (qn, r)
    } else {
        linalg::parse_density(d)?
    };
    let dim = 1 << n;
    let obs = args
        .get("obs")
        .or_else(|| args.get("可观测量"))
        .ok_or_else(|| "missing `obs`".to_string())?;
    let val = if let Some(s) = obs.as_str() {
        linalg::expect_pauli(&rho, dim, n, s)?
    } else {
        let (_od, om) = linalg::value_as_matrix(obs)?;
        linalg::expect_matrix(&rho, dim, &om, dim)?
    };
    Ok(json!(val))
});

q_ffi!(quantum_density_draw, |args: &Value| {
    let d = args
        .get("density")
        .or_else(|| args.get("密度"))
        .ok_or_else(|| "missing `density`".to_string())?;
    let kind = args
        .get("kind")
        .or_else(|| args.get("种类"))
        .and_then(|v| v.as_str())
        .unwrap_or("hinton")
        .to_ascii_lowercase();
    let path = args
        .get("path")
        .or_else(|| args.get("路径"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let (n, rho) = linalg::parse_density(d)?;
    let dim = 1 << n;
    let nested = linalg::flat_to_nested(&rho, dim);
    let (kind_out, svg) = match kind.as_str() {
        "hinton" | "欣顿" => ("hinton", draw::hinton_svg(&nested, "ρ hinton")),
        "city" | "城市" => ("city", draw::city_svg(&nested, "ρ")),
        "density" | "密度图" => ("density", draw::density_cells_svg(&nested, "ρ")),
        "paulivec" | "泡利向量" => {
            let labels = linalg::all_pauli_labels(n)?;
            let mut vals = Vec::with_capacity(labels.len());
            for lab in &labels {
                vals.push(linalg::expect_pauli(&rho, dim, n, lab)?);
            }
            ("paulivec", draw::paulivec_svg(&labels, &vals, "Pauli"))
        }
        other => {
            return Err(format!(
                "unknown density draw kind `{other}` (hinton|city|density|paulivec)"
            ));
        }
    };
    record_plot(&svg, path)?;
    Ok(json!({
        "_type": "quantum_svg",
        "kind": kind_out,
        "qubits": n,
        "svg": svg,
    }))
});

q_ffi!(quantum_kron, |args: &Value| {
    let a = args
        .get("a")
        .or_else(|| args.get("左"))
        .ok_or_else(|| "kron needs `a`".to_string())?;
    let b = args
        .get("b")
        .or_else(|| args.get("右"))
        .ok_or_else(|| "kron needs `b`".to_string())?;
    if (a.get("amplitudes").is_some()
        || a.get("_type").and_then(|t| t.as_str()) == Some("quantum_state"))
        && (b.get("amplitudes").is_some()
            || b.get("_type").and_then(|t| t.as_str()) == Some("quantum_state"))
    {
        let (na, aa) = linalg::amps_from_state_or_circuit(a)?;
        let (nb, bb) = linalg::amps_from_state_or_circuit(b)?;
        let out = linalg::kronecker_amps(&aa, &bb);
        return Ok(json!({
            "_type": "quantum_state",
            "qubits": na + nb,
            "dim": out.len(),
            "amplitudes": sim::amps_to_json(&out),
        }));
    }
    let (_na, ma) = linalg::value_as_matrix(a)?;
    let (_nb, mb) = linalg::value_as_matrix(b)?;
    let da = (ma.len() as f64).sqrt() as usize;
    let db = (mb.len() as f64).sqrt() as usize;
    if da * da != ma.len() || db * db != mb.len() {
        return Err("kron: matrices must be square".into());
    }
    let out = linalg::kronecker(&ma, da, &mb, db);
    let dim = da * db;
    let qubits_f = (dim as f64).log2();
    if (qubits_f - qubits_f.round()).abs() < 1e-9 {
        let qubits = qubits_f.round() as usize;
        Ok(linalg::density_handle(qubits, &out))
    } else {
        Ok(json!({
            "_type": "quantum_density",
            "dim": dim,
            "matrix": sim::matrix_to_json(&linalg::flat_to_nested(&out, dim)),
        }))
    }
});

q_ffi!(quantum_schmidt, |args: &Value| {
    let src = args
        .get("state")
        .or_else(|| args.get("态"))
        .or_else(|| args.get("circuit"))
        .or_else(|| args.get("电路"))
        .ok_or_else(|| "schmidt needs `state` or `circuit`".to_string())?;
    let cut = arg_u(args, "cut")
        .or_else(|_| arg_u(args, "分割"))
        .unwrap_or(1);
    let (n, amps) = linalg::amps_from_state_or_circuit(src)?;
    linalg::schmidt_decompose(&amps, n, cut)
});

q_ffi!(quantum_fidelity, |args: &Value| {
    let a = args
        .get("a")
        .or_else(|| args.get("左"))
        .ok_or_else(|| "fidelity needs `a`".to_string())?;
    let b = args
        .get("b")
        .or_else(|| args.get("右"))
        .ok_or_else(|| "fidelity needs `b`".to_string())?;
    let (_, aa) = linalg::amps_from_state_or_circuit(a)?;
    let (_, bb) = linalg::amps_from_state_or_circuit(b)?;
    Ok(json!(linalg::fidelity_pure(&aa, &bb)?))
});

fn parse_usize_list(v: &Value) -> Result<Vec<usize>, String> {
    match v {
        Value::Array(a) => a
            .iter()
            .map(|x| {
                x.as_u64()
                    .or_else(|| x.as_i64().map(|i| i as u64))
                    .or_else(|| x.as_str().and_then(|s| s.trim().parse().ok()))
                    .map(|u| u as usize)
                    .ok_or_else(|| "bad keep list entry".to_string())
            })
            .collect(),
        Value::Number(n) => Ok(vec![n.as_u64().unwrap_or(0) as usize]),
        Value::String(s) => s
            .split(|c| c == ',' || c == ' ')
            .filter(|t| !t.is_empty())
            .map(|t| t.parse().map_err(|_| format!("bad keep `{t}`")))
            .collect(),
        _ => Err("keep must be list, number, or comma string".into()),
    }
}

fn register(host: &MarqdoHostApi, name: &str, params: &str, fn_ptr: PluginFn) -> c_int {
    let register = match host.register_fn {
        Some(f) => f,
        None => return 1,
    };
    let n = CString::new(name).unwrap();
    let p = CString::new(params).unwrap();
    unsafe { register(host.userdata, n.as_ptr(), p.as_ptr(), fn_ptr) }
}

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_abi_version() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_shutdown() {}

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_init(host: *const MarqdoHostApi) -> c_int {
    if host.is_null() {
        return 1;
    }
    let host = &*host;
    HOST_ALLOC = host.alloc;
    HOST_FREE = host.free;
    HOST_QUERY = host.host_query;
    HOST_USERDATA = host.userdata;
    let regs = [
        ("quantum_ping", "", quantum_ping as PluginFn),
        (
            "quantum_circuit_new",
            "qubits,steps",
            quantum_circuit_new as PluginFn,
        ),
        ("quantum_h", "circuit,qubit", quantum_h as PluginFn),
        ("quantum_x", "circuit,qubit", quantum_x as PluginFn),
        ("quantum_y", "circuit,qubit", quantum_y as PluginFn),
        ("quantum_z", "circuit,qubit", quantum_z as PluginFn),
        ("quantum_s", "circuit,qubit", quantum_s as PluginFn),
        ("quantum_t", "circuit,qubit", quantum_t as PluginFn),
        ("quantum_i", "circuit,qubit", quantum_i as PluginFn),
        ("quantum_rx", "circuit,qubit,theta", quantum_rx as PluginFn),
        ("quantum_ry", "circuit,qubit,theta", quantum_ry as PluginFn),
        ("quantum_rz", "circuit,qubit,theta", quantum_rz as PluginFn),
        (
            "quantum_cx",
            "circuit,control,target",
            quantum_cx as PluginFn,
        ),
        (
            "quantum_cz",
            "circuit,control,target",
            quantum_cz as PluginFn,
        ),
        ("quantum_swap", "circuit,a,b", quantum_swap as PluginFn),
        (
            "quantum_simulate",
            "circuit",
            quantum_simulate as PluginFn,
        ),
        (
            "quantum_probabilities",
            "circuit",
            quantum_probabilities as PluginFn,
        ),
        ("quantum_run", "circuit,shots,seed", quantum_run as PluginFn),
        ("quantum_barrier", "circuit", quantum_barrier as PluginFn),
        (
            "quantum_measure",
            "circuit,qubits",
            quantum_measure as PluginFn,
        ),
        ("quantum_append", "circuit,op", quantum_append as PluginFn),
        ("quantum_noise", "circuit,kind,p", quantum_noise as PluginFn),
        ("quantum_state", "circuit", quantum_state as PluginFn),
        (
            "quantum_draw_circuit",
            "circuit,path,kind,qubit",
            quantum_draw_circuit as PluginFn,
        ),
        // `matrix` is optional (custom unitary); omit from required bind list.
        ("quantum_gate_new", "name,theta", quantum_gate_new as PluginFn),
        (
            "quantum_gate_from_matrix",
            "matrix",
            quantum_gate_from_matrix as PluginFn,
        ),
        (
            "quantum_gate_matrix",
            "gate",
            quantum_gate_matrix as PluginFn,
        ),
        (
            "quantum_gate_matches_matrix",
            "gate,matrix,tol",
            quantum_gate_matches_matrix as PluginFn,
        ),
        (
            "quantum_gate_draw",
            "gate,path,kind",
            quantum_gate_draw as PluginFn,
        ),
        (
            "quantum_apply",
            "circuit,gate,qubits",
            quantum_apply as PluginFn,
        ),
        (
            "quantum_density_from_state",
            "state",
            quantum_density_from_state as PluginFn,
        ),
        (
            "quantum_density_from_matrix",
            "matrix",
            quantum_density_from_matrix as PluginFn,
        ),
        (
            "quantum_density_matrix",
            "density",
            quantum_density_matrix as PluginFn,
        ),
        (
            "quantum_density_purity",
            "density",
            quantum_density_purity as PluginFn,
        ),
        (
            "quantum_density_partial_trace",
            "density,keep",
            quantum_density_partial_trace as PluginFn,
        ),
        (
            "quantum_density_eig",
            "density",
            quantum_density_eig as PluginFn,
        ),
        (
            "quantum_density_expect",
            "density,obs",
            quantum_density_expect as PluginFn,
        ),
        (
            "quantum_density_draw",
            "density,path,kind",
            quantum_density_draw as PluginFn,
        ),
        ("quantum_kron", "a,b", quantum_kron as PluginFn),
        (
            "quantum_schmidt",
            "state,cut",
            quantum_schmidt as PluginFn,
        ),
        ("quantum_fidelity", "a,b", quantum_fidelity as PluginFn),
    ];
    for (name, params, f) in regs {
        if register(host, name, params, f) != 0 {
            return 1;
        }
    }
    0
}
