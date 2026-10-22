//! Host function registry and dispatch.

use std::collections::HashMap;

use crate::host::{
    cli, csv, datetime, encoding, fs, hash, html_ops, json, log, math, net, path, re, secrets, stats,
    sys, text_ops, time, toml_ops, url_ops, uuid_ops, writeback, HostContext,
};
#[cfg(feature = "exec-host")]
use crate::host::{foreign, subtask};
#[cfg(feature = "plugin-host")]
use crate::host::plugin;
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
    // math
    Pi = 26,
    EConst = 27,
    Num = 28,
    MathAdd = 29,
    MathSub = 30,
    MathMul = 31,
    MathDiv = 32,
    MathPow = 33,
    MathNeg = 34,
    MathSin = 35,
    MathCos = 36,
    MathTan = 37,
    MathAsin = 38,
    MathAcos = 39,
    MathAtan = 40,
    MathSqrt = 41,
    MathAbs = 42,
    MathLn = 43,
    MathExp = 44,
    MathFloor = 45,
    MathCeil = 46,
    MathMin = 47,
    MathMax = 48,
    Random = 49,
    RandomInt = 50,
    Seed = 51,
    Formula = 52,
    Simplify = 53,
    Expand = 54,
    Diff = 55,
    Subs = 56,
    Eval = 57,
    Solve = 58,
    Plot = 59,
    PlotPoints = 60,
    PlotConic = 61,
    ForeignSetCmd = 62,
    ForeignRun = 63,
    ForeignRunLang = 64,
    ForeignLangs = 65,
    DotenvLoad = 66,
    HttpRequest = 67,
    JsonQuote = 68,
    PluginLoad = 69,
    PluginUnload = 70,
    PluginList = 71,
    ExtNativePath = 72,
    MapSet = 76,
    ListAppend = 77,
    WritebackRecord = 84,
    WritebackGet = 85,
    WritebackClear = 86,
    WritebackList = 87,
    SubtaskSpawn = 88,
    SubtaskPoll = 89,
    SubtaskJoin = 90,
    SubtaskKill = 91,
    SubtaskWaitAll = 92,
    WritebackEnsure = 93,
    OuterCallLine = 94,
    TextPatch = 95,
    ApplyPatchBlocks = 96,
    WritebackScanPath = 97,
    HttpPostSse = 98,
    OpenAiSseParse = 99,
    StreamPublish = 100,
    CollectionPut = 101,
    ListPrepend = 102,
    ListConcat = 103,
    ListInsert = 104,
    ListSetAt = 105,
    ListRemoveAt = 106,
    ListSlice = 107,
    ListContains = 108,
    ListIndexOf = 109,
    ListReverse = 110,
    ListFirst = 111,
    ListLast = 112,
    MapGet = 113,
    MapDelete = 114,
    MapHas = 115,
    MapKeys = 116,
    MapValues = 117,
    MapItems = 118,
    MapMerge = 119,
    MapSize = 120,
    CollectionClear = 121,
    CookieParse = 122,
    MultipartParse = 123,
    MarkdownParse = 124,
    ReIsMatch = 125,
    ReFind = 126,
    ReFindAll = 127,
    ReReplace = 128,
    ReSplit = 129,
    EncodingBase64Encode = 130,
    EncodingBase64Decode = 131,
    EncodingHexEncode = 132,
    EncodingHexDecode = 133,
    PathJoin = 134,
    PathSplit = 135,
    PathFileName = 136,
    PathParent = 137,
    PathExtension = 138,
    PathNormalize = 139,
    PathIsAbsolute = 140,
    FsCopyFile = 141,
    FsMove = 142,
    FsMakeTemp = 143,
    HashSha256 = 144,
    HashSha1 = 145,
    HashMd5 = 146,
    HashHmacSha256 = 147,
    SecretsTokenHex = 148,
    SecretsTokenUrlsafe = 149,
    TextContains = 150,
    TextStartsWith = 151,
    TextEndsWith = 152,
    TextReplace = 153,
    TextToUpper = 154,
    TextToLower = 155,
    TextRepeat = 156,
    TextPad = 157,
    CsvParse = 158,
    CsvStringify = 159,
    ListSort = 160,
    ListSortBy = 161,
    ListUnique = 162,
    ListChunk = 163,
    ListZip = 164,
    ListFlatten = 165,
    CliParse = 166,
    LogSetLevel = 167,
    LogLine = 168,
    UuidV4 = 169,
    DatetimeNow = 170,
    DatetimeFromUnix = 171,
    DatetimeToUnix = 172,
    DatetimeParse = 173,
    DatetimeFormat = 174,
    DatetimeAdd = 175,
    DatetimeInZone = 176,
    UrlParse = 177,
    UrlQueryParse = 178,
    UrlQueryStringify = 179,
    TomlParse = 180,
    HtmlEscape = 181,
    HtmlUnescape = 182,
    EncodingBase32Encode = 183,
    EncodingBase32Decode = 184,
    FsMakeDirs = 185,
    FsRemoveTree = 186,
    FsStat = 187,
    FsWalk = 188,
    StatsMean = 189,
    StatsMedian = 190,
    StatsStdev = 191,
    Meta = 192,
    MetaGet = 193,
    ModuleSource = 194,
    ArtifactKind = 195,
    ForeignInvoke = 196,
    ForeignProbe = 197,
    ForeignRegister = 198,
    ForeignUnset = 199,
    ForeignLoadConfig = 200,
    ForeignCall = 201,
    ForeignRunMany = 202,
    ForeignCallMany = 203,
    ForeignSessionOpen = 204,
    ForeignSessionEval = 205,
    ForeignSessionSet = 206,
    ForeignSessionGet = 207,
    ForeignSessionNames = 208,
    ForeignSessionClose = 209,
    ForeignSessionList = 210,
    ForeignSessionInterrupt = 211,
    /// `<TAB>` M7: `access(集合, 键)` — polymorphic collection access (legacy index).
    Access = 212,
}

/// Single source of truth for every host function. `<TAB>` slimming
///
/// Replaces five hand-written matches (id, name, required, optional) that
/// had already drifted; `call_host` keeps its one-line arms.
pub struct HostFnDef {
    pub id: u16,
    pub f: HostFn,
    pub name: &'static str,
    pub required: &'static [&'static str],
    pub optional: &'static [&'static str],
}

pub const HOST_FN_TABLE: &[HostFnDef] = &[
    HostFnDef { id: 1, f: HostFn::ReadText, name: "host_read_text", required: &["path"], optional: &[] },
    HostFnDef { id: 2, f: HostFn::WriteText, name: "host_write_text", required: &["path", "text"], optional: &[] },
    HostFnDef { id: 3, f: HostFn::AppendText, name: "host_append_text", required: &["path", "text"], optional: &[] },
    HostFnDef { id: 4, f: HostFn::Exists, name: "host_exists", required: &["path"], optional: &[] },
    HostFnDef { id: 5, f: HostFn::ListDir, name: "host_list_dir", required: &["path"], optional: &[] },
    HostFnDef { id: 6, f: HostFn::MakeDir, name: "host_make_dir", required: &["path"], optional: &[] },
    HostFnDef { id: 7, f: HostFn::Remove, name: "host_remove", required: &["path"], optional: &[] },
    HostFnDef { id: 8, f: HostFn::NowUnix, name: "host_now_unix", required: &[], optional: &[] },
    HostFnDef { id: 9, f: HostFn::NowMs, name: "host_now_ms", required: &[], optional: &[] },
    HostFnDef { id: 10, f: HostFn::FormatTime, name: "host_format_time", required: &["unix", "pattern"], optional: &[] },
    HostFnDef { id: 11, f: HostFn::SleepMs, name: "host_sleep_ms", required: &["ms"], optional: &[] },
    HostFnDef { id: 12, f: HostFn::ParseTime, name: "host_parse_time", required: &["text", "pattern"], optional: &[] },
    HostFnDef { id: 13, f: HostFn::EnvGet, name: "host_env_get", required: &["name"], optional: &[] },
    HostFnDef { id: 14, f: HostFn::EnvSet, name: "host_env_set", required: &["name", "value"], optional: &[] },
    HostFnDef { id: 15, f: HostFn::Args, name: "host_args", required: &[], optional: &[] },
    HostFnDef { id: 16, f: HostFn::Cwd, name: "host_cwd", required: &[], optional: &[] },
    HostFnDef { id: 17, f: HostFn::Exit, name: "host_exit", required: &["code"], optional: &[] },
    HostFnDef { id: 18, f: HostFn::Exec, name: "host_exec", required: &["cmd"], optional: &["args", "capture"] },
    HostFnDef { id: 19, f: HostFn::JsonParse, name: "host_json_parse", required: &["text"], optional: &[] },
    HostFnDef { id: 20, f: HostFn::JsonStringify, name: "host_json_stringify", required: &["value"], optional: &["indent"] },
    HostFnDef { id: 21, f: HostFn::JsonGet, name: "host_json_get", required: &["value", "key"], optional: &[] },
    HostFnDef { id: 22, f: HostFn::JsonKeys, name: "host_json_keys", required: &["value"], optional: &[] },
    HostFnDef { id: 23, f: HostFn::HttpGet, name: "host_http_get", required: &["url"], optional: &["headers"] },
    HostFnDef { id: 24, f: HostFn::HttpPost, name: "host_http_post", required: &["url", "body"], optional: &["content_type", "headers"] },
    HostFnDef { id: 25, f: HostFn::UrlEncode, name: "host_url_encode", required: &["text"], optional: &[] },
    HostFnDef { id: 26, f: HostFn::Pi, name: "host_pi", required: &[], optional: &[] },
    HostFnDef { id: 27, f: HostFn::EConst, name: "host_e", required: &[], optional: &[] },
    HostFnDef { id: 28, f: HostFn::Num, name: "host_num", required: &["value"], optional: &[] },
    HostFnDef { id: 29, f: HostFn::MathAdd, name: "host_math_add", required: &["a", "b"], optional: &[] },
    HostFnDef { id: 30, f: HostFn::MathSub, name: "host_math_sub", required: &["a", "b"], optional: &[] },
    HostFnDef { id: 31, f: HostFn::MathMul, name: "host_math_mul", required: &["a", "b"], optional: &[] },
    HostFnDef { id: 32, f: HostFn::MathDiv, name: "host_math_div", required: &["a", "b"], optional: &[] },
    HostFnDef { id: 33, f: HostFn::MathPow, name: "host_math_pow", required: &["a", "b"], optional: &[] },
    HostFnDef { id: 34, f: HostFn::MathNeg, name: "host_math_neg", required: &["value"], optional: &[] },
    HostFnDef { id: 35, f: HostFn::MathSin, name: "host_math_sin", required: &["value"], optional: &[] },
    HostFnDef { id: 36, f: HostFn::MathCos, name: "host_math_cos", required: &["value"], optional: &[] },
    HostFnDef { id: 37, f: HostFn::MathTan, name: "host_math_tan", required: &["value"], optional: &[] },
    HostFnDef { id: 38, f: HostFn::MathAsin, name: "host_math_asin", required: &["value"], optional: &[] },
    HostFnDef { id: 39, f: HostFn::MathAcos, name: "host_math_acos", required: &["value"], optional: &[] },
    HostFnDef { id: 40, f: HostFn::MathAtan, name: "host_math_atan", required: &["value"], optional: &[] },
    HostFnDef { id: 41, f: HostFn::MathSqrt, name: "host_math_sqrt", required: &["value"], optional: &[] },
    HostFnDef { id: 42, f: HostFn::MathAbs, name: "host_math_abs", required: &["value"], optional: &[] },
    HostFnDef { id: 43, f: HostFn::MathLn, name: "host_math_ln", required: &["value"], optional: &[] },
    HostFnDef { id: 44, f: HostFn::MathExp, name: "host_math_exp", required: &["value"], optional: &[] },
    HostFnDef { id: 45, f: HostFn::MathFloor, name: "host_math_floor", required: &["value"], optional: &[] },
    HostFnDef { id: 46, f: HostFn::MathCeil, name: "host_math_ceil", required: &["value"], optional: &[] },
    HostFnDef { id: 47, f: HostFn::MathMin, name: "host_math_min", required: &["a", "b"], optional: &[] },
    HostFnDef { id: 48, f: HostFn::MathMax, name: "host_math_max", required: &["a", "b"], optional: &[] },
    HostFnDef { id: 49, f: HostFn::Random, name: "host_random", required: &[], optional: &[] },
    HostFnDef { id: 50, f: HostFn::RandomInt, name: "host_random_int", required: &["min", "max"], optional: &[] },
    HostFnDef { id: 51, f: HostFn::Seed, name: "host_seed", required: &["value"], optional: &[] },
    HostFnDef { id: 52, f: HostFn::Formula, name: "host_formula", required: &["value"], optional: &[] },
    HostFnDef { id: 53, f: HostFn::Simplify, name: "host_simplify", required: &["value"], optional: &[] },
    HostFnDef { id: 54, f: HostFn::Expand, name: "host_expand", required: &["value"], optional: &[] },
    HostFnDef { id: 55, f: HostFn::Diff, name: "host_diff", required: &["formula", "var"], optional: &[] },
    HostFnDef { id: 56, f: HostFn::Subs, name: "host_subs", required: &["formula", "var", "value"], optional: &[] },
    HostFnDef { id: 57, f: HostFn::Eval, name: "host_eval", required: &["formula", "var", "value"], optional: &[] },
    HostFnDef { id: 58, f: HostFn::Solve, name: "host_solve", required: &["formula", "var"], optional: &["min", "max"] },
    HostFnDef { id: 59, f: HostFn::Plot, name: "host_plot", required: &["formula", "var", "min", "max"], optional: &["steps", "path", "derivative", "grid"] },
    HostFnDef { id: 60, f: HostFn::PlotPoints, name: "host_plot_points", required: &["xs", "ys"], optional: &["path", "grid"] },
    HostFnDef { id: 61, f: HostFn::PlotConic, name: "host_plot_conic", required: &["kind", "a"], optional: &["b", "h", "k", "path", "grid"] },
    HostFnDef { id: 62, f: HostFn::ForeignSetCmd, name: "host_foreign_set_cmd", required: &["lang", "cmd"], optional: &["args"] },
    HostFnDef { id: 63, f: HostFn::ForeignRun, name: "host_foreign_run", required: &["code"], optional: &["stdin", "env", "cwd", "timeout", "args", "max_output"] },
    HostFnDef { id: 64, f: HostFn::ForeignRunLang, name: "host_foreign_run_lang", required: &["lang", "source"], optional: &["stdin", "env", "cwd", "timeout", "args", "max_output"] },
    HostFnDef { id: 65, f: HostFn::ForeignLangs, name: "host_foreign_langs", required: &[], optional: &[] },
    HostFnDef { id: 66, f: HostFn::DotenvLoad, name: "host_dotenv_load", required: &[], optional: &["path"] },
    HostFnDef { id: 67, f: HostFn::HttpRequest, name: "host_http_request", required: &["method", "url"], optional: &["body", "content_type", "headers"] },
    HostFnDef { id: 68, f: HostFn::JsonQuote, name: "host_json_quote", required: &["text"], optional: &[] },
    HostFnDef { id: 69, f: HostFn::PluginLoad, name: "host_plugin_load", required: &["path"], optional: &[] },
    HostFnDef { id: 70, f: HostFn::PluginUnload, name: "host_plugin_unload", required: &[], optional: &[] },
    HostFnDef { id: 71, f: HostFn::PluginList, name: "host_plugin_list", required: &[], optional: &[] },
    HostFnDef { id: 72, f: HostFn::ExtNativePath, name: "host_ext_native_path", required: &["name"], optional: &[] },
    HostFnDef { id: 76, f: HostFn::MapSet, name: "host_map_set", required: &["map", "key", "value"], optional: &[] },
    HostFnDef { id: 77, f: HostFn::ListAppend, name: "host_list_append", required: &["list", "item"], optional: &[] },
    HostFnDef { id: 84, f: HostFn::WritebackRecord, name: "host_writeback_record", required: &["value"], optional: &["at_end", "key", "line"] },
    HostFnDef { id: 85, f: HostFn::WritebackGet, name: "host_writeback_get", required: &[], optional: &["at_end", "key", "line"] },
    HostFnDef { id: 86, f: HostFn::WritebackClear, name: "host_writeback_clear", required: &[], optional: &["at_end", "key", "line"] },
    HostFnDef { id: 87, f: HostFn::WritebackList, name: "host_writeback_list", required: &[], optional: &[] },
    HostFnDef { id: 88, f: HostFn::SubtaskSpawn, name: "host_subtask_spawn", required: &[], optional: &["path", "fn", "args", "code", "lang", "source", "stdin", "quiet"] },
    HostFnDef { id: 89, f: HostFn::SubtaskPoll, name: "host_subtask_poll", required: &["id"], optional: &[] },
    HostFnDef { id: 90, f: HostFn::SubtaskJoin, name: "host_subtask_join", required: &["id"], optional: &[] },
    HostFnDef { id: 91, f: HostFn::SubtaskKill, name: "host_subtask_kill", required: &["id"], optional: &[] },
    HostFnDef { id: 92, f: HostFn::SubtaskWaitAll, name: "host_subtask_wait_all", required: &[], optional: &[] },
    HostFnDef { id: 93, f: HostFn::WritebackEnsure, name: "host_writeback_ensure", required: &["key"], optional: &["placeholder", "line"] },
    HostFnDef { id: 94, f: HostFn::OuterCallLine, name: "host_outer_call_line", required: &[], optional: &[] },
    HostFnDef { id: 95, f: HostFn::TextPatch, name: "host_text_patch", required: &["path", "find", "replace"], optional: &[] },
    HostFnDef { id: 96, f: HostFn::ApplyPatchBlocks, name: "host_apply_patch_blocks", required: &["path", "text", "soft"], optional: &[] },
    HostFnDef { id: 97, f: HostFn::WritebackScanPath, name: "host_writeback_scan_path", required: &["path"], optional: &[] },
    HostFnDef { id: 98, f: HostFn::HttpPostSse, name: "host_http_post_sse", required: &["url", "body"], optional: &["content_type", "headers", "echo"] },
    HostFnDef { id: 99, f: HostFn::OpenAiSseParse, name: "host_openai_sse_parse", required: &["text"], optional: &["echo"] },
    HostFnDef { id: 100, f: HostFn::StreamPublish, name: "host_stream_publish", required: &["event"], optional: &[] },
    HostFnDef { id: 101, f: HostFn::CollectionPut, name: "host_collection_put", required: &["in", "at", "value"], optional: &[] },
    HostFnDef { id: 102, f: HostFn::ListPrepend, name: "host_list_prepend", required: &["list", "item"], optional: &[] },
    HostFnDef { id: 103, f: HostFn::ListConcat, name: "host_list_concat", required: &["a", "b"], optional: &[] },
    HostFnDef { id: 104, f: HostFn::ListInsert, name: "host_list_insert", required: &["list", "index", "item"], optional: &[] },
    HostFnDef { id: 105, f: HostFn::ListSetAt, name: "host_list_set_at", required: &["list", "index", "item"], optional: &[] },
    HostFnDef { id: 106, f: HostFn::ListRemoveAt, name: "host_list_remove_at", required: &["list", "index"], optional: &[] },
    HostFnDef { id: 107, f: HostFn::ListSlice, name: "host_list_slice", required: &["list", "start"], optional: &["end"] },
    HostFnDef { id: 108, f: HostFn::ListContains, name: "host_list_contains", required: &["list", "item"], optional: &[] },
    HostFnDef { id: 109, f: HostFn::ListIndexOf, name: "host_list_index_of", required: &["list", "item"], optional: &[] },
    HostFnDef { id: 110, f: HostFn::ListReverse, name: "host_list_reverse", required: &["list"], optional: &[] },
    HostFnDef { id: 111, f: HostFn::ListFirst, name: "host_list_first", required: &["list"], optional: &[] },
    HostFnDef { id: 112, f: HostFn::ListLast, name: "host_list_last", required: &["list"], optional: &[] },
    HostFnDef { id: 113, f: HostFn::MapGet, name: "host_map_get", required: &["map", "key"], optional: &[] },
    HostFnDef { id: 114, f: HostFn::MapDelete, name: "host_map_delete", required: &["map", "key"], optional: &[] },
    HostFnDef { id: 115, f: HostFn::MapHas, name: "host_map_has", required: &["map", "key"], optional: &[] },
    HostFnDef { id: 116, f: HostFn::MapKeys, name: "host_map_keys", required: &["map"], optional: &[] },
    HostFnDef { id: 117, f: HostFn::MapValues, name: "host_map_values", required: &["map"], optional: &[] },
    HostFnDef { id: 118, f: HostFn::MapItems, name: "host_map_items", required: &["map"], optional: &[] },
    HostFnDef { id: 119, f: HostFn::MapMerge, name: "host_map_merge", required: &["a", "b"], optional: &[] },
    HostFnDef { id: 120, f: HostFn::MapSize, name: "host_map_size", required: &["map"], optional: &[] },
    HostFnDef { id: 121, f: HostFn::CollectionClear, name: "host_collection_clear", required: &["value"], optional: &[] },
    HostFnDef { id: 122, f: HostFn::CookieParse, name: "host_cookie_parse", required: &["text", "is_response"], optional: &[] },
    HostFnDef { id: 123, f: HostFn::MultipartParse, name: "host_multipart_parse", required: &["body", "boundary"], optional: &[] },
    HostFnDef { id: 124, f: HostFn::MarkdownParse, name: "host_markdown_parse", required: &["text"], optional: &[] },
    HostFnDef { id: 125, f: HostFn::ReIsMatch, name: "host_re_is_match", required: &["text", "pattern"], optional: &[] },
    HostFnDef { id: 126, f: HostFn::ReFind, name: "host_re_find", required: &["text", "pattern"], optional: &[] },
    HostFnDef { id: 127, f: HostFn::ReFindAll, name: "host_re_find_all", required: &["text", "pattern"], optional: &[] },
    HostFnDef { id: 128, f: HostFn::ReReplace, name: "host_re_replace", required: &["text", "pattern", "with"], optional: &["count"] },
    HostFnDef { id: 129, f: HostFn::ReSplit, name: "host_re_split", required: &["text", "pattern"], optional: &[] },
    HostFnDef { id: 130, f: HostFn::EncodingBase64Encode, name: "host_encoding_base64_encode", required: &["text"], optional: &[] },
    HostFnDef { id: 131, f: HostFn::EncodingBase64Decode, name: "host_encoding_base64_decode", required: &["text"], optional: &[] },
    HostFnDef { id: 132, f: HostFn::EncodingHexEncode, name: "host_encoding_hex_encode", required: &["text"], optional: &[] },
    HostFnDef { id: 133, f: HostFn::EncodingHexDecode, name: "host_encoding_hex_decode", required: &["text"], optional: &[] },
    HostFnDef { id: 134, f: HostFn::PathJoin, name: "host_path_join", required: &["a", "b"], optional: &[] },
    HostFnDef { id: 135, f: HostFn::PathSplit, name: "host_path_split", required: &["path"], optional: &[] },
    HostFnDef { id: 136, f: HostFn::PathFileName, name: "host_path_file_name", required: &["path"], optional: &[] },
    HostFnDef { id: 137, f: HostFn::PathParent, name: "host_path_parent", required: &["path"], optional: &[] },
    HostFnDef { id: 138, f: HostFn::PathExtension, name: "host_path_extension", required: &["path"], optional: &[] },
    HostFnDef { id: 139, f: HostFn::PathNormalize, name: "host_path_normalize", required: &["path"], optional: &[] },
    HostFnDef { id: 140, f: HostFn::PathIsAbsolute, name: "host_path_is_absolute", required: &["path"], optional: &[] },
    HostFnDef { id: 141, f: HostFn::FsCopyFile, name: "host_copy_file", required: &["src", "dest"], optional: &[] },
    HostFnDef { id: 142, f: HostFn::FsMove, name: "host_move", required: &["src", "dest"], optional: &[] },
    HostFnDef { id: 143, f: HostFn::FsMakeTemp, name: "host_make_temp", required: &[], optional: &["prefix"] },
    HostFnDef { id: 144, f: HostFn::HashSha256, name: "host_hash_sha256", required: &["text"], optional: &[] },
    HostFnDef { id: 145, f: HostFn::HashSha1, name: "host_hash_sha1", required: &["text"], optional: &[] },
    HostFnDef { id: 146, f: HostFn::HashMd5, name: "host_hash_md5", required: &["text"], optional: &[] },
    HostFnDef { id: 147, f: HostFn::HashHmacSha256, name: "host_hash_hmac_sha256", required: &["key", "text"], optional: &[] },
    HostFnDef { id: 148, f: HostFn::SecretsTokenHex, name: "host_secrets_token_hex", required: &[], optional: &["n"] },
    HostFnDef { id: 149, f: HostFn::SecretsTokenUrlsafe, name: "host_secrets_token_urlsafe", required: &[], optional: &["n"] },
    HostFnDef { id: 150, f: HostFn::TextContains, name: "host_text_contains", required: &["text", "sub"], optional: &[] },
    HostFnDef { id: 151, f: HostFn::TextStartsWith, name: "host_text_starts_with", required: &["text", "prefix"], optional: &[] },
    HostFnDef { id: 152, f: HostFn::TextEndsWith, name: "host_text_ends_with", required: &["text", "suffix"], optional: &[] },
    HostFnDef { id: 153, f: HostFn::TextReplace, name: "host_text_replace", required: &["text", "old", "new"], optional: &["count"] },
    HostFnDef { id: 154, f: HostFn::TextToUpper, name: "host_text_to_upper", required: &["text"], optional: &[] },
    HostFnDef { id: 155, f: HostFn::TextToLower, name: "host_text_to_lower", required: &["text"], optional: &[] },
    HostFnDef { id: 156, f: HostFn::TextRepeat, name: "host_text_repeat", required: &["text", "n"], optional: &[] },
    HostFnDef { id: 157, f: HostFn::TextPad, name: "host_text_pad", required: &["text", "width"], optional: &["fill", "align"] },
    HostFnDef { id: 158, f: HostFn::CsvParse, name: "host_csv_parse", required: &["text"], optional: &[] },
    HostFnDef { id: 159, f: HostFn::CsvStringify, name: "host_csv_stringify", required: &["rows"], optional: &[] },
    HostFnDef { id: 160, f: HostFn::ListSort, name: "host_list_sort", required: &["list"], optional: &[] },
    HostFnDef { id: 161, f: HostFn::ListSortBy, name: "host_list_sort_by", required: &["list", "key"], optional: &[] },
    HostFnDef { id: 162, f: HostFn::ListUnique, name: "host_list_unique", required: &["list"], optional: &[] },
    HostFnDef { id: 163, f: HostFn::ListChunk, name: "host_list_chunk", required: &["list", "size"], optional: &[] },
    HostFnDef { id: 164, f: HostFn::ListZip, name: "host_list_zip", required: &["a", "b"], optional: &[] },
    HostFnDef { id: 165, f: HostFn::ListFlatten, name: "host_list_flatten", required: &["list"], optional: &[] },
    HostFnDef { id: 166, f: HostFn::CliParse, name: "host_cli_parse", required: &[], optional: &["args"] },
    HostFnDef { id: 167, f: HostFn::LogSetLevel, name: "host_log_set_level", required: &["level"], optional: &[] },
    HostFnDef { id: 168, f: HostFn::LogLine, name: "host_log_line", required: &["level", "text"], optional: &["fields"] },
    HostFnDef { id: 169, f: HostFn::UuidV4, name: "host_uuid_v4", required: &[], optional: &[] },
    HostFnDef { id: 170, f: HostFn::DatetimeNow, name: "host_datetime_now", required: &[], optional: &[] },
    HostFnDef { id: 171, f: HostFn::DatetimeFromUnix, name: "host_datetime_from_unix", required: &["unix"], optional: &["zone"] },
    HostFnDef { id: 172, f: HostFn::DatetimeToUnix, name: "host_datetime_to_unix", required: &["dt"], optional: &[] },
    HostFnDef { id: 173, f: HostFn::DatetimeParse, name: "host_datetime_parse", required: &["text"], optional: &["pattern"] },
    HostFnDef { id: 174, f: HostFn::DatetimeFormat, name: "host_datetime_format", required: &["dt"], optional: &["style"] },
    HostFnDef { id: 175, f: HostFn::DatetimeAdd, name: "host_datetime_add", required: &["dt"], optional: &["days", "hours", "minutes", "seconds"] },
    HostFnDef { id: 176, f: HostFn::DatetimeInZone, name: "host_datetime_in_zone", required: &["dt", "zone"], optional: &[] },
    HostFnDef { id: 177, f: HostFn::UrlParse, name: "host_url_parse", required: &["text"], optional: &[] },
    HostFnDef { id: 178, f: HostFn::UrlQueryParse, name: "host_url_query_parse", required: &["text"], optional: &[] },
    HostFnDef { id: 179, f: HostFn::UrlQueryStringify, name: "host_url_query_stringify", required: &["map"], optional: &[] },
    HostFnDef { id: 180, f: HostFn::TomlParse, name: "host_toml_parse", required: &["text"], optional: &[] },
    HostFnDef { id: 181, f: HostFn::HtmlEscape, name: "host_html_escape", required: &["text"], optional: &[] },
    HostFnDef { id: 182, f: HostFn::HtmlUnescape, name: "host_html_unescape", required: &["text"], optional: &[] },
    HostFnDef { id: 183, f: HostFn::EncodingBase32Encode, name: "host_encoding_base32_encode", required: &["text"], optional: &[] },
    HostFnDef { id: 184, f: HostFn::EncodingBase32Decode, name: "host_encoding_base32_decode", required: &["text"], optional: &[] },
    HostFnDef { id: 185, f: HostFn::FsMakeDirs, name: "host_make_dirs", required: &["path"], optional: &[] },
    HostFnDef { id: 186, f: HostFn::FsRemoveTree, name: "host_remove_tree", required: &["path"], optional: &[] },
    HostFnDef { id: 187, f: HostFn::FsStat, name: "host_stat", required: &["path"], optional: &[] },
    HostFnDef { id: 188, f: HostFn::FsWalk, name: "host_walk", required: &["path"], optional: &[] },
    HostFnDef { id: 189, f: HostFn::StatsMean, name: "host_stats_mean", required: &["list"], optional: &[] },
    HostFnDef { id: 190, f: HostFn::StatsMedian, name: "host_stats_median", required: &["list"], optional: &[] },
    HostFnDef { id: 191, f: HostFn::StatsStdev, name: "host_stats_stdev", required: &["list"], optional: &[] },
    HostFnDef { id: 192, f: HostFn::Meta, name: "host_meta", required: &[], optional: &[] },
    HostFnDef { id: 193, f: HostFn::MetaGet, name: "host_meta_get", required: &["key"], optional: &[] },
    HostFnDef { id: 194, f: HostFn::ModuleSource, name: "host_module_source", required: &[], optional: &[] },
    HostFnDef { id: 195, f: HostFn::ArtifactKind, name: "host_artifact_kind", required: &[], optional: &[] },
    HostFnDef { id: 196, f: HostFn::ForeignInvoke, name: "host_foreign_invoke", required: &[], optional: &["code", "lang", "source", "stdin", "input", "env", "cwd", "timeout", "args", "output", "max_output", "cmd", "keep"] },
    HostFnDef { id: 197, f: HostFn::ForeignProbe, name: "host_foreign_probe", required: &[], optional: &["lang"] },
    HostFnDef { id: 198, f: HostFn::ForeignRegister, name: "host_foreign_register", required: &["lang"], optional: &["cmd", "args", "template", "ext", "env", "version_args"] },
    HostFnDef { id: 199, f: HostFn::ForeignUnset, name: "host_foreign_unset", required: &["lang"], optional: &[] },
    HostFnDef { id: 200, f: HostFn::ForeignLoadConfig, name: "host_foreign_load_config", required: &[], optional: &["path"] },
    HostFnDef { id: 201, f: HostFn::ForeignCall, name: "host_foreign_call", required: &[], optional: &["code", "lang", "source", "input", "stdin", "env", "cwd", "timeout", "args"] },
    HostFnDef { id: 202, f: HostFn::ForeignRunMany, name: "host_foreign_run_many", required: &["runs"], optional: &["concurrency"] },
    HostFnDef { id: 203, f: HostFn::ForeignCallMany, name: "host_foreign_call_many", required: &["inputs"], optional: &["code", "lang", "source", "cmd", "env", "cwd", "timeout", "args", "max_output", "concurrency"] },
    HostFnDef { id: 204, f: HostFn::ForeignSessionOpen, name: "host_foreign_session_open", required: &["lang"], optional: &["cmd", "env", "cwd", "timeout"] },
    HostFnDef { id: 205, f: HostFn::ForeignSessionEval, name: "host_foreign_session_eval", required: &["id"], optional: &["code", "source", "input", "stdin", "timeout"] },
    HostFnDef { id: 206, f: HostFn::ForeignSessionSet, name: "host_foreign_session_set", required: &["id", "name", "value"], optional: &[] },
    HostFnDef { id: 207, f: HostFn::ForeignSessionGet, name: "host_foreign_session_get", required: &["id", "name"], optional: &[] },
    HostFnDef { id: 208, f: HostFn::ForeignSessionNames, name: "host_foreign_session_names", required: &["id"], optional: &[] },
    HostFnDef { id: 209, f: HostFn::ForeignSessionClose, name: "host_foreign_session_close", required: &["id"], optional: &[] },
    HostFnDef { id: 210, f: HostFn::ForeignSessionList, name: "host_foreign_session_list", required: &[], optional: &[] },
    HostFnDef { id: 211, f: HostFn::ForeignSessionInterrupt, name: "host_foreign_session_interrupt", required: &["id"], optional: &[] },
    HostFnDef { id: 212, f: HostFn::Access, name: "host_access", required: &["value", "key"], optional: &[] },
];


impl HostFn {
    pub fn from_u16(id: u16) -> Option<Self> {
        HOST_FN_TABLE.iter().find(|d| d.id == id).map(|d| d.f)
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
            // Aliases so optional `headers` / `content_type` bind like plot.
            "host_http_get" | "http_get" => Self::HttpGet,
            "host_http_post" | "http_post" => Self::HttpPost,
            "host_http_post_sse" | "http_post_sse" => Self::HttpPostSse,
            "host_openai_sse_parse" | "openai_sse_parse" => Self::OpenAiSseParse,
            "host_stream_publish" | "stream_publish" => Self::StreamPublish,
            "host_url_encode" => Self::UrlEncode,
            "host_pi" => Self::Pi,
            "host_e" => Self::EConst,
            "host_num" => Self::Num,
            "host_math_add" => Self::MathAdd,
            "host_math_sub" => Self::MathSub,
            "host_math_mul" => Self::MathMul,
            "host_math_div" => Self::MathDiv,
            "host_math_pow" => Self::MathPow,
            "host_math_neg" => Self::MathNeg,
            "host_math_sin" => Self::MathSin,
            "host_math_cos" => Self::MathCos,
            "host_math_tan" => Self::MathTan,
            "host_math_asin" => Self::MathAsin,
            "host_math_acos" => Self::MathAcos,
            "host_math_atan" => Self::MathAtan,
            "host_math_sqrt" => Self::MathSqrt,
            "host_math_abs" => Self::MathAbs,
            "host_math_ln" => Self::MathLn,
            "host_math_exp" => Self::MathExp,
            "host_math_floor" => Self::MathFloor,
            "host_math_ceil" => Self::MathCeil,
            "host_math_min" => Self::MathMin,
            "host_math_max" => Self::MathMax,
            "host_random" => Self::Random,
            "host_random_int" => Self::RandomInt,
            "host_seed" => Self::Seed,
            "host_formula" => Self::Formula,
            "host_simplify" => Self::Simplify,
            "host_expand" => Self::Expand,
            "host_diff" => Self::Diff,
            "host_subs" => Self::Subs,
            "host_eval" => Self::Eval,
            "host_solve" => Self::Solve,
            // Friendly aliases so L1 `plot` / `绘图` get host optionals (`grid`, `path`, …)
            // instead of thin wrappers that only forward required params.
            "host_plot" | "plot" | "绘图" => Self::Plot,
            "host_plot_points" | "plot_points" | "绘点列" => Self::PlotPoints,
            "host_plot_conic" | "plot_conic" | "绘圆锥" => Self::PlotConic,
            "host_foreign_set_cmd" => Self::ForeignSetCmd,
            "host_foreign_run" => Self::ForeignRun,
            "host_foreign_run_lang" => Self::ForeignRunLang,
            "host_foreign_invoke" | "foreign_invoke" => Self::ForeignInvoke,
            "host_foreign_probe" | "foreign_probe" => Self::ForeignProbe,
            "host_foreign_register" | "foreign_register" => Self::ForeignRegister,
            "host_foreign_unset" | "foreign_unset" => Self::ForeignUnset,
            "host_foreign_load_config" | "foreign_load_config" => Self::ForeignLoadConfig,
            "host_foreign_call" | "foreign_call" => Self::ForeignCall,
            "host_foreign_run_many" | "foreign_run_many" => Self::ForeignRunMany,
            "host_foreign_call_many" | "foreign_call_many" => Self::ForeignCallMany,
            "host_foreign_session_open" | "foreign_session_open" => Self::ForeignSessionOpen,
            "host_foreign_session_eval" | "foreign_session_eval" => Self::ForeignSessionEval,
            "host_foreign_session_set" | "foreign_session_set" => Self::ForeignSessionSet,
            "host_foreign_session_get" | "foreign_session_get" => Self::ForeignSessionGet,
            "host_foreign_session_names" | "foreign_session_names" => Self::ForeignSessionNames,
            "host_foreign_session_close" | "foreign_session_close" => Self::ForeignSessionClose,
            "host_foreign_session_list" | "foreign_session_list" => Self::ForeignSessionList,
            "host_foreign_session_interrupt" | "foreign_session_interrupt" => Self::ForeignSessionInterrupt,
            "host_foreign_langs" => Self::ForeignLangs,
            "host_dotenv_load" | "load_dotenv" => Self::DotenvLoad,
            "host_http_request" | "http_request" => Self::HttpRequest,
            "host_json_quote" | "quote" => Self::JsonQuote,
            "host_plugin_load" | "plugin_load" => Self::PluginLoad,
            "host_plugin_unload" | "plugin_unload" => Self::PluginUnload,
            "host_plugin_list" | "plugin_list" => Self::PluginList,
            "host_access" | "access" | "取" | "取值" => Self::Access,
            "host_ext_native_path" | "ext_native_path" => Self::ExtNativePath,
                                                "host_map_set" | "map_set" => Self::MapSet,
            "host_list_append" | "list_append" => Self::ListAppend,
                                                                                    "host_writeback_record" | "writeback_record" => Self::WritebackRecord,
            "host_writeback_get" | "writeback_get" => Self::WritebackGet,
            "host_writeback_clear" | "writeback_clear" => Self::WritebackClear,
            "host_writeback_list" | "writeback_list" => Self::WritebackList,
            "host_writeback_ensure" | "writeback_ensure" => Self::WritebackEnsure,
            "host_outer_call_line" | "outer_call_line" => Self::OuterCallLine,
            "host_text_patch" | "text_patch" => Self::TextPatch,
            "host_apply_patch_blocks" | "apply_patch_blocks" => Self::ApplyPatchBlocks,
            "host_writeback_scan_path" | "writeback_scan_path" => Self::WritebackScanPath,
            "host_subtask_spawn" | "subtask_spawn" => Self::SubtaskSpawn,
            "host_subtask_poll" | "subtask_poll" => Self::SubtaskPoll,
            "host_subtask_join" | "subtask_join" => Self::SubtaskJoin,
            "host_subtask_kill" | "subtask_kill" => Self::SubtaskKill,
            "host_subtask_wait_all" | "subtask_wait_all" => Self::SubtaskWaitAll,
            "host_collection_put" | "collection_put" => Self::CollectionPut,
            "host_list_prepend" | "list_prepend" => Self::ListPrepend,
            "host_list_concat" | "list_concat" => Self::ListConcat,
            "host_list_insert" | "list_insert" => Self::ListInsert,
            "host_list_set_at" | "list_set_at" => Self::ListSetAt,
            "host_list_remove_at" | "list_remove_at" => Self::ListRemoveAt,
            "host_list_slice" | "list_slice" => Self::ListSlice,
            "host_list_contains" | "list_contains" => Self::ListContains,
            "host_list_index_of" | "list_index_of" => Self::ListIndexOf,
            "host_list_reverse" | "list_reverse" => Self::ListReverse,
            "host_list_first" | "list_first" => Self::ListFirst,
            "host_list_last" | "list_last" => Self::ListLast,
            "host_map_get" | "map_get" => Self::MapGet,
            "host_map_delete" | "map_delete" => Self::MapDelete,
            "host_map_has" | "map_has" => Self::MapHas,
            "host_map_keys" | "map_keys" => Self::MapKeys,
            "host_map_values" | "map_values" => Self::MapValues,
            "host_map_items" | "map_items" => Self::MapItems,
            "host_map_merge" | "map_merge" => Self::MapMerge,
            "host_map_size" | "map_size" => Self::MapSize,
            "host_collection_clear" | "collection_clear" => Self::CollectionClear,
            "host_cookie_parse" | "cookie_parse" => Self::CookieParse,
            "host_multipart_parse" | "multipart_parse" => Self::MultipartParse,
            "host_markdown_parse" | "markdown_parse" => Self::MarkdownParse,
            "host_re_is_match" | "re_is_match" => Self::ReIsMatch,
            "host_re_find" | "re_find" => Self::ReFind,
            "host_re_find_all" | "re_find_all" => Self::ReFindAll,
            "host_re_replace" | "re_replace" => Self::ReReplace,
            "host_re_split" | "re_split" => Self::ReSplit,
            "host_encoding_base64_encode" | "encoding_base64_encode" => Self::EncodingBase64Encode,
            "host_encoding_base64_decode" | "encoding_base64_decode" => Self::EncodingBase64Decode,
            "host_encoding_hex_encode" | "encoding_hex_encode" => Self::EncodingHexEncode,
            "host_encoding_hex_decode" | "encoding_hex_decode" => Self::EncodingHexDecode,
            "host_path_join" | "path_join" => Self::PathJoin,
            "host_path_split" | "path_split" => Self::PathSplit,
            "host_path_file_name" | "path_file_name" => Self::PathFileName,
            "host_path_parent" | "path_parent" => Self::PathParent,
            "host_path_extension" | "path_extension" => Self::PathExtension,
            "host_path_normalize" | "path_normalize" => Self::PathNormalize,
            "host_path_is_absolute" | "path_is_absolute" => Self::PathIsAbsolute,
            "host_copy_file" | "copy_file" => Self::FsCopyFile,
            "host_move" | "move_path" => Self::FsMove,
            "host_make_temp" | "make_temp" => Self::FsMakeTemp,
            "host_hash_sha256" | "hash_sha256" => Self::HashSha256,
            "host_hash_sha1" | "hash_sha1" => Self::HashSha1,
            "host_hash_md5" | "hash_md5" => Self::HashMd5,
            "host_hash_hmac_sha256" | "hash_hmac_sha256" => Self::HashHmacSha256,
            "host_secrets_token_hex" | "secrets_token_hex" => Self::SecretsTokenHex,
            "host_secrets_token_urlsafe" | "secrets_token_urlsafe" => Self::SecretsTokenUrlsafe,
            "host_text_contains" | "text_contains" => Self::TextContains,
            "host_text_starts_with" | "text_starts_with" => Self::TextStartsWith,
            "host_text_ends_with" | "text_ends_with" => Self::TextEndsWith,
            "host_text_replace" | "text_replace" => Self::TextReplace,
            "host_text_to_upper" | "text_to_upper" => Self::TextToUpper,
            "host_text_to_lower" | "text_to_lower" => Self::TextToLower,
            "host_text_repeat" | "text_repeat" => Self::TextRepeat,
            "host_text_pad" | "text_pad" => Self::TextPad,
            "host_csv_parse" | "csv_parse" => Self::CsvParse,
            "host_csv_stringify" | "csv_stringify" => Self::CsvStringify,
            "host_list_sort" | "list_sort" => Self::ListSort,
            "host_list_sort_by" | "list_sort_by" => Self::ListSortBy,
            "host_list_unique" | "list_unique" => Self::ListUnique,
            "host_list_chunk" | "list_chunk" => Self::ListChunk,
            "host_list_zip" | "list_zip" => Self::ListZip,
            "host_list_flatten" | "list_flatten" => Self::ListFlatten,
            "host_cli_parse" | "cli_parse" => Self::CliParse,
            "host_log_set_level" | "log_set_level" => Self::LogSetLevel,
            "host_log_line" | "log_line" => Self::LogLine,
            "host_uuid_v4" | "uuid_v4" => Self::UuidV4,
            "host_datetime_now" | "datetime_now" => Self::DatetimeNow,
            "host_datetime_from_unix" | "datetime_from_unix" => Self::DatetimeFromUnix,
            "host_datetime_to_unix" | "datetime_to_unix" => Self::DatetimeToUnix,
            "host_datetime_parse" | "datetime_parse" => Self::DatetimeParse,
            "host_datetime_format" | "datetime_format" => Self::DatetimeFormat,
            "host_datetime_add" | "datetime_add" => Self::DatetimeAdd,
            "host_datetime_in_zone" | "datetime_in_zone" => Self::DatetimeInZone,
            "host_url_parse" | "url_parse" => Self::UrlParse,
            "host_url_query_parse" | "url_query_parse" => Self::UrlQueryParse,
            "host_url_query_stringify" | "url_query_stringify" => Self::UrlQueryStringify,
            "host_toml_parse" | "toml_parse" => Self::TomlParse,
            "host_html_escape" | "html_escape" => Self::HtmlEscape,
            "host_html_unescape" | "html_unescape" => Self::HtmlUnescape,
            "host_encoding_base32_encode" | "encoding_base32_encode" => Self::EncodingBase32Encode,
            "host_encoding_base32_decode" | "encoding_base32_decode" => Self::EncodingBase32Decode,
            "host_make_dirs" | "make_dirs" => Self::FsMakeDirs,
            "host_remove_tree" | "remove_tree" => Self::FsRemoveTree,
            "host_stat" | "stat" => Self::FsStat,
            "host_walk" | "walk" => Self::FsWalk,
            "host_stats_mean" | "stats_mean" => Self::StatsMean,
            "host_stats_median" | "stats_median" => Self::StatsMedian,
            "host_stats_stdev" | "stats_stdev" => Self::StatsStdev,
            "host_meta" | "meta" => Self::Meta,
            "host_meta_get" | "meta_get" => Self::MetaGet,
            "host_module_source" | "module_source" => Self::ModuleSource,
            "host_artifact_kind" | "artifact_kind" => Self::ArtifactKind,
            _ => return None,
        })
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }

    pub fn name(self) -> &'static str {
        HOST_FN_TABLE
            .iter()
            .find(|d| d.f.as_u16() == self.as_u16())
            .map(|d| d.name)
            .unwrap_or("host_unknown")
    }

    pub fn required_params(self) -> &'static [&'static str] {
        HOST_FN_TABLE
            .iter()
            .find(|d| d.f.as_u16() == self.as_u16())
            .map(|d| d.required)
            .unwrap_or(&[])
    }

    pub fn optional_params(self) -> &'static [&'static str] {
        HOST_FN_TABLE
            .iter()
            .find(|d| d.f.as_u16() == self.as_u16())
            .map(|d| d.optional)
            .unwrap_or(&[])
    }

    pub fn all_params(self) -> Vec<&'static str> {
        let mut v = self.required_params().to_vec();
        v.extend_from_slice(self.optional_params());
        v
    }
}

pub fn call_host(
    ctx: &mut HostContext,
    fn_id: HostFn,
    bound: &HashMap<String, Value>,
) -> Result<Value, String> {
    match fn_id {
        HostFn::Access => {
            let base = require(bound, "value")?;
            let key = match require(bound, "key")? {
                Value::Text(t) => t.clone(),
                Value::Int(i) => i.to_string(),
                other => crate::builtin::builtin_str(other).as_display(),
            };
            crate::builtin::builtin_footnote_get(base, &key)
        }
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
        HostFn::FsMakeDirs => fs::make_dirs(ctx, require(bound, "path")?),
        HostFn::FsRemoveTree => fs::remove_tree(ctx, require(bound, "path")?),
        HostFn::FsStat => fs::stat(ctx, require(bound, "path")?),
        HostFn::FsWalk => fs::walk(ctx, require(bound, "path")?),
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
        HostFn::DotenvLoad => sys::dotenv_load(ctx, bound.get("path")),
        HostFn::Args => sys::args(ctx),
        HostFn::Cwd => sys::cwd(ctx),
        HostFn::Meta => sys::meta(ctx),
        HostFn::MetaGet => sys::meta_get(ctx, require(bound, "key")?),
        HostFn::ModuleSource => crate::host::agent_rt::module_source(ctx),
        HostFn::ArtifactKind => sys::artifact_kind(ctx),
        HostFn::Exit => sys::exit(ctx, require(bound, "code")?),
        HostFn::Exec => sys::exec(
            ctx,
            require(bound, "cmd")?,
            bound.get("args"),
            bound.get("capture"),
        ),
        HostFn::JsonParse => json::parse(require(bound, "text")?),
        HostFn::JsonStringify => {
            json::stringify(require(bound, "value")?, bound.get("indent"))
        }
        HostFn::JsonGet => json::get(require(bound, "value")?, require(bound, "key")?),
        HostFn::JsonKeys => json::keys(require(bound, "value")?),
        HostFn::JsonQuote => json::quote(require(bound, "text")?),
        HostFn::HttpGet => {
            #[cfg(feature = "net-host")]
            {
                net::http_get(ctx, require(bound, "url")?, bound.get("headers"))
            }
            #[cfg(not(feature = "net-host"))]
            {
                let _ = (ctx, bound);
                Err("http_get unavailable in browser wasm".into())
            }
        }
        HostFn::HttpPost => {
            #[cfg(feature = "net-host")]
            {
                net::http_post(
                    ctx,
                    require(bound, "url")?,
                    require(bound, "body")?,
                    bound.get("content_type"),
                    bound.get("headers"),
                )
            }
            #[cfg(not(feature = "net-host"))]
            {
                let _ = (ctx, bound);
                Err("http_post unavailable in browser wasm".into())
            }
        }
        HostFn::HttpPostSse => {
            #[cfg(feature = "net-host")]
            {
                net::http_post_sse(
                    ctx,
                    require(bound, "url")?,
                    require(bound, "body")?,
                    bound.get("content_type"),
                    bound.get("headers"),
                    bound.get("echo"),
                )
            }
            #[cfg(not(feature = "net-host"))]
            {
                let _ = (ctx, bound);
                Err("http_post_sse unavailable in browser wasm".into())
            }
        }
        HostFn::OpenAiSseParse => {
            net::openai_sse_parse(require(bound, "text")?, bound.get("echo"))
        }
        HostFn::StreamPublish => {
            crate::host::event_bus::publish(require(bound, "event")?);
            Ok(Value::Bool(true))
        }
        HostFn::HttpRequest => {
            #[cfg(feature = "net-host")]
            {
                net::http_request(
                    ctx,
                    require(bound, "method")?,
                    require(bound, "url")?,
                    bound.get("body"),
                    bound.get("content_type"),
                    bound.get("headers"),
                )
            }
            #[cfg(not(feature = "net-host"))]
            {
                let _ = (ctx, bound);
                Err("http_request unavailable in browser wasm".into())
            }
        }
        HostFn::UrlEncode => net::url_encode(require(bound, "text")?),
        HostFn::Pi => math::pi(),
        HostFn::EConst => math::e_const(),
        HostFn::Num => math::to_num(require(bound, "value")?),
        HostFn::MathAdd => math::binary_num("add", require(bound, "a")?, require(bound, "b")?),
        HostFn::MathSub => math::binary_num("sub", require(bound, "a")?, require(bound, "b")?),
        HostFn::MathMul => math::binary_num("mul", require(bound, "a")?, require(bound, "b")?),
        HostFn::MathDiv => math::binary_num("div", require(bound, "a")?, require(bound, "b")?),
        HostFn::MathPow => math::binary_num("pow", require(bound, "a")?, require(bound, "b")?),
        HostFn::MathMin => math::binary_num("min", require(bound, "a")?, require(bound, "b")?),
        HostFn::MathMax => math::binary_num("max", require(bound, "a")?, require(bound, "b")?),
        HostFn::MathNeg => math::unary_num("neg", require(bound, "value")?),
        HostFn::MathSin => math::unary_num("sin", require(bound, "value")?),
        HostFn::MathCos => math::unary_num("cos", require(bound, "value")?),
        HostFn::MathTan => math::unary_num("tan", require(bound, "value")?),
        HostFn::MathAsin => math::unary_num("asin", require(bound, "value")?),
        HostFn::MathAcos => math::unary_num("acos", require(bound, "value")?),
        HostFn::MathAtan => math::unary_num("atan", require(bound, "value")?),
        HostFn::MathSqrt => math::unary_num("sqrt", require(bound, "value")?),
        HostFn::MathAbs => math::unary_num("abs", require(bound, "value")?),
        HostFn::MathLn => math::unary_num("ln", require(bound, "value")?),
        HostFn::MathExp => math::unary_num("exp", require(bound, "value")?),
        HostFn::MathFloor => math::unary_num("floor", require(bound, "value")?),
        HostFn::MathCeil => math::unary_num("ceil", require(bound, "value")?),
        HostFn::Random => math::random(ctx),
        HostFn::RandomInt => {
            math::random_int(ctx, require(bound, "min")?, require(bound, "max")?)
        }
        HostFn::Seed => math::seed(ctx, require(bound, "value")?),
        HostFn::Formula => math::formula_parse(require(bound, "value")?),
        HostFn::Simplify => math::simplify(require(bound, "value")?),
        HostFn::Expand => math::expand(require(bound, "value")?),
        HostFn::Diff => math::diff(require(bound, "formula")?, require(bound, "var")?),
        HostFn::Subs => math::subs(
            require(bound, "formula")?,
            require(bound, "var")?,
            require(bound, "value")?,
        ),
        HostFn::Eval => math::eval_at(
            require(bound, "formula")?,
            require(bound, "var")?,
            require(bound, "value")?,
        ),
        HostFn::Solve => math::solve(
            require(bound, "formula")?,
            require(bound, "var")?,
            bound.get("min"),
            bound.get("max"),
        ),
        HostFn::Plot => math::plot(
            ctx,
            require(bound, "formula")?,
            require(bound, "var")?,
            require(bound, "min")?,
            require(bound, "max")?,
            bound.get("steps"),
            bound.get("path"),
            bound.get("derivative"),
            bound.get("grid"),
        ),
        HostFn::PlotPoints => math::plot_points(
            ctx,
            require(bound, "xs")?,
            require(bound, "ys")?,
            bound.get("path"),
            bound.get("grid"),
        ),
        HostFn::PlotConic => math::plot_conic(
            ctx,
            require(bound, "kind")?,
            require(bound, "a")?,
            bound.get("b"),
            bound.get("h"),
            bound.get("k"),
            bound.get("path"),
            bound.get("grid"),
        ),
        HostFn::ForeignSetCmd => {
            #[cfg(feature = "exec-host")]
            {
                foreign::set_cmd(
                    ctx,
                    require(bound, "lang")?,
                    require(bound, "cmd")?,
                    bound.get("args"),
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignRun => {
            #[cfg(feature = "exec-host")]
            {
                foreign::run_with_options(
                    ctx,
                    require(bound, "code")?,
                    bound.get("stdin"),
                    bound,
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignRunLang => {
            #[cfg(feature = "exec-host")]
            {
                foreign::run_lang_with_options(
                    ctx,
                    require(bound, "lang")?,
                    require(bound, "source")?,
                    bound.get("stdin"),
                    bound,
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignInvoke => {
            #[cfg(feature = "exec-host")]
            {
                foreign::invoke(
                    ctx,
                    foreign::InvokeArgs {
                        code: bound.get("code"),
                        lang: bound.get("lang"),
                        source: bound.get("source"),
                        stdin: bound.get("stdin"),
                        input: bound.get("input"),
                        env: bound.get("env"),
                        cwd: bound.get("cwd"),
                        timeout: bound.get("timeout"),
                        args: bound.get("args"),
                        output: bound.get("output"),
                        max_output: bound.get("max_output"),
                        cmd: bound.get("cmd"),
                        keep: bound.get("keep"),
                    },
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignProbe => {
            #[cfg(feature = "exec-host")]
            {
                foreign::probe(ctx, bound.get("lang"))
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignRegister => {
            #[cfg(feature = "exec-host")]
            {
                foreign::register(
                    ctx,
                    require(bound, "lang")?,
                    bound.get("cmd"),
                    bound.get("args"),
                    bound.get("template"),
                    bound.get("ext"),
                    bound.get("env"),
                    bound.get("version_args"),
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignUnset => {
            #[cfg(feature = "exec-host")]
            {
                foreign::unset(ctx, require(bound, "lang")?)
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignLoadConfig => {
            #[cfg(feature = "exec-host")]
            {
                foreign::load_config(ctx, bound.get("path"))
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignCall => {
            #[cfg(feature = "exec-host")]
            {
                foreign::call(
                    ctx,
                    foreign::InvokeArgs {
                        code: bound.get("code"),
                        lang: bound.get("lang"),
                        source: bound.get("source"),
                        stdin: bound.get("stdin"),
                        input: bound.get("input"),
                        env: bound.get("env"),
                        cwd: bound.get("cwd"),
                        timeout: bound.get("timeout"),
                        args: bound.get("args"),
                        output: None,
                        max_output: None,
                        cmd: None,
                        keep: None,
                    },
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignRunMany => {
            #[cfg(feature = "exec-host")]
            {
                foreign::run_many(
                    ctx,
                    require(bound, "runs")?,
                    bound.get("concurrency"),
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignCallMany => {
            #[cfg(feature = "exec-host")]
            {
                foreign::call_many(
                    ctx,
                    foreign::InvokeArgs {
                        code: bound.get("code"),
                        lang: bound.get("lang"),
                        source: bound.get("source"),
                        stdin: None,
                        input: None,
                        env: bound.get("env"),
                        cwd: bound.get("cwd"),
                        timeout: bound.get("timeout"),
                        args: bound.get("args"),
                        output: None,
                        max_output: bound.get("max_output"),
                        cmd: bound.get("cmd"),
                        keep: None,
                    },
                    require(bound, "inputs")?,
                    bound.get("concurrency"),
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignSessionOpen => {
            #[cfg(feature = "exec-host")]
            {
                crate::host::foreign_session::open(
                    ctx,
                    require(bound, "lang")?,
                    bound.get("cmd"),
                    bound.get("env"),
                    bound.get("cwd"),
                    bound.get("timeout"),
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignSessionEval => {
            #[cfg(feature = "exec-host")]
            {
                crate::host::foreign_session::eval_with_stdin(
                    ctx,
                    require(bound, "id")?,
                    bound.get("code"),
                    bound.get("source"),
                    bound.get("input"),
                    bound.get("stdin").and_then(|v| v.reveal_text()),
                    bound.get("timeout"),
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignSessionSet => {
            #[cfg(feature = "exec-host")]
            {
                crate::host::foreign_session::set(
                    ctx,
                    require(bound, "id")?,
                    require(bound, "name")?,
                    require(bound, "value")?,
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignSessionGet => {
            #[cfg(feature = "exec-host")]
            {
                crate::host::foreign_session::get(
                    ctx,
                    require(bound, "id")?,
                    require(bound, "name")?,
                )
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignSessionNames => {
            #[cfg(feature = "exec-host")]
            {
                crate::host::foreign_session::names(ctx, require(bound, "id")?)
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignSessionClose => {
            #[cfg(feature = "exec-host")]
            {
                crate::host::foreign_session::close(ctx, require(bound, "id")?)
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignSessionInterrupt => {
            #[cfg(feature = "exec-host")]
            {
                crate::host::foreign_session::interrupt(ctx, require(bound, "id")?)
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("foreign unavailable in browser wasm".into())
            }
        }
        HostFn::ForeignSessionList => {
            #[cfg(feature = "exec-host")]
            {
                crate::host::foreign_session::list(ctx)
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = ctx;
                Ok(Value::List(Vec::new()))
            }
        }
        HostFn::ForeignLangs => {
            #[cfg(feature = "exec-host")]
            {
                foreign::langs(ctx)
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = ctx;
                Ok(Value::List(Vec::new()))
            }
        }
        HostFn::PluginLoad => {
            #[cfg(feature = "plugin-host")]
            {
                plugin::load(ctx, require(bound, "path")?)
            }
            #[cfg(not(feature = "plugin-host"))]
            {
                let _ = (ctx, bound);
                Err("plugin.load unavailable in browser wasm".into())
            }
        }
        HostFn::PluginUnload => {
            #[cfg(feature = "plugin-host")]
            {
                plugin::unload(ctx)
            }
            #[cfg(not(feature = "plugin-host"))]
            {
                let _ = ctx;
                Err("plugin.unload unavailable in browser wasm".into())
            }
        }
        HostFn::PluginList => {
            #[cfg(feature = "plugin-host")]
            {
                plugin::list(ctx)
            }
            #[cfg(not(feature = "plugin-host"))]
            {
                let _ = ctx;
                Ok(Value::List(Vec::new()))
            }
        }
        HostFn::ExtNativePath => {
            let name = require(bound, "name")?;
            let s = match name {
                Value::Text(t) => t.as_str(),
                _ => return Err("name must be text".into()),
            };
            #[cfg(feature = "cli")]
            {
                Ok(match crate::ext_cli::installed_native_path(s) {
                    Some(p) => Value::Text(p.to_string_lossy().into_owned()),
                    None => Value::None,
                })
            }
            #[cfg(not(feature = "cli"))]
            {
                let _ = s;
                Ok(Value::None)
            }
        }
        HostFn::MapSet => {
            super::collection::map_set(require(bound, "map")?, require(bound, "key")?, require(bound, "value")?)
        }
        HostFn::ListAppend => {
            super::collection::list_append(require(bound, "list")?, require(bound, "item")?)
        }
        HostFn::CollectionPut => super::collection::collection_put(
            require(bound, "in")?,
            require(bound, "at")?,
            require(bound, "value")?,
        ),
        HostFn::ListPrepend => {
            super::collection::list_prepend(require(bound, "list")?, require(bound, "item")?)
        }
        HostFn::ListConcat => {
            super::collection::list_concat(require(bound, "a")?, require(bound, "b")?)
        }
        HostFn::ListInsert => {
            let idx = match require(bound, "index")? {
                Value::Int(n) => *n,
                _ => return Err("index must be int".into()),
            };
            super::collection::list_insert(require(bound, "list")?, idx, require(bound, "item")?)
        }
        HostFn::ListSetAt => {
            let idx = match require(bound, "index")? {
                Value::Int(n) => *n,
                _ => return Err("index must be int".into()),
            };
            super::collection::list_set_at(require(bound, "list")?, idx, require(bound, "item")?)
        }
        HostFn::ListRemoveAt => {
            let idx = match require(bound, "index")? {
                Value::Int(n) => *n,
                _ => return Err("index must be int".into()),
            };
            super::collection::list_remove_at(require(bound, "list")?, idx)
        }
        HostFn::ListSlice => super::collection::list_slice_bound(
            require(bound, "list")?,
            require(bound, "start")?,
            bound.get("end"),
        ),
        HostFn::ListContains => {
            super::collection::list_contains(require(bound, "list")?, require(bound, "item")?)
        }
        HostFn::ListIndexOf => {
            super::collection::list_index_of(require(bound, "list")?, require(bound, "item")?)
        }
        HostFn::ListReverse => super::collection::list_reverse(require(bound, "list")?),
        HostFn::ListFirst => super::collection::list_first(require(bound, "list")?),
        HostFn::ListLast => super::collection::list_last(require(bound, "list")?),
        HostFn::MapGet => super::collection::map_get(require(bound, "map")?, require(bound, "key")?),
        HostFn::MapDelete => super::collection::map_delete(require(bound, "map")?, require(bound, "key")?),
        HostFn::MapHas => super::collection::map_has(require(bound, "map")?, require(bound, "key")?),
        HostFn::MapKeys => super::collection::map_keys(require(bound, "map")?),
        HostFn::MapValues => super::collection::map_values(require(bound, "map")?),
        HostFn::MapItems => super::collection::map_items(require(bound, "map")?),
        HostFn::MapMerge => super::collection::map_merge(require(bound, "a")?, require(bound, "b")?),
        HostFn::MapSize => super::collection::map_size(require(bound, "map")?),
        HostFn::CollectionClear => super::collection::collection_clear(require(bound, "value")?),
        HostFn::CookieParse => net::cookie_parse(require(bound, "text")?, bound.get("is_response")),
        HostFn::MultipartParse => net::multipart_parse(require(bound, "body")?, require(bound, "boundary")?),
        HostFn::MarkdownParse => net::markdown_parse(require(bound, "text")?),
        HostFn::ReIsMatch => re::is_match(require(bound, "text")?, require(bound, "pattern")?),
        HostFn::ReFind => re::find(require(bound, "text")?, require(bound, "pattern")?),
        HostFn::ReFindAll => re::find_all(require(bound, "text")?, require(bound, "pattern")?),
        HostFn::ReReplace => re::replace(
            require(bound, "text")?,
            require(bound, "pattern")?,
            require(bound, "with")?,
            bound.get("count"),
        ),
        HostFn::ReSplit => re::split(require(bound, "text")?, require(bound, "pattern")?),
        HostFn::EncodingBase64Encode => encoding::base64_encode(require(bound, "text")?),
        HostFn::EncodingBase64Decode => encoding::base64_decode(require(bound, "text")?),
        HostFn::EncodingHexEncode => encoding::hex_encode(require(bound, "text")?),
        HostFn::EncodingHexDecode => encoding::hex_decode(require(bound, "text")?),
        HostFn::EncodingBase32Encode => encoding::base32_encode(require(bound, "text")?),
        HostFn::EncodingBase32Decode => encoding::base32_decode(require(bound, "text")?),
        HostFn::TomlParse => toml_ops::parse(require(bound, "text")?),
        HostFn::HtmlEscape => html_ops::escape(require(bound, "text")?),
        HostFn::HtmlUnescape => html_ops::unescape(require(bound, "text")?),
        HostFn::PathJoin => path::join(require(bound, "a")?, require(bound, "b")?),
        HostFn::PathSplit => path::split(require(bound, "path")?),
        HostFn::PathFileName => path::file_name(require(bound, "path")?),
        HostFn::PathParent => path::parent(require(bound, "path")?),
        HostFn::PathExtension => path::extension(require(bound, "path")?),
        HostFn::PathNormalize => path::normalize(require(bound, "path")?),
        HostFn::PathIsAbsolute => path::is_absolute(require(bound, "path")?),
        HostFn::FsCopyFile => {
            fs::copy_file(ctx, require(bound, "src")?, require(bound, "dest")?)
        }
        HostFn::FsMove => fs::move_path(ctx, require(bound, "src")?, require(bound, "dest")?),
        HostFn::FsMakeTemp => fs::make_temp(ctx, bound.get("prefix")),
        HostFn::HashSha256 => hash::sha256(require(bound, "text")?),
        HostFn::HashSha1 => hash::sha1(require(bound, "text")?),
        HostFn::HashMd5 => hash::md5(require(bound, "text")?),
        HostFn::HashHmacSha256 => {
            hash::hmac_sha256(require(bound, "key")?, require(bound, "text")?)
        }
        HostFn::SecretsTokenHex => secrets::token_hex(bound.get("n")),
        HostFn::SecretsTokenUrlsafe => secrets::token_urlsafe(bound.get("n")),
        HostFn::TextContains => {
            text_ops::contains(require(bound, "text")?, require(bound, "sub")?)
        }
        HostFn::TextStartsWith => {
            text_ops::starts_with(require(bound, "text")?, require(bound, "prefix")?)
        }
        HostFn::TextEndsWith => {
            text_ops::ends_with(require(bound, "text")?, require(bound, "suffix")?)
        }
        HostFn::TextReplace => text_ops::replace(
            require(bound, "text")?,
            require(bound, "old")?,
            require(bound, "new")?,
            bound.get("count"),
        ),
        HostFn::TextToUpper => text_ops::to_upper(require(bound, "text")?),
        HostFn::TextToLower => text_ops::to_lower(require(bound, "text")?),
        HostFn::TextRepeat => text_ops::repeat(require(bound, "text")?, require(bound, "n")?),
        HostFn::TextPad => text_ops::pad(
            require(bound, "text")?,
            require(bound, "width")?,
            bound.get("fill"),
            bound.get("align"),
        ),
        HostFn::CsvParse => csv::parse(require(bound, "text")?),
        HostFn::CsvStringify => csv::stringify(require(bound, "rows")?),
        HostFn::ListSort => super::collection::list_sort(require(bound, "list")?),
        HostFn::ListSortBy => {
            super::collection::list_sort_by(require(bound, "list")?, require(bound, "key")?)
        }
        HostFn::ListUnique => super::collection::list_unique(require(bound, "list")?),
        HostFn::ListChunk => {
            super::collection::list_chunk(require(bound, "list")?, require(bound, "size")?)
        }
        HostFn::ListZip => {
            super::collection::list_zip(require(bound, "a")?, require(bound, "b")?)
        }
        HostFn::ListFlatten => super::collection::list_flatten(require(bound, "list")?),
        HostFn::CliParse => cli::parse(ctx, bound.get("args")),
        HostFn::LogSetLevel => log::set_level(ctx, require(bound, "level")?),
        HostFn::LogLine => log::line(
            ctx,
            require(bound, "level")?,
            require(bound, "text")?,
            bound.get("fields"),
        ),
        HostFn::StatsMean => stats::mean(require(bound, "list")?),
        HostFn::StatsMedian => stats::median(require(bound, "list")?),
        HostFn::StatsStdev => stats::stdev(require(bound, "list")?),
        HostFn::UuidV4 => uuid_ops::v4(),
        HostFn::DatetimeNow => datetime::now(),
        HostFn::DatetimeFromUnix => {
            datetime::from_unix(require(bound, "unix")?, bound.get("zone"))
        }
        HostFn::DatetimeToUnix => datetime::to_unix(require(bound, "dt")?),
        HostFn::DatetimeParse => datetime::parse(require(bound, "text")?, bound.get("pattern")),
        HostFn::DatetimeFormat => datetime::format(require(bound, "dt")?, bound.get("style")),
        HostFn::DatetimeAdd => datetime::add(
            require(bound, "dt")?,
            bound.get("days"),
            bound.get("hours"),
            bound.get("minutes"),
            bound.get("seconds"),
        ),
        HostFn::DatetimeInZone => {
            datetime::in_zone(require(bound, "dt")?, require(bound, "zone")?)
        }
        HostFn::UrlParse => url_ops::parse(require(bound, "text")?),
        HostFn::UrlQueryParse => url_ops::query_parse(require(bound, "text")?),
        HostFn::UrlQueryStringify => url_ops::query_stringify(require(bound, "map")?),
        HostFn::OuterCallLine => Ok(Value::Int(
            ctx.call_site_lines
                .first()
                .copied()
                .unwrap_or(ctx.current_line) as i64,
        )),
        HostFn::TextPatch => fs::text_patch(
            ctx,
            require(bound, "path")?,
            require(bound, "find")?,
            require(bound, "replace")?,
        ),
        HostFn::ApplyPatchBlocks => fs::apply_patch_blocks(
            ctx,
            require(bound, "path")?,
            require(bound, "text")?,
            bound.get("soft"),
        ),
        HostFn::WritebackScanPath => writeback::scan_path(ctx, require(bound, "path")?),
        HostFn::WritebackRecord => writeback::record(
            ctx,
            require(bound, "value")?,
            bound.get("at_end"),
            bound.get("key"),
            bound.get("line"),
        ),
        HostFn::WritebackGet => writeback::get(
            ctx,
            bound.get("at_end"),
            bound.get("key"),
            bound.get("line"),
        ),
        HostFn::WritebackClear => writeback::clear(
            ctx,
            bound.get("at_end"),
            bound.get("key"),
            bound.get("line"),
        ),
        HostFn::WritebackList => writeback::list(ctx),
        HostFn::WritebackEnsure => {
            let placeholder = bound
                .get("placeholder")
                .cloned()
                .unwrap_or_else(|| Value::Text("pending".into()));
            writeback::ensure_slot(
                ctx,
                require(bound, "key")?,
                &placeholder,
                bound.get("line"),
            )
        }
        HostFn::SubtaskSpawn => {
            #[cfg(feature = "exec-host")]
            {
                subtask::spawn(ctx, bound)
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("subtask unavailable in browser wasm".into())
            }
        }
        HostFn::SubtaskPoll => {
            #[cfg(feature = "exec-host")]
            {
                subtask::poll(ctx, require(bound, "id")?)
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("subtask unavailable in browser wasm".into())
            }
        }
        HostFn::SubtaskJoin => {
            #[cfg(feature = "exec-host")]
            {
                subtask::join(ctx, require(bound, "id")?)
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("subtask unavailable in browser wasm".into())
            }
        }
        HostFn::SubtaskKill => {
            #[cfg(feature = "exec-host")]
            {
                subtask::kill(ctx, require(bound, "id")?)
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = (ctx, bound);
                Err("subtask unavailable in browser wasm".into())
            }
        }
        HostFn::SubtaskWaitAll => {
            #[cfg(feature = "exec-host")]
            {
                subtask::wait_all(ctx)
            }
            #[cfg(not(feature = "exec-host"))]
            {
                let _ = ctx;
                Err("subtask unavailable in browser wasm".into())
            }
        }
    }
}

fn require<'a>(bound: &'a HashMap<String, Value>, key: &str) -> Result<&'a Value, String> {
    bound
        .get(key)
        .ok_or_else(|| format!("host call missing `{key}`"))
}

#[cfg(test)]
mod tests {
    use super::HostFn;

    /// Every host id must round-trip through `from_u16` / `as_u16`, and every
    /// canonical name through `from_name` / `name`. This pins the drift the
    /// audit found (`from_name` accepted `http_get` while `name()` emitted
    /// `host_http_get`) without rewriting the five tables.
    #[test]
    fn every_assigned_id_round_trips() {
        // The id space is intentionally sparse: retired ids are never reused
        // (bytecode from an older build must not silently change meaning).
        let mut seen = 0;
        for id in 1..=400u16 {
            let Some(f) = HostFn::from_u16(id) else {
                continue; // retired id
            };
            seen += 1;
            assert_eq!(f.as_u16(), id, "id {id} → {f:?} → {}", f.as_u16());
        }
        assert!(seen >= 200, "expected ~200 live host ids, got {seen}");
    }

    #[test]
    fn every_id_has_a_name_that_round_trips() {
        for id in 1..=300u16 {
            let Some(f) = HostFn::from_u16(id) else {
                continue;
            };
            let name = f.name();
            assert!(!name.is_empty(), "id {id} has an empty name");
            assert_eq!(
                HostFn::from_name(name),
                Some(f),
                "id {id}: name `{name}` does not round-trip"
            );
        }
    }

    #[test]
    fn names_are_unique() {
        let mut seen = std::collections::HashMap::new();
        for id in 1..=300u16 {
            let Some(f) = HostFn::from_u16(id) else {
                continue;
            };
            let name = f.name();
            if let Some(prev) = seen.insert(name.to_string(), id) {
                panic!("duplicate host name `{name}` at ids {prev} and {id}");
            }
        }
    }

    #[test]
    fn required_and_optional_params_are_disjoint() {
        for id in 1..=300u16 {
            let Some(f) = HostFn::from_u16(id) else {
                continue;
            };
            let req = f.required_params();
            let opt = f.optional_params();
            for r in req {
                assert!(
                    !opt.contains(r),
                    "{:?} lists `{r}` as both required and optional",
                    f
                );
            }
            let all = f.all_params();
            let mut uniq = all.clone();
            uniq.sort_unstable();
            uniq.dedup();
            assert_eq!(uniq.len(), all.len(), "{f:?} has duplicate params");
        }
    }
}

