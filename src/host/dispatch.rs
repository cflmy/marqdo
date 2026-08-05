//! Host function registry and dispatch.

use std::collections::HashMap;

use crate::host::{fs, json, net, sys, time, HostContext};
use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum HostFn {
    ReadText = 1,
    WriteText = 2,
    AppendText = 3,
    Exists = 4,
    ListDir = 5,
    MakeDir = 6,
    Remove = 7,
    NowUnix = 8,
    NowMs = 9,
    FormatTime = 10,
    SleepMs = 11,
    ParseTime = 12,
    EnvGet = 13,
    EnvSet = 14,
    Args = 15,
    Cwd = 16,
    Exit = 17,
    Exec = 18,
    JsonParse = 19,
    JsonStringify = 20,
    JsonGet = 21,
    JsonKeys = 22,
    HttpGet = 23,
    HttpPost = 24,
    UrlEncode = 25,
}

impl HostFn {
    pub fn from_u16(id: u16) -> Option<Self> {
        Some(match id {
            1 => Self::ReadText,
            2 => Self::WriteText,
            3 => Self::AppendText,
            4 => Self::Exists,
            5 => Self::ListDir,
            6 => Self::MakeDir,
            7 => Self::Remove,
            8 => Self::NowUnix,
            9 => Self::NowMs,
            10 => Self::FormatTime,
            11 => Self::SleepMs,
            12 => Self::ParseTime,
            13 => Self::EnvGet,
            14 => Self::EnvSet,
            15 => Self::Args,
            16 => Self::Cwd,
            17 => Self::Exit,
            18 => Self::Exec,
            19 => Self::JsonParse,
            20 => Self::JsonStringify,
            21 => Self::JsonGet,
            22 => Self::JsonKeys,
            23 => Self::HttpGet,
            24 => Self::HttpPost,
            25 => Self::UrlEncode,
            _ => return None,
        })
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "host_read_text" => Self::ReadText,
            "host_write_text" => Self::WriteText,
            "host_append_text" => Self::AppendText,
            "host_exists" => Self::Exists,
            "host_list_dir" => Self::ListDir,
            "host_make_dir" => Self::MakeDir,
            "host_remove" => Self::Remove,
            "host_now_unix" => Self::NowUnix,
            "host_now_ms" => Self::NowMs,
            "host_format_time" => Self::FormatTime,
            "host_sleep_ms" => Self::SleepMs,
            "host_parse_time" => Self::ParseTime,
            "host_env_get" => Self::EnvGet,
            "host_env_set" => Self::EnvSet,
            "host_args" => Self::Args,
            "host_cwd" => Self::Cwd,
            "host_exit" => Self::Exit,
            "host_exec" => Self::Exec,
            "host_json_parse" => Self::JsonParse,
            "host_json_stringify" => Self::JsonStringify,
            "host_json_get" => Self::JsonGet,
            "host_json_keys" => Self::JsonKeys,
            "host_http_get" => Self::HttpGet,
            "host_http_post" => Self::HttpPost,
            "host_url_encode" => Self::UrlEncode,
            _ => return None,
        })
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::ReadText => "host_read_text",
            Self::WriteText => "host_write_text",
            Self::AppendText => "host_append_text",
            Self::Exists => "host_exists",
            Self::ListDir => "host_list_dir",
            Self::MakeDir => "host_make_dir",
            Self::Remove => "host_remove",
            Self::NowUnix => "host_now_unix",
            Self::NowMs => "host_now_ms",
            Self::FormatTime => "host_format_time",
            Self::SleepMs => "host_sleep_ms",
            Self::ParseTime => "host_parse_time",
            Self::EnvGet => "host_env_get",
            Self::EnvSet => "host_env_set",
            Self::Args => "host_args",
            Self::Cwd => "host_cwd",
            Self::Exit => "host_exit",
            Self::Exec => "host_exec",
            Self::JsonParse => "host_json_parse",
            Self::JsonStringify => "host_json_stringify",
            Self::JsonGet => "host_json_get",
            Self::JsonKeys => "host_json_keys",
            Self::HttpGet => "host_http_get",
            Self::HttpPost => "host_http_post",
            Self::UrlEncode => "host_url_encode",
        }
    }

    /// Required params first; see `optional_params` for trailing optionals.
    pub fn required_params(self) -> &'static [&'static str] {
        match self {
            Self::ReadText | Self::Exists | Self::ListDir | Self::MakeDir | Self::Remove => {
                &["path"]
            }
            Self::WriteText | Self::AppendText => &["path", "text"],
            Self::NowUnix | Self::NowMs | Self::Args | Self::Cwd => &[],
            Self::FormatTime => &["unix", "pattern"],
            Self::ParseTime => &["text", "pattern"],
            Self::SleepMs => &["ms"],
            Self::EnvGet => &["name"],
            Self::EnvSet => &["name", "value"],
            Self::Exit => &["code"],
            Self::Exec => &["cmd"],
            Self::JsonParse | Self::UrlEncode => &["text"],
            Self::JsonStringify | Self::JsonKeys => &["value"],
            Self::JsonGet => &["value", "key"],
            Self::HttpGet => &["url"],
            Self::HttpPost => &["url", "body"],
        }
    }

    pub fn optional_params(self) -> &'static [&'static str] {
        match self {
            Self::JsonStringify => &["indent"],
            Self::HttpPost => &["content_type"],
            Self::Exec => &["args"],
            _ => &[],
        }
    }

    pub fn all_params(self) -> Vec<&'static str> {
        let mut v = self.required_params().to_vec();
        v.extend_from_slice(self.optional_params());
        v
    }
}

pub fn call_host(
    ctx: &HostContext,
    fn_id: HostFn,
    bound: &HashMap<String, Value>,
) -> Result<Value, String> {
    match fn_id {
        HostFn::ReadText => fs::read_text(ctx, require(bound, "path")?),
        HostFn::WriteText => {
            fs::write_text(ctx, require(bound, "path")?, require(bound, "text")?)
        }
        HostFn::AppendText => {
            fs::append_text(ctx, require(bound, "path")?, require(bound, "text")?)
        }
        HostFn::Exists => fs::exists(ctx, require(bound, "path")?),
        HostFn::ListDir => fs::list_dir(ctx, require(bound, "path")?),
        HostFn::MakeDir => fs::make_dir(ctx, require(bound, "path")?),
        HostFn::Remove => fs::remove(ctx, require(bound, "path")?),
        HostFn::NowUnix => time::now_unix(),
        HostFn::NowMs => time::now_ms(),
        HostFn::FormatTime => {
            time::format_time(require(bound, "unix")?, require(bound, "pattern")?)
        }
        HostFn::SleepMs => time::sleep_ms(ctx, require(bound, "ms")?),
        HostFn::ParseTime => {
            time::parse_time(require(bound, "text")?, require(bound, "pattern")?)
        }
        HostFn::EnvGet => sys::env_get(require(bound, "name")?),
        HostFn::EnvSet => sys::env_set(require(bound, "name")?, require(bound, "value")?),
        HostFn::Args => sys::args(ctx),
        HostFn::Cwd => sys::cwd(ctx),
        HostFn::Exit => sys::exit(ctx, require(bound, "code")?),
        HostFn::Exec => sys::exec(ctx, require(bound, "cmd")?, bound.get("args")),
        HostFn::JsonParse => json::parse(require(bound, "text")?),
        HostFn::JsonStringify => {
            json::stringify(require(bound, "value")?, bound.get("indent"))
        }
        HostFn::JsonGet => json::get(require(bound, "value")?, require(bound, "key")?),
        HostFn::JsonKeys => json::keys(require(bound, "value")?),
        HostFn::HttpGet => net::http_get(ctx, require(bound, "url")?),
        HostFn::HttpPost => net::http_post(
            ctx,
            require(bound, "url")?,
            require(bound, "body")?,
            bound.get("content_type"),
        ),
        HostFn::UrlEncode => net::url_encode(require(bound, "text")?),
    }
}

fn require<'a>(bound: &'a HashMap<String, Value>, key: &str) -> Result<&'a Value, String> {
    bound
        .get(key)
        .ok_or_else(|| format!("host call missing `{key}`"))
}
