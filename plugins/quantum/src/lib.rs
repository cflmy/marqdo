//! Marqdo quantum plugin (C ABI v2): state-vector circuit simulation.

mod draw;
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

q_ffi!(quantum_draw_circuit, |args: &Value| {
    let c = circuit_of(args)?;
    let svg = draw::circuit_svg(c)?;
    let path = args
        .get("path")
        .or_else(|| args.get("路径"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    // Prefer view/CLI plots channel; still return structured value for authors.
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
        "kind": "circuit",
        "qubits": qubits,
        "ops": ops_n,
        "svg": svg,
    }))
});

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
        (
            "quantum_draw_circuit",
            "circuit,path",
            quantum_draw_circuit as PluginFn,
        ),
    ];
    for (name, params, f) in regs {
        if register(host, name, params, f) != 0 {
            return 1;
        }
    }
    0
}
