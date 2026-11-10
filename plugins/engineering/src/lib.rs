//! Native boundaries for executable engineering documents. Business ordering,
//! version decisions and ordinary file reads/writes belong in .mq.md.
//! No Python interpreter, generated shell program or secret-valued tool result.
use std::{
    collections::BTreeMap,
    ffi::CStr,
    fs::{self, File},
    io::{Read, Write},
    os::raw::{c_char, c_int},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use hmac::{Hmac, Mac};
use marqdo_plugin_sdk::*;
use sha2::{Digest, Sha256};
use serde_json::{json, Value};

type Answer = Result<Value, String>;
const OUTPUT: usize = 256 * 1024;
const HTTP_MAX: u64 = 16 * 1024 * 1024;
fn field<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v.get(key).and_then(Value::as_str).ok_or_else(|| format!("missing text field {key}"))
}
fn hash(b: &[u8]) -> String { hex(&Sha256::digest(b)) }
fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }
fn read_bounded(mut r: impl Read, max: u64) -> Result<Vec<u8>, String> {
    let mut b = Vec::new();
    r.by_ref().take(max + 1).read_to_end(&mut b).map_err(|e| format!("read failed ({:?})",e.kind()))?;
    if b.len() as u64 > max { return Err("bounded input limit exceeded".into()); }
    Ok(b)
}
fn scoped(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let p = root.join(relative).canonicalize().map_err(|_| "path does not exist")?;
    if !p.starts_with(root) { return Err("artifact path escapes the selected root".into()); }
    Ok(p)
}
fn homes(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for key in ["MARQDO_RELEASE_CREDENTIAL_HOME", "USERPROFILE", "HOME"] {
        if let Some(s) = std::env::var_os(key) { out.push(PathBuf::from(s)); }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        for p in std::iter::once(root).chain(root.ancestors().skip(1)) {
            if p.join(".git").exists() {
                if let Ok(m) = p.metadata() {
                    unsafe {
                        let mut entry: libc::passwd = std::mem::zeroed();
                        let mut result = std::ptr::null_mut();
                        let mut buf = vec![0u8; 32768];
                        if libc::getpwuid_r(m.uid(), &mut entry, buf.as_mut_ptr().cast(), buf.len(), &mut result) == 0
                            && !result.is_null() && !entry.pw_dir.is_null() {
                            out.push(PathBuf::from(CStr::from_ptr(entry.pw_dir).to_string_lossy().into_owned()));
                        }
                    }
                }
            }
        }
    }
    out.dedup();
    out
}
fn redact(mut text: String, private: &[String]) -> String {
    for token in private.iter().filter(|s| !s.is_empty()) { text = text.replace(token, "<secret>"); }
    for (k, v) in std::env::vars() {
        if v.len() >= 8 && ["TOKEN", "SECRET", "PASSWORD", "ACCESS_KEY"].iter().any(|x| k.contains(x)) {
            text = text.replace(&v, "<secret>");
        }
    }
    text
}
struct Output { code: i32, stdout: String, stderr: String, timed_out: bool, truncated: bool }
fn pipe(mut input: impl Read + Send + 'static, limit: usize) -> mpsc::Receiver<(Vec<u8>, bool)> {
    let (send, recv) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = Vec::new(); let mut part = [0u8; 8192]; let mut truncated = false;
        while let Ok(n) = input.read(&mut part) {
            if n == 0 { break; }
            let keep = (limit - buf.len()).min(n);
            buf.extend_from_slice(&part[..keep]); truncated |= keep < n;
        }
        let _ = send.send((buf, truncated));
    });
    recv
}
fn execute(root: &Path, argv: &[String], input: Option<String>, env: &BTreeMap<String, String>,
           timeout_ms: u64, max_output: usize) -> Result<Output, String> {
    if argv.is_empty() || argv[0].is_empty() { return Err("argv must not be empty".into()); }
    if !(1..=3_600_000).contains(&timeout_ms) || !(1..=16_777_216).contains(&max_output) {
        return Err("timeout/output limit outside supported range".into());
    }
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]).current_dir(root).envs(env)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(unix)] {
        use std::os::unix::process::CommandExt;
        unsafe { cmd.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 { return Err(std::io::Error::last_os_error()); }
            Ok(())
        }); }
    }
    let mut child = cmd.spawn().map_err(|_| format!("cannot start {}", argv[0]))?;
    let stdout = pipe(child.stdout.take().unwrap(), max_output);
    let stderr = pipe(child.stderr.take().unwrap(), max_output);
    let mut stdin = child.stdin.take().unwrap();
    thread::spawn(move || { if let Some(text) = input { let _ = stdin.write_all(text.as_bytes()); } });
    let started = Instant::now(); let mut timed_out = false;
    let code = loop {
        match child.try_wait().map_err(|_| "cannot observe process")? {
            Some(status) => break status.code().unwrap_or(1),
            None if started.elapsed() >= Duration::from_millis(timeout_ms) => {
                timed_out = true;
                #[cfg(unix)] unsafe { libc::kill(-(child.id() as i32), libc::SIGKILL); }
                #[cfg(windows)] { let _ = Command::new("taskkill").args(["/PID", &child.id().to_string(), "/T", "/F"]).output(); }
                let _ = child.kill(); let _ = child.wait(); break 124;
            }
            None => thread::sleep(Duration::from_millis(10)),
        }
    };
    // Descendants retaining inherited pipes must not prevent a bounded return.
    let (out, out_clip) = stdout.recv_timeout(Duration::from_secs(1)).unwrap_or((Vec::new(), true));
    let (err, err_clip) = stderr.recv_timeout(Duration::from_secs(1)).unwrap_or((Vec::new(), true));
    Ok(Output { code, stdout: String::from_utf8_lossy(&out).into(), stderr: String::from_utf8_lossy(&err).into(),
                timed_out, truncated: out_clip || err_clip })
}
fn github_token(root: &Path) -> Option<String> {
    for key in ["GH_TOKEN", "GITHUB_TOKEN"] {
        if let Ok(s) = std::env::var(key) { if !s.is_empty() { return Some(s); } }
    }
    let args = ["git", "-c", &format!("safe.directory={}", root.display()), "credential", "fill"]
        .into_iter().map(str::to_string).collect::<Vec<_>>();
    let env = BTreeMap::from([("GIT_TERMINAL_PROMPT".into(), "0".into())]);
    if let Ok(p) = execute(root, &args, Some("protocol=https\nhost=github.com\n\n".into()), &env, 15_000, OUTPUT) {
        if let Some(s) = p.stdout.lines().find_map(|s| s.strip_prefix("password=")) { return Some(s.into()); }
    }
    for home in homes(root) {
        if let Ok(text) = fs::read_to_string(home.join(".config/gh/hosts.yml")) {
            if let Some(s) = text.lines().find_map(|s| s.trim().strip_prefix("oauth_token:")) {
                return Some(s.trim().trim_matches(['"', '\'']).into());
            }
        }
    }
    None
}

fn git_config(env: &mut BTreeMap<String,String>, key: &str, value: String) {
    let start = env.get("GIT_CONFIG_COUNT").cloned().or_else(|| std::env::var("GIT_CONFIG_COUNT").ok())
        .and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    env.insert(format!("GIT_CONFIG_KEY_{start}"),key.into());
    env.insert(format!("GIT_CONFIG_VALUE_{start}"),value);
    env.insert("GIT_CONFIG_COUNT".into(),(start+1).to_string());
}

fn command(root: &Path, p: &Value) -> Answer {
    let mut argv = p.get("argv").and_then(Value::as_array).ok_or("argv must be a list")?
        .iter().map(|v| v.as_str().map(str::to_string).ok_or("argv items must be text".to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let cwd = scoped(root, p.get("cwd").and_then(Value::as_str).unwrap_or("."))?;
    let mut env = BTreeMap::new(); let mut private = Vec::new();
    // Trust only the explicitly selected workspace, only in this child tree.
    // Cargo/Go VCS probes inherit the same scoped Git metadata access.
    git_config(&mut env,"safe.directory",root.display().to_string());
    let exe = argv.first().and_then(|s| Path::new(s).file_stem()).and_then(|s| s.to_str()).unwrap_or("");
    if ["bash", "sh", "pwsh", "powershell"].contains(&exe) && argv.iter().any(|s| ["-c", "-Command", "-EncodedCommand"].contains(&s.as_str())) {
        return Err("use argument vectors or maintained external scripts, not generated shell programs".into());
    }
    if exe == "git" {
        if argv.iter().any(|s| ["--force", "-f", "--force-with-lease", "--no-verify"].contains(&s.as_str()) || s.starts_with("--force=")) {
            return Err("forced Git updates and hook bypass are forbidden".into());
        }
        argv.splice(1..1, ["-c".into(), format!("safe.directory={}", cwd.display())]);
        env.insert("GIT_TERMINAL_PROMPT".into(), "0".into());
        if argv.iter().any(|s| ["fetch", "push", "ls-remote"].contains(&s.as_str())) {
            if let Some(token) = github_token(root) {
                // URL-scoped headers stay in the child environment, not argv.
                let auth = format!("Authorization: Basic {}", base64(format!("x-access-token:{token}").as_bytes()));
                for url in ["https://proxy.cflmy.top/github.com/", "https://github.com/"] {
                    git_config(&mut env,&format!("http.{url}.extraheader"),auth.clone());
                }
                private.push(token); private.push(auth);
                argv.splice(1..1, ["-c".into(), "http.version=HTTP/1.1".into(), "-c".into(), "http.postBuffer=524288000".into()]);
            }
        }
    }
    if let Some(e) = p.get("env").and_then(Value::as_object) {
        for (k, v) in e { env.insert(k.clone(), v.as_str().ok_or("env values must be text")?.into()); }
    }
    // Child Git trace output could expose internally attached authentication.
    for key in ["GIT_TRACE", "GIT_TRACE_CURL", "GIT_CURL_VERBOSE"] { env.insert(key.into(), "0".into()); }
    let timeout = p.get("timeout_ms").and_then(Value::as_u64)
        .unwrap_or_else(|| p.get("timeout").and_then(Value::as_u64).unwrap_or(120) * 1000);
    let max = p.get("max_output").and_then(Value::as_u64).unwrap_or(OUTPUT as u64) as usize;
    let out = execute(&cwd, &argv, p.get("stdin").and_then(Value::as_str).map(str::to_string), &env, timeout, max)?;
    Ok(json!({"code":out.code,"stdout":redact(out.stdout,&private),"stderr":redact(out.stderr,&private),
        "timed_out":out.timed_out,"truncated":out.truncated}))
}
fn http_json(root: &Path, p: &Value) -> Answer {
    let path = field(p, "path")?;
    if !path.starts_with("/repos/cflmy/marqdo/") || path.contains("..") { return Err("GitHub path is outside the project".into()); }
    let method = p.get("method").and_then(Value::as_str).unwrap_or("GET");
    if !["GET", "PATCH", "POST"].contains(&method) { return Err("unsupported GitHub method".into()); }
    let bases = std::env::var("MARQDO_RELEASE_API_BASE").map(|s| vec![s])
        .unwrap_or_else(|_| vec!["https://proxy.cflmy.top/api.github.com".into(), "https://api.github.com".into()]);
    let token = github_token(root);
    for (i, base) in bases.iter().enumerate() {
        let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(35)).build();
        let url = format!("{}{path}", base.trim_end_matches('/'));
        let mut req = agent.request(method, &url).set("Accept", "application/vnd.github+json")
            .set("User-Agent", "marqdo-engineering/1").set("X-GitHub-Api-Version", "2022-11-28");
        if let Some(token) = &token { req = req.set("Authorization", &format!("Bearer {token}")); }
        let response = if let Some(body) = p.get("data").filter(|v| !v.is_null()) {
            req.set("Content-Type", "application/json").send_string(&body.to_string())
        } else { req.call() };
        match response {
            Ok(r) => {
                let body = read_bounded(r.into_reader(), HTTP_MAX)
                    .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|_| "GitHub response is not JSON".into()));
                match body {
                    Ok(value) => return Ok(value),
                    Err(_) if method != "POST" && i + 1 < bases.len() => continue,
                    Err(message) => return Err(message),
                }
            }
            Err(ureq::Error::Status(404,_)) if p.get("missing").and_then(Value::as_bool).unwrap_or(false) => return Ok(Value::Null),
            Err(ureq::Error::Status(code,_)) => {
                if method == "POST" || i + 1 == bases.len() || ![429,500,502,503,504].contains(&code) {
                    return Err(format!("GitHub HTTP {code}"));
                }
            }
            Err(_) if method == "POST" || i + 1 == bases.len() => return Err("GitHub network request failed; uncertain mutations are not replayed".into()),
            Err(_) => {}
        }
    }
    Err("GitHub request failed after proxy and direct routes".into())
}
fn archive(root: &Path, p: &Value) -> Answer {
    let path = scoped(root, field(p,"path")?)?;
    let file = File::open(path).map_err(|_|"cannot open archive")?;
    let mut zip = zip::ZipArchive::new(file).map_err(|_|"invalid ZIP")?;
    if zip.len() > 100000 { return Err("ZIP has too many entries".into()); }
    let mut seen=std::collections::HashSet::new();
    let mut entries = Vec::new(); let mut metadata = serde_json::Map::new(); let mut total = 0u64; let mut actual = 0u64;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(|_|"cannot read ZIP entry")?;
        let name = f.name().replace('\\', "/");
        if !seen.insert(name.clone()) || name.as_bytes().get(1)==Some(&b':') {return Err("duplicate or drive-qualified ZIP path".into());}
        if Path::new(&name).components().any(|c| matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_))) {
            return Err("unsafe ZIP path".into());
        }
        if f.unix_mode().is_some_and(|m|m & 0o170000 == 0o120000) { return Err("ZIP symlink is not supported".into()); }
        total = total.checked_add(f.size()).ok_or("ZIP expansion overflow")?;
        if total > 2 * 1024 * 1024 * 1024 { return Err("ZIP expands beyond 2 GiB".into()); }
        let mut selected = Vec::new();
        let keep = ["VERSION","ext/VERSION","Cargo.toml","extension/package.json"].contains(&name.as_str());
        let mut buf=[0u8;65536];
        loop {
            let n=f.read(&mut buf).map_err(|_|"ZIP content/CRC validation failed")?;
            if n==0 {break;}
            actual = actual.checked_add(n as u64).ok_or("ZIP expansion overflow")?;
            if actual > 2 * 1024 * 1024 * 1024 { return Err("ZIP expands beyond 2 GiB".into()); }
            if keep && selected.len()<16384 {selected.extend_from_slice(&buf[..n.min(16384-selected.len())]);}
        }
        if keep { metadata.insert(name.clone(),json!(String::from_utf8(selected).map_err(|_|"ZIP metadata is not UTF-8")?)); }
        entries.push(name);
    }
    Ok(json!({"entries":entries,"metadata":metadata}))
}
fn hmac(key:&[u8], data:&str)->Vec<u8> {
    let mut mac=Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts all key lengths");
    mac.update(data.as_bytes());mac.finalize().into_bytes().to_vec()
}
fn r2_settings(root:&Path)->Result<BTreeMap<String,String>,String> {
    let mut settings=std::env::vars().filter(|(k,_)|k.starts_with("R2_")).collect::<BTreeMap<_,_>>();
    for home in homes(root) {
        if let Ok(text)=fs::read_to_string(home.join(".marqdo/r2.env")) {
            for line in text.lines().map(str::trim).filter(|s|!s.starts_with('#')) {
                if let Some((k,v))=line.split_once('=') {settings.entry(k.trim().into()).or_insert_with(||v.trim().trim_matches(['\'','"']).into());}
            }
        }
    }
    for k in ["R2_ENDPOINT","R2_BUCKET","R2_ACCESS_KEY_ID","R2_SECRET_ACCESS_KEY"] {
        if settings.get(k).is_none_or(String::is_empty) {return Err(format!("missing {k}; configure ~/.marqdo/r2.env"));}
    }
    Ok(settings)
}
fn r2_put(root:&Path,p:&Value)->Answer {
    let s=r2_settings(root)?;
    let key=field(p,"key")?;
    if !key.bytes().all(|b|b.is_ascii_alphanumeric() || b"/.-_".contains(&b)) || key.contains("..") {
        return Err("unsafe R2 key".into());
    }
    let body=if let Some(path)=p.get("path").and_then(Value::as_str) {
        read_bounded(File::open(scoped(root,path)?).map_err(|_|"cannot open upload")?,256*1024*1024)?
    } else {field(p,"text")?.as_bytes().to_vec()};
    let digest=hash(&body);
    if let Some(expected)=p.get("sha256").and_then(Value::as_str) {
        if digest!=expected {return Err("upload bytes changed after verification".into());}
    }
    let endpoint=s["R2_ENDPOINT"].trim_end_matches('/');
    let host=endpoint.split_once("://").ok_or("R2 endpoint needs a scheme")?.1;
    if host.contains('/') || !(endpoint.starts_with("https://") || endpoint.starts_with("http://127.0.0.1:")) {
        return Err("R2 endpoint must be HTTPS (loopback HTTP is for isolated tests)".into());
    }
    let uri=format!("/{}/{key}",s["R2_BUCKET"]);
    let date=chrono::Utc::now();let amz=date.format("%Y%m%dT%H%M%SZ").to_string();let day=date.format("%Y%m%d").to_string();
    let region=s.get("R2_REGION").map(String::as_str).unwrap_or("auto");
    let mime=if key.ends_with("/VERSION"){"text/plain; charset=utf-8"}else{"application/zip"};
    let headers=format!("content-type:{mime}\nhost:{host}\nx-amz-content-sha256:{digest}\nx-amz-date:{amz}\n");
    let signed="content-type;host;x-amz-content-sha256;x-amz-date";
    let canonical=format!("PUT\n{uri}\n\n{headers}\n{signed}\n{digest}");
    let scope=format!("{day}/{region}/s3/aws4_request");
    let string=format!("AWS4-HMAC-SHA256\n{amz}\n{scope}\n{}",hash(canonical.as_bytes()));
    let k=hmac(format!("AWS4{}",s["R2_SECRET_ACCESS_KEY"]).as_bytes(),&day);
    let k=hmac(&k,region);let k=hmac(&k,"s3");let k=hmac(&k,"aws4_request");
    let auth=format!("AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed}, Signature={}",s["R2_ACCESS_KEY_ID"],hex(&hmac(&k,&string)));
    let response=ureq::AgentBuilder::new().timeout(Duration::from_secs(180)).build()
        .put(&format!("{endpoint}{uri}")).set("Content-Type",mime).set("x-amz-content-sha256",&digest)
        .set("x-amz-date",&amz).set("Authorization",&auth).send_bytes(&body);
    match response {
        Ok(r) if [200,201].contains(&r.status()) => Ok(json!({"key":key,"sha256":digest,"size":body.len()})),
        Ok(r)=>Err(format!("R2 HTTP {}",r.status())),
        Err(ureq::Error::Status(code,_))=>Err(format!("R2 HTTP {code}")),
        Err(_)=>Err("R2 upload failed or timed out; verify before retrying".into())
    }
}

fn git_inspect(root:&Path)->Answer {
    if !root.join(".git").exists() {return Err("inspect requires a Git checkout".into());}
    let query=|args:&[&str]|->Result<String,String>{
        let argv=std::iter::once("git").chain(args.iter().copied()).map(|s|json!(s)).collect::<Vec<_>>();
        let out=command(root,&json!({"argv":argv}))?;
        if out["code"]!=0 {return Err("Git metadata query failed".into());}
        Ok(out["stdout"].as_str().unwrap_or("").trim().into())
    };
    Ok(json!({"root":root.display().to_string(),"branch":query(&["branch","--show-current"])?,
        "commit":query(&["rev-parse","HEAD"])?,"changes":query(&["status","--porcelain"])?,
        "tags":query(&["tag","--list"])?.lines().collect::<Vec<_>>(),
        "extension_on_main":query(&["ls-files","vscode-extension"])?,
        "origin":query(&["remote","get-url","origin"])?}))
}
fn base64(bytes:&[u8])->String {
    const C:&[u8]=b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out=String::new();
    for part in bytes.chunks(3) {
        let a=part[0] as usize;let b=*part.get(1).unwrap_or(&0) as usize;let c=*part.get(2).unwrap_or(&0) as usize;
        out.push(C[a>>2] as char);out.push(C[((a&3)<<4)|(b>>4)] as char);
        out.push(if part.len()>1 {C[((b&15)<<2)|(c>>6)] as char}else{'='});
        out.push(if part.len()>2 {C[c&63] as char}else{'='});
    }
    out
}



fn expression(e:&marqdo::ast::Expr)->Option<String> {
    use marqdo::ast::{Expr,Literal,InterpPart,UnaryOp,BinaryOp,IndexKey};
    Some(match e {
        Expr::Literal(Literal::None)=>"None".into(),
        Expr::Literal(Literal::Bool(v))=>if *v{"True".into()}else{"False".into()},
        Expr::Literal(Literal::Int(v))=>v.to_string(),
        Expr::Literal(Literal::Num(v))=>{let s=v.to_string();if s.contains('.')||s.contains('e')||s.contains('E'){s}else{format!("{s}.0")}},
        Expr::Literal(Literal::Text(v))=>{
            if v.contains('`'){return None;} serde_json::to_string(v).ok()?
        },
        Expr::Var(v)=>format!("`{v}`"),
        Expr::Interp(parts)=>{
            let mut text=String::new();
            for p in parts {match p {
                InterpPart::Lit(v)=>{if v.contains('`'){return None;}text.push_str(v);},
                InterpPart::Var(v)=>text.push_str(&format!("`{v}`")),
                _=>return None
            }}
            serde_json::to_string(&text).ok()?
        },
        Expr::Unary{op,expr}=>match op {UnaryOp::Neg=>format!("-{}",expression(expr)?),UnaryOp::Not=>format!("(not {})",expression(expr)?)},
        Expr::Binary{op,left,right}=>format!("({} {} {})",expression(left)?,match op {
            BinaryOp::Add=>"+",BinaryOp::Sub=>"-",BinaryOp::Mul=>"*",BinaryOp::Div=>"/",
            BinaryOp::Eq=>"==",BinaryOp::Ne=>"!=",BinaryOp::Lt=>"<",BinaryOp::Le=>"<=",
            BinaryOp::Gt=>">",BinaryOp::Ge=>">=",BinaryOp::And=>"and",BinaryOp::Or=>"or"},expression(right)?),
        Expr::Call(call)=>call_source(call,false)?,
        Expr::List(items)=>format!("[{}]",items.iter().map(expression).collect::<Option<Vec<_>>>()?.join(", ")),
        // Dynamic map construction is deliberately not emitted as an alternate
        // author DSL: keep its real bound GFM table at the structural boundary.
        Expr::Map(_)|Expr::Formula(_)|Expr::Code(_)=>return None,
        Expr::Index{base,label}=>format!("access({}, {})",expression(base)?,match label {
            IndexKey::Lit(v)=>serde_json::to_string(v).ok()?,IndexKey::Var(v)=>format!("`{v}`")}),
        Expr::Occluded{label,payload}=>format!("[{}]({})",label,payload.as_deref().map(expression).transpose_option()?),
    })
}
trait OptionalText {fn transpose_option(self)->Option<String>;}
impl OptionalText for Option<Option<String>> {
    fn transpose_option(self)->Option<String>{match self{None=>Some(String::new()),Some(v)=>v}}
}
fn call_source(call:&marqdo::ast::CallExpr,statement:bool)->Option<String> {
    use marqdo::ast::Arg;
    let call=call.with_modifiers_expanded().ok()?;
    let target=if let Some(path)=&call.path{path.join(".")}
        else if let Some(recv)=&call.receiver{format!("`{recv}`.{}",call.callee)}
        else{call.callee.clone()};
    let args=call.args.iter().map(|a|match a {
        Arg::Named{name,value}=>Some(format!("{name}={}",expression(value)?)),
        Arg::Positional(value)=>expression(value)
    }).collect::<Option<Vec<_>>>()?;
    if statement || call.receiver.is_some() {
        let target=if target.contains(char::is_whitespace){format!("[{target}]")}else{target};
        Some(format!("> {target} {}",args.join(" ")).trim_end().into())
    } else {
        if target.contains(char::is_whitespace){return None;}
        Some(format!("{target}({})",args.join(", ")))
    }
}
fn semantic_expr(e:&marqdo::ast::Expr)->Value {
    use marqdo::ast::{Expr,Arg};
    match e {
        Expr::Occluded{payload,..}=>payload.as_deref().map(semantic_expr).unwrap_or(json!(["none"])),
        Expr::Call(c)=>{
            let c=c.for_evaluation().unwrap_or_else(|_|c.clone());
            json!(["call",c.callee,c.path,c.receiver,c.args.iter().map(|a|match a{
                Arg::Named{name,value}=>json!(["named",name,semantic_expr(value)]),
                Arg::Positional(v)=>json!(["positional",semantic_expr(v)])
            }).collect::<Vec<_>>()])
        },
        Expr::List(v)=>json!(["list",v.iter().map(semantic_expr).collect::<Vec<_>>()]),
        Expr::Map(v)=>json!(["map",v.iter().map(|(k,v)|json!([k,semantic_expr(v)])).collect::<Vec<_>>()]),
        Expr::Unary{op,expr}=>json!(["unary",format!("{op:?}"),semantic_expr(expr)]),
        Expr::Binary{op,left,right}=>json!(["binary",format!("{op:?}"),semantic_expr(left),semantic_expr(right)]),
        _=>json!(format!("{e:?}"))
    }
}
fn semantic_statements(body:&[marqdo::ast::Stmt])->Vec<Value> {
    use marqdo::ast::{Expr,Stmt};
    body.iter().map(|s|match s {
        Stmt::Assign{name,value,..}=>json!(["assign",name,semantic_expr(value)]),
        Stmt::Return{value,..}=>json!(["return",semantic_expr(value)]),
        Stmt::Call{call,..}=>json!(["eval",semantic_expr(&Expr::Call(call.clone()))]),
        Stmt::Expr{value,..}=>json!(["eval",semantic_expr(value)]),
        Stmt::Branch{arms,..}=>json!(["branch",arms.iter().map(|a|json!([a.condition.as_ref().map(semantic_expr),semantic_statements(&a.body)])).collect::<Vec<_>>()]),
        Stmt::While{condition,body,..}=>json!(["while",semantic_expr(condition),semantic_statements(body)]),
        Stmt::ForEach{item,collection,body,..}=>json!(["foreach",item,collection,semantic_statements(body)])
    }).collect()
}
fn statement_locations(body:&[marqdo::ast::Stmt])->Vec<Value> {
    use marqdo::ast::Stmt;
    let mut out=Vec::new();
    for s in body {match s {
        Stmt::Assign{name,value,span,end_line}=>out.push(json!({"kind":"assign","name":name,"line":span.line,"end_line":end_line,"rendered":expression(value)})),
        Stmt::Return{value,span}=>out.push(json!({"kind":"return","line":span.line,"rendered":expression(value)})),
        Stmt::Call{call,span}=>out.push(json!({"kind":"call","line":span.line,"callee":call.path.as_ref().map(|p|p.join(".")).unwrap_or_else(||call.callee.clone()),"rendered":call_source(call,true)})),
        Stmt::Expr{value,span}=>out.push(json!({"kind":"eval","line":span.line,"rendered":expression(value)})),
        Stmt::Branch{arms,..}=>for arm in arms{out.extend(statement_locations(&arm.body));},
        Stmt::While{body,..}|Stmt::ForEach{body,..}=>out.extend(statement_locations(body))
    }}
    out
}

fn analyze(p:&Value)->Answer {
    use marqdo::ast::{Expr,Function,Literal};
    fn default(e:&Expr)->Value {match e {
        Expr::Literal(Literal::Text(t))=>json!({"text":t}),
        Expr::Literal(Literal::None)=>json!({"source":"None"}),
        Expr::Literal(Literal::Bool(v))=>json!({"source":if *v{"True"}else{"False"}}),
        Expr::Literal(Literal::Int(v))=>json!({"source":v.to_string()}),
        Expr::Literal(Literal::Num(v))=>json!({"source":if v.fract()==0.0 { format!("{v:.1}") } else {v.to_string()}}),
        _=>json!({"debug":format!("{e:?}")})
    }}
    fn units(fs:&[Function],body:bool)->Vec<Value> {fs.iter().map(|f|json!({
        "name":f.name,"line":f.span.line,"level":f.level,"base":f.base,
        "params":f.params.iter().map(|p|json!({"name":p.name,"line":p.span.line,"inferred":p.inferred,"default":p.default.as_ref().map(default)})).collect::<Vec<_>>(),
        "contract":f.contract.as_ref().map(|c|json!({"params":c.params.iter().map(|p|json!({"name":p.name,"ty":p.ty.name(),"desc":p.desc})).collect::<Vec<_>>(),"returns":c.returns.as_ref().map(|r|json!({"ty":r.ty.name(),"desc":r.desc}))})),
        "children":units(&f.children,body),"semantic":semantic_statements(&f.body),"statements":statement_locations(&f.body),"body":if body{Some(format!("{:?}",f.body))}else{None}
    })).collect()}
    let source=field(p,"source")?;
    let module=marqdo::parse::parse_source(source).map_err(|e|format!("{e:#}"))?;
    let diagnostics=marqdo::check::check_module(&module,None).iter().map(|d|d.to_json()).collect::<Vec<_>>();
    let ratios=marqdo::narrative_ratio::measure_source_units(source,&module.functions).iter()
        .map(|r|json!({"name":r.name,"line":r.line,"narrative":r.narrative,"total":r.total,"ratio":r.ratio})).collect::<Vec<_>>();
    Ok(json!({"schema_version":1,"functions":units(&module.functions,p.get("include_body").and_then(Value::as_bool).unwrap_or(false)),
        "ratios":ratios,"diagnostics":diagnostics}))
}


fn gzip_text(p: &Value) -> Answer {
    use base64::Engine;
    let data = p.get("data").ok_or("missing portable bytes")?;
    if data.get("_type").and_then(Value::as_str) != Some("bytes") {
        return Err("gzip expects portable bytes".into());
    }
    let encoded = field(data,"base64")?;
    if encoded.len() > 24 * 1024 * 1024 { return Err("compressed input exceeds limit".into()); }
    let bytes = base64::engine::general_purpose::STANDARD.decode(encoded).map_err(|_| "invalid base64 bytes")?;
    let limit = p.get("max_bytes").and_then(Value::as_u64).unwrap_or(16 * 1024 * 1024);
    if limit == 0 || limit > 64 * 1024 * 1024 { return Err("invalid gzip output limit".into()); }
    let plain = read_bounded(flate2::read::MultiGzDecoder::new(bytes.as_slice()),limit)?;
    Ok(json!(String::from_utf8(plain).map_err(|_| "gzip text is not UTF-8")?))
}

fn dispatch(request:&Value)->Answer {
    let root=PathBuf::from(request.get("root").and_then(Value::as_str).unwrap_or(".")).canonicalize().map_err(|_|"root does not exist")?;
    let op=field(request,"operation")?;
    let empty=json!({});let p=request.get("payload").unwrap_or(&empty);
    match op {
        "command"=>command(&root,p),
        "git_inspect"=>git_inspect(&root),
        "github"=>http_json(&root,p),
        "archive"=>archive(&root,p),
        "gzip_text"=>gzip_text(p),
        "analyze"=>analyze(p),
        "r2_put"=>r2_put(&root,p),
        _=>Err("unknown engineering boundary operation".into())
    }
}
unsafe extern "C" fn engineering_request(args:*const c_char,out:*mut *mut c_char,err:*mut *mut c_char)->c_int {
    let args=match parse_args(args){Ok(v)=>v,Err(e)=>{set_err(err,&e);return 1;}};
    let request=args.get("request").unwrap_or(&args);
    let result=match dispatch(request) {
        Ok(value)=>json!({"ok":true,"value":value,"error":null}),
        Err(message)=>json!({"ok":false,"value":null,"error":{"code":"engineering.boundary","message":message,"operation":request.get("operation")}})
    };
    set_out(out,&result.to_string());0
}
marqdo_plugin!(2,[("engineering_request","request",engineering_request)]);

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize,Ordering};
    static NEXT:AtomicUsize=AtomicUsize::new(0);
    fn fixture()->PathBuf {
        let p=std::env::temp_dir().join(format!("mq-engineering-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
        fs::create_dir_all(&p).unwrap(); p
    }
    #[cfg(unix)]
    #[test]
    fn argv_input_is_literal_and_noisy_output_is_bounded() {
        let dir=fixture();
        let input="quotes \" ' $HOME $(never-run) 😀\n".repeat(1000);
        let result=command(&dir,&json!({"argv":["/bin/cat"],"stdin":input})).unwrap();
        assert_eq!(result["code"],0);assert_eq!(result["stdout"],input);
        let result=command(&dir,&json!({"argv":["/usr/bin/head","-c","65536","/dev/zero"],"max_output":128})).unwrap();
        assert_eq!(result["code"],0);assert_eq!(result["stdout"].as_str().unwrap().len(),128);
        assert_eq!(result["truncated"],true);
        fs::remove_dir_all(dir).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn timeout_and_nonzero_exit_remain_observable() {
        let dir=fixture();let started=Instant::now();
        let result=command(&dir,&json!({"argv":["/bin/sleep","2"],"timeout_ms":20})).unwrap();
        assert_eq!(result["timed_out"],true);assert_eq!(result["code"],124);
        assert!(started.elapsed()<Duration::from_secs(2));
        let result=command(&dir,&json!({"argv":["/bin/false"]})).unwrap();
        assert_eq!(result["code"],1);assert_eq!(result["timed_out"],false);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn unsafe_transport_shortcuts_are_rejected_before_spawn() {
        let dir=fixture();
        for argv in [json!(["git","push","--force"]),json!(["git","commit","--no-verify"]),json!(["bash","-c","echo forbidden"])] {
            assert!(command(&dir,&json!({"argv":argv})).is_err());
        }
        fs::remove_dir_all(dir).unwrap();
    }
    fn make_zip(path:&Path,name:&str,payload:&[u8]) {
        let mut out=zip::ZipWriter::new(File::create(path).unwrap());
        out.start_file(name,zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored)).unwrap();
        out.write_all(payload).unwrap();out.finish().unwrap();
    }
    #[test]
    fn zip_inspection_checks_metadata_paths_and_crc_without_extraction() {
        let dir=fixture();let path=dir.join("fixture.zip");
        make_zip(&path,"ext/VERSION",b"2.6.0\r\n");
        let result=archive(&dir,&json!({"path":"fixture.zip"})).unwrap();
        assert_eq!(result["metadata"]["ext/VERSION"],"2.6.0\r\n");
        assert!(!dir.join("ext").exists());
        make_zip(&path,"../escape",b"untrusted");
        assert!(archive(&dir,&json!({"path":"fixture.zip"})).is_err());
        make_zip(&path,"payload",b"unique CRC fixture payload");
        let mut bytes=fs::read(&path).unwrap();
        let index=bytes.windows(b"unique CRC fixture payload".len()).position(|x|x==b"unique CRC fixture payload").unwrap();
        bytes[index]^=1;fs::write(&path,bytes).unwrap();
        assert!(archive(&dir,&json!({"path":"fixture.zip"})).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn gzip_text_validates_format_utf8_and_expansion_limit() {
        use base64::Engine;
        fn compressed(payload:&[u8])->Vec<u8> {
            let mut writer=flate2::write::GzEncoder::new(Vec::new(),flate2::Compression::default());
            writer.write_all(payload).unwrap();writer.finish().unwrap()
        }
        let mut bytes=compressed(b"Package: marqdo\n");bytes.extend(compressed(b"Version: 2.6.0\n"));
        let data=json!({"_type":"bytes","base64":base64::engine::general_purpose::STANDARD.encode(&bytes)});
        assert_eq!(gzip_text(&json!({"data":data})).unwrap(),"Package: marqdo\nVersion: 2.6.0\n");
        assert!(gzip_text(&json!({"data":data,"max_bytes":2})).is_err());
        assert!(gzip_text(&json!({"data":{"_type":"bytes","base64":"invalid"}})).is_err());
        let data=json!({"_type":"bytes","base64":base64::engine::general_purpose::STANDARD.encode(compressed(&[0xff]))});
        assert!(gzip_text(&json!({"data":data})).is_err());
    }
    #[test]
    fn analysis_uses_the_real_parser_without_loading_imports() {
        let source="---\nmarkup: occlusion\nimport:\n  absent: /does/not/exist.mq.md\n---\n## answer\n\n完成计算后返回*42*。\n";
        let result=analyze(&json!({"source":source})).unwrap();
        assert_eq!(result["functions"][0]["name"],"answer");
        assert!(result["diagnostics"].as_array().unwrap().is_empty());
        assert_eq!(result["functions"][0]["semantic"][0][0],"return");
    }
}
