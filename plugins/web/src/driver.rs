//! Driver URL schemes for `# db` / `# cache` / `# storage` (design ext-web-drivers).

/// Which SQL backend a `# db` URL selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbKind {
    Sqlite,
    Postgres,
}

pub fn db_kind(url: &str) -> DbKind {
    let u = url.trim().to_ascii_lowercase();
    if u.starts_with("postgres://") || u.starts_with("postgresql://") {
        DbKind::Postgres
    } else {
        DbKind::Sqlite
    }
}

/// Rewrite SQL written with `?` placeholders into Postgres `$1,$2,…` form.
pub fn rewrite_placeholders_pg(sql: &str) -> String {
    let mut out = String::with_capacity(sql.len() + 8);
    let mut n = 0usize;
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '?' {
            n += 1;
            out.push('$');
            out.push_str(&n.to_string());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_from_url() {
        assert_eq!(db_kind("sqlite:a.db"), DbKind::Sqlite);
        assert_eq!(db_kind("postgres://u:p@h/db"), DbKind::Postgres);
        assert_eq!(db_kind("postgresql://localhost/x"), DbKind::Postgres);
    }

    #[test]
    fn rewrite_qmark() {
        assert_eq!(
            rewrite_placeholders_pg("SELECT * FROM t WHERE a = ? AND b IN (?, ?)"),
            "SELECT * FROM t WHERE a = $1 AND b IN ($2, $3)"
        );
    }
}
