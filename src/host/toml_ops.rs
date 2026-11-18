//! Standards-based TOML; datetime scalars become explicit ISO text at Value boundary.
use crate::{host::as_text,value::Value};

pub fn parse(text: &Value) -> Result<Value,String> {
    let parsed: toml::Value = toml::from_str(as_text(text,"text")?).map_err(|e|format!("toml.parse: {e}"))?;
    fn convert(v: toml::Value) -> Result<Value,String> {
        Ok(match v {
            toml::Value::String(s) => Value::Text(s),
            toml::Value::Integer(n) => Value::Int(n),
            toml::Value::Float(n) if n.is_finite() => Value::Num(n),
            toml::Value::Float(_) => return Err("toml: non-finite numbers unsupported".into()),
            toml::Value::Boolean(b) => Value::Bool(b),
            toml::Value::Datetime(dt) => Value::Text(dt.to_string()),
            toml::Value::Array(xs) => Value::List(xs.into_iter().map(convert).collect::<Result<_,_>>()?),
            toml::Value::Table(fields) => Value::Map(fields.into_iter().map(|(k,v)|convert(v).map(|v|(k,v))).collect::<Result<_,_>>()?),
        })
    }
    convert(parsed)
}

pub fn stringify(value:&Value)->Result<Value,String>{
    fn convert(v:&Value)->Result<toml::Value,String>{
        Ok(match v {
            Value::Text(s)=>toml::Value::String(s.clone()),
            Value::Int(n)=>toml::Value::Integer(*n),
            Value::Num(n) if n.is_finite()=>toml::Value::Float(*n),
            Value::Bool(b)=>toml::Value::Boolean(*b),
            Value::List(xs)=>toml::Value::Array(xs.iter().map(convert).collect::<Result<_,_>>()?),
            Value::Map(fields)=>toml::Value::Table(fields.iter().map(|(k,v)|convert(v).map(|v|(k.clone(),v))).collect::<Result<_,_>>()?),
            _=>return Err("toml.stringify supports text/finite numbers/bool/list/map; None has no TOML representation".into()),
        })
    }
    if !matches!(value,Value::Map(_)){return Err("toml root must be map".into())}
    toml::to_string_pretty(&convert(value)?).map(Value::Text).map_err(|e|format!("toml.stringify: {e}"))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let t = r#"
# comment
name = "marqdo"
count = 3
ok = true
[pkg]
version = "0.1"
"#;
        let v = parse(&Value::Text(t.into())).unwrap();
        let Value::Map(entries) = v else { panic!() };
        let get = |k: &str| {
            entries
                .iter()
                .find(|(kk, _)| kk == k)
                .map(|(_, v)| v.clone())
                .unwrap()
        };
        assert_eq!(get("name"), Value::Text("marqdo".into()));
        assert_eq!(get("count"), Value::Int(3));
        assert_eq!(get("ok"), Value::Bool(true));
        let Value::Map(pkg) = get("pkg") else { panic!() };
        assert_eq!(
            pkg.iter().find(|(k, _)| k == "version").unwrap().1,
            Value::Text("0.1".into())
        );
    }
}
