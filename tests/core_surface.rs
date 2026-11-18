//! T1.3 核心表面守卫：`doc/design/core-surface.md` ⇄ `marqdo::parse::CORE_CONSTRUCTS`。
//!
//! 三重断言（反遮盖）：
//! 1. 文档 id 列与代码注册表**双向集合相等**——改任一侧不同步 → 红；
//! 2. id 不重复；
//! 3. 每行「样例」列指向的文件真实存在——删/改名样例 → 红。
//!
//! 任何新增核心标记都必须同步文档 + 本注册表 + 样例（`doc/design/layers.md` 准入戒律）。

use marqdo::parse::CORE_CONSTRUCTS;

/// 解析构造表：首格为 kebab-case 反引号 id 的行 → `(id, 样例路径)`。
fn construct_rows(doc: &str) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    for line in doc.lines() {
        let line = line.trim();
        if !line.starts_with("| `") {
            continue;
        }
        let cells: Vec<&str> = line.split('|').collect();
        if cells.len() < 3 {
            continue;
        }
        let id = cells[1].trim().trim_matches('`').to_string();
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
            continue; // 非构造行（如 LineKind 大类名首字母大写，天然排除）
        }
        let sample = line
            .split('`')
            .find_map(|s| {
                let s = s.trim();
                if s.starts_with("tests/") || s.starts_with("examples/") {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        rows.push((id, sample));
    }
    rows
}

#[test]
fn core_surface_doc_matches_registry() {
    let doc = include_str!("../doc/design/core-surface.md");
    let mut doc_ids: Vec<String> = construct_rows(doc).into_iter().map(|(id, _)| id).collect();
    doc_ids.sort();
    let mut code_ids: Vec<String> = CORE_CONSTRUCTS
        .iter()
        .map(|(id, _)| (*id).to_string())
        .collect();
    code_ids.sort();
    assert_eq!(
        doc_ids, code_ids,
        "core-surface.md 与 CORE_CONSTRUCTS 不一致（T1.3 守卫）"
    );
}

#[test]
fn core_surface_ids_unique() {
    let doc = include_str!("../doc/design/core-surface.md");
    let ids: Vec<String> = construct_rows(doc).into_iter().map(|(id, _)| id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(ids.len(), sorted.len(), "core-surface.md 存在重复 id");
    let mut code: Vec<&str> = CORE_CONSTRUCTS.iter().map(|(id, _)| *id).collect();
    let before = code.len();
    code.sort_unstable();
    code.dedup();
    assert_eq!(before, code.len(), "CORE_CONSTRUCTS 存在重复 id");
}

#[test]
fn core_surface_samples_exist() {
    let doc = include_str!("../doc/design/core-surface.md");
    for (id, sample) in construct_rows(doc) {
        assert!(!sample.is_empty(), "构造 `{id}` 缺「样例」列");
        assert!(
            std::path::Path::new(&sample).exists(),
            "构造 `{id}` 样例不存在：{sample}"
        );
    }
}

#[test]
fn core_surface_registry_ids_are_kebab() {
    for (id, shape) in CORE_CONSTRUCTS {
        assert!(
            id.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
            "CORE_CONSTRUCTS id 非 kebab：{id}"
        );
        assert!(!shape.is_empty(), "CORE_CONSTRUCTS `{id}` 缺外形描述");
    }
}
