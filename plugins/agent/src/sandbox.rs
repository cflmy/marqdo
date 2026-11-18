//! Forward `sandbox_*` plugin calls to host_query (CoW lives in the host).

use super::{
    host_query_json_args, parse_args, set_err, set_out,
};
use std::os::raw::c_char;

fn forward(
    query_name: &str,
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> i32 {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("{query_name}: {e}"));
            return 1;
        }
    };
    match host_query_json_args(query_name, &v) {
        Ok(j) => {
            set_out(out_json, &j.to_string());
            0
        }
        Err(e) => {
            set_err(err_msg, &e);
            1
        }
    }
}

macro_rules! sandbox_ffi {
    ($fn_name:ident, $query:expr) => {
        pub unsafe extern "C" fn $fn_name(
            args_json: *const c_char,
            out_json: *mut *mut c_char,
            err_msg: *mut *mut c_char,
        ) -> i32 {
            forward($query, args_json, out_json, err_msg)
        }
    };
}

sandbox_ffi!(sandbox_open, "sandbox_open");
sandbox_ffi!(sandbox_write, "sandbox_write");
sandbox_ffi!(sandbox_read, "sandbox_read");
sandbox_ffi!(sandbox_remove, "sandbox_remove");
sandbox_ffi!(sandbox_diff, "sandbox_diff");
sandbox_ffi!(sandbox_commit, "sandbox_commit");
sandbox_ffi!(sandbox_discard, "sandbox_discard");
sandbox_ffi!(sandbox_inspect, "sandbox_inspect");
sandbox_ffi!(sandbox_run, "sandbox_run");
