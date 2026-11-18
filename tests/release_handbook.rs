//! Isolated publication ordering and package-role checks through the real handbooks.
use std::{fs,io::{Read,Write},net::{TcpListener,TcpStream},path::PathBuf,process::{Command,Stdio},
    sync::{Arc,atomic::{AtomicBool,Ordering}},thread,time::{Duration,Instant}};
use serde_json::{json,Value};
fn root()->PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }
fn native()->Command { let mut cmd=Command::new(env!("CARGO_BIN_EXE_marqdo"));cmd.current_dir(root());cmd }
fn invoke(file:&str,function:&str,args:Value,effects:&str,work:&std::path::Path,env:&[(String,String)])->Value {
    let mut cmd=native();
    cmd.args(["call",file,function,"--args-json","-","--effects",effects,"--fs-root",work.to_str().unwrap(),"--json"]);
    for (k,v) in env {cmd.env(k,v);}
    let mut child=cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(args.to_string().as_bytes()).unwrap();
    let out=child.wait_with_output().unwrap();
    assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stdout));
    let answer:Value=serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(answer["ok"],true,"{answer}");answer["result"].clone()
}
fn engineering()->PathBuf {
    let name=if cfg!(target_os="windows"){"engineering.dll"}else if cfg!(target_os="macos"){"libengineering.dylib"}else{"libengineering.so"};
    let lib=root().join("target/debug").join(name);
    if !lib.is_file() {
        let result=invoke("lib/sys.mq.md","exec",json!({"cmd":"cargo","args":["build","-p","marqdo_plugin_engineering","--offline"],"capture":true}),"process.exec",&root(),&[]);
        assert_eq!(result["code"],0,"{result}");
    }
    assert!(lib.is_file());lib
}
#[derive(Debug)]
struct Request { path:String, body:Vec<u8>, headers:String }
fn request(mut stream:TcpStream)->Request {
    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let mut data=Vec::new();let mut chunk=[0u8;4096];
    let header_end=loop {
        let n=stream.read(&mut chunk).unwrap();assert!(n>0);data.extend_from_slice(&chunk[..n]);
        assert!(data.len()<65536);
        if let Some(i)=data.windows(4).position(|v|v==b"\r\n\r\n"){break i+4;}
    };
    let headers=String::from_utf8(data[..header_end].to_vec()).unwrap();
    let length=headers.lines().find_map(|line|line.split_once(':').filter(|(k,_)|k.eq_ignore_ascii_case("content-length"))
        .map(|(_,v)|v.trim().parse::<usize>().unwrap())).unwrap_or(0);
    while data.len()<header_end+length {
        let n=stream.read(&mut chunk).unwrap();assert!(n>0);data.extend_from_slice(&chunk[..n]);
    }
    let path=headers.lines().next().unwrap().split_whitespace().nth(1).unwrap().to_string();
    Request{path,body:data[header_end..header_end+length].to_vec(),headers}
}
fn upload(fail_at:Option<usize>)->(Value,Vec<Request>) {
    let work=std::env::temp_dir().join(format!("mq-r2-order-{}-{}",std::process::id(),fail_at.unwrap_or(0)));
    fs::create_dir_all(&work).unwrap();
    let lib=engineering();
    let listener=TcpListener::bind("127.0.0.1:0").unwrap();listener.set_nonblocking(true).unwrap();
    let address=listener.local_addr().unwrap();
    let done=Arc::new(AtomicBool::new(false));let server_done=done.clone();
    let server=thread::spawn(move||{
        let started=Instant::now();let mut events=Vec::new();
        while started.elapsed()<Duration::from_secs(30) {
            match listener.accept() {
                Ok((mut stream,_))=>{
                    let event=request(stream.try_clone().unwrap());
                    events.push(event);
                    let status=if fail_at==Some(events.len()){"503 Service Unavailable"}else{"200 OK"};
                    stream.write_all(format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes()).unwrap();
                },
                Err(e) if e.kind()==std::io::ErrorKind::WouldBlock=>{
                    if server_done.load(Ordering::SeqCst){break;}
                    thread::sleep(Duration::from_millis(5));
                },
                Err(e)=>panic!("{e}"),
            }
        }
        events
    });
    let names=["marqdo-9.8.7-ext.zip","marqdo-9.8.7-native-x86_64-pc-windows-msvc.zip","marqdo-9.8.7-native-x86_64-unknown-linux-gnu.zip"];
    let mut files=Vec::new();
    for name in names {
        fs::write(work.join(name),b"isolated transport fixture\n").unwrap();
        let digest=invoke("lib/fs.mq.md","file_hash",json!({"path":name}),"fs.read",&work,&[]);
        files.push(json!({"name":name,"path":name,"sha256":digest}));
    }
    let env=vec![
        ("MARQDO_ENGINEERING_PLUGIN".into(),lib.display().to_string()),
        ("MARQDO_RELEASE_CREDENTIAL_HOME".into(),work.display().to_string()),
        ("R2_ENDPOINT".into(),format!("http://{address}")),
        ("R2_BUCKET".into(),"fixture".into()),
        ("R2_REGION".into(),"auto".into()),
        ("R2_ACCESS_KEY_ID".into(),"fixture-key".into()),
        ("R2_SECRET_ACCESS_KEY".into(),"fixture-secret-never-real".into()),
    ];
    let result=invoke("skills/marqdo-release/asset_io.mq.md","upload",json!({"version":"9.8.7","files":files}),"plugin",&work,&env);
    done.store(true,Ordering::SeqCst);
    let events=server.join().unwrap();
    assert!(!result.to_string().contains("fixture-secret"));
    fs::remove_dir_all(work).unwrap();
    (result,events)
}
#[test]
fn all_six_packs_are_written_before_either_version_pointer() {
    let (result,events)=upload(None);assert_eq!(result["ok"],true,"{result}");
    assert_eq!(events.len(),8,"{events:?}");
    for request in &events[..6] {
        assert!(request.path.ends_with(".zip"));assert_eq!(request.body,b"isolated transport fixture\n");
        assert!(request.headers.contains("AWS4-HMAC-SHA256"));
        assert!(!request.headers.contains("fixture-secret"));
    }
    assert_eq!(events[6].path,"/fixture/v9.8.7/VERSION");
    assert_eq!(events[7].path,"/fixture/latest/VERSION");
    assert_eq!(events[6].body,b"9.8.7\n");assert_eq!(events[7].body,b"9.8.7\n");
}
#[test]
fn failed_pack_upload_stops_before_publication_pointers() {
    let (result,events)=upload(Some(4));assert_eq!(result["ok"],false,"{result}");
    assert_eq!(events.len(),4,"{events:?}");
    assert!(events.iter().all(|r|!r.path.ends_with("/VERSION")));
}
#[test]
fn source_role_checks_package_version_and_declared_native_capability() {
    let bad=invoke("skills/marqdo-release/assets.mq.md","content",json!({
        "version":"9.8.7","record":{"name":"marqdo-9.8.7-source.zip","entries":["Cargo.toml"],
        "metadata":{"Cargo.toml":"[package]\nname=\"fixture\"\nversion = \"1.0.0\"\n[dependencies.fake]\nversion = \"9.8.7\"\n"}}
    }),"none",&root(),&[]);
    assert_eq!(bad["ok"],false);assert_eq!(bad["error"]["code"],"release.source_version");
    let bad=invoke("skills/marqdo-release/assets.mq.md","matrix",json!({"records":[
        {"name":"marqdo-9.8.7-ext.zip","entries":["ext/engineering/index.mq.md"]},
        {"name":"marqdo-9.8.7-native-x86_64-pc-windows-msvc.zip","entries":[]},
        {"name":"marqdo-9.8.7-native-x86_64-unknown-linux-gnu.zip","entries":["native/libengineering.so"]}
    ]}),"none",&root(),&[]);
    assert_eq!(bad["ok"],false);assert_eq!(bad["error"]["code"],"release.native_capability");
}

#[test]
fn source_audit_keeps_failed_evidence_and_does_not_execute_documents() {
    let work=std::env::temp_dir().join(format!("mq-handbook-audit-{}",std::process::id()));
    fs::create_dir_all(&work).unwrap();
    fs::write(work.join("content.mq.md"),"# Read this page\n\nThis heading contains explanatory prose only.\n").unwrap();
    fs::write(work.join("uncontracted.mq.md"),"---\nmarkup: occlusion\n---\n\n## task\n\nAfter choosing this fixed result we return *42*.\n").unwrap();
    fs::write(work.join("broken.mq.md"),"---\nmarkup: occlusion\n---\n\n## task\n\nWe **x = [Broken expression](1 +)**.\n").unwrap();
    let env=vec![("MARQDO_ENGINEERING_PLUGIN".into(),engineering().display().to_string())];
    let report=invoke("skills/marqdo-dev/files.mq.md","audit",json!({"paths":["content.mq.md","uncontracted.mq.md","broken.mq.md"]}),"plugin,fs.read",&work,&env);
    assert_eq!(report["ok"],false,"{report}");
    assert_eq!(report["error"]["code"],"authoring.incomplete");
    let records=report["value"]["reports"].as_array().unwrap();
    assert_eq!(records.len(),3);
    assert_eq!(records[0]["ok"],true,"{records:?}");
    assert_eq!(records[1]["missing_contracts"][0]["name"],"task");
    assert_eq!(records[2]["ok"],false);
    assert!(!records[2]["diagnostics"].as_array().unwrap().is_empty());
    fs::remove_dir_all(work).unwrap();
}

#[test]
fn release_commit_preflight_preserves_wrong_branch_directory_and_unrelated_index() {
    let work=std::env::temp_dir().join(format!("mq-commit-scope-{}",std::process::id()));
    fs::create_dir_all(work.join("nested")).unwrap();
    let git=|args:&[&str]|{
        let out=Command::new("git").current_dir(&work).args(args).output().unwrap();
        assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stderr));out.stdout
    };
    git(&["init","-b","main"]);
    git(&["config","user.name","Isolated handbook fixture"]);
    git(&["config","user.email","fixture@example.invalid"]);
    git(&["config","commit.gpgSign","false"]);
    fs::write(work.join("base.txt"),"base\n").unwrap();
    git(&["add","base.txt"]);git(&["commit","-m","prepare fixture"]);
    fs::write(work.join("reviewed.txt"),"reviewed\n").unwrap();
    fs::write(work.join("other.txt"),"unrelated\n").unwrap();
    fs::write(work.join("nested/leaf.txt"),"leaf\n").unwrap();
    let env=vec![("MARQDO_ENGINEERING_PLUGIN".into(),engineering().display().to_string())];
    let args=|paths:Vec<&str>|json!({"paths":paths,"message":"must remain a preflight"});
    git(&["checkout","-b","topic"]);
    let result=invoke("skills/marqdo-release/local.mq.md","commit",args(vec!["reviewed.txt"]),"plugin,fs.read",&work,&env);
    assert_eq!(result["error"]["code"],"release.commit_branch","{result}");
    assert!(git(&["diff","--cached","--name-only"]).is_empty());
    git(&["checkout","main"]);
    let result=invoke("skills/marqdo-release/local.mq.md","commit",args(vec!["nested"]),"plugin,fs.read",&work,&env);
    assert_eq!(result["error"]["code"],"release.commit_paths","{result}");
    assert!(git(&["diff","--cached","--name-only"]).is_empty());
    git(&["add","other.txt"]);let index=git(&["diff","--cached","--name-only"]);
    let result=invoke("skills/marqdo-release/local.mq.md","commit",args(vec!["reviewed.txt"]),"plugin,fs.read",&work,&env);
    assert_eq!(result["error"]["code"],"release.staged_unrelated","{result}");
    assert_eq!(git(&["diff","--cached","--name-only"]),index);
    assert_eq!(String::from_utf8(git(&["log","-1","--format=%s"])).unwrap().trim(),"prepare fixture");
    git(&["restore","--staged","other.txt"]);
    fs::remove_file(work.join("base.txt")).unwrap();
    git(&["add","--","base.txt"]);
    let committed=invoke("skills/marqdo-release/local.mq.md","commit",
        json!({"paths":["base.txt","reviewed.txt"],"message":"retry preserves staged removal"}),
        "plugin,fs.read",&work,&env);
    assert_eq!(committed["ok"],true,"{committed}");
    assert_eq!(String::from_utf8(git(&["log","-1","--format=%s"])).unwrap().trim(),"retry preserves staged removal");
    assert!(git(&["ls-files","base.txt"]).is_empty());
    assert_eq!(String::from_utf8(git(&["ls-files","reviewed.txt"])).unwrap().trim(),"reviewed.txt");
    assert!(!git(&["status","--porcelain","other.txt"]).is_empty());
    fs::remove_dir_all(work).unwrap();
}

#[test]
fn packaging_metadata_and_vendor_preparation_use_the_native_handbook() {
    let work=std::env::temp_dir().join(format!("mq-package-native-{}",std::process::id()));
    fs::create_dir_all(work.join("debian")).unwrap();
    fs::create_dir_all(work.join("vendor/fixture/src")).unwrap();
    fs::write(work.join("Cargo.toml"),"[package]\nname=\"fixture\"\nversion=\"9.8.7\"\n[dependencies.other]\nversion=\"1.0.0\"\n").unwrap();
    fs::write(work.join("debian/changelog"),"marqdo (8.0.0-2ppa4~jammy) jammy; urgency=medium\n\nKeep the reviewed history.\n").unwrap();
    let package="skills/marqdo-release/package.mq.md";
    assert_eq!(invoke(package,"upstream",json!({}),"fs.read",&work,&[]),"9.8.7");
    assert_eq!(invoke(package,"debian_version",json!({"series":"noble"}),"fs.read",&work,&[]),"9.8.7-2ppa4~noble");
    let invalid=invoke(package,"debian_version",json!({"series":"noble;invalid"}),"fs.read",&work,&[]);
    assert_eq!(invalid["ok"],false);
    assert_eq!(invoke(package,"vendor_config",json!({}),"fs.write",&work,&[])["ok"],true);
    assert_eq!(fs::read_to_string(work.join(".cargo/config.toml")).unwrap(),"[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n");
    fs::write(work.join("vendor/fixture/Cargo.toml.orig"),"generated backup").unwrap();
    fs::write(work.join("vendor/fixture/src/lib.rs"),"retained source").unwrap();
    fs::write(work.join("vendor/fixture/.cargo-checksum.json"),json!({"package":"retained-package-hash","files":{"Cargo.toml.orig":"backup-hash","src/lib.rs":"retained-file-hash"}}).to_string()).unwrap();
    let cleaned=invoke(package,"sanitize_vendor",json!({}),"fs.write",&work,&[]);
    assert_eq!(cleaned["ok"],true,"{cleaned}");
    assert_eq!(cleaned["value"]["removed"],1);assert_eq!(cleaned["value"]["changed"],1);
    let checksum:Value=serde_json::from_slice(&fs::read(work.join("vendor/fixture/.cargo-checksum.json")).unwrap()).unwrap();
    assert_eq!(checksum["package"],"retained-package-hash");
    assert_eq!(checksum["files"]["src/lib.rs"],"retained-file-hash");
    assert!(checksum["files"].get("Cargo.toml.orig").is_none());
    assert!(!work.join("vendor/fixture/Cargo.toml.orig").exists());
    assert_eq!(fs::read_to_string(work.join("vendor/fixture/src/lib.rs")).unwrap(),"retained source");
    fs::remove_dir_all(work).unwrap();
}

#[test]
fn independent_pack_build_preserves_sources_requires_all_libraries_and_never_overwrites() {
    let work=std::env::temp_dir().join(format!("mq-independent-packs-{}",std::process::id()));
    fs::create_dir_all(work.join("ext/example")).unwrap();
    fs::create_dir_all(work.join("ext/native")).unwrap();
    fs::create_dir_all(work.join("compiled")).unwrap();
    fs::write(work.join("ext/VERSION"),"9.8.7\n").unwrap();
    fs::write(work.join("ext/example/index.mq.md"),"Readable source retained byte-for-byte.\n").unwrap();
    fs::write(work.join("ext/native/unrelated.so"),"do not include in sources").unwrap();
    let env=vec![("MARQDO_ENGINEERING_PLUGIN".into(),engineering().display().to_string())];
    let args=json!({"version":"9.8.7","platform":"x86_64-unknown-linux-gnu","directory":"out","native_dir":"compiled"});
    let invoke_pack=||invoke("skills/marqdo-release/packs.mq.md","build",args.clone(),"plugin,fs.write",&work,&env);
    let missing=invoke_pack();
    assert_eq!(missing["error"]["code"],"packs.library","{missing}");
    assert!(!work.join("out").exists());
    for name in ["libweb.so","libagent.so","libquantum.so","liblinalg.so","libllm.so","libengineering.so"] {
        fs::write(work.join("compiled").join(name),format!("format-only fixture: {name}\n")).unwrap();
    }
    let built=invoke_pack();
    assert_eq!(built["ok"],true,"{built}");
    let value=&built["value"];
    let sources=&value["source"];
    assert!(sources["archive"]["entries"].as_array().unwrap().contains(&json!("ext/example/index.mq.md")));
    assert!(!sources["archive"]["entries"].as_array().unwrap().iter().any(|v|v.as_str().unwrap().contains("unrelated.so")));
    assert_eq!(sources["archive"]["metadata"]["ext/VERSION"],"9.8.7\n");
    let native_path=work.join(value["native"]["path"].as_str().unwrap());
    let native_before=fs::read(&native_path).unwrap();
    let mut archive=zip::ZipArchive::new(std::io::Cursor::new(&native_before)).unwrap();
    let mut selected=String::new();
    archive.by_name("native/libengineering.so").unwrap().read_to_string(&mut selected).unwrap();
    assert_eq!(selected,"format-only fixture: libengineering.so\n");
    assert_eq!(invoke_pack()["error"]["code"],"packs.output_exists");
    assert_eq!(fs::read(native_path).unwrap(),native_before);
    assert!(!fs::read_dir(&work).unwrap().any(|e|e.unwrap().file_name().to_string_lossy().starts_with(".release-pack")));
    assert_eq!(fs::read_to_string(work.join("ext/example/index.mq.md")).unwrap(),"Readable source retained byte-for-byte.\n");
    fs::remove_dir_all(work).unwrap();
}

#[test]
fn independent_release_creation_checks_exact_tag_and_preserves_existing_publication() {
    let work=std::env::temp_dir().join(format!("mq-ext-create-{}",std::process::id()));
    fs::create_dir_all(&work).unwrap();
    let notes="本次变更 / Changes\nOnly isolated fixture data; no live publication.\n";
    fs::write(work.join("notes.md"),notes).unwrap();
    let listener=TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address=listener.local_addr().unwrap();
    let done=Arc::new(AtomicBool::new(false));let server_done=done.clone();
    let sha="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let server=thread::spawn(move||{
        let mut events=Vec::new();let mut created=false;let started=Instant::now();
        while started.elapsed()<Duration::from_secs(30) {
            match listener.accept() {
                Ok((mut stream,_))=>{
                    let event=request(stream.try_clone().unwrap());
                    let (status,body)=if event.path.ends_with("/git/ref/tags/ext-v9.8.7") {
                        ("200 OK",json!({"object":{"type":"commit","sha":sha}}))
                    } else if event.headers.starts_with("POST ") && event.path=="/repos/cflmy/marqdo/releases" {
                        created=true;
                        ("201 Created",json!({"id":77,"tag_name":"ext-v9.8.7","assets":[]}))
                    } else if event.path.ends_with("/releases/tags/ext-v9.8.7") {
                        if created {("200 OK",json!({"id":77,"tag_name":"ext-v9.8.7","assets":[]}))}
                        else {("404 Not Found",json!({"message":"not found"}))}
                    } else {panic!("unexpected fixture request: {event:?}")};
                    events.push(event);
                    let body=body.to_string();
                    write!(stream,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
                }
                Err(e) if e.kind()==std::io::ErrorKind::WouldBlock=>{
                    if server_done.load(Ordering::SeqCst){break}
                    thread::sleep(Duration::from_millis(5));
                }
                Err(e)=>panic!("{e}"),
            }
        }
        events
    });
    let env=vec![
        ("MARQDO_ENGINEERING_PLUGIN".into(),engineering().display().to_string()),
        ("MARQDO_RELEASE_API_BASE".into(),format!("http://{address}")),
        ("GH_TOKEN".into(),"isolated-fixture-token".into()),
    ];
    let file="skills/marqdo-release/github.mq.md";
    let wrong=invoke(file,"create_ext",json!({"version":"9.8.7","path":"notes.md","sha":"different","root":work}),"plugin,fs.read",&work,&env);
    assert_eq!(wrong["error"]["code"],"release.source_mismatch","{wrong}");
    let args=json!({"version":"9.8.7","path":"notes.md","sha":sha,"root":work});
    let first=invoke(file,"create_ext",args.clone(),"plugin,fs.read",&work,&env);
    assert_eq!(first["ok"],true,"{first}");assert_eq!(first["value"]["id"],77);
    let again=invoke(file,"create_ext",args,"plugin,fs.read",&work,&env);
    assert_eq!(again["ok"],true,"{again}");assert_eq!(again["value"]["id"],77);
    done.store(true,Ordering::SeqCst);
    let events=server.join().unwrap();
    let posts:Vec<_>=events.iter().filter(|e|e.headers.starts_with("POST ")).collect();
    assert_eq!(posts.len(),1,"{events:?}");
    let body:Value=serde_json::from_slice(&posts[0].body).unwrap();
    assert_eq!(body["tag_name"],"ext-v9.8.7");
    assert_eq!(body["target_commitish"],sha);
    assert_eq!(body["body"],notes);
    assert_eq!(body["draft"],false);
    assert!(events.iter().all(|e|!e.headers.starts_with("PATCH ") && !e.headers.starts_with("DELETE ")));
    fs::remove_dir_all(work).unwrap();
}

#[test]
fn extension_ci_rejects_receipt_path_and_byte_drift_before_remote_mutation() {
    let work=std::env::temp_dir().join(format!("mq-ext-receipt-{}",std::process::id()));
    fs::create_dir_all(work.join(".marqdo/release")).unwrap();
    fs::create_dir_all(work.join("dist/ext-release")).unwrap();
    let source="dist/ext-release/marqdo-9.8.7-ext.zip";
    let native_file="dist/ext-release/marqdo-9.8.7-native-x86_64-unknown-linux-gnu.zip";
    for path in [source,native_file] {fs::write(work.join(path),"reviewed fixture bytes").unwrap();}
    let source_sha=invoke("lib/fs.mq.md","file_hash",json!({"path":source}),"fs.read",&work,&[]);
    let native_sha=invoke("lib/fs.mq.md","file_hash",json!({"path":native_file}),"fs.read",&work,&[]);
    let mut receipt=json!({"version":"9.8.7","platform":"x86_64-unknown-linux-gnu",
        "source":{"path":source,"sha256":source_sha},"native":{"path":native_file,"sha256":native_sha}});
    let save=|value:&Value|fs::write(work.join(".marqdo/release/packs.json"),value.to_string()).unwrap();
    save(&receipt);
    let env=vec![
        ("MARQDO_ENGINEERING_PLUGIN".into(),engineering().display().to_string()),
        ("GITHUB_REF_NAME".into(),"ext-v9.8.7".into()),
        ("RUNNER_OS".into(),"Linux".into()),
        ("GITHUB_SHA".into(),"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()),
        ("MARQDO_RELEASE_API_BASE".into(),"http://127.0.0.1:1".into()),
        ("GH_TOKEN".into(),"isolated-fixture-token".into()),
    ];
    fs::write(work.join(native_file),"changed after verification").unwrap();
    let changed=invoke("skills/marqdo-release/packs.mq.md","attach_ci",json!({}),"plugin,fs.read,environment.read",&work,&env);
    assert_eq!(changed["error"]["code"],"packs.changed","{changed}");
    receipt["source"]["path"]=json!("another-candidate.zip");save(&receipt);
    let foreign=invoke("skills/marqdo-release/packs.mq.md","attach_ci",json!({}),"plugin,fs.read,environment.read",&work,&env);
    assert_eq!(foreign["error"]["code"],"packs.receipt_path","{foreign}");
    fs::remove_dir_all(work).unwrap();
}
