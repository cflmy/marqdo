//! Bounded filesystem tasks. Public authors use lib/fs, lib/path and lib/bytes.
use super::{as_text, encoding, fs as legacy, result, sandbox_fs};
use crate::{
    host::{HostContext, HostFn},
    value::Value,
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
type Args = HashMap<String, Value>;
const MAX_READ: usize = 64 * 1024 * 1024;
fn arg<'a>(b: &'a Args, k: &str) -> Result<&'a Value, String> {
    b.get(k).ok_or_else(|| format!("missing {k}"))
}
fn text<'a>(b: &'a Args, k: &str) -> Result<&'a str, String> {
    as_text(arg(b, k)?, k)
}
fn number(b: &Args, k: &str) -> Result<u64, String> {
    match arg(b, k)? {
        Value::Int(n) if *n >= 0 => Ok(*n as u64),
        _ => Err(format!("{k} must be nonnegative integer")),
    }
}
fn flag(b: &Args, k: &str) -> bool {
    matches!(b.get(k), Some(Value::Bool(true)))
}
fn native(ctx: &HostContext) -> Result<(), String> {
    if ctx.sandbox_session.is_some() {
        Err("operation unavailable in CoW sandbox; use write_text/remove_file".into())
    } else {
        Ok(())
    }
}
fn writable(ctx: &HostContext) -> Result<(), String> {
    if ctx.allow_fs_write() {
        Ok(())
    } else {
        Err("filesystem write denied by host policy".into())
    }
}
fn path(ctx: &HostContext, b: &Args, k: &str) -> Result<PathBuf, String> {
    ctx.resolve_path(text(b, k)?)
}
fn ioerr(e: std::io::Error) -> String {
    let code = match e.kind() {
        std::io::ErrorKind::NotFound => "not_found",
        std::io::ErrorKind::PermissionDenied => "permission_denied",
        std::io::ErrorKind::AlreadyExists => "conflict",
        _ => "operation_failed",
    };
    format!("{code}: {e}")
}
fn limit(b: &Args, k: &str) -> Result<usize, String> {
    let n = number(b, k)?;
    if n == 0 || n > MAX_READ as u64 {
        return Err(format!("{k} must be 1..{MAX_READ}"));
    }
    Ok(n as usize)
}
pub(super) fn bytes_value(bytes: &[u8]) -> Value {
    Value::Map(vec![
        ("_type".into(), Value::Text("bytes".into())),
        ("base64".into(), Value::Text(encoding::encode_base64(bytes))),
    ])
}
pub(super) fn bytes(v: &Value) -> Result<Vec<u8>, String> {
    let Value::Map(fields) = v else {
        return Err("expected bytes record from lib/bytes or fs.read_bytes".into());
    };
    if !fields
        .iter()
        .any(|(k, v)| k == "_type" && v == &Value::Text("bytes".into()))
    {
        return Err("expected _type bytes".into());
    }
    let text = fields
        .iter()
        .find(|(k, _)| k == "base64")
        .ok_or("missing bytes.base64")?;
    encoding::decode_base64(as_text(&text.1, "base64")?)
}
fn read(ctx: &HostContext, p: &Path, max: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    if let Some(bytes) = sandbox_fs::cow_read_bytes(ctx, p)? {
        if bytes.len() > max {
            return Err("file exceeds max_bytes".into());
        }
        return Ok(bytes);
    }
    fs::File::open(p)
        .map_err(ioerr)?
        .take(max as u64 + 1)
        .read_to_end(&mut out)
        .map_err(ioerr)?;
    if out.len() > max {
        return Err("file exceeds max_bytes".into());
    }
    Ok(out)
}
fn atomic(ctx: &HostContext, p: &Path, body: &[u8]) -> Result<(), String> {
    writable(ctx)?;
    if sandbox_fs::cow_write_bytes(ctx, p, body)? {
        return Ok(());
    }
    let parent = p.parent().ok_or("file needs parent")?;
    fs::create_dir_all(parent).map_err(ioerr)?;
    // create_new reserves the sibling name. Never truncate an existing temp file.
    let mut reserved = None;
    for _ in 0..128 {
        let mut entropy = [0u8; 16];
        getrandom::getrandom(&mut entropy).map_err(|e| e.to_string())?;
        let name = entropy
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let temp = parent.join(format!(".mq-{name}.tmp"));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
        {
            Ok(file) => {
                reserved = Some((temp, file));
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(ioerr(e)),
        }
    }
    let (temp, mut file) = reserved.ok_or("cannot reserve atomic write file")?;
    let result = (|| {
        file.write_all(body).map_err(ioerr)?;
        file.sync_all().map_err(ioerr)?;
        drop(file);
        fs::rename(&temp, p).map_err(ioerr)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
pub(super) fn call(ctx: &mut HostContext, id: HostFn, b: &Args) -> Result<Value, String> {
    let answer = match id {
        HostFn::FsTryReadText => {
            let answer = (|| {
                let data = read(ctx, &path(ctx, b, "path")?, 8 * 1024 * 1024)?;
                String::from_utf8(data)
                    .map(Value::Text)
                    .map_err(|e| format!("invalid_encoding: {e}"))
            })();
            return Ok(result::capture("fs.read_text", answer));
        }
        HostFn::FsTryWriteText => {
            return Ok(result::capture(
                "fs.write_text",
                legacy::write_text(ctx, arg(b, "path")?, arg(b, "text")?),
            ))
        }
        HostFn::TryJsonParse => {
            return Ok(result::capture(
                "json.parse",
                super::json::parse(arg(b, "text")?).map_err(|e| format!("invalid_json: {e}")),
            ))
        }
        HostFn::TryCsvParse => {
            return Ok(result::capture(
                "csv.parse",
                super::csv::parse(arg(b, "text")?).map_err(|e| format!("invalid_csv: {e}")),
            ))
        }
        HostFn::BytesFromText => bytes_value(text(b, "text")?.as_bytes()),
        HostFn::BytesFromBase64 => bytes_value(&encoding::decode_base64(text(b, "text")?)?),
        HostFn::BytesToText => Value::Text(
            String::from_utf8(bytes(arg(b, "data")?)?)
                .map_err(|e| format!("bytes are not UTF-8: {e}"))?,
        ),
        HostFn::BytesToBase64 => Value::Text(encoding::encode_base64(&bytes(arg(b, "data")?)?)),
        HostFn::BytesLen => Value::Int(bytes(arg(b, "data")?)?.len() as i64),
        HostFn::BytesSlice => {
            let data = bytes(arg(b, "data")?)?;
            let start = usize::try_from(number(b, "start")?)
                .unwrap_or(usize::MAX)
                .min(data.len());
            let end = usize::try_from(number(b, "end")?)
                .unwrap_or(usize::MAX)
                .min(data.len());
            bytes_value(if start > end { &[] } else { &data[start..end] })
        }
        HostFn::FsReadBytes => {
            bytes_value(&read(ctx, &path(ctx, b, "path")?, limit(b, "max_bytes")?)?)
        }
        HostFn::FsWriteBytes => {
            let p = path(ctx, b, "path")?;
            atomic(ctx, &p, &bytes(arg(b, "data")?)?)?;
            Value::None
        }
        HostFn::FsReadChunk => {
            let p = path(ctx, b, "path")?;
            let offset = number(b, "offset")?;
            let size = limit(b, "size")?;
            let mut data = vec![0u8; size];
            let (n, total) = if let Some(bytes) = sandbox_fs::cow_read_bytes(ctx, &p)? {
                let start = usize::try_from(offset)
                    .unwrap_or(usize::MAX)
                    .min(bytes.len());
                let end = start.saturating_add(size).min(bytes.len());
                data[..end - start].copy_from_slice(&bytes[start..end]);
                (end - start, bytes.len() as u64)
            } else {
                let mut file = fs::File::open(&p).map_err(ioerr)?;
                let total = file.metadata().map_err(ioerr)?.len();
                file.seek(SeekFrom::Start(offset)).map_err(ioerr)?;
                let n = file.take(size as u64).read(&mut data).map_err(ioerr)?;
                (n, total)
            };
            data.truncate(n);
            let next = offset.checked_add(n as u64).ok_or("offset overflow")?;
            Value::Map(vec![
                ("data".into(), bytes_value(&data)),
                ("next_offset".into(), Value::Int(next as i64)),
                ("eof".into(), Value::Bool(next >= total)),
            ])
        }
        HostFn::FsAppendBytes => {
            writable(ctx)?;
            let p = path(ctx, b, "path")?;
            let data = bytes(arg(b, "data")?)?;
            if ctx.sandbox_session.is_some() {
                let mut body = read(ctx, &p, MAX_READ).or_else(|e| {
                    if matches!(legacy::exists(ctx, arg(b, "path")?), Ok(Value::Bool(false))) {
                        Ok(vec![])
                    } else {
                        Err(e)
                    }
                })?;
                body.extend_from_slice(&data);
                sandbox_fs::cow_write_bytes(ctx, &p, &body)?;
            } else {
                let mut f = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&p)
                    .map_err(ioerr)?;
                f.write_all(&data).map_err(ioerr)?;
            }
            Value::None
        }
        HostFn::FsAtomicWrite | HostFn::FsWriteChanged => {
            let p = path(ctx, b, "path")?;
            let body = text(b, "text")?.as_bytes();
            if id == HostFn::FsWriteChanged && read(ctx, &p, MAX_READ).ok().as_deref() == Some(body)
            {
                return Ok(Value::Bool(false));
            }
            atomic(ctx, &p, body)?;
            if id == HostFn::FsWriteChanged {
                Value::Bool(true)
            } else {
                Value::None
            }
        }
        HostFn::FsWriteChecked => {
            let change = (|| {
                let p = path(ctx, b, "path")?;
                let expected = text(b, "expected_sha256")?;
                let current = read(ctx, &p, MAX_READ)?;
                let actual = format!("{:x}", Sha256::digest(&current));
                if actual != expected {
                    return Err("conflict: file content changed".into());
                }
                atomic(ctx, &p, text(b, "text")?.as_bytes()).map(|_| Value::None)
            })();
            return Ok(result::capture("fs.write_checked", change));
        }
        HostFn::FsReadLines => {
            let data = read(ctx, &path(ctx, b, "path")?, limit(b, "max_bytes")?)?;
            let text = String::from_utf8(data).map_err(|e| format!("file is not UTF-8: {e}"))?;
            Value::List(text.lines().map(|s| Value::Text(s.into())).collect())
        }
        HostFn::FsWriteLines => {
            let Value::List(lines) = arg(b, "lines")? else {
                return Err("lines must be list of text".into());
            };
            let lines = lines
                .iter()
                .map(|v| as_text(v, "line"))
                .collect::<Result<Vec<_>, _>>()?;
            let mut body = lines.join("\n");
            if !lines.is_empty() {
                body.push('\n')
            };
            atomic(ctx, &path(ctx, b, "path")?, body.as_bytes())?;
            Value::None
        }
        HostFn::FsRemoveFile => {
            writable(ctx)?;
            let p = path(ctx, b, "path")?;
            if p.is_dir() {
                return Err("remove_file refuses directory".into());
            }
            if !matches!(legacy::exists(ctx, arg(b, "path")?), Ok(Value::Bool(true))) {
                return Err("not_found: remove_file target absent".into());
            }
            if !sandbox_fs::cow_remove(ctx, &p)? {
                fs::remove_file(p).map_err(ioerr)?;
            }
            Value::None
        }
        HostFn::FsRemoveEmptyDir => {
            writable(ctx)?;
            native(ctx)?;
            fs::remove_dir(path(ctx, b, "path")?).map_err(ioerr)?;
            Value::None
        }
        HostFn::FsTempDir => {
            writable(ctx)?;
            native(ctx)?;
            let prefix = text(b, "prefix")?;
            if prefix.contains('/') || prefix.contains('\\') || prefix.contains("..") {
                return Err("temp prefix must be simple name".into());
            }
            let mut created = None;
            for _ in 0..128 {
                let mut entropy = [0u8; 16];
                getrandom::getrandom(&mut entropy).map_err(|e| e.to_string())?;
                let name = format!(
                    "{prefix}-{}",
                    entropy
                        .iter()
                        .map(|n| format!("{n:02x}"))
                        .collect::<String>()
                );
                let p = ctx.resolve_path(&name)?;
                match fs::create_dir(p) {
                    Ok(()) => {
                        created = Some(Value::Text(name));
                        break;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(e) => return Err(ioerr(e)),
                }
            }
            created.ok_or("unable to reserve temporary directory")?
        }
        HostFn::FsFileHash => {
            let p = path(ctx, b, "path")?;
            let mut hasher = Sha256::new();
            if let Some(bytes) = sandbox_fs::cow_read_bytes(ctx, &p)? {
                hasher.update(bytes)
            } else {
                let mut f = fs::File::open(p).map_err(ioerr)?;
                let mut buffer = [0u8; 65536];
                loop {
                    let n = f.read(&mut buffer).map_err(ioerr)?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
            }
            Value::Text(format!("{:x}", hasher.finalize()))
        }
        HostFn::FsFind => {
            let root = path(ctx, b, "path")?;
            let pattern = glob(text(b, "pattern")?)?;
            let excluded = patterns(arg(b, "exclude")?)?;
            let max_depth = number(b, "max_depth")? as usize;
            let kind = text(b, "kind")?;
            if !matches!(kind, "file" | "dir" | "all") {
                return Err("find kind must be file/dir/all".into());
            }
            let mut files = Vec::new();
            find(
                ctx,
                &root,
                Path::new(""),
                &pattern,
                &excluded,
                max_depth,
                kind,
                &mut files,
            )?;
            Value::List(files)
        }
        HostFn::FsCopyTree | HostFn::FsSyncTree => {
            writable(ctx)?;
            native(ctx)?;
            let src = path(ctx, b, "src")?;
            let dest = path(ctx, b, "dest")?;
            if dest.starts_with(&src) || src.starts_with(&dest) {
                return Err("copy/sync trees must be disjoint".into());
            }
            if !src.is_dir() {
                return Err("src must be directory".into());
            }
            let excluded = patterns(arg(b, "exclude")?)?;
            let mut files = Vec::new();
            find(
                ctx,
                &src,
                Path::new(""),
                &glob("**")?,
                &excluded,
                256,
                "file",
                &mut files,
            )?;
            let mut jobs = Vec::new();
            for name in files {
                let name = as_text(&name, "name")?;
                let from = ctx.resolve_path(src.join(name).to_str().ok_or("non UTF-8 path")?)?;
                let to = ctx.resolve_path(dest.join(name).to_str().ok_or("non UTF-8 path")?)?;
                if to.exists() {
                    if id == HostFn::FsSyncTree && files_equal(&from, &to)? {
                        continue;
                    }
                    if !flag(b, "overwrite") {
                        return Err(format!("destination conflict: {}", to.display()));
                    }
                }
                jobs.push((from, to, name.to_string()));
            }
            let dry = flag(b, "dry_run");
            let mut changed = Vec::new();
            for (from, to, name) in jobs {
                if !dry {
                    if let Some(parent) = to.parent() {
                        fs::create_dir_all(parent).map_err(ioerr)?;
                    }
                    fs::copy(from, to).map_err(ioerr)?;
                }
                changed.push(Value::Text(name));
            }
            Value::List(changed)
        }
        HostFn::FsRenameMany => {
            writable(ctx)?;
            native(ctx)?;
            let Value::List(rows) = arg(b, "moves")? else {
                return Err("moves must be record list".into());
            };
            let mut jobs = Vec::new();
            let mut destinations = std::collections::HashSet::new();
            let mut sources = std::collections::HashSet::new();
            for row in rows {
                let Value::Map(fields) = row else {
                    return Err("move needs src,dest".into());
                };
                let mut paths = Vec::new();
                for key in ["src", "dest"] {
                    let value = fields
                        .iter()
                        .find(|(k, _)| k == key)
                        .ok_or("move needs src,dest")?;
                    paths.push(ctx.resolve_path(as_text(&value.1, key)?)?);
                }
                if !paths[0].is_file() {
                    return Err("rename_many source must be file".into());
                }
                if !sources.insert(paths[0].clone()) {
                    return Err("conflict: repeated rename source".into());
                }
                if paths[1].exists() || !destinations.insert(paths[1].clone()) {
                    return Err("rename_many destination conflict".into());
                }
                jobs.push((paths.remove(0), paths.remove(0)));
            }
            // Preflight all paths. Execution is ordered and deliberately not transactional.
            if !flag(b, "dry_run") {
                for (from, to) in &jobs {
                    if let Some(parent) = to.parent() {
                        fs::create_dir_all(parent).map_err(ioerr)?;
                    }
                    fs::rename(from, to).map_err(ioerr)?;
                }
            }
            Value::Int(jobs.len() as i64)
        }
        HostFn::PathRelative => {
            let normalized = super::path::normalize(arg(b, "path")?)?;
            let normalized_base = super::path::normalize(arg(b, "base")?)?;
            let path = PathBuf::from(as_text(&normalized, "path")?);
            let base = PathBuf::from(as_text(&normalized_base, "base")?);
            if path.is_absolute() != base.is_absolute() {
                return Err("relative paths must share absolute/relative form".into());
            }
            let a: Vec<_> = path.components().collect();
            let b: Vec<_> = base.components().collect();
            if path.is_absolute() && a.first() != b.first() {
                return Err("relative paths have different roots".into());
            }
            let common = a.iter().zip(&b).take_while(|(a, b)| a == b).count();
            let mut out = PathBuf::new();
            for _ in common..b.len() {
                out.push("..")
            }
            for c in &a[common..] {
                out.push(c.as_os_str())
            }
            Value::Text(if out.as_os_str().is_empty() {
                ".".into()
            } else {
                out.to_string_lossy().replace('\\', "/")
            })
        }
        HostFn::PathWithExtension => {
            let mut p = PathBuf::from(text(b, "path")?);
            let ext = text(b, "extension")?;
            if ext.contains('/') || ext.contains('\\') {
                return Err("extension cannot contain path separator".into());
            }
            p.set_extension(ext.trim_start_matches('.'));
            Value::Text(p.to_string_lossy().replace('\\', "/"))
        }
        _ => return Err("not a filesystem task".into()),
    };
    Ok(answer)
}
fn patterns(v: &Value) -> Result<Vec<regex::Regex>, String> {
    if matches!(v, Value::None) {
        return Ok(vec![]);
    }
    let Value::List(xs) = v else {
        return Err("exclude must be list of patterns".into());
    };
    xs.iter().map(|v| glob(as_text(v, "pattern")?)).collect()
}
fn glob(pattern: &str) -> Result<regex::Regex, String> {
    let mut out = String::from("^");
    let mut cs = pattern.chars().peekable();
    while let Some(c) = cs.next() {
        match c {
            '*' => {
                if cs.peek() == Some(&'*') {
                    cs.next();
                    if cs.peek() == Some(&'/') {
                        cs.next();
                        out.push_str("(?:.*/)?")
                    } else {
                        out.push_str(".*")
                    }
                } else {
                    out.push_str("[^/]*")
                }
            }
            '?' => out.push_str("[^/]"),
            _ => out.push_str(&regex::escape(&c.to_string())),
        }
    }
    out.push('$');
    regex::Regex::new(&out).map_err(|e| e.to_string())
}
fn find(
    ctx: &HostContext,
    root: &Path,
    rel: &Path,
    pattern: &regex::Regex,
    exclude: &[regex::Regex],
    depth: usize,
    kind: &str,
    out: &mut Vec<Value>,
) -> Result<(), String> {
    if depth == 0 {
        return Ok(());
    }
    let mut entries = fs::read_dir(root.join(rel))
        .map_err(ioerr)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(ioerr)?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let child = rel.join(entry.file_name());
        let name = child.to_string_lossy().replace('\\', "/");
        let ty = entry.file_type().map_err(ioerr)?;
        if exclude
            .iter()
            .any(|r| r.is_match(&name) || r.is_match(&format!("{name}/")))
        {
            continue;
        }
        // No symlink traversal. Reads returned by this finder still use resolve_path.
        if ty.is_symlink() {
            continue;
        }
        ctx.resolve_path(entry.path().to_str().ok_or("non UTF-8 path")?)?;
        if pattern.is_match(&name)
            && ((ty.is_file() && kind != "dir") || (ty.is_dir() && kind != "file"))
        {
            if out.len() >= 100_000 {
                return Err("find exceeds 100000 entries".into());
            }
            out.push(Value::Text(name))
        }
        if ty.is_dir() {
            find(ctx, root, &child, pattern, exclude, depth - 1, kind, out)?
        }
    }
    Ok(())
}

fn files_equal(a: &Path, b: &Path) -> Result<bool, String> {
    if fs::metadata(a).map_err(ioerr)?.len() != fs::metadata(b).map_err(ioerr)?.len() {
        return Ok(false);
    }
    let mut a = std::io::BufReader::new(fs::File::open(a).map_err(ioerr)?);
    let mut b = std::io::BufReader::new(fs::File::open(b).map_err(ioerr)?);
    use std::io::BufRead;
    loop {
        let left = a.fill_buf().map_err(ioerr)?;
        let right = b.fill_buf().map_err(ioerr)?;
        if left.is_empty() || right.is_empty() {
            return Ok(left.is_empty() && right.is_empty());
        }
        let n = left.len().min(right.len());
        if left[..n] != right[..n] {
            return Ok(false);
        }
        a.consume(n);
        b.consume(n);
    }
}

#[cfg(test)]
mod overlay_tests {
    use super::*;
    #[test]
    fn byte_append_copy_and_patch_stay_in_the_overlay_until_commit() {
        let root = std::env::temp_dir().join(format!("mq-byte-overlay-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("note.md"), "old text").unwrap();
        let mut ctx =
            HostContext::for_run(Some(&root.join("main.mq.md")), Default::default(), vec![]);
        let session = sandbox_fs::open(Some(root.to_str().unwrap()), &ctx).unwrap();
        let id = session["id"].as_str().unwrap().to_string();
        ctx.sandbox_session = Some(id.clone());
        legacy::text_patch(
            &ctx,
            &Value::Text("note.md".into()),
            &Value::Text("old".into()),
            &Value::Text("new".into()),
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(root.join("note.md")).unwrap(),
            "old text"
        );
        assert_eq!(
            legacy::read_text(&ctx, &Value::Text("note.md".into())).unwrap(),
            Value::Text("new text".into())
        );
        let mut args = Args::from([
            ("path".into(), Value::Text("data.bin".into())),
            ("data".into(), bytes_value(&[0, 255])),
        ]);
        call(&mut ctx, HostFn::FsWriteBytes, &args).unwrap();
        args.insert("data".into(), bytes_value(&[128]));
        call(&mut ctx, HostFn::FsAppendBytes, &args).unwrap();
        legacy::copy_file(
            &ctx,
            &Value::Text("data.bin".into()),
            &Value::Text("copied.bin".into()),
        )
        .unwrap();
        assert_eq!(
            sandbox_fs::cow_read_bytes(&ctx, &root.join("copied.bin"))
                .unwrap()
                .unwrap(),
            [0, 255, 128]
        );
        assert!(!root.join("data.bin").exists());
        assert!(!root.join("copied.bin").exists());
        assert!(legacy::make_dirs(&ctx, &Value::Text("unsupported".into())).is_err());
        assert!(!root.join("unsupported").exists());
        sandbox_fs::discard(&id).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
