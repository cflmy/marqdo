//! Optional ZIP tasks with preflight, expansion limits and no path traversal.
use crate::{
    host::{HostContext, HostFn},
    value::Value,
};
use std::collections::HashMap;
#[cfg(feature = "net-host")]
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    path::Path,
};
type Args = HashMap<String, Value>;
#[cfg(feature = "net-host")]
fn arg<'a>(b: &'a Args, k: &str) -> Result<&'a Value, String> {
    b.get(k).ok_or_else(|| format!("missing {k}"))
}
pub(super) fn call(ctx: &mut HostContext, id: HostFn, b: &Args) -> Result<Value, String> {
    #[cfg(feature = "net-host")]
    let answer = operate(ctx, id, b);
    #[cfg(not(feature = "net-host"))]
    let answer = {
        let _ = (ctx, id, b);
        Err("ZIP unavailable: build with net-host (native default)".into())
    };
    Ok(super::result::capture(
        if id == HostFn::ZipPack {
            "archive.pack"
        } else {
            "archive.unpack"
        },
        answer,
    ))
}
#[cfg(feature = "net-host")]
fn operate(ctx: &mut HostContext, id: HostFn, b: &Args) -> Result<Value, String> {
    if !ctx.allow_fs_write() {
        return Err("filesystem write denied by host policy".into());
    }
    if ctx.sandbox_session.is_some() {
        return Err("ZIP unavailable in CoW sandbox".into());
    }
    let src = ctx.resolve_path(super::as_text(arg(b, "src")?, "src")?)?;
    let dest = ctx.resolve_path(super::as_text(arg(b, "dest")?, "dest")?)?;
    let max = match arg(b, "max_bytes")? {
        Value::Int(n) if *n > 0 && *n <= 1024 * 1024 * 1024 => *n as u64,
        _ => return Err("max_bytes must be 1..1073741824".into()),
    };
    if dest.exists() {
        return Err("destination conflict: ZIP output must not exist".into());
    }
    if id == HostFn::ZipPack {
        if !src.is_dir() || dest.starts_with(&src) {
            return Err("pack src must be directory and dest outside src".into());
        }
        let mut args = HashMap::new();
        args.insert("path".into(), Value::Text(src.to_string_lossy().into()));
        args.insert("pattern".into(), Value::Text("**".into()));
        args.insert("exclude".into(), Value::None);
        args.insert("max_depth".into(), Value::Int(256));
        args.insert("kind".into(), Value::Text("file".into()));
        let Value::List(files) = super::fs_ops::call(ctx, HostFn::FsFind, &args)? else {
            unreachable!()
        };
        let total = files.iter().try_fold(0u64, |n, name| {
            let p = src.join(super::as_text(name, "name")?);
            let size = fs::metadata(p).map_err(|e| e.to_string())?.len();
            n.checked_add(size)
                .ok_or_else(|| "archive size overflow".to_string())
        })?;
        if total > max {
            return Err("archive source exceeds max_bytes".into());
        }
        let parent = dest.parent().ok_or("archive needs parent")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let temp = parent.join(format!(".mq-zip-{}", random_name()?));
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        let written = (|| {
            let mut writer = zip::ZipWriter::new(file);
            let mut buffer = [0u8; 65536];
            let mut seen = 0u64;
            for name in &files {
                let name = super::as_text(name, "name")?;
                writer
                    .start_file(
                        name,
                        zip::write::FileOptions::default()
                            .compression_method(zip::CompressionMethod::Deflated),
                    )
                    .map_err(|e| e.to_string())?;
                let p = ctx.resolve_path(src.join(name).to_str().ok_or("non UTF-8 path")?)?;
                let mut file = fs::File::open(p).map_err(|e| e.to_string())?;
                loop {
                    let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    seen += n as u64;
                    if seen > max {
                        return Err("archive source grew past max_bytes".into());
                    }
                    writer.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
                }
            }
            writer
                .finish()
                .map_err(|e| e.to_string())?
                .sync_all()
                .map_err(|e| e.to_string())?;
            // hard_link is an atomic no-clobber publish on the same filesystem.
            fs::hard_link(&temp, &dest).map_err(|e| e.to_string())?;
            Ok(Value::Int(files.len() as i64))
        })();
        let _ = fs::remove_file(&temp);
        return written;
    }
    let file = fs::File::open(src).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    if zip.len() > 100000 {
        return Err("archive exceeds 100000 entries".into());
    }
    let mut names = HashSet::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = entry
            .enclosed_name()
            .ok_or("archive path escapes destination")?;
        if entry.name().contains('\\') || name == Path::new("") || !names.insert(name.to_path_buf())
        {
            return Err("invalid or duplicate archive path".into());
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err("archive symlinks unsupported".into());
        }
        total = total
            .checked_add(entry.size())
            .ok_or("archive size overflow")?;
        if total > max {
            return Err("archive expansion exceeds max_bytes".into());
        }
    }
    fs::create_dir_all(dest.parent().ok_or("destination needs parent")?)
        .map_err(|e| e.to_string())?;
    fs::create_dir(&dest).map_err(|e| e.to_string())?;
    let extracted = (|| {
        let mut total = 0u64;
        let mut buffer = [0u8; 65536];
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
            let path = ctx.resolve_path(
                dest.join(entry.enclosed_name().ok_or("invalid archive path")?)
                    .to_str()
                    .ok_or("non UTF-8 path")?,
            )?;
            if entry.is_dir() {
                fs::create_dir_all(path).map_err(|e| e.to_string())?;
                continue;
            }
            fs::create_dir_all(path.parent().ok_or("entry needs parent")?)
                .map_err(|e| e.to_string())?;
            let mut file = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path)
                .map_err(|e| e.to_string())?;
            loop {
                let n = entry.read(&mut buffer).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                total += n as u64;
                if total > max {
                    return Err("archive expansion exceeds max_bytes".into());
                }
                file.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
            }
        }
        Ok(Value::Int(zip.len() as i64))
    })();
    if extracted.is_err() {
        let _ = fs::remove_dir_all(&dest);
    }
    extracted
}
#[cfg(feature = "net-host")]
fn random_name() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
