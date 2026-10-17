//! Language presets and specs for `lib/foreign` (pure data; no process spawning).
//!
//! The host runner resolves a `LangSpec`, then builds the command. Keeping this
//! module free of `std::process` lets the browser/wasm build share the syntax and
//! preset tables.

use std::path::PathBuf;

/// One language's invocation recipe.
#[derive(Debug, Clone, Default)]
pub struct LangSpec {
    pub lang: String,
    /// Simple interpreter prefix; the script path is appended.
    pub argv: Vec<String>,
    /// Optional full argv template; supports `{file}`/`{inline}`/… placeholders.
    pub template: Option<Vec<String>>,
    pub ext: Option<String>,
    pub env: Vec<(String, String)>,
    /// Arguments appended to the executable for `probe` (e.g. `--version`).
    pub version_args: Vec<String>,
    /// Persistent session bootstrap available.
    pub session: bool,
    /// Directory for resolving a relative interpreter path (config file dir).
    pub base_dir: Option<PathBuf>,
}

impl LangSpec {
    fn simple(lang: &str, argv: &[&str], ext: &str, version: &[&str], session: bool) -> Self {
        Self {
            lang: lang.to_string(),
            argv: argv.iter().map(|s| (*s).to_string()).collect(),
            template: None,
            ext: Some(ext.to_string()),
            env: Vec::new(),
            version_args: version.iter().map(|s| (*s).to_string()).collect(),
            session,
            base_dir: None,
        }
    }

    fn templated(lang: &str, template: &[&str], ext: &str, version: &[&str]) -> Self {
        Self {
            lang: lang.to_string(),
            argv: vec![template[0].to_string()],
            template: Some(template.iter().map(|s| (*s).to_string()).collect()),
            ext: Some(ext.to_string()),
            env: Vec::new(),
            version_args: version.iter().map(|s| (*s).to_string()).collect(),
            session: false,
            base_dir: None,
        }
    }
}

pub fn default_python_argv() -> Vec<String> {
    #[cfg(windows)]
    {
        vec!["python".into()]
    }
    #[cfg(not(windows))]
    {
        vec!["python3".into()]
    }
}

/// Canonical preset id for a language or alias. Unknown names pass through lowercased.
pub fn canonical(lang: &str) -> String {
    let l = lang.to_ascii_lowercase();
    match l.as_str() {
        "python3" | "py" => "python".into(),
        "javascript" | "js" => "node".into(),
        "shell" | "sh" => "bash".into(),
        "rscript" => "r".into(),
        "pwsh" | "ps1" => "powershell".into(),
        "sqlite3" => "sqlite".into(),
        "typescript" => "tsx".into(),
        _ => l,
    }
}

/// Built-in preset for a language/alias.
pub fn preset_spec(lang: &str) -> Option<LangSpec> {
    let id = canonical(lang);
    Some(match id.as_str() {
        "python" => LangSpec {
            argv: default_python_argv(),
            ..LangSpec::simple("python", &["python3"], "py", &["--version"], true)
        },
        "node" => LangSpec::simple("node", &["node"], "js", &["-v"], true),
        "deno" => LangSpec::simple("deno", &["deno", "run"], "ts", &["--version"], false),
        "bun" => LangSpec::simple("bun", &["bun"], "js", &["--version"], false),
        "ruby" => LangSpec::simple("ruby", &["ruby"], "rb", &["--version"], false),
        "perl" => LangSpec::simple("perl", &["perl"], "pl", &["--version"], false),
        "php" => LangSpec::templated("php", &["php", "-r", "{inline}"], "php", &["--version"]),
        "lua" => LangSpec::simple("lua", &["lua"], "lua", &["-v"], false),
        "r" => LangSpec::simple("r", &["Rscript"], "R", &["--version"], false),
        "julia" => LangSpec::simple("julia", &["julia"], "jl", &["--version"], false),
        "go" => LangSpec::templated("go", &["go", "run", "{file}"], "go", &["version"]),
        "bash" => LangSpec::simple("bash", &["bash"], "sh", &["--version"], false),
        "powershell" => {
            #[cfg(windows)]
            {
                LangSpec::simple("powershell", &["powershell"], "ps1", &["--version"], false)
            }
            #[cfg(not(windows))]
            {
                LangSpec::simple("powershell", &["pwsh"], "ps1", &["--version"], false)
            }
        }
        "sqlite" => LangSpec::simple("sqlite", &["sqlite3"], "sql", &["--version"], false),
        "tsx" => LangSpec::simple("tsx", &["tsx"], "ts", &["--version"], false),
        _ => return None,
    })
}

/// Sorted preset ids (for `langs`).
pub fn preset_ids() -> Vec<String> {
    [
        "python", "node", "deno", "bun", "ruby", "perl", "php", "lua", "r", "julia", "go",
        "bash", "powershell", "sqlite", "tsx",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

/// Display string for the command box (e.g. `python3`, `go run`, `php -r`).
pub fn default_cmd_display(lang: &str) -> String {
    match preset_spec(lang) {
        Some(spec) => {
            if let Some(t) = &spec.template {
                let mut parts = Vec::new();
                for x in t {
                    if x.contains('{') {
                        break;
                    }
                    parts.push(x.clone());
                }
                if parts.is_empty() {
                    lang.to_string()
                } else {
                    parts.join(" ")
                }
            } else {
                spec.argv.join(" ")
            }
        }
        None => lang.to_string(),
    }
}

/// Default argv for v1 compatibility (simple presets only).
pub fn default_argv(lang: &str) -> Option<Vec<String>> {
    preset_spec(lang).map(|s| s.argv)
}

/// Session support for a language (based on preset).
pub fn session_supported(lang: &str) -> bool {
    preset_spec(lang).map(|s| s.session).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_canonicalize() {
        assert_eq!(canonical("py"), "python");
        assert_eq!(canonical("javascript"), "node");
        assert_eq!(canonical("sh"), "bash");
        assert_eq!(canonical("unknown"), "unknown");
    }

    #[test]
    fn presets_cover_docs() {
        assert_eq!(preset_ids().len(), 15);
        assert!(preset_spec("python").is_some());
        assert!(preset_spec("node").is_some());
        assert!(preset_spec("go").unwrap().template.is_some());
        assert!(preset_spec("php").unwrap().template.is_some());
    }

    #[test]
    fn display_uses_template_prefix() {
        assert_eq!(default_cmd_display("go"), "go run");
        assert_eq!(default_cmd_display("php"), "php -r");
    }
}
