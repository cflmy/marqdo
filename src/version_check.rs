//! Optional check against GitHub Releases (`marqdo version --check`).

use std::cmp::Ordering;

const REPO: &str = "cflmy/marqdo";

pub fn print_version() {
    println!("marqdo {}", env!("CARGO_PKG_VERSION"));
}

pub fn check_latest() -> Result<(), String> {
    let current = env!("CARGO_PKG_VERSION");
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let response = ureq::get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", &format!("marqdo/{current}"))
        .call()
        .map_err(|e| format!("version check failed: {e}"))?;
    let body = response
        .into_string()
        .map_err(|e| format!("version check read failed: {e}"))?;
    let tag = parse_tag_name(&body).ok_or_else(|| "version check: missing tag_name".to_string())?;
    let latest = tag.trim_start_matches('v');
    match compare_semver(latest, current) {
        Ordering::Greater => {
            println!("Update available: v{latest} (you have {current})");
            println!("https://github.com/{REPO}/releases/latest");
        }
        _ => println!("marqdo {current} is up to date (latest v{latest})."),
    }
    Ok(())
}

fn parse_tag_name(json: &str) -> Option<String> {
    let needle = "\"tag_name\"";
    let start = json.find(needle)? + needle.len();
    let rest = json[start..].trim_start();
    if !rest.starts_with(':') {
        return None;
    }
    let rest = rest[1..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let rest = &rest[1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn compare_semver(a: &str, b: &str) -> Ordering {
    let pa: Vec<u32> = a
        .split('-')
        .next()
        .unwrap_or(a)
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    let pb: Vec<u32> = b
        .split('-')
        .next()
        .unwrap_or(b)
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    for i in 0..3 {
        let da = pa.get(i).copied().unwrap_or(0);
        let db = pb.get(i).copied().unwrap_or(0);
        match da.cmp(&db) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_release_tag() {
        let json = r#"{"tag_name":"v0.1.2","name":"Marqdo"}"#;
        assert_eq!(parse_tag_name(json).as_deref(), Some("v0.1.2"));
    }

    #[test]
    fn semver_order() {
        assert_eq!(compare_semver("0.1.2", "0.1.1"), Ordering::Greater);
        assert_eq!(compare_semver("0.1.1", "0.1.2"), Ordering::Less);
    }
}
