//! Native document data and filesystem addresses must compose under an explicit root.
use marqdo::{Backend, call::{call_file_with, parse_call_args_json, CallEffects, CallOptions}, value::Value};
use std::{fs, path::PathBuf, sync::atomic::{AtomicU64, Ordering}};
static SEQUENCE:AtomicU64=AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new()->Self {
        let root=std::env::temp_dir().join(format!("mq-document-data-{}-{}",std::process::id(),SEQUENCE.fetch_add(1,Ordering::Relaxed)));
        fs::create_dir_all(&root).unwrap();Self(root.canonicalize().unwrap())
    }
}
impl Drop for Scratch {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
fn root()->PathBuf {PathBuf::from(env!("CARGO_MANIFEST_DIR"))}
fn invoke(file:&str,function:&str,args:serde_json::Value,backend:Backend,work:&Scratch)->anyhow::Result<Value>{
    call_file_with(&root().join(file),function,&parse_call_args_json(&args.to_string()).unwrap(),backend,
        CallOptions{effects:CallEffects::parse("fs.read").unwrap(),fs_root:Some(work.0.clone()),..Default::default()})
}
fn field<'a>(value:&'a Value,key:&str)->&'a Value {
    let Value::Map(fields)=value else {panic!("expected record: {value:?}")};
    &fields.iter().find(|(name,_)|name==key).unwrap().1
}
#[test]
fn markdown_data_reading_is_inert_and_preserves_native_values_on_both_backends(){
    let work=Scratch::new();
    fs::write(work.0.join("marker"),"keep").unwrap();
    let backtick=char::from(96);
    let source=format!("---\nmarkup: occlusion\nimport:\n  fs: lib/fs.mq.md\n---\n# main\n\n**[Remove the marker](fs.remove(path=\"marker\"))**\n\n{b}{b}{b}node\nthrow new Error(\"must not run\");\n{b}{b}{b}\n\n## 数据\n\n| @ | id | enabled | count | body | empty |\n|---|---|---|---|---|---|\n| 1 | \"a\" | False | -3 | \"*literal*\" | None |\n",b=backtick);
    fs::write(work.0.join("data.md"),source).unwrap();
    for backend in [Backend::Tree,Backend::Bytecode] {
        for (file,function,args) in [
            ("lib/document.mq.md","read_table",serde_json::json!({"path":"data.md","title":"数据"})),
            ("lib/文档.mq.md","读取表值",serde_json::json!({"路径":"data.md","标题":"数据"})),
        ] {
            let actual=invoke(file,function,args,backend,&work).unwrap();
            let Value::List(rows)=actual else {panic!("expected record rows")};
            assert_eq!(rows.len(),1);
            assert_eq!(field(&rows[0],"enabled"),&Value::Bool(false));
            assert_eq!(field(&rows[0],"count"),&Value::Int(-3));
            assert_eq!(field(&rows[0],"body"),&Value::Text("*literal*".into()));
            assert_eq!(field(&rows[0],"empty"),&Value::None);
            assert_eq!(fs::read_to_string(work.0.join("marker")).unwrap(),"keep");
        }
    }
}
#[test]
fn filesystem_resolution_uses_the_granted_root_and_creates_nothing(){
    let work=Scratch::new();
    for backend in [Backend::Tree,Backend::Bytecode] {
        for (file,function,args) in [
            ("lib/fs.mq.md","resolve",serde_json::json!({"path":"unused/../db/new.sqlite"})),
            ("lib/文件.mq.md","解析路径",serde_json::json!({"路径":"unused/../db/new.sqlite"})),
        ] {
            let actual=invoke(file,function,args,backend,&work).unwrap();
            assert_eq!(actual,Value::Text(work.0.join("db/new.sqlite").to_str().unwrap().into()));
            assert!(!work.0.join("db").exists());
        }
        let outside=work.0.parent().unwrap().join("mq-not-authorized.sqlite");
        let error=invoke("lib/fs.mq.md","resolve",serde_json::json!({"path":outside}),backend,&work).unwrap_err();
        assert!(error.to_string().contains("escapes sandbox root"),"{error:#}");
    }
}
#[cfg(unix)]
#[test]
fn filesystem_resolution_rejects_links_outside_the_root(){
    let work=Scratch::new();
    std::os::unix::fs::symlink(work.0.parent().unwrap(),work.0.join("escape")).unwrap();
    for backend in [Backend::Tree,Backend::Bytecode] {
        let error=invoke("lib/fs.mq.md","resolve",serde_json::json!({"path":"escape/new.sqlite"}),backend,&work).unwrap_err();
        assert!(error.to_string().contains("escapes sandbox root"),"{error:#}");
    }
}
