//! Thin statistics for Mid2 M10 (`lib/stats`). Not a dataframe library.

use crate::value::Value;

fn as_nums(list: &Value) -> Result<Vec<f64>, String> {
    let Value::List(xs) = list else {
        return Err("stats: list of numbers required".into());
    };
    let mut out = Vec::with_capacity(xs.len());
    for (i, x) in xs.iter().enumerate() {
        match x {
            Value::Int(n) => out.push(*n as f64),
            Value::Num(n) => out.push(*n),
            _ => return Err(format!("stats: item {i} must be number")),
        }
    }
    Ok(out)
}

fn num_val(n: f64) -> Value {
    if n.fract() == 0.0 && n.abs() < (i64::MAX as f64) {
        Value::Int(n as i64)
    } else {
        Value::Num(n)
    }
}

pub fn mean(list: &Value) -> Result<Value, String> {
    let xs = as_nums(list)?;
    if xs.is_empty() {
        return Err("stats.mean: empty list".into());
    }
    let sum: f64 = xs.iter().sum();
    Ok(Value::Num(sum / xs.len() as f64))
}

pub fn median(list: &Value) -> Result<Value, String> {
    let mut xs = as_nums(list)?;
    if xs.is_empty() {
        return Err("stats.median: empty list".into());
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = xs.len();
    let m = if n % 2 == 1 {
        xs[n / 2]
    } else {
        (xs[n / 2 - 1] + xs[n / 2]) / 2.0
    };
    Ok(num_val(m))
}

/// Sample standard deviation (n >= 2), matching common `stdev` semantics.
pub fn stdev(list: &Value) -> Result<Value, String> {
    let xs = as_nums(list)?;
    if xs.len() < 2 {
        return Err("stats.stdev: need at least 2 values".into());
    }
    let n = xs.len() as f64;
    let mean = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| {
        let d = x - mean;
        d * d
    }).sum::<f64>()
        / (n - 1.0);
    Ok(Value::Num(var.sqrt()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let xs = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        let m = mean(&xs).unwrap();
        assert_eq!(m, Value::Num(2.5));
        assert_eq!(median(&xs).unwrap(), Value::Num(2.5));
        let s = stdev(&xs).unwrap();
        let Value::Num(v) = s else { panic!() };
        assert!((v - 1.2909944487358056).abs() < 1e-9);
    }
}
