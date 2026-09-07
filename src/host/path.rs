//! Path algebra for Mid M2 (`lib/path`) — no I/O.

use std::path::{Component, Path, PathBuf};

use crate::value::Value;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} needs text")),
    }
}

fn path_to_text(p: &Path) -> String {
    if p.as_os_str().is_empty() {
        ".".into()
    } else {
        p.to_string_lossy().into_owned()
    }
}

pub fn join(a: &Value, b: &Value) -> Result<Value, String> {
    let a = as_text(a, "a")?;
    let b = as_text(b, "b")?;
    let out = Path::new(a).join(b);
    Ok(Value::Text(path_to_text(&out)))
}

pub fn split(path: &Value) -> Result<Value, String> {
    let p = as_text(path, "path")?;
    let mut parts = Vec::new();
    for c in Path::new(p).components() {
        match c {
            Component::RootDir | Component::CurDir | Component::ParentDir => {
                parts.push(Value::Text(c.as_os_str().to_string_lossy().into_owned()));
            }
            Component::Prefix(prefix) => {
                parts.push(Value::Text(prefix.as_os_str().to_string_lossy().into_owned()));
            }
            Component::Normal(s) => parts.push(Value::Text(s.to_string_lossy().into_owned())),
        }
    }
    Ok(Value::List(parts))
}

pub fn file_name(path: &Value) -> Result<Value, String> {
    let p = as_text(path, "path")?;
    Ok(Value::Text(
        Path::new(p)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
    ))
}

pub fn parent(path: &Value) -> Result<Value, String> {
    let p = as_text(path, "path")?;
    let parent = Path::new(p).parent();
    Ok(Value::Text(match parent {
        Some(par) if !par.as_os_str().is_empty() => path_to_text(par),
        _ => ".".into(),
    }))
}

pub fn extension(path: &Value) -> Result<Value, String> {
    let p = as_text(path, "path")?;
    Ok(Value::Text(
        Path::new(p)
            .extension()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
    ))
}

pub fn normalize(path: &Value) -> Result<Value, String> {
    let p = as_text(path, "path")?;
    let mut out = PathBuf::new();
    for c in Path::new(p).components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                match out.components().next_back() {
                    Some(Component::Normal(_)) => {
                        out.pop();
                    }
                    Some(Component::RootDir) | Some(Component::Prefix(_)) => {}
                    _ => out.push(".."),
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    Ok(Value::Text(path_to_text(&out)))
}

pub fn is_absolute(path: &Value) -> Result<Value, String> {
    let p = as_text(path, "path")?;
    Ok(Value::Bool(Path::new(p).is_absolute()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_and_name() {
        let j = join(
            &Value::Text("a/b".into()),
            &Value::Text("c.txt".into()),
        )
        .unwrap();
        assert_eq!(j, Value::Text("a/b/c.txt".into()));
        assert_eq!(
            file_name(&Value::Text("a/b/c.txt".into())).unwrap(),
            Value::Text("c.txt".into())
        );
        assert_eq!(
            extension(&Value::Text("c.txt".into())).unwrap(),
            Value::Text("txt".into())
        );
    }
}
