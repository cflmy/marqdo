//! Value-independent argument binding. Effects finish before this plan is used.
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArgumentPlan {
    pub parameters: Vec<Option<usize>>,
    pub assignments: Vec<(String, usize)>,
}
pub(crate) fn plan(
    params: &[String],
    names: &[Option<String>],
    allow_missing: bool,
) -> Result<ArgumentPlan, String> {
    plan_with(params, names, |_| allow_missing)
}

pub(crate) fn plan_with_omissions(
    params: &[String],
    names: &[Option<String>],
    omissions: &[bool],
) -> Result<ArgumentPlan, String> {
    if params.len() != omissions.len() {
        return Err("argument omission mask count mismatch".into());
    }
    plan_with(params, names, |index| omissions[index])
}

fn plan_with(
    params: &[String],
    names: &[Option<String>],
    may_omit: impl Fn(usize) -> bool,
) -> Result<ArgumentPlan, String> {
    let mut named = HashMap::new();
    let mut positionals = Vec::new();
    for (index, name) in names.iter().enumerate() {
        match name {
            Some(name) => {
                if named.insert(name.clone(), index).is_some() {
                    return Err(format!("duplicate named argument `{name}`"));
                }
            }
            None => positionals.push(index),
        }
    }
    let mut pos = 0;
    let mut parameters = Vec::new();
    let mut assignments = Vec::new();
    for (parameter_index, param) in params.iter().enumerate() {
        let index = if let Some(index) = named.get(param) {
            Some(*index)
        } else if let Some(index) = positionals.get(pos) {
            pos += 1;
            Some(*index)
        } else if may_omit(parameter_index) {
            None
        } else {
            return Err(format!("missing argument for parameter `{param}`"));
        };
        parameters.push(index);
        if let Some(index) = index {
            assignments.push((param.clone(), index));
        }
    }
    if pos < positionals.len() {
        return Err(format!(
            "too many positional arguments ({} extra)",
            positionals.len() - pos
        ));
    }
    for (index, name) in names.iter().enumerate() {
        if let Some(name) = name {
            if !params.contains(name) {
                assignments.push((name.clone(), index));
            }
        }
    }
    Ok(ArgumentPlan {
        parameters,
        assignments,
    })
}
pub(crate) fn bind<T: Clone>(
    params: &[String],
    names: &[Option<String>],
    values: &[T],
    allow_missing: bool,
) -> Result<HashMap<String, T>, String> {
    if names.len() != values.len() {
        return Err("argument layout/value count mismatch".into());
    }
    let plan = plan(params, names, allow_missing)?;
    Ok(plan
        .assignments
        .into_iter()
        .map(|(name, index)| (name, values[index].clone()))
        .collect())
}

pub(crate) fn plan_evidence(
    params: &[String],
    names: &[Option<String>],
    omissions: &[bool],
) -> serde_json::Value {
    let result = match plan_with_omissions(params, names, omissions) {
        Ok(plan) => {
            serde_json::json!({"ok":true,"parameters":plan.parameters,"assignments":plan.assignments})
        }
        Err(error) => serde_json::json!({"ok":false,"error":error}),
    };
    serde_json::json!({"params":params,"arguments":names,"omissions":omissions,"result":result})
}
pub(crate) fn builtin_signature(name: &str) -> Option<(&'static [&'static str], bool)> {
    Some(match name {
        "print" => (&["text"], false),
        "input" => (&["prompt"], true),
        "len" | "str" | "int" | "type" | "trim" => (&["value"], false),
        "split" | "join" => (&["value", "sep"], false),
        "at" => (&["value", "index"], false),
        "call_fn" => (&["name"], false),
        _ => return None,
    })
}
