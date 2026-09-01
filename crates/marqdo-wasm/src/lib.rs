//! Browser export surface for Marqdo (ADR 0002 / C1–C3).
//!
//! Raw `cdylib` ABI (no wasm-bindgen CLI):
//! - `mq_alloc` / `mq_dealloc`
//! - `mq_run` — one-shot `# main` (C1)
//! - `mq_boot` — start session; return value often a `wire` table (C3)
//! - `mq_call` — `(name_ptr, name_len, args_ptr, args_len) ->` JSON result (C3)
//! - `mq_version`
//!
//! Packed results: little-endian `u32` length + UTF-8 JSON
//! `{ ok, stdout, error, value }`.

use std::alloc::{alloc, dealloc, Layout};
use std::cell::RefCell;
use std::ptr;
use std::slice;

use marqdo::{run_source, value_as_json, BrowserSession, RunOptions};
use serde_json::json;

thread_local! {
    static SESSION: RefCell<Option<BrowserSession>> = const { RefCell::new(None) };
}

/// Allocate `size` bytes in WASM linear memory (alignment 1).
#[no_mangle]
pub extern "C" fn mq_alloc(size: usize) -> *mut u8 {
    if size == 0 {
        return ptr::null_mut();
    }
    let layout = match Layout::from_size_align(size, 1) {
        Ok(l) => l,
        Err(_) => return ptr::null_mut(),
    };
    unsafe { alloc(layout) }
}

/// Free a buffer previously returned by [`mq_alloc`], [`mq_run`], [`mq_boot`], or [`mq_call`].
#[no_mangle]
pub extern "C" fn mq_dealloc(ptr: *mut u8, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }
    let layout = match Layout::from_size_align(size, 1) {
        Ok(l) => l,
        Err(_) => return,
    };
    unsafe { dealloc(ptr, layout) }
}

/// One-shot run (C1). Does not keep a session.
#[no_mangle]
pub extern "C" fn mq_run(ptr: *const u8, len: usize) -> *mut u8 {
    let source = match read_utf8(ptr, len) {
        Ok(s) => s,
        Err(e) => return pack_json(&err_json(&e)),
    };
    pack_json(&run_to_json(&source))
}

/// Boot a session: run `# main`, keep entry env. Replaces any prior session.
#[no_mangle]
pub extern "C" fn mq_boot(ptr: *const u8, len: usize) -> *mut u8 {
    let source = match read_utf8(ptr, len) {
        Ok(s) => s,
        Err(e) => return pack_json(&err_json(&e)),
    };
    match BrowserSession::boot(&source) {
        Ok((sess, value)) => {
            let stdout = String::new();
            let value_json = match value_as_json(&value) {
                Ok(j) => j,
                Err(e) => return pack_json(&err_json(&e)),
            };
            SESSION.with(|s| *s.borrow_mut() = Some(sess));
            pack_json(&json!({
                "ok": true,
                "stdout": stdout,
                "error": null,
                "value": value_json,
            }))
        }
        Err(e) => {
            SESSION.with(|s| *s.borrow_mut() = None);
            pack_json(&err_json(&format!("{e:#}")))
        }
    }
}

/// Call a function in the current session.
/// `args` is UTF-8 JSON object of named arguments (may be `{}`).
#[no_mangle]
pub extern "C" fn mq_call(
    name_ptr: *const u8,
    name_len: usize,
    args_ptr: *const u8,
    args_len: usize,
) -> *mut u8 {
    let name = match read_utf8(name_ptr, name_len) {
        Ok(s) => s,
        Err(e) => return pack_json(&err_json(&e)),
    };
    let args = match read_utf8(args_ptr, args_len) {
        Ok(s) => s,
        Err(e) => return pack_json(&err_json(&e)),
    };
    SESSION.with(|cell| {
        let mut guard = cell.borrow_mut();
        let Some(sess) = guard.as_mut() else {
            return pack_json(&err_json("no session — call mq_boot first"));
        };
        match sess.call(&name, &args) {
            Ok(value) => {
                let stdout = sess.take_stdout();
                let value_json = match value_as_json(&value) {
                    Ok(j) => j,
                    Err(e) => return pack_json(&err_json(&e)),
                };
                pack_json(&json!({
                    "ok": true,
                    "stdout": stdout,
                    "error": null,
                    "value": value_json,
                }))
            }
            Err(e) => pack_json(&err_json(&format!("{e:#}"))),
        }
    })
}

/// Package version (NUL-terminated C string in static memory — do not free).
#[no_mangle]
pub extern "C" fn mq_version() -> *const u8 {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr()
}

fn read_utf8(ptr: *const u8, len: usize) -> Result<String, String> {
    if ptr.is_null() || len == 0 {
        return Ok(String::new());
    }
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };
    String::from_utf8(bytes.to_vec()).map_err(|e| format!("not utf-8: {e}"))
}

fn run_to_json(source: &str) -> serde_json::Value {
    let mut opts = RunOptions::default();
    opts.allow_fs_write = false;
    opts.allow_exec = false;
    opts.allow_net = false;
    opts.sleep_limit_ms = Some(0);

    match run_source(source, &opts) {
        Ok(cap) => match value_as_json(&cap.value) {
            Ok(value) => json!({
                "ok": true,
                "stdout": cap.stdout,
                "error": null,
                "value": value,
            }),
            Err(e) => err_json(&e),
        },
        Err(e) => err_json(&format!("{e:#}")),
    }
}

fn err_json(msg: &str) -> serde_json::Value {
    json!({
        "ok": false,
        "stdout": "",
        "error": msg,
        "value": null,
    })
}

fn pack_json(v: &serde_json::Value) -> *mut u8 {
    let s = v.to_string();
    let bytes = s.as_bytes();
    let total = 4 + bytes.len();
    let out = mq_alloc(total);
    if out.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        let len = bytes.len() as u32;
        ptr::copy_nonoverlapping(len.to_le_bytes().as_ptr(), out, 4);
        ptr::copy_nonoverlapping(bytes.as_ptr(), out.add(4), bytes.len());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_hello_json() {
        let v = run_to_json("# main\n\n> print text=Hello World!\n");
        assert_eq!(v["ok"], true);
        assert_eq!(v["stdout"], "Hello World!\n");
    }
}
