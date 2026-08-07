//! Official `lib/*.mq.md` baked into the binary (fallback when disk `lib/` is absent).

use include_dir::{include_dir, Dir};

static LIB: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/lib");

/// Read `lib/<remainder>` from the embedded tree (e.g. `text.mq.md`).
pub fn read_file(remainder: &str) -> Option<String> {
    let key = remainder.replace('\\', "/");
    LIB.get_file(&key)
        .and_then(|f| f.contents_utf8().map(|s| s.to_string()))
}

pub fn has_file(remainder: &str) -> bool {
    let key = remainder.replace('\\', "/");
    LIB.get_file(&key).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_has_core_libs() {
        assert!(has_file("text.mq.md"));
        assert!(has_file("subtask.mq.md"));
        assert!(has_file("writeback.mq.md"));
        assert!(read_file("math.mq.md")
            .unwrap()
            .contains("host_math_add"));
    }
}
