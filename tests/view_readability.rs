//! Reader exports must remain faithful to source facts without evaluating them.
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mq-view-reading-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn html_files(path: &std::path::Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            html_files(&p, files)
        } else if p.extension().is_some_and(|e| e == "html") {
            files.push(p)
        }
    }
}
fn export(source: &std::path::Path, out: &std::path::Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_marqdo"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["view", "output", "--no-exec", "--out"])
        .arg(out)
        .arg(source)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut files = Vec::new();
    html_files(out, &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .find(|s| s.contains("id=\"structure\""))
        .unwrap()
}
#[test]
fn single_file_reading_ignores_unrelated_name_collisions_and_never_runs_logic() {
    let temp = Scratch::new();
    for dir in ["one", "two"] {
        let dir = temp.0.join(dir);
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("same.mq.md"), "# main\n\n*1*\n").unwrap();
    }
    let marker = temp.0.join("must-not-exist");
    let source = temp.0.join("entry.mq.md");
    let text=format!("---\nimport:\n  fs: lib/fs.mq.md\n---\n# main\n\n**[写入标记](fs.write_text(path={}, text=\"read must not execute\"))**\n",serde_json::to_string(&marker.to_string_lossy()).unwrap());
    fs::write(&source, &text).unwrap();
    let html = export(&source, &temp.0.join("out"));
    assert!(!marker.exists(), "--no-exec performed the occluded write");
    assert!(
        html.contains("class=\"knowledge-panel\""),
        "knowledge failed: {html}"
    );
    assert!(!html.contains("knowledge-error\""));
    let structure = html
        .split("<div class=\"structure\" id=\"structure\">")
        .nth(1)
        .unwrap()
        .split("class=\"knowledge-panel\"")
        .next()
        .unwrap();
    assert!(structure.contains("写入标记</button>"));
    assert!(structure.contains("class=\"annot-chip annot-implementation\""));
    assert!(
        !structure.contains("class=\"annot-chip annot-bind"),
        "unrun source acquired a runtime result"
    );
}
#[test]
fn knowledge_cards_use_real_contracts_and_keep_verification_states_separate() {
    let temp = Scratch::new();
    let source = temp.0.join("reading.mq.md");
    fs::write(&source, include_str!("../examples/view-reading.mq.md")).unwrap();
    let html = export(&source, &temp.0.join("out"));
    let panel = html
        .split("aria-label=\"可执行知识\"")
        .nth(1)
        .unwrap()
        .split("id=\"mq-knowledge-presentation\"")
        .next()
        .unwrap();
    for label in [
        "需要什么输入",
        "返回什么",
        "调用边界",
        "当前有哪些依据",
        "测试记录",
        "内容来源",
        "领域审阅",
        "技术详情",
    ] {
        assert!(panel.contains(label), "missing {label}");
    }
    assert!(panel.contains("每件商品的价格"));
    assert!(panel.contains("不包含额外税费或优惠"));
    assert!(panel.contains("没有可用记录") && panel.contains("尚未进行领域审阅"));
    assert!(!panel.contains("Signature</dt>") && !panel.contains("Verification</dt>"));
    assert!(panel.contains("未检查锁定状态"));
    let structure = html
        .split("<div class=\"structure\" id=\"structure\">")
        .nth(1)
        .unwrap();
    assert!(
        structure.contains("由作者补充具体业务</button><span class=\"occl-stub\">待补充实际逻辑")
    );
}

#[test]
fn automatic_reader_marks_callable_modules_without_executing_tools_or_hiding_program_errors() {
    let temp = Scratch::new();
    let source = temp.0.join("tools.mq.md");
    fs::write(&source, "---\nmarkup: occlusion\nimport:\n  fs: lib/fs.mq.md\n---\n\n## write_marker\n\nWe **[Write only when explicitly called](fs.write_text(path=\"must-not-exist\", text=\"unexpected automatic call\"))**.\nReturn *None*.\n\n## ask\n\nReturn *[Read only when called](input(prompt=\"Tool input\"))*.\n").unwrap();
    let render = |file: &std::path::Path, output: &std::path::Path| {
        let out = Command::new(env!("CARGO_BIN_EXE_marqdo"))
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["view", "output", "--out"]).arg(output).arg(file).output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let mut paths = Vec::new(); html_files(output, &mut paths);
        paths.iter().map(|p| fs::read_to_string(p).unwrap())
            .find(|s| s.contains("id=\"structure\"")).unwrap()
    };
    let html = render(&source, &temp.0.join("library-out"));
    assert!(html.contains(">module</span>"), "module status was lost");
    assert!(html.contains("这是可调用模块，未自动执行"));
    assert!(!html.contains(">fail</span>"), "a callable library became a failed program");
    assert!(!temp.0.join("must-not-exist").exists());
    assert!(!html.contains("<form class=\"stdin-form\""), "a helper's input prompted during library reading");

    let broken = temp.0.join("broken.mq.md");
    fs::write(&broken, "---\nmarkup: occlusion\n---\n\n## main\n\nReturn *[Real program failure](1 / 0)*.\n").unwrap();
    let html = render(&broken, &temp.0.join("broken-out"));
    assert!(html.contains(">fail</span>"), "a real entry failure was hidden");
    assert!(!html.contains(">module</span>"));
}

#[test]
fn typed_details_keep_calls_links_and_content_readable_without_executing() {
    let temp = Scratch::new();
    let source = temp.0.join("details.mq.md");
    fs::write(&source, include_str!("../examples/view-details.mq.md")).unwrap();
    fs::write(temp.0.join("view-reading.mq.md"), "## next\n\nReturn *1*.\n").unwrap();
    export(&temp.0, &temp.0.join("out"));
    let html = fs::read_to_string(temp.0.join("out/pages/details.mq.md.html")).unwrap();
    for fragment in ["data-logic-kind=\"call\"", "<dt>单价</dt>", "<dt>数量</dt>",
        "class=\"logic-definition\"", "class=\"logic-values\"", "<dt>名称</dt>",
        "data-logic-kind=\"empty\"", "待补充实际逻辑",
        "href=\"view-reading.mq.md.html\"", "href=\"https://github.com/cflmy/marqdo\""] {
        assert!(html.contains(fragment), "missing typed reading detail: {fragment}");
    }
    assert!(!html.contains("class=\"annot-chip annot-bind"), "reading acquired runtime results");
}
