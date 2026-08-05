//! Canonical names and Chinese aliases for L0 keywords / builtins / params.
//!
//! See `doc/design/keywords-i18n.md`. No language mode — aliases are equivalent.

/// Map a literal / logic keyword to its English canonical form, if reserved.
pub fn canonical_keyword(name: &str) -> Option<&'static str> {
    match name {
        "True" | "真" => Some("True"),
        "False" | "假" => Some("False"),
        "None" | "空" => Some("None"),
        "and" | "且" => Some("and"),
        "or" | "或" => Some("or"),
        "not" | "非" => Some("not"),
        _ => None,
    }
}

pub fn is_reserved_keyword(name: &str) -> bool {
    canonical_keyword(name).is_some()
}

/// Map a builtin callee (EN or ZH alias) to its English canonical name.
/// Host primitives without ZH L0 aliases only match English.
pub fn canonical_builtin(name: &str) -> Option<&'static str> {
    if let Some(h) = crate::host::HostFn::from_name(name) {
        return Some(h.name());
    }
    match name {
        "print" | "打印" => Some("print"),
        "input" | "输入" => Some("input"),
        "len" | "长度" => Some("len"),
        "str" | "文本" => Some("str"),
        "int" | "整数" => Some("int"),
        "type" => Some("type"),
        "trim" => Some("trim"),
        "split" => Some("split"),
        "join" => Some("join"),
        "at" => Some("at"),
        _ => None,
    }
}

/// Normalize a named-argument key for a builtin to the English param name.
pub fn canonical_param(builtin: &str, param: &str) -> String {
    let b = canonical_builtin(builtin).unwrap_or(builtin);
    match (b, param) {
        ("print", "text" | "内容") => "text".into(),
        ("input", "prompt" | "提示") => "prompt".into(),
        ("len" | "str" | "int" | "type" | "trim", "value" | "值") => "value".into(),
        ("split" | "join", "value" | "值") => "value".into(),
        ("split" | "join", "sep" | "分隔") => "sep".into(),
        ("at", "value" | "值") => "value".into(),
        ("at", "index" | "下标") => "index".into(),
        (
            "host_plot" | "host_plot_points" | "host_plot_conic",
            "grid" | "网格",
        ) => "grid".into(),
        ("host_plot" | "host_plot_points" | "host_plot_conic", "path" | "路径") => {
            "path".into()
        }
        ("host_plot", "steps" | "步数") => "steps".into(),
        ("host_plot", "derivative" | "导数") => "derivative".into(),
        ("host_foreign_run", "code" | "代码") => "code".into(),
        ("host_foreign_run" | "host_foreign_run_lang", "stdin" | "标准输入") => "stdin".into(),
        ("host_foreign_set_cmd" | "host_foreign_run_lang", "lang" | "语言") => "lang".into(),
        ("host_foreign_set_cmd", "cmd" | "命令") => "cmd".into(),
        ("host_foreign_run_lang", "source" | "源码") => "source".into(),
        _ => param.to_string(),
    }
}

/// Rewrite callee + named args to English canonical forms when this is a builtin.
/// Returns the callee to dispatch on (canonical builtin or original user/fn name).
pub fn normalize_call_callee_and_args(callee: &str, args: &mut [crate::ast::Arg]) -> String {
    match canonical_builtin(callee) {
        Some(canon) => {
            for a in args.iter_mut() {
                if let crate::ast::Arg::Named { name, .. } = a {
                    *name = canonical_param(canon, name);
                }
            }
            canon.to_string()
        }
        None => callee.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_aliases() {
        assert_eq!(canonical_keyword("真"), Some("True"));
        assert_eq!(canonical_keyword("且"), Some("and"));
        assert!(is_reserved_keyword("空"));
    }

    #[test]
    fn builtin_aliases() {
        assert_eq!(canonical_builtin("打印"), Some("print"));
        assert_eq!(canonical_builtin("长度"), Some("len"));
        assert_eq!(canonical_param("打印", "内容"), "text");
        assert_eq!(canonical_param("print", "内容"), "text");
    }
}
