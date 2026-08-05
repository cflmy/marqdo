//! Host entry points for math / formula / random / plot.

use std::collections::HashMap;

use crate::formula::{self, Expr};
use crate::host::fs;
use crate::host::{HostContext, PlotArtifact};
use crate::value::Value;

pub fn as_f64(v: &Value) -> Result<f64, String> {
    match v {
        Value::Num(n) => Ok(*n),
        Value::Int(n) => Ok(*n as f64),
        Value::Text(s) => s
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("cannot convert to num: {s:?}")),
        _ => Err("expected num (or int/text)".into()),
    }
}

fn opt<'a>(v: Option<&'a Value>) -> Option<&'a Value> {
    match v {
        None | Some(Value::None) => None,
        Some(v) => Some(v),
    }
}

pub fn as_text(v: &Value) -> Result<&str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err("expected text".into()),
    }
}

pub fn as_bool(v: &Value) -> bool {
    v.truthy()
}

pub fn formula_from_value(v: &Value) -> Result<Expr, String> {
    match v {
        Value::Formula(e) => Ok(e.clone()),
        Value::Text(s) => formula::parse(s),
        Value::Num(n) => Ok(Expr::Num(*n)),
        Value::Int(n) => Ok(Expr::Num(*n as f64)),
        _ => Err("expected formula or expression text".into()),
    }
}

fn num_val(n: f64) -> Value {
    Value::Num(n)
}

pub fn pi() -> Result<Value, String> {
    Ok(num_val(std::f64::consts::PI))
}

pub fn e_const() -> Result<Value, String> {
    Ok(num_val(std::f64::consts::E))
}

pub fn to_num(v: &Value) -> Result<Value, String> {
    Ok(num_val(as_f64(v)?))
}

pub fn unary_num(op: &str, x: &Value) -> Result<Value, String> {
    let x = as_f64(x)?;
    let y = match op {
        "neg" => -x,
        "sin" => x.sin(),
        "cos" => x.cos(),
        "tan" => x.tan(),
        "asin" => x.asin(),
        "acos" => x.acos(),
        "atan" => x.atan(),
        "sqrt" => x.sqrt(),
        "abs" => x.abs(),
        "ln" | "log" => x.ln(),
        "exp" => x.exp(),
        "floor" => x.floor(),
        "ceil" => x.ceil(),
        _ => return Err(format!("unknown unary math op `{op}`")),
    };
    Ok(num_val(y))
}

pub fn binary_num(op: &str, a: &Value, b: &Value) -> Result<Value, String> {
    let a = as_f64(a)?;
    let b = as_f64(b)?;
    let y = match op {
        "add" => a + b,
        "sub" => a - b,
        "mul" => a * b,
        "div" => {
            if b == 0.0 {
                return Err("division by zero".into());
            }
            a / b
        }
        "pow" => a.powf(b),
        "min" => a.min(b),
        "max" => a.max(b),
        _ => return Err(format!("unknown binary math op `{op}`")),
    };
    Ok(num_val(y))
}

pub fn seed(ctx: &mut HostContext, v: &Value) -> Result<Value, String> {
    let n = as_f64(v)? as u64;
    ctx.rng = n ^ 0xA5A5_A5A5_A5A5_A5A5;
    if ctx.rng == 0 {
        ctx.rng = 0xC0FFEE;
    }
    Ok(Value::None)
}

pub fn random(ctx: &mut HostContext) -> Result<Value, String> {
    Ok(num_val(ctx.next_f64()))
}

pub fn random_int(ctx: &mut HostContext, lo: &Value, hi: &Value) -> Result<Value, String> {
    let lo = as_f64(lo)? as i64;
    let hi = as_f64(hi)? as i64;
    if hi < lo {
        return Err("random_int: max < min".into());
    }
    let span = (hi - lo + 1) as u64;
    let r = ctx.next_u64() % span;
    Ok(Value::Int(lo + r as i64))
}

pub fn formula_parse(text: &Value) -> Result<Value, String> {
    let s = as_text(text)?;
    Ok(Value::Formula(formula::parse(s)?))
}

pub fn simplify(v: &Value) -> Result<Value, String> {
    Ok(Value::Formula(formula::simplify(&formula_from_value(v)?)))
}

pub fn expand(v: &Value) -> Result<Value, String> {
    Ok(Value::Formula(formula::expand(&formula_from_value(v)?)))
}

pub fn diff(formula: &Value, var: &Value) -> Result<Value, String> {
    let e = formula_from_value(formula)?;
    let var = as_text(var)?;
    Ok(Value::Formula(formula::diff(&e, var)))
}

pub fn subs(formula: &Value, var: &Value, value: &Value) -> Result<Value, String> {
    let e = formula_from_value(formula)?;
    let var = as_text(var)?;
    let val = formula_from_value(value)?;
    let out = formula::subs(&e, var, &val);
    if let Expr::Num(n) = out {
        Ok(num_val(n))
    } else {
        Ok(Value::Formula(out))
    }
}

pub fn eval_at(formula: &Value, var: &Value, value: &Value) -> Result<Value, String> {
    let e = formula_from_value(formula)?;
    let var = as_text(var)?;
    let x = as_f64(value)?;
    let mut env = HashMap::new();
    env.insert(var.to_string(), x);
    Ok(num_val(formula::eval_f64(&e, &env)?))
}

pub fn solve(
    formula: &Value,
    var: &Value,
    min: Option<&Value>,
    max: Option<&Value>,
) -> Result<Value, String> {
    let e = formula_from_value(formula)?;
    let var = as_text(var)?;
    let xmin = opt(min).map(as_f64).transpose()?.unwrap_or(-100.0);
    let xmax = opt(max).map(as_f64).transpose()?.unwrap_or(100.0);
    let roots = formula::solve(&e, var, xmin, xmax)?;
    Ok(Value::List(roots.into_iter().map(num_val).collect()))
}

fn maybe_write_plot(
    ctx: &mut HostContext,
    path: Option<&Value>,
    svg: String,
) -> Result<Value, String> {
    let path_s = match opt(path) {
        Some(v) => Some(as_text(v)?.to_string()),
        None => None,
    };
    if let Some(ref p) = path_s {
        fs::write_text(ctx, &Value::Text(p.clone()), &Value::Text(svg.clone()))?;
    }
    ctx.plots.push(PlotArtifact {
        path: path_s,
        svg: svg.clone(),
    });
    Ok(Value::Text(svg))
}

fn plot_style(grid: Option<&Value>) -> Result<formula::PlotStyle, String> {
    let grid = match opt(grid) {
        None => true,
        Some(v) => as_bool(v),
    };
    Ok(formula::PlotStyle { grid })
}

pub fn plot(
    ctx: &mut HostContext,
    formula: &Value,
    var: &Value,
    min: &Value,
    max: &Value,
    steps: Option<&Value>,
    path: Option<&Value>,
    derivative: Option<&Value>,
    grid: Option<&Value>,
) -> Result<Value, String> {
    let mut e = formula_from_value(formula)?;
    let var_s = as_text(var)?;
    if opt(derivative).map(as_bool).unwrap_or(false) {
        e = formula::diff(&e, var_s);
    }
    let xmin = as_f64(min)?;
    let xmax = as_f64(max)?;
    let steps = match opt(steps) {
        Some(v) => as_f64(v)? as usize,
        None => 200,
    };
    let style = plot_style(grid)?;
    let svg = formula::plot_function_svg(&e, var_s, xmin, xmax, steps, style)?;
    maybe_write_plot(ctx, path, svg)
}

pub fn plot_points(
    ctx: &mut HostContext,
    xs: &Value,
    ys: &Value,
    path: Option<&Value>,
    grid: Option<&Value>,
) -> Result<Value, String> {
    let xs = match xs {
        Value::List(xs) => xs
            .iter()
            .map(as_f64)
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err("plot_points needs list xs".into()),
    };
    let ys = match ys {
        Value::List(ys) => ys
            .iter()
            .map(as_f64)
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err("plot_points needs list ys".into()),
    };
    let style = plot_style(grid)?;
    let svg = formula::plot_points_svg(&xs, &ys, style)?;
    maybe_write_plot(ctx, path, svg)
}

pub fn plot_conic(
    ctx: &mut HostContext,
    kind: &Value,
    a: &Value,
    b: Option<&Value>,
    h: Option<&Value>,
    k: Option<&Value>,
    path: Option<&Value>,
    grid: Option<&Value>,
) -> Result<Value, String> {
    let kind = as_text(kind)?;
    let a = as_f64(a)?;
    let b = opt(b).map(as_f64).transpose()?.unwrap_or(a);
    let h = opt(h).map(as_f64).transpose()?.unwrap_or(0.0);
    let k = opt(k).map(as_f64).transpose()?.unwrap_or(0.0);
    let style = plot_style(grid)?;
    let svg = formula::plot_conic_svg(kind, a, b, h, k, style)?;
    maybe_write_plot(ctx, path, svg)
}
