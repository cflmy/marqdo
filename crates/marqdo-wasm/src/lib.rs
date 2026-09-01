//! Browser export surface for Marqdo (ADR 0002 / C1).
//!
//! Raw `cdylib` ABI (no wasm-bindgen CLI required):
//! - `mq_alloc` / `mq_dealloc` — JS writes UTF-8 source into linear memory
//! - `mq_run` — `(ptr, len) -> ptr` to a length-prefixed UTF-8 JSON result
//!
//! Result JSON: `{ ok, stdout, error, value }`.

use std::alloc::{alloc, dealloc, Layout};
use std::ptr;
use std::slice;

use marqdo::{run_source, RunOptions};

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

/// Free a buffer previously returned by [`mq_alloc`] or [`mq_run`].
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

/// Run Marqdo source. Input: UTF-8 at `ptr..ptr+len`.
/// Output: heap buffer = little-endian `u32` byte length + UTF-8 JSON.
/// Caller must `mq_dealloc` the returned pointer with `4 + json_len`.
#[no_mangle]
pub extern "C" fn mq_run(ptr: *const u8, len: usize) -> *mut u8 {
    let source = if ptr.is_null() || len == 0 {
        ""
    } else {
        match std::str::from_utf8(unsafe { slice::from_raw_parts(ptr, len) }) {
            Ok(s) => s,
            Err(e) => {
                return pack_json(&serde_json::json!({
                    "ok": false,
                    "stdout": "",
                    "error": format!("source is not utf-8: {e}"),
                    "value": null,
                }));
            }
        }
    };
    pack_json(&run_to_json(source))
}

/// Package version (NUL-terminated C string in static memory — do not free).
#[no_mangle]
pub extern "C" fn mq_version() -> *const u8 {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr()
}

fn run_to_json(source: &str) -> serde_json::Value {
    let mut opts = RunOptions::default();
    opts.allow_fs_write = false;
    opts.allow_exec = false;
    opts.allow_net = false;
    opts.sleep_limit_ms = Some(0);

    match run_source(source, &opts) {
        Ok(cap) => serde_json::json!({
            "ok": true,
            "stdout": cap.stdout,
            "error": null,
            "value": cap.value.as_display(),
        }),
        Err(e) => serde_json::json!({
            "ok": false,
            "stdout": "",
            "error": format!("{e:#}"),
            "value": null,
        }),
    }
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
