//! Document session: cells, kernels, journals, state store.
//!
//! Two journals (do not mix names):
//! - **execution journal** (`JournalRecord`): UI timing / status / errors.
//! - **oracle journal** (`OracleRecord`): Ω step recordings for `replay_check`
//!   (source hash, stdout, result, ok, code).
//!
//! Stateful cells live here, owned by the foreign runtime. The view reads a
//! `会话 = > foreign.session ...` handle; no table names are reserved, and a
//! document without that handle gets stateless one-shot cells.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::foreign::registry;
use crate::host::foreign_session;
use crate::host::HostContext;
use crate::value::Value;

mod control;
pub mod packing;
pub use control::DocumentControl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellStatus {
    Never,
    Running,
    Clean,
    Dirty,
    Stale,
    Error,
    Unlisted,
}

impl CellStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Running => "running",
            Self::Clean => "clean",
            Self::Dirty => "dirty",
            Self::Stale => "stale",
            Self::Error => "error",
            Self::Unlisted => "unlisted",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub name: String,
    pub lang: String,
    pub source: String,
    pub line_start: u32,
    pub line_end: u32,
    pub source_hash: String,
    pub kernel: String,
    /// Per-cell raw stdin from foreign.session inputs=.
    pub stdin: String,
    pub listed: bool,
    pub status: CellStatus,
    pub runs: u64,
    pub last_ms: u64,
    pub last_ok: bool,
    pub last_error: String,
    /// Captured stdout from the last evaluation (recording surface for replay_check).
    pub last_stdout: String,
    pub last_code: i64,
    /// Value returned by the last successful evaluation, used by explicit persist.
    pub last_result: Value,
}

#[derive(Debug, Clone)]
pub struct KernelDecl {
    pub name: String,
    pub lang: String,
    pub cmd: Option<String>,
    pub implicit: bool,
}

#[derive(Debug, Clone)]
pub struct JournalRecord {
    pub seq: u64,
    pub cell: String,
    pub kernel: String,
    pub lang: String,
    pub ok: bool,
    pub code: i64,
    pub ms: u64,
    pub timestamp_ms: i64,
    pub error: String,
}

/// One recorded Ω step: what a later `replay_check` compares against.
#[derive(Debug, Clone)]
pub struct OracleRecord {
    pub value: Value,
    pub stdin: String,
    pub seq: u64,
    pub cell: String,
    pub kernel: String,
    pub lang: String,
    pub source_hash: String,
    pub ok: bool,
    pub code: i64,
    pub stdout: String,
    pub result: Value,
}

impl OracleRecord {
    fn to_value(&self) -> Value {
                    Value::Map(vec![
                        ("value".into(), self.value.clone()),
                        ("seq".into(), Value::Int(self.seq as i64)),
                        ("stdin".into(), Value::Text(self.stdin.clone())),
                        ("cell".into(), Value::Text(self.cell.clone())),
                        ("kernel".into(), Value::Text(self.kernel.clone())),
                        ("lang".into(), Value::Text(self.lang.clone())),
                        ("source_hash".into(), Value::Text(self.source_hash.clone())),
                        ("ok".into(), Value::Bool(self.ok)),
                        ("code".into(), Value::Int(self.code)),
                        ("stdout".into(), Value::Text(self.stdout.clone())),
                        ("result".into(), self.result.clone()),
                    ])
    }
}

pub struct DocumentSession {
    pub control: DocumentControl,
    replay_controls: Vec<control::ControlEvent>,
    replay_boundary: Option<replay::ReplayBoundary>,
    source_hash: String,
    initial_values: Vec<(String, String, Value)>,
    initial_image: Value,
    action_history: Vec<replay::HistoryEntry>,
    action_depth: usize,
    replay_fresh: bool,
    pub path: PathBuf,
    root: PathBuf,
    pub persistent: bool,
    pub cells: Vec<Cell>,
    pub kernels: Vec<KernelDecl>,
    pub order: Vec<String>,
    pub journal: Vec<JournalRecord>,
    pub oracle_journal: Vec<OracleRecord>,
    pub value_store: HashMap<String, Value>,
    pub cursor: usize,
    pub generation: u64,
    pub opened_ms: i64,
    host: HostContext,
    kernel_sessions: HashMap<String, String>,
    seq: u64,
}

fn hash_source(source: &str) -> String {
    let mut h = Sha256::new();
    h.update(source.as_bytes());
    format!("{:x}", h.finalize())
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Convert a table value into row maps. Handles both row tables (`| @ | ... |`)
/// and ordinary column tables; no source text is scanned.
fn table_rows(v: &Value) -> Result<Vec<HashMap<String, Value>>, String> {
    match v {
        Value::None => Ok(Vec::new()),
        Value::List(rows) => {
            let mut out = Vec::with_capacity(rows.len());
            for (i, row) in rows.iter().enumerate() {
                let Value::Map(pairs) = row else {
                    return Err(format!("view table row {} is not a map", i + 1));
                };
                out.push(
                    pairs
                        .iter()
                        .filter(|(k, _)| k != "@")
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                );
            }
            Ok(out)
        }
        Value::Map(cols) => {
            // A single-row table may evaluate to a map of scalar fields rather
            // than a column map. Treat that map as one record.
            if !cols.iter().all(|(_, v)| matches!(v, Value::List(_))) {
                let row = cols
                    .iter()
                    .filter(|(k, _)| k != "@")
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                return Ok(vec![row]);
            }
            let mut width = 0usize;
            for (_, value) in cols {
                let Value::List(xs) = value else {
                    unreachable!();
                };
                width = width.max(xs.len());
            }
            let mut out = Vec::with_capacity(width);
            for i in 0..width {
                let mut row = HashMap::new();
                for (name, value) in cols {
                    if let Value::List(xs) = value {
                        if let Some(v) = xs.get(i) {
                            row.insert(name.clone(), v.clone());
                        }
                    }
                }
                out.push(row);
            }
            Ok(out)
        }
        _ => Err("view plan binding is not a table".into()),
    }
}

fn plan_tables(
    bindings: &HashMap<String, Value>,
) -> Result<
    (
        Option<Vec<HashMap<String, Value>>>,
        Option<Vec<HashMap<String, Value>>>,
        Option<Vec<HashMap<String, Value>>>,
        Option<Vec<HashMap<String, Value>>>,
    ),
    String,
> {
    let Some(plan) = bindings.get("会话") else {
        return Ok((None, None, None, None));
    };
    let Value::Map(fields) = plan else {
        // `会话` is also a common low-level session id binding (e.g. from
        // foreign.session_open). It is not a view plan; leave cells stateless.
        return Ok((None, None, None, None));
    };
    let field = |name: &str| {
        fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    };
    let kind = field("kind").map(|v| value_text(&v)).unwrap_or_default();
    if kind != "foreign-session" {
        return Err("`会话` must be a foreign.session handle".into());
    }
    let kernels = match field("kernels") {
        Some(Value::None) | None => None,
        Some(v) => Some(table_rows(&v)?),
    };
    let cells = match field("cells") {
        Some(Value::None) | None => None,
        Some(v) => Some(table_rows(&v)?),
    };
    let values = match field("values") {
        Some(Value::None) | None => None,
        Some(v) => Some(table_rows(&v)?),
    };
    let inputs = match field("inputs") {
        Some(Value::None) | None => None,
        Some(v) => Some(table_rows(&v)?),
    };
    Ok((kernels, cells, values, inputs))
}

fn col_value<'a>(row: &'a HashMap<String, Value>, names: &[&str]) -> Option<&'a Value> {
    for n in names {
        if let Some(v) = row.get(*n) {
            if !matches!(v, Value::None) {
                return Some(v);
            }
        }
    }
    None
}

fn value_text(v: &Value) -> String {
    match v {
        Value::Text(s) => s.clone(),
        Value::None => String::new(),
        other => other.as_display(),
    }
}

fn col_text(row: &HashMap<String, Value>, names: &[&str]) -> Option<String> {
    col_value(row, names).map(value_text).filter(|s| !s.trim().is_empty())
}

/// Evaluate the entry object's declarations with the real interpreter. Tables
/// are ordinary Marqdo values; the view does not re-parse source text.
fn capture_bindings(
    path: &Path,
    root: &Path,
    module: &crate::ast::Module,
    source: &str,
) -> Result<HashMap<String, Value>, String> {
    let mut host = HostContext::for_capture(Some(path), crate::host::HostCaps::default());
    host.cwd = path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
    host.fs_root = Some(root.to_path_buf());
    host.entry_path = Some(path.to_path_buf());
    host.entry_source = Some(source.to_string());
    host.entry_metadata = module.metadata.clone();
    let mut interp = crate::interp::Interpreter::with_capture(Some(path), false).with_host(host);
    interp
        .capture_declarations(module, &["会话"])
        .map_err(|e| format!("view plan evaluation: {e}"))
}

fn collect_cells(stmts: &[crate::ast::Stmt], out: &mut Vec<Cell>, seen: &mut HashSet<String>) -> Result<(), String> {
    for stmt in stmts {
        match stmt {
            crate::ast::Stmt::Assign {
                name,
                value: crate::ast::Expr::Code(c),
                span,
                end_line,
            } => {
                if !seen.insert(name.clone()) {
                    return Err(format!(
                        "cell `{name}` is assigned more than once (line {})",
                        span.line
                    ));
                }
                out.push(Cell {
                    name: name.clone(),
                    lang: c.lang.clone(),
                    source: c.source.clone(),
                    line_start: span.line,
                    line_end: *end_line,
                    source_hash: hash_source(&c.source),
                    kernel: String::new(),
                    stdin: String::new(),
                    listed: false,
                    status: CellStatus::Never,
                    runs: 0,
                    last_ms: 0,
                    last_ok: false,
                    last_error: String::new(),
                    last_stdout: String::new(),
                    last_code: 0,
                    last_result: Value::None,
                });
            }
            crate::ast::Stmt::Branch { arms, .. } => {
                for arm in arms {
                    collect_cells(&arm.body, out, seen)?;
                }
            }
            crate::ast::Stmt::While { body, .. } | crate::ast::Stmt::ForEach { body, .. } => {
                collect_cells(body, out, seen)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn fun_has_plan(fun: &crate::ast::Function) -> bool {
    fun.body.iter().any(|stmt| match stmt {
        crate::ast::Stmt::Assign { name, .. } => name == "会话",
        _ => false,
    }) || fun.children.iter().any(fun_has_plan)
}

fn has_nested_plan(module: &crate::ast::Module, entry_name: &str) -> bool {
    module
        .functions
        .iter()
        .find(|f| f.name == entry_name)
        .map(|entry| entry.children.iter().any(fun_has_plan))
        .unwrap_or(false)
}

mod replay;

impl DocumentSession {
    pub(crate) fn adopt_control(&mut self, control: DocumentControl) {
        self.control = control;
        for (name, id) in &self.kernel_sessions {
            if let Some(session) = self.host.foreign_sessions.get(id) {
                self.control.register(name, session.interrupt_handle.clone());
            }
        }
    }
    pub fn open(path: &Path, root: &Path) -> Result<Self, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        Self::open_source(path, root, &source, true)
    }

    fn open_source(path: &Path, root: &Path, source: &str, initialize: bool) -> Result<Self, String> {
        let bind_ctx = crate::binding::BindingContext::from_binds(
            &[],
            crate::binding::cwd_for_path(Some(path)),
        );
        let module = crate::load::load_module_with_source(path, source, &bind_ctx)
            .map_err(|e| format!("load {}: {e}", path.display()))?;
        let entry = module
            .functions
            .iter()
            .find(|f| f.name == "main")
            .or_else(|| {
                let objs: Vec<_> = module.functions.iter().filter(|f| f.is_object()).collect();
                if objs.len() == 1 {
                    Some(objs[0])
                } else {
                    None
                }
            })
            .ok_or_else(|| "notebook: no `main` entry (or sole object) to index".to_string())?;
        let mut cells = Vec::new();
        let mut seen = HashSet::new();
        collect_cells(&entry.body, &mut cells, &mut seen)?;
        let declarations = capture_bindings(path, root, &module, &source)?;
        if !declarations.contains_key("会话") && has_nested_plan(&module, &entry.name) {
            return Err(
                "foreign.session: plan must be a top-level binding (`会话` = > foreign.session ...)"
                    .into(),
            );
        }

        let (plan_kernels, plan_cells, plan_values, plan_inputs) = plan_tables(&declarations)?;
        let persistent = matches!(
            declarations.get("会话"),
            Some(Value::Map(fields))
                if fields
                    .iter()
                    .any(|(k, v)| k == "kind" && value_text(v) == "foreign-session")
        );
        let mut kernels: Vec<KernelDecl> = Vec::new();
        let mut kernel_map: HashMap<String, KernelDecl> = HashMap::new();
        if let Some(rows) = plan_kernels {
            if rows.is_empty() {
                return Err("foreign.session: at least one kernel row is required".into());
            }
            for row in rows {
                let name = col_text(&row, &["名称", "Name", "name"])
                    .ok_or_else(|| "foreign.session kernels: missing 名称/Name".to_string())?;
                let lang = col_text(&row, &["语言", "Language", "language", "lang"])
                    .ok_or_else(|| format!("foreign.session[{name}]: missing 语言/Language"))?;
                let lang = registry::canonical(&lang);
                let cmd = col_text(&row, &["命令", "Command", "command", "cmd"]);
                if kernel_map.contains_key(&name) {
                    return Err(format!("foreign.session: kernel `{name}` is declared twice"));
                }
                let decl = KernelDecl {
                    name: name.clone(),
                    lang,
                    cmd,
                    implicit: false,
                };
                kernel_map.insert(name.clone(), decl.clone());
                kernels.push(decl);
            }
        }

        let mut order: Vec<String> = Vec::new();
        let session_rows = plan_cells;
        if let Some(rows) = session_rows {
            if rows.is_empty() {
                return Err("foreign.session cells: at least one cell row is required".into());
            }
            let by_name: HashMap<String, usize> = cells
                .iter()
                .enumerate()
                .map(|(i, c)| (c.name.clone(), i))
                .collect();
            let mut listed = HashSet::new();
            for row in rows {
                let cell_name = col_text(&row, &["单元", "Cell", "cell", "单元名"])
                    .ok_or_else(|| "foreign.session cells: missing 单元/Cell".to_string())?;
                let kernel_name = col_text(&row, &["内核", "Kernel", "kernel"])
                    .ok_or_else(|| format!("foreign.session[{cell_name}]: missing 内核/Kernel"))?;
                let Some(&ci) = by_name.get(&cell_name) else {
                    return Err(format!("foreign.session: unknown cell `{cell_name}`"));
                };
                if !listed.insert(cell_name.clone()) {
                    return Err(format!("foreign.session: cell `{cell_name}` is listed twice"));
                }
                let kernel = resolve_kernel(&mut kernels, &mut kernel_map, &kernel_name)?;
                if registry::canonical(&cells[ci].lang) != registry::canonical(&kernel.lang) {
                    return Err(format!(
                        "foreign.session: cell `{cell_name}` ({}) cannot use kernel `{}` ({})",
                        cells[ci].lang, kernel.name, kernel.lang
                    ));
                }
                cells[ci].kernel = kernel.name.clone();
                cells[ci].listed = true;
                order.push(cell_name);
            }
            for cell in &mut cells {
                if !cell.listed {
                    cell.status = CellStatus::Unlisted;
                }
            }
        } else if persistent {
            for cell in &mut cells {
                let kernel = ensure_default_kernel(
                    &mut kernels,
                    &mut kernel_map,
                    &cell.lang,
                );
                cell.kernel = kernel;
                cell.listed = true;
                order.push(cell.name.clone());
            }
        } else {
            for cell in &mut cells {
                cell.kernel = "one-shot".into();
                cell.listed = true;
                order.push(cell.name.clone());
            }
        }

        let mut host = HostContext::default();
        let doc_dir = path.parent().unwrap_or_else(|| Path::new("."));
        host.cwd = doc_dir.to_path_buf();
        host.fs_root = Some(root.to_path_buf());
        host.entry_path = Some(path.to_path_buf());
        host.entry_metadata = module.metadata.clone();
        let source_hash = hash_source(&source);
        host.entry_source = Some(source.to_string());

        let mut session = Self {
            control: DocumentControl::default(),
            replay_controls: Vec::new(),
            replay_boundary: None,
            source_hash,
            initial_values: Vec::new(),
            initial_image: Value::None,
            action_history: Vec::new(),
            action_depth: 0,
            replay_fresh: false,
            path: path.to_path_buf(),
            root: root.to_path_buf(),
            persistent,
            cells,
            kernels,
            order,
            journal: Vec::new(),
            oracle_journal: Vec::new(),
            value_store: HashMap::new(),
            cursor: 0,
            generation: 1,
            opened_ms: now_ms(),
            host,
            kernel_sessions: HashMap::new(),
            seq: 0,
        };
        if let Some(rows) = plan_values {
            for row in rows {
                let kernel = col_text(&row, &["内核", "Kernel", "kernel"])
                    .ok_or_else(|| "foreign.session values: missing 内核/Kernel".to_string())?;
                let name = col_text(&row, &["名称", "Name", "name"])
                    .ok_or_else(|| "foreign.session values: missing 名称/Name".to_string())?;
                let value = col_value(&row, &["值", "Value", "value", "数据"])
                    .ok_or_else(|| format!("foreign.session values[{name}]: missing 值/Value"))?
                    .clone();
                if session.kernel(&kernel).is_none() { return Err(format!("unknown kernel `{kernel}`")); }
                if !foreign_session::valid_name(&name) { return Err("initial value: name must be an identifier".into()); }
                super::oracle::admit(&value)?;
                session.initial_values.push((kernel, name, value));
            }
        }
        if let Some(rows) = plan_inputs {
            for row in rows {
                let cell = col_text(&row, &["单元", "Cell", "cell", "单元名"])
                    .ok_or_else(|| "foreign.session inputs: missing 单元/Cell".to_string())?;
                let raw = col_value(&row, &["stdin", "标准输入", "输入"])
                    .map(value_text)
                    .unwrap_or_default();
                if let Some(i) = session.cell_index(&cell) {
                    session.cells[i].stdin = raw;
                }
            }
        }
        session.initial_image = session.plan_image();
        if initialize { session.initialize_values()?; }
        session.replay_fresh = true;
        Ok(session)
    }

    /// Re-read the document, preserving kernels/state/journal. Changed cells become dirty.
    fn reload_source(&mut self, source: &str, root: &Path) -> Result<(), String> {
        let fresh = Self::open_source(&self.path, root, source, false)?;
        self.install_reload(fresh)
    }

    fn install_reload(&mut self, mut fresh: Self) -> Result<(), String> {
        let new_source = fresh.host.entry_source.clone();
        let new_metadata = fresh.host.entry_metadata.clone();
        let old_kernels = self.kernels.clone();
        let old: HashMap<String, Cell> = self
            .cells
            .iter()
            .cloned()
            .map(|c| (c.name.clone(), c))
            .collect();
        for c in &mut fresh.cells {
            if let Some(o) = old.get(&c.name) {
                c.runs = o.runs;
                c.last_ms = o.last_ms;
                c.last_ok = o.last_ok;
                c.last_error = o.last_error.clone();
                c.last_stdout = o.last_stdout.clone();
                c.last_code = o.last_code;
                c.last_result = o.last_result.clone();
                c.status = if o.source_hash != c.source_hash || o.kernel != c.kernel {
                    CellStatus::Dirty
                } else if o.status == CellStatus::Running {
                    CellStatus::Dirty
                } else {
                    o.status
                };
            }
        }
        fresh.host = std::mem::replace(&mut self.host, HostContext::default());
        fresh.host.entry_source = new_source;
        fresh.host.entry_metadata = new_metadata;
        fresh.initial_image = self.initial_image.clone();
        fresh.action_history = std::mem::take(&mut self.action_history);
        fresh.control = self.control.clone();
        fresh.replay_controls = std::mem::take(&mut self.replay_controls);
        fresh.replay_boundary = self.replay_boundary.take();
        fresh.action_depth = self.action_depth;
        fresh.generation = self.generation + 1;
        fresh.kernel_sessions = std::mem::take(&mut self.kernel_sessions);
        fresh.value_store = std::mem::take(&mut self.value_store);
        fresh.journal = std::mem::take(&mut self.journal);
        fresh.oracle_journal = std::mem::take(&mut self.oracle_journal);
        fresh.seq = self.seq;
        fresh.replay_fresh = false;
        fresh.cursor = self.cursor.min(fresh.order.len());
        fresh.opened_ms = self.opened_ms;
        let known: HashSet<String> = fresh.kernels.iter().filter(|k|
            old_kernels.iter().any(|old| old.name == k.name && old.lang == k.lang && old.cmd == k.cmd)
        ).map(|k| k.name.clone()).collect();
        let orphans: Vec<String> = fresh
            .kernel_sessions
            .keys()
            .filter(|k| !known.contains(*k))
            .cloned()
            .collect();
        for k in orphans {
            if let Some(id) = fresh.kernel_sessions.remove(&k) {
                let _ = foreign_session::close(&mut fresh.host, &Value::Handle(id));
            }
        }
        *self = fresh;
        Ok(())
    }

    pub fn title(&self) -> String {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("session")
            .trim_end_matches(".mq")
            .to_string()
    }

    fn cell_index(&self, name: &str) -> Option<usize> {
        self.cells.iter().position(|c| c.name == name)
    }

    fn kernel(&self, name: &str) -> Option<&KernelDecl> {
        self.kernels.iter().find(|k| k.name == name)
    }

    fn ensure_kernel_session(&mut self, kernel_name: &str) -> Result<String, String> {
        self.replay_fresh = false;
        if let Some(id) = self.kernel_sessions.get(kernel_name) {
            if self.host.foreign_sessions.contains_key(id) {
                return Ok(id.clone());
            }
        }
        let decl = self
            .kernel(kernel_name)
            .ok_or_else(|| format!("unknown kernel `{kernel_name}`"))?
            .clone();
        let cmd_value = decl.cmd.map(Value::Text);
        let lang = Value::Text(decl.lang.clone());
        let id = foreign_session::open(
            &mut self.host,
            &lang,
            cmd_value.as_ref(),
            None,
            None,
            None,
        )?;
        let id = match id {
            Value::Handle(s) => s,
            _ => return Err("session_open returned non-handle id".into()),
        };
        for (kernel, name, value) in self.initial_values.clone() {
            if kernel == kernel_name {
                if let Err(error) = foreign_session::set(&mut self.host, &Value::Handle(id.clone()), &Value::Text(name), &value) {
                    let _ = foreign_session::close(&mut self.host, &Value::Handle(id.clone()));
                    return Err(error);
                }
            }
        }
        self.kernel_sessions.insert(kernel_name.to_string(), id.clone());
        self.control.register(kernel_name, self.host.foreign_sessions[&id].interrupt_handle.clone());
        Ok(id)
    }

    fn record_result(&mut self, cell: &str, kernel: &str, record: &Value, stdin: Option<&str>) -> Result<(), String> {
        let field = |k: &str| match record {
            Value::Map(entries) => entries
                .iter()
                .find(|(kk, _)| kk == k)
                .map(|(_, v)| v.clone()),
            _ => None,
        };
        let ok = matches!(field("ok"), Some(Value::Bool(true)));
        let ms = match field("ms") {
            Some(Value::Int(n)) => n as u64,
            _ => 0,
        };
        let error = field("error")
            .and_then(|v| v.reveal_text().map(|s| s.to_string()))
            .unwrap_or_default();
        let code = match field("code") {
            Some(Value::Int(n)) => n,
            _ => 0,
        };
        let stdout = field("stdout")
            .and_then(|v| v.reveal_text().map(|s| s.to_string()))
            .unwrap_or_default();
        let (lang, source_hash) = self
            .cell_index(cell)
            .map(|i| {
                (
                    self.cells[i].lang.clone(),
                    self.cells[i].source_hash.clone(),
                )
            })
            .unwrap_or_default();
        // An explicit null is a semantic result, even when stdout is nonempty.
        // Keep display output separate; only legacy records lacking the field
        // can use their stdout as a fallback.
        let last_result = field("result").unwrap_or_else(|| {
                field("stdout")
                    .and_then(|v| v.reveal_text().map(|s| Value::Text(s.to_string())))
                    .unwrap_or(Value::None)
            });
        if let Some(i) = self.cell_index(cell) {
            let c = &mut self.cells[i];
            c.runs += 1;
            c.last_ms = ms;
            c.last_ok = ok;
            c.last_error = error.clone();
            c.last_stdout = stdout.clone();
            c.last_code = code;
            c.last_result = last_result.clone();
            c.status = if ok { CellStatus::Clean } else { CellStatus::Error };
        }
        self.seq += 1;
        self.journal.push(JournalRecord {
            seq: self.seq,
            cell: cell.to_string(),
            kernel: kernel.to_string(),
            lang: lang.clone(),
            ok,
            code,
            ms,
            timestamp_ms: now_ms(),
            error,
        });
        let value = super::oracle::eval_observation(record)?;
        super::oracle::append(&mut self.host, "eval", &value)?;
        self.oracle_journal.push(OracleRecord {
            value,
            stdin: stdin.unwrap_or_default().to_string(),
            seq: self.seq,
            cell: cell.to_string(),
            kernel: kernel.to_string(),
            lang,
            source_hash,
            ok,
            code,
            stdout,
            result: last_result,
        });
        self.check_replay_cell_boundary();
        Ok(())
    }

    pub fn run_cell(&mut self, cell_name: &str, kernel_override: Option<&str>) -> Result<Value, String> {
        self.run_cell_with_stdin(cell_name, kernel_override, None)
    }

    fn run_cell_with_stdin_inner(
        &mut self,
        cell_name: &str,
        kernel_override: Option<&str>,
        stdin: Option<&str>,
    ) -> Result<Value, String> {
        self.replay_fresh = false;
        let ci = self
            .cell_index(cell_name)
            .ok_or_else(|| format!("unknown cell `{cell_name}`"))?;
        if !self.cells[ci].listed && kernel_override.is_none() {
            return Err(format!("cell `{cell_name}` is not listed in the foreign session"));
        }
        let stdin_owned = stdin.map(str::to_string).or_else(|| {
            let s = self.cells[ci].stdin.clone();
            if s.is_empty() { None } else { Some(s) }
        });
        let stdin = stdin_owned.as_deref();
        if !self.persistent {
            self.cells[ci].kernel = "one-shot".into();
            self.cells[ci].status = CellStatus::Running;
            let lang = self.cells[ci].lang.clone();
            let source = self.cells[ci].source.clone();
            let result = crate::host::foreign::invoke_for_view(
                &self.root,
                &lang,
                &source,
                None,
                stdin,
                None,
            );
            return match result {
                Ok(record) => {
                    self.record_result(cell_name, "one-shot", &record, stdin)?;
                    Ok(record)
                }
                Err(e) => {
                    if let Some(i) = self.cell_index(cell_name) {
                        self.cells[i].status = CellStatus::Error;
                        self.cells[i].last_error = e.clone();
                    }
                    Err(e)
                }
            };
        }
        let kernel = match kernel_override {
            Some(k) => {
                if self.kernel(k).is_none() {
                    return Err(format!("unknown kernel `{k}`"));
                }
                let ci_lang = registry::canonical(&self.cells[ci].lang);
                let klang = registry::canonical(
                    &self.kernel(k).map(|d| d.lang.clone()).unwrap_or_default(),
                );
                if ci_lang != klang {
                    return Err(format!(
                        "cell `{cell_name}` ({}) cannot run in kernel `{k}` ({klang})",
                        self.cells[ci].lang
                    ));
                }
                self.cells[ci].kernel = k.to_string();
                self.cells[ci].listed = true;
                k.to_string()
            }
            None => self.cells[ci].kernel.clone(),
        };
        self.cells[ci].status = CellStatus::Running;
        let result = (|| -> Result<Value, String> {
            let id = self.ensure_kernel_session(&kernel)?;
            if self.control.cancelled() { return Err("document action interrupted before evaluation".into()); }
            let source = Value::Text(self.cells[ci].source.clone());
            let record = foreign_session::eval_with_stdin(
                &mut self.host,
                &Value::Handle(id),
                None,
                Some(&source),
                None,
                stdin,
                None,
            )?;
            self.record_result(cell_name, &kernel, &record, stdin)?;
            Ok(record)
        })();
        if result.is_err() {
            self.cells[ci].status = CellStatus::Error;
        }
        result
    }

    pub fn step(&mut self) -> Result<Value, String> {
        self.step_with_stdin(None)
    }

    fn step_with_stdin_inner(&mut self, stdin: Option<&str>) -> Result<Value, String> {
        if self.cursor >= self.order.len() {
            return Ok(Value::Map(vec![
                ("done".into(), Value::Bool(true)),
                ("cursor".into(), Value::Int(self.cursor as i64)),
            ("generation".into(), Value::Int(self.generation as i64)),
            ]));
        }
        let name = self.order[self.cursor].clone();
        let record = self.run_cell_with_stdin(&name, None, stdin)?;
        let ci = self.cell_index(&name).unwrap_or(0);
        let kernel = self.cells[ci].kernel.clone();
        self.cursor += 1;
        let mut fields = match record {
            Value::Map(m) => m,
            other => vec![("result".into(), other)],
        };
        fields.push(("cell".into(), Value::Text(name)));
        fields.push(("kernel".into(), Value::Text(kernel)));
        fields.push(("cursor".into(), Value::Int(self.cursor as i64)));
        Ok(Value::Map(fields))
    }

    pub fn run_all(&mut self, restart: bool) -> Result<Value, String> {
        self.run_all_with_stdin(restart, None)
    }

    pub fn run_all_with_stdin(
        &mut self,
        restart: bool,
        stdin: Option<&str>,
    ) -> Result<Value, String> {
        self.run_all_with_inputs(restart, stdin, None)
    }

    fn run_all_with_inputs_inner(
        &mut self,
        restart: bool,
        stdin: Option<&str>,
        inputs: Option<&HashMap<String, String>>,
    ) -> Result<Value, String> {
        if restart {
            self.restart_all()?;
        }
        self.cursor = 0;
        let mut records = Vec::new();
        let mut failed = 0i64;
        for name in self.order.clone() {
            if self.control.cancelled() { break; }
            if !self.replay_batch_can_continue() { break; }
            let seq_before = self.seq;
            let cell_stdin = inputs
                .and_then(|m| m.get(&name))
                .map(String::as_str)
                .or(stdin);
            let record = match self.run_cell_with_stdin(&name, None, cell_stdin) {
                Ok(record) => record,
                Err(error) => {
                    if let Some(i) = self.cell_index(&name) {
                        self.cells[i].status = CellStatus::Error;
                        self.cells[i].last_error = error.clone();
                    }
                    Value::Map(vec![
                        ("ok".into(), Value::Bool(false)),
                        ("stdout".into(), Value::Text(String::new())),
                        ("stderr".into(), Value::Text(String::new())),
                        ("error".into(), Value::Text(error)),
                        ("code".into(), Value::Int(-1)),
                        ("ms".into(), Value::Int(0)),
                    ])
                }
            };
            let ok = match &record {
                Value::Map(m) => matches!(
                    m.iter().find(|(k, _)| k == "ok").map(|(_, v)| v),
                    Some(Value::Bool(true))
                ),
                _ => false,
            };
            if !ok {
                failed += 1;
            }
            let mut m = match record {
                Value::Map(m) => m,
                other => vec![("result".into(), other)],
            };
            m.push(("cell".into(), Value::Text(name)));
            let record = Value::Map(m);
            self.check_replay_batch_record(&record, seq_before);
            records.push(record);
            self.cursor += 1;
        }
        let mut summary = vec![
            ("ok".into(), Value::Bool(failed == 0 && !self.control.cancelled())),
            ("ran".into(), Value::Int(records.len() as i64)),
            ("failed".into(), Value::Int(failed)),
            ("records".into(), Value::List(records)),
        ];
        if self.control.cancelled() {
            summary.push(("cancelled".into(), Value::Bool(true)));
            summary.push(("remaining".into(), Value::Int((self.order.len() - self.cursor) as i64)));
        }
        Ok(Value::Map(summary))
    }

    fn interrupt_inner(&mut self) -> Result<Value, String> {
        let mut ids: Vec<String> = self.kernel_sessions.keys().cloned().collect();
        ids.sort();
        let mut out = Vec::new();
        for name in ids {
            if let Some(id) = self.kernel_sessions.get(&name) {
                match foreign_session::interrupt(&mut self.host, &Value::Handle(id.clone())) {
                    Ok(_) => out.push(Value::Text(name)),
                    Err(e) => out.push(Value::Text(format!("{name}: {e}"))),
                }
            }
        }
        Ok(Value::List(out))
    }

    fn restart_kernel_inner(&mut self, name: &str) -> Result<Value, String> {
        if let Some(id) = self.kernel_sessions.remove(name) {
            let _ = foreign_session::close(&mut self.host, &Value::Handle(id));
        }
        for c in &mut self.cells {
            if c.kernel == name {
                c.status = CellStatus::Never;
                c.last_error.clear();
            }
        }
        Ok(Value::Bool(true))
    }

    fn restart_all_inner(&mut self) -> Result<Value, String> {
        let mut names: Vec<String> = self.kernel_sessions.keys().cloned().collect();
        names.sort();
        for name in names {
            self.restart_kernel(&name)?;
        }
        self.cursor = 0;
        for c in &mut self.cells {
            c.status = if c.listed { CellStatus::Never } else { CellStatus::Unlisted };
        }
        Ok(Value::Bool(true))
    }

    fn assign_kernel_inner(&mut self, cell: &str, kernel: &str) -> Result<Value, String> {
        let ci = self
            .cell_index(cell)
            .ok_or_else(|| format!("unknown cell `{cell}`"))?;
        let decl = self
            .kernel(kernel)
            .ok_or_else(|| format!("unknown kernel `{kernel}`"))?
            .clone();
        if registry::canonical(&self.cells[ci].lang) != registry::canonical(&decl.lang) {
            return Err(format!(
                "cell `{cell}` ({}) cannot use kernel `{kernel}` ({})",
                self.cells[ci].lang, decl.lang
            ));
        }
        self.cells[ci].kernel = kernel.to_string();
        self.cells[ci].listed = true;
        self.cells[ci].status = CellStatus::Dirty;
        self.generation += 1;
        Ok(Value::None)
    }

    fn add_cell_inner(
        &mut self,
        cell: &str,
        lang: &str,
        source: &str,
        kernel: &str,
    ) -> Result<Value, String> {
        if self.cell_index(cell).is_some() {
            return Err(format!("cell `{cell}` already exists"));
        }
        let lang = registry::canonical(lang);
        let decl = self
            .kernel(kernel)
            .ok_or_else(|| format!("unknown kernel `{kernel}`"))?
            .clone();
        if registry::canonical(&decl.lang) != lang {
            return Err(format!(
                "cell `{cell}` ({lang}) cannot use kernel `{kernel}` ({})",
                decl.lang
            ));
        }
        self.cells.push(Cell {
            name: cell.to_string(),
            lang,
            source: source.to_string(),
            line_start: 0,
            line_end: 0,
            source_hash: hash_source(source),
            kernel: kernel.to_string(),
            stdin: String::new(),
            listed: true,
            status: CellStatus::Never,
            runs: 0,
            last_ms: 0,
            last_ok: false,
            last_error: String::new(),
            last_stdout: String::new(),
            last_code: 0,
            last_result: Value::None,
        });
        self.order.push(cell.to_string());
        self.generation += 1;
        Ok(Value::Bool(true))
    }

    fn remove_cell_inner(&mut self, cell: &str) -> Result<Value, String> {
        let Some(i) = self.cell_index(cell) else {
            return Err(format!("unknown cell `{cell}`"));
        };
        self.cells.remove(i);
        self.order.retain(|name| name != cell);
        self.journal.retain(|j| j.cell != cell);
        self.generation += 1;
        Ok(Value::Bool(true))
    }

    fn set_value_inner(&mut self, kernel: &str, name: &str, value: Value) -> Result<Value, String> {
        let out = self.push_value(kernel, name, value)?;
        self.generation += 1;
        Ok(out)
    }

    fn state_list_inner(&mut self) -> Result<Value, String> {
        let marqdo = self
            .value_store
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
        let mut kernels = Vec::new();
        for k in &self.kernels {
            let live = self.kernel_sessions.get(&k.name).cloned();
            let mut vars = Vec::new();
            if let Some(id) = live {
                if self.host.foreign_sessions.contains_key(&id) {
                    if let Ok(names) = foreign_session::names(&mut self.host, &Value::Handle(id)) {
                        if let Value::List(xs) = names {
                            vars = xs;
                        }
                    }
                }
            }
            kernels.push(Value::Map(vec![
                ("name".into(), Value::Text(k.name.clone())),
                ("lang".into(), Value::Text(k.lang.clone())),
                ("vars".into(), Value::List(vars)),
            ]));
        }
        Ok(Value::Map(vec![
            ("marqdo".into(), Value::Map(marqdo)),
            ("kernels".into(), Value::List(kernels)),
        ]))
    }

    fn push_value_inner(&mut self, kernel: &str, name: &str, value: Value) -> Result<Value, String> {
        if self.kernel(kernel).is_none() { return Err(format!("unknown kernel `{kernel}`")); }
        if !foreign_session::valid_name(name) { return Err("session_set: name must be an identifier".into()); }
        super::oracle::admit(&value)?;
        let id = self.ensure_kernel_session(kernel)?;
        foreign_session::set(
            &mut self.host,
            &Value::Handle(id),
            &Value::Text(name.to_string()),
            &value,
        )?;
        self.value_store.insert(name.to_string(), value);
        Ok(Value::None)
    }

    fn pull_value_inner(&mut self, kernel: &str, name: &str) -> Result<Value, String> {
        if self.kernel(kernel).is_none() { return Err(format!("unknown kernel `{kernel}`")); }
        if !foreign_session::valid_name(name) { return Err("session_get: name must be an identifier".into()); }
        let id = self.ensure_kernel_session(kernel)?;
        let value = foreign_session::get(
            &mut self.host,
            &Value::Handle(id),
            &Value::Text(name.to_string()),
        )?;
        super::oracle::append(&mut self.host, "get", &value)?;
        self.value_store
            .insert(name.to_string(), value.clone());
        Ok(value)
    }

    pub fn summary(&self) -> Value {
        let cells = self
            .cells
            .iter()
            .map(|c| {
                Value::Map(vec![
                    ("name".into(), Value::Text(c.name.clone())),
                    ("lang".into(), Value::Text(c.lang.clone())),
                    ("kernel".into(), Value::Text(c.kernel.clone())),
                    ("listed".into(), Value::Bool(c.listed)),
                    ("status".into(), Value::Text(c.status.as_str().into())),
                    ("line_start".into(), Value::Int(c.line_start as i64)),
                    ("line_end".into(), Value::Int(c.line_end as i64)),
                    ("runs".into(), Value::Int(c.runs as i64)),
                    ("last_ms".into(), Value::Int(c.last_ms as i64)),
                    ("last_ok".into(), Value::Bool(c.last_ok)),
                    ("last_error".into(), Value::Text(c.last_error.clone())),
                    ("stdin".into(), Value::Text(c.stdin.clone())),
                ])
            })
            .collect();
        let kernels = self
            .kernels
            .iter()
            .map(|k| {
                Value::Map(vec![
                    ("name".into(), Value::Text(k.name.clone())),
                    ("lang".into(), Value::Text(k.lang.clone())),
                    ("implicit".into(), Value::Bool(k.implicit)),
                    (
                        "live".into(),
                        Value::Bool(self.kernel_sessions.contains_key(&k.name)),
                    ),
                ])
            })
            .collect();
        Value::Map(vec![
            ("title".into(), Value::Text(self.title())),
            ("path".into(), Value::Text(self.path.display().to_string())),
            ("persistent".into(), Value::Bool(self.persistent)),
            ("cells".into(), Value::List(cells)),
            ("kernels".into(), Value::List(kernels)),
            ("order".into(), Value::List(
                self.order.iter().cloned().map(Value::Text).collect(),
            )),
            ("cursor".into(), Value::Int(self.cursor as i64)),
            ("generation".into(), Value::Int(self.generation as i64)),
            ("journal".into(), Value::List(
                self.journal
                    .iter()
                    .map(|j| {
                        Value::Map(vec![
                            ("seq".into(), Value::Int(j.seq as i64)),
                            ("cell".into(), Value::Text(j.cell.clone())),
                            ("kernel".into(), Value::Text(j.kernel.clone())),
                            ("lang".into(), Value::Text(j.lang.clone())),
                            ("ok".into(), Value::Bool(j.ok)),
                            ("code".into(), Value::Int(j.code)),
                            ("ms".into(), Value::Int(j.ms as i64)),
                            ("timestamp_ms".into(), Value::Int(j.timestamp_ms)),
                            ("error".into(), Value::Text(j.error.clone())),
                        ])
                    })
                    .collect(),
            )),
            ("opened_ms".into(), Value::Int(self.opened_ms)),
        ])
    }

    /// Clear one cell's displayed output/record; kernels and live state survive.
    fn clear_cell_inner(&mut self, cell: &str) -> Result<Value, String> {
        let i = self
            .cell_index(cell)
            .ok_or_else(|| format!("unknown cell `{cell}`"))?;
        self.cells[i].runs = 0;
        self.cells[i].last_ms = 0;
        self.cells[i].last_ok = false;
        self.cells[i].last_error.clear();
        self.cells[i].last_stdout.clear();
        self.cells[i].last_code = 0;
        self.cells[i].last_result = Value::None;
        if self.cells[i].listed {
            self.cells[i].status = CellStatus::Never;
        }
        self.journal.retain(|j| j.cell != cell);
        self.oracle_journal.retain(|j| j.cell != cell);
        Ok(self.summary())
    }

    /// Clear displayed outputs and per-cell records; kernels and live state survive.
    /// Export plan, explicit values, kernel values and journal as JSON-safe data.
    pub fn export_snapshot(&mut self) -> Result<Value, String> {
        let kernels = self.tracked("inspect",Value::Map(Vec::new()), |session| session.inspect_kernels())?;
        let cells = self
            .cells
            .iter()
            .map(|c| {
                Value::Map(vec![
                    ("name".into(), Value::Text(c.name.clone())),
                    ("lang".into(), Value::Text(c.lang.clone())),
                    ("kernel".into(), Value::Text(c.kernel.clone())),
                    ("source".into(), Value::Text(c.source.clone())),
                    ("source_hash".into(), Value::Text(c.source_hash.clone())),
                    ("stdin".into(), Value::Text(c.stdin.clone())),
                    ("status".into(), Value::Text(c.status.as_str().into())),
                    ("runs".into(), Value::Int(c.runs as i64)),
                    ("last_ok".into(), Value::Bool(c.last_ok)),
                    // Recording fields: replay_check compares a later run against these.
                    ("ok".into(), Value::Bool(c.last_ok)),
                    ("code".into(), Value::Int(c.last_code)),
                    ("stdout".into(), Value::Text(c.last_stdout.clone())),
                    ("result".into(), c.last_result.clone()),
                ])
            })
            .collect();
        let journal = self.journal_json();
        Ok(Value::Map(vec![
            ("document_hash".into(), Value::Text(self.source_hash.clone())),
            ("version".into(), Value::Int(5)),
            ("initial_image".into(), self.initial_image.clone()),
            ("action_count".into(), Value::Int(self.action_history.len() as i64)),
            ("actions".into(), self.action_history_json()),
            ("observation_journal".into(), Value::List(self.host.oracle_observations.clone())),
            ("oracle_count".into(), Value::Int(self.complete_oracle_rows().len() as i64)),
            ("path".into(), Value::Text(self.path.display().to_string())),
            ("persistent".into(), Value::Bool(self.persistent)),
            ("generation".into(), Value::Int(self.generation as i64)),
            ("kernels".into(), kernels),
            ("cells".into(), Value::List(cells)),
            ("order".into(), Value::List(
                self.order.iter().cloned().map(Value::Text).collect(),
            )),
            ("journal".into(), journal),
            ("oracle_journal".into(), Value::List(self.complete_oracle_rows())),
        ]))
    }

    /// Import JSON-safe values from an exported snapshot. Source identity and
    /// cell names must already match; no arbitrary objects are deserialized.
    fn import_snapshot_inner(&mut self, snapshot: &Value) -> Result<Value, String> {
        replay::validate_control(snapshot)?;
        let Value::Map(fields) = snapshot else {
            return Err("snapshot must be a map".into());
        };
        let field = |name: &str| {
            fields
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
        };
        let Some(Value::List(kernels)) = field("kernels") else {
            return Err("snapshot: missing kernels".into());
        };
        if field("document_hash") != Some(Value::Text(self.source_hash.clone())) {
            return Err("snapshot: document hash differs".into());
        }
        // Validate all crossing payloads before mutating any live foreign store.
        for kernel in &kernels {
            let Value::Map(kfields) = kernel else { return Err("snapshot: malformed kernel".into()) };
            let Some((_, Value::Text(name))) = kfields.iter().find(|(k, _)| k == "name")
                else { return Err("snapshot: kernel name missing".into()) };
            if self.kernel(name).is_none() { return Err(format!("snapshot: unknown kernel `{name}`")); }
            let Some((_, Value::List(values))) = kfields.iter().find(|(k, _)| k == "values")
                else { return Err("snapshot: values missing".into()) };
            for item in values {
                let Value::Map(vfields) = item else { return Err("snapshot: malformed value".into()) };
                let Some((_, Value::Text(name))) = vfields.iter().find(|(k, _)| k == "name")
                    else { return Err("snapshot: value name missing".into()) };
                if !foreign_session::valid_name(name) { return Err("snapshot: invalid value name".into()); }
                let Some((_, value)) = vfields.iter().find(|(k, _)| k == "value")
                    else { return Err("snapshot: value missing".into()) };
                super::oracle::admit(value)?;
            }
        }
        let mut restored = 0i64;
        for kernel in kernels {
            let Value::Map(kfields) = kernel else { continue };
            let get = |name: &str| {
                kfields
                    .iter()
                    .find(|(k, _)| k == name)
                    .map(|(_, v)| v.clone())
            };
            let Some(Value::Text(name)) = get("name") else { continue };
            if self.kernel(&name).is_none() {
                return Err(format!("snapshot: unknown kernel `{name}`"));
            }
            let Some(Value::List(values)) = get("values") else { continue };
            let id = self.ensure_kernel_session(&name)?;
            for item in values {
                let Value::Map(vfields) = item else { continue };
                let getv = |name: &str| {
                    vfields
                        .iter()
                        .find(|(k, _)| k == name)
                        .map(|(_, v)| v.clone())
                };
                let Some(Value::Text(vname)) = getv("name") else { continue };
                let Some(value) = getv("value") else { continue };
                foreign_session::set(
                    &mut self.host,
                    &Value::Handle(id.clone()),
                    &Value::Text(vname),
                    &value,
                )?;
                restored += 1;
            }
        }
        if let Some(Value::List(cells)) = field("cells") {
            for item in cells {
                let Value::Map(cfields) = item else { continue };
                let getc = |name: &str| {
                    cfields
                        .iter()
                        .find(|(k, _)| k == name)
                        .map(|(_, v)| v.clone())
                };
                let Some(Value::Text(name)) = getc("name") else { continue };
                let Some(i) = self.cell_index(&name) else { continue };
                if let Some(Value::Text(status)) = getc("status") {
                    self.cells[i].status = match status.as_str() {
                        "clean" => CellStatus::Clean,
                        "error" => CellStatus::Error,
                        "dirty" => CellStatus::Dirty,
                        "stale" => CellStatus::Stale,
                        "unlisted" => CellStatus::Unlisted,
                        _ => CellStatus::Never,
                    };
                }
            }
        }
        self.replay_fresh = false;
        self.generation += 1;
        Ok(Value::Int(restored))
    }

    /// Replay recorded cells in order, using the snapshot's per-cell stdin.
    /// Re-runs only; does not compare against the recording. Prefer
    /// [`Self::replay_check`] when equivalence against a recording matters.
    fn replay_snapshot_inner(&mut self, snapshot: &Value) -> Result<Value, String> {
        let Value::Map(fields) = snapshot else {
            return Err("snapshot must be a map".into());
        };
        let cells = fields
            .iter()
            .find(|(k, _)| k == "cells")
            .and_then(|(_, v)| match v {
                Value::List(xs) => Some(xs.clone()),
                _ => None,
            })
            .ok_or_else(|| "snapshot: missing cells".to_string())?;
        let mut records = Vec::new();
        for item in cells {
            let Value::Map(cfields) = item else { continue };
            let getc = |name: &str| {
                cfields
                    .iter()
                    .find(|(k, _)| k == name)
                    .map(|(_, v)| v.clone())
            };
            let Some(Value::Text(name)) = getc("name") else { continue };
            if self.cell_index(&name).is_none() {
                continue;
            }
            if !self.replay_batch_can_continue() { break; }
            let stdin = getc("stdin").and_then(|v| match v {
                Value::Text(s) if !s.is_empty() => Some(s),
                _ => None,
            });
            let record = self.run_cell_with_stdin(&name, None, stdin.as_deref())?;
            self.check_replay_batch_record(&record, self.seq.saturating_sub(1));
            let mut m = match record {
                Value::Map(m) => m,
                other => vec![("result".into(), other)],
            };
            m.push(("cell".into(), Value::Text(name)));
            records.push(Value::Map(m));
        }
        Ok(Value::Map(vec![
            ("ok".into(), Value::Bool(true)),
            ("replayed".into(), Value::Int(records.len() as i64)),
            ("records".into(), Value::List(records)),
        ]))
    }

    /// Explicitly persist the last cell result through lib/writeback.
    fn persist_cell_inner(&mut self, cell: &str, key: &str) -> Result<Value, String> {
        let i = self
            .cell_index(cell)
            .ok_or_else(|| format!("unknown cell `{cell}`"))?;
        let value = self.cells[i].last_result.clone();
        if self.cells[i].runs == 0 || !self.cells[i].last_ok {
            return Err(format!("cell `{cell}` has no successful result to persist"));
        }
        let key = if key.is_empty() { cell } else { key };
        let line = self.cells[i].line_end.max(self.cells[i].line_start);
        crate::host::writeback::record(
            &mut self.host,
            &value,
            None,
            Some(&Value::Text(key.to_string())),
            Some(&Value::Int(line as i64)),
        )?;
        self.generation += 1;
        Ok(Value::Bool(true))
    }

    fn clear_outputs_inner(&mut self) -> Value {
        for cell in &mut self.cells {
            cell.runs = 0;
            cell.last_ms = 0;
            cell.last_ok = false;
            cell.last_error.clear();
            cell.last_stdout.clear();
            cell.last_code = 0;
            cell.last_result = Value::None;
            if cell.listed {
                cell.status = CellStatus::Never;
            }
        }
        self.journal.clear();
        self.oracle_journal.clear();
        self.cursor = 0;
        self.summary()
    }

    pub fn journal_json(&self) -> Value {
        Value::List(
            self.journal
                .iter()
                .map(|j| {
                    Value::Map(vec![
                        ("seq".into(), Value::Int(j.seq as i64)),
                        ("cell".into(), Value::Text(j.cell.clone())),
                        ("kernel".into(), Value::Text(j.kernel.clone())),
                        ("lang".into(), Value::Text(j.lang.clone())),
                        ("ok".into(), Value::Bool(j.ok)),
                        ("code".into(), Value::Int(j.code)),
                        ("ms".into(), Value::Int(j.ms as i64)),
                        ("timestamp_ms".into(), Value::Int(j.timestamp_ms)),
                        ("error".into(), Value::Text(j.error.clone())),
                    ])
                })
                .collect(),
        )
    }

    pub fn oracle_journal_json(&self) -> Value {
        Value::List(self.oracle_journal.iter().map(OracleRecord::to_value).collect())
    }

    fn close_inner(&mut self) {
        self.replay_fresh = false;
        let ids: Vec<String> = self.kernel_sessions.values().cloned().collect();
        for id in ids {
            let _ = foreign_session::close(&mut self.host, &Value::Handle(id));
        }
        self.kernel_sessions.clear();
    }
}

fn resolve_kernel(
    kernels: &mut Vec<KernelDecl>,
    map: &mut HashMap<String, KernelDecl>,
    name: &str,
) -> Result<KernelDecl, String> {
    if let Some(k) = map.get(name) {
        return Ok(k.clone());
    }
    if registry::preset_spec(name).is_some() {
        let lang = registry::canonical(name);
        let decl = KernelDecl {
            name: lang.clone(),
            lang,
            cmd: None,
            implicit: true,
        };
        map.insert(decl.name.clone(), decl.clone());
        kernels.push(decl.clone());
        return Ok(decl);
    }
    Err(format!(
        "foreign.session: kernel `{name}` is not declared in kernels"
    ))
}

fn ensure_default_kernel(
    kernels: &mut Vec<KernelDecl>,
    map: &mut HashMap<String, KernelDecl>,
    lang: &str,
) -> String {
    let lang = registry::canonical(lang);
    let name = format!("default:{lang}");
    if !map.contains_key(&name) {
        let decl = KernelDecl {
            name: name.clone(),
            lang,
            cmd: None,
            implicit: true,
        };
        map.insert(name.clone(), decl.clone());
        kernels.push(decl);
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_rows_reads_row_records_and_columns() {
        let row_table = Value::List(vec![Value::Map(vec![
            ("@".into(), Value::Text("1".into())),
            ("名称".into(), Value::Text("分析".into())),
            ("语言".into(), Value::Text("python".into())),
        ])]);
        let rows = table_rows(&row_table).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(col_text(&rows[0], &["名称"]).as_deref(), Some("分析"));

        let column_table = Value::Map(vec![
            ("名称".into(), Value::List(vec![Value::Text("分析".into())])),
            ("语言".into(), Value::List(vec![Value::Text("python".into())])),
        ]);
        let rows = table_rows(&column_table).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(col_text(&rows[0], &["语言"]).as_deref(), Some("python"));
    }

    #[test]
    fn plan_uses_evaluated_bindings() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/view-plan-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let source = r#"---
title: evaluated plan test
import:
  foreign: lib/foreign.mq.md
---

# main

`lang` = python

`分析内核` =

| @ | 名称 | 语言 |
|---|------|------|
| 1 | 分析 | `lang` |

`分析单元` =

| @ | 单元 | 内核 |
|---|------|------|
| 1 | 单元 | 分析 |

`会话` = > foreign.session kernels=`分析内核` cells=`分析单元`

`单元` =
```python
print(1)
```
"#;
        std::fs::write(&path, source).unwrap();
        let s = DocumentSession::open(&path, &root).unwrap();
        assert_eq!(s.cells.len(), 1);
        assert_eq!(s.cells[0].kernel, "分析");
        assert_eq!(s.kernels[0].lang, "python");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn view_open_accepts_a_document_without_cells() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/view-empty-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::write(&path, "# main\n\n> print text=hello\n").unwrap();
        let s = DocumentSession::open(&path, &root).unwrap();
        assert!(s.cells.is_empty());
        let _ = std::fs::remove_file(&path);
    }

    fn node_available() -> bool {
        std::process::Command::new("node")
            .args(["-e", "process.exit(0)"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn field(record: &Value, key: &str) -> Option<Value> {
        match record {
            Value::Map(entries) => entries
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone()),
            _ => None,
        }
    }

    #[test]
    fn shared_state_and_isolation() {
        if !node_available() {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/notebook-test-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let source = r#"---
title: shared state test
import:
  foreign: lib/foreign.mq.md
---

# main

`内核表` =

| 名称 | 语言 | 命令 | 说明 |
|------|------|------|------|
| 分析 | node | | shared |
| 绘图 | node | | isolated |

`会话表` =

| 单元 | 内核 | 说明 |
|------|------|------|
| 设置 | 分析 | write |
| 读取 | 分析 | read |
| 隔离 | 绘图 | isolated |

`会话` = > foreign.session kernels=`内核表` cells=`会话表`

`设置` =
```node
let shared = 41;
```

`读取` =
```node
console.log(shared + 1);
```

`隔离` =
```node
console.log(typeof shared === "undefined" ? "isolated" : "leaked");
```
"#;
        std::fs::write(&path, source).unwrap();
        let mut s = DocumentSession::open(&path, &root).unwrap();
        assert_eq!(
            s.order,
            vec!["设置".to_string(), "读取".to_string(), "隔离".to_string()]
        );
        let r = s.run_cell("设置", None).unwrap();
        assert_eq!(field(&r, "ok"), Some(Value::Bool(true)));
        let r = s.run_cell("读取", None).unwrap();
        assert_eq!(
            field(&r, "stdout").and_then(|v| v.reveal_text().map(|s| s.to_string())),
            Some("42\n".to_string())
        );
        let r = s.run_cell("隔离", None).unwrap();
        assert_eq!(
            field(&r, "stdout").and_then(|v| v.reveal_text().map(|s| s.to_string())),
            Some("isolated\n".to_string())
        );
        s.restart_all().unwrap();
        let r = s.run_cell("读取", None).unwrap();
        assert_eq!(field(&r, "ok"), Some(Value::Bool(false)));
        s.close();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn magic_table_names_without_session_are_ignored() {
        if !node_available() {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/view-stateless-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let source = r#"# main

`内核表` =

| 名称 | 语言 |
|------|------|
| 分析 | node |

`会话表` =

| 单元 | 内核 |
|------|------|
| a | 分析 |

`a` =
```node
console.log("stateless");
```
"#;
        std::fs::write(&path, source).unwrap();
        let mut s = DocumentSession::open(&path, &root).unwrap();
        assert!(!s.persistent);
        assert!(s.kernels.is_empty(), "magic tables must not create kernels");
        assert_eq!(s.order, vec!["a".to_string()]);
        assert_eq!(s.cells[0].kernel, "one-shot");
        let r = s.run_cell("a", None).unwrap();
        assert_eq!(
            field(&r, "stdout").and_then(|v| v.reveal_text().map(|s| s.to_string())),
            Some("stateless\n".to_string())
        );
        assert_eq!(s.cells[0].status, CellStatus::Clean);
        s.clear_outputs();
        assert_eq!(s.cells[0].status, CellStatus::Never);
        assert!(s.journal.is_empty());
        s.close();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn clear_outputs_keeps_persistent_kernel_state() {
        if !node_available() {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/view-clear-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let source = r#"---
title: clear outputs
import:
  foreign: lib/foreign.mq.md
---

# main

`内核` =

| 名称 | 语言 |
|------|------|
| 分析 | node |

`单元` =

| 单元 | 内核 |
|------|------|
| a | 分析 |

`会话` = > foreign.session kernels=`内核` cells=`单元`

`a` =
```node
globalThis.kept = (globalThis.kept || 0) + 1;
console.log(globalThis.kept);
```
"#;
        std::fs::write(&path, source).unwrap();
        let mut s = DocumentSession::open(&path, &root).unwrap();
        let r = s.run_cell("a", None).unwrap();
        assert_eq!(
            field(&r, "stdout").and_then(|v| v.reveal_text().map(|s| s.to_string())),
            Some("1\n".to_string())
        );
        s.clear_outputs();
        assert_eq!(s.cells[0].status, CellStatus::Never);
        assert!(s.journal.is_empty());
        let r = s.run_cell("a", None).unwrap();
        assert_eq!(
            field(&r, "stdout").and_then(|v| v.reveal_text().map(|s| s.to_string())),
            Some("2\n".to_string()),
            "clear outputs must not restart the kernel"
        );
        s.close();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn snapshot_export_import_restores_values() {
        if !node_available() {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("tests/notebook/shared.mq.md");
        let mut a = DocumentSession::open(&path, &root).unwrap();
        a.set_value("分析", "rate", Value::Int(42)).unwrap();
        let snap = a.export_snapshot().unwrap();
        a.close();
        let mut b = DocumentSession::open(&path, &root).unwrap();
        let mut wrong_source = snap.clone();
        replace_field(&mut wrong_source, "document_hash", Value::Text("different-document".into()));
        assert!(b.import_snapshot(&wrong_source).unwrap_err().contains("hash"));
        assert!(b.kernel_sessions.is_empty());
        let restored = b.import_snapshot(&snap).unwrap();
        assert!(matches!(restored, Value::Int(n) if n > 0));
        let rate = b.pull_value("分析", "rate").unwrap();
        assert_eq!(rate, Value::Int(42));
        b.close();
    }

    #[test]
    fn nested_plan_is_rejected_at_open() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/view-nested-plan-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let source = "# main\n\n## view\n\n`会话` = > foreign.session\n";
        std::fs::write(&path, source).unwrap();
        let err = match DocumentSession::open(&path, &root) {
            Ok(_) => panic!("nested plan should be rejected"),
            Err(e) => e,
        };
        assert!(err.contains("top-level"), "{err}");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn nested_plan_helper_detects_child_binding() {
        let src = "# main\n\n## view\n\n`会话` = > foreign.session\n";
        let module = crate::parse::parse_source(src).unwrap();
        assert!(has_nested_plan(&module, "main"));
    }

    #[test]
    fn runtime_plan_add_remove_and_generation() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("tests/notebook/shared.mq.md");
        let mut s = DocumentSession::open(&path, &root).unwrap();
        let before = s.generation;
        s.add_cell("extra", "node", "console.log('extra');", "分析")
            .unwrap();
        assert!(s.generation > before);
        assert!(s.cells.iter().any(|c| c.name == "extra"));
        s.assign_kernel("extra", "分析").unwrap();
        s.remove_cell("extra").unwrap();
        assert!(!s.cells.iter().any(|c| c.name == "extra"));
        assert!(s.generation > before + 1);
        s.close();
    }

    #[test]
    fn persist_cell_writes_a_keyed_slot() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/view-persist-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let source = r#"# main

`a` =
```python
print(950)
```
"#;
        std::fs::write(&path, source).unwrap();
        let mut s = DocumentSession::open(&path, &root).unwrap();
        s.run_cell("a", None).unwrap();
        s.persist_cell("a", "total").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("marqdo-out total"), "{text}");
        s.close();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn plan_inputs_feed_each_cell() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/view-plan-input-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let source = r#"---
title: plan inputs test
import:
  foreign: lib/foreign.mq.md
---

# main

`内核` =
| 名称 | 语言 |
|------|------|
| py | python |

`单元` =
| 单元 | 内核 |
|------|------|
| a | py |

`输入` =
| 单元 | stdin |
|------|-------|
| a | hello plan |

`会话` = > foreign.session kernels=`内核` cells=`单元` inputs=`输入`

`a` =
```python
import sys
print(sys.stdin.read().strip())
```
"#;
        std::fs::write(&path, source).unwrap();
        let mut s = DocumentSession::open(&path, &root).unwrap();
        let summary = s.run_all_with_stdin(false, None).unwrap();
        let records = match &summary {
            Value::Map(entries) => entries.iter().find(|(k, _)| k == "records").map(|(_, v)| v.clone()),
            _ => None,
        };
        let Some(Value::List(records)) = records else { panic!("no records") };
        assert_eq!(
            field(&records[0], "stdout").and_then(|v| v.reveal_text().map(|s| s.to_string())),
            Some("hello plan\n".to_string())
        );
        s.close();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn persistent_node_cell_accepts_raw_stdin() {
        if !node_available() {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/view-node-stdin-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let source = r#"---
title: persistent node stdin test
import:
  foreign: lib/foreign.mq.md
---

# main

`内核` =
| 名称 | 语言 |
|------|------|
| js | node |

`单元` =
| 单元 | 内核 |
|------|------|
| a | js |

`会话` = > foreign.session kernels=`内核` cells=`单元`

`a` =
```node
(async () => {
  await new Promise((resolve) => {
    let input = "";
    process.stdin.on("data", (chunk) => { input += chunk; });
    process.stdin.on("end", () => { console.log(input.trim()); resolve(); });
  });
})();
```
"#;
        std::fs::write(&path, source).unwrap();
        let mut s = DocumentSession::open(&path, &root).unwrap();
        let record = s
            .run_cell_with_stdin("a", None, Some("hello node"))
            .unwrap();
        assert_eq!(
            field(&record, "stdout").and_then(|v| v.reveal_text().map(|s| s.to_string())),
            Some("hello node\n".to_string())
        );
        s.run_cell_with_stdin("a", None, Some("second input 中文")).unwrap();
        let snapshot = s.export_snapshot().unwrap();
        s.close();
        let mut fresh = DocumentSession::open(&path, &root).unwrap();
        let replay = fresh.replay_check(&snapshot).unwrap();
        assert_eq!(field(&replay, "ok"), Some(Value::Bool(true)));
        assert_eq!(field(&replay, "matched"), Some(Value::Int(2)));
        fresh.close();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn persistent_python_cell_accepts_raw_stdin() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/view-persistent-stdin-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let source = r#"---
title: persistent stdin test
import:
  foreign: lib/foreign.mq.md
---

# main

`内核` =
| 名称 | 语言 |
|------|------|
| py | python |

`单元` =
| 单元 | 内核 |
|------|------|
| a | py |

`会话` = > foreign.session kernels=`内核` cells=`单元`

`a` =
```python
import sys
print(sys.stdin.read().strip())
```
"#;
        std::fs::write(&path, source).unwrap();
        let mut s = DocumentSession::open(&path, &root).unwrap();
        let record = s
            .run_cell_with_stdin("a", None, Some("hello persistent"))
            .unwrap();
        assert_eq!(
            field(&record, "stdout").and_then(|v| v.reveal_text().map(|s| s.to_string())),
            Some("hello persistent\n".to_string())
        );
        s.run_cell_with_stdin("a", None, Some("second input 中文")).unwrap();
        let snapshot = s.export_snapshot().unwrap();
        s.close();
        let mut fresh = DocumentSession::open(&path, &root).unwrap();
        let replay = fresh.replay_check(&snapshot).unwrap();
        assert_eq!(field(&replay, "ok"), Some(Value::Bool(true)));
        assert_eq!(field(&replay, "matched"), Some(Value::Int(2)));
        fresh.close();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn stateless_cell_accepts_stdin() {
        if !node_available() {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/view-stdin-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let source = r#"# main

`a` =
```node
const fs = require("fs");
const input = fs.readFileSync(0, "utf8").trim();
console.log("got:" + input);
```
"#;
        std::fs::write(&path, source).unwrap();
        let mut s = DocumentSession::open(&path, &root).unwrap();
        let record = s
            .run_cell_with_stdin("a", None, Some("hello stdin"))
            .unwrap();
        assert_eq!(
            field(&record, "stdout").and_then(|v| v.reveal_text().map(|s| s.to_string())),
            Some("got:hello stdin\n".to_string())
        );
        s.close();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn run_all_continues_after_a_cell_error() {
        if !node_available() {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/view-run-all-error-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let source = r#"# main

`a` =
```node
throw new Error("boom");
```

`b` =
```node
console.log("after error");
```
"#;
        std::fs::write(&path, source).unwrap();
        let mut s = DocumentSession::open(&path, &root).unwrap();
        let summary = s.run_all(false).unwrap();
        let records = match &summary {
            Value::Map(entries) => entries.iter().find(|(k, _)| k == "records").map(|(_, v)| v.clone()),
            _ => None,
        };
        let Some(Value::List(records)) = records else { panic!("no records") };
        assert_eq!(records.len(), 2);
        assert_eq!(field(&records[0], "ok"), Some(Value::Bool(false)));
        assert_eq!(field(&records[1], "ok"), Some(Value::Bool(true)));
        assert!(field(&records[1], "stdout")
            .and_then(|v| v.reveal_text().map(|s| s.to_string()))
            .unwrap_or_default()
            .contains("after error"));
        s.close();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn reload_marks_changed_cell_dirty() {
        if !node_available() {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/notebook-reload-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let src = "# main\n\n`a` =\n```node\nconsole.log(1);\n```\n";
        std::fs::write(&path, src).unwrap();
        let mut s = DocumentSession::open(&path, &root).unwrap();
        let r = s.run_cell("a", None).unwrap();
        assert_eq!(field(&r, "ok"), Some(Value::Bool(true)));
        assert_eq!(s.cells[0].status, CellStatus::Clean);
        let before = s.oracle_journal_json();
        let src2 = "# main\n\n`a` =\n```node\nconsole.log(2);\n```\n";
        std::fs::write(&path, src2).unwrap();
        s.reload(&root).unwrap();
        assert_eq!(s.cells[0].status, CellStatus::Dirty);
        assert!(s.cells[0].source.contains("console.log(2)"));
        assert_eq!(s.oracle_journal_json(), before, "reload must retain observations of the old source");
        s.close();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn oracle_journal_records_stdout_separate_from_execution_journal() {
        if !node_available() {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("tests/notebook/shared.mq.md");
        let mut s = DocumentSession::open(&path, &root).unwrap();
        s.run_all(true).unwrap();
        assert!(!s.journal.is_empty());
        assert_eq!(s.oracle_journal.len(), s.journal.len());
        let read = s
            .oracle_journal
            .iter()
            .find(|j| j.cell == "读取")
            .expect("oracle entry for 读取");
        assert!(read.ok);
        assert!(
            read.stdout.contains('4') || read.stdout.contains("42"),
            "oracle must capture stdout, got {:?}",
            read.stdout
        );
        assert!(!read.source_hash.is_empty());
        // Execution journal must not pretend to be the oracle (no stdout field).
        let exec = s.journal.iter().find(|j| j.cell == "读取").unwrap();
        assert_eq!(exec.ok, read.ok);
        s.close();
    }

    #[test]
    fn replay_check_matches_exported_recording() {
        if !node_available() {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("tests/notebook/shared.mq.md");
        let mut a = DocumentSession::open(&path, &root).unwrap();
        a.run_all(true).unwrap();
        let snap = a.export_snapshot().unwrap();
        a.close();

        let mut b = DocumentSession::open(&path, &root).unwrap();
        let check = b.replay_check(&snap).unwrap();
        assert_eq!(field(&check, "ok"), Some(Value::Bool(true)));
        assert_eq!(field(&check, "mismatched"), Some(Value::Int(0)));
        let matched = field(&check, "matched").and_then(|v| match v {
            Value::Int(n) => Some(n),
            _ => None,
        });
        assert!(matched.unwrap_or(0) >= 1, "expected at least one compared cell");
        b.close();
    }

    #[test]
    fn replay_check_localizes_stdout_mismatch() {
        if !node_available() {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("tests/notebook/shared.mq.md");
        let mut a = DocumentSession::open(&path, &root).unwrap();
        a.run_all(true).unwrap();
        let mut snap = a.export_snapshot().unwrap();
        a.close();

        // Tamper with the recording for cell 读取.
        if let Value::Map(ref mut fields) = snap {
            if let Some((_, Value::List(cells))) =
                fields.iter_mut().find(|(k, _)| k == "oracle_journal")
            {
                for cell in cells.iter_mut() {
                    let Value::Map(cfields) = cell else { continue };
                    let name = cfields
                        .iter()
                        .find(|(k, _)| k == "cell")
                        .and_then(|(_, v)| v.reveal_text().map(|s| s.to_string()));
                    if name.as_deref() == Some("读取") {
                        if let Some((_, v)) = cfields.iter_mut().find(|(k, _)| k == "stdout") {
                            *v = Value::Text("wrong-recording\n".into());
                        }
                    }
                }
            }
        }

        replay::sync_expected_cells(&mut snap).unwrap();
        let mut b = DocumentSession::open(&path, &root).unwrap();
        let check = b.replay_check(&snap).unwrap();
        assert_eq!(field(&check, "ok"), Some(Value::Bool(false)));
        assert_eq!(field(&check, "mismatched"), Some(Value::Int(1)));
        assert_eq!(field(&check, "skipped"), Some(Value::Int(1)));
        let Some(Value::List(checks)) = field(&check, "checks") else {
            panic!("missing checks");
        };
        let bad = checks.iter().find(|c| {
            field(c, "cell")
                .and_then(|v| v.reveal_text().map(|s| s.to_string()))
                .as_deref()
                == Some("读取")
        });
        let Some(bad) = bad else { panic!("missing 读取 check") };
        assert_eq!(field(bad, "match"), Some(Value::Bool(false)));
        let diff = field(bad, "diff")
            .and_then(|v| v.reveal_text().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(diff.contains("stdout"), "diff should localize to stdout: {diff}");
        b.close();
    }
    fn snapshot_list_mut<'a>(snap: &'a mut Value, key: &str) -> &'a mut Vec<Value> {
        let Value::Map(m) = snap else { panic!("map") };
        let (_, Value::List(xs)) = m.iter_mut().find(|(k, _)| k == key).unwrap()
            else { panic!("list") };
        xs
    }

    fn replace_field(v: &mut Value, key: &str, value: Value) {
        let Value::Map(m) = v else { panic!("map") };
        m.iter_mut().find(|(k, _)| k == key).unwrap().1 = value;
    }

    #[test]
    fn replay_check_rejects_incomplete_recordings_before_execution() {
        assert!(node_available(), "replay experiment requires node");
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("tests/notebook/shared.mq.md");
        let mut a = DocumentSession::open(&path, &root).unwrap();
        a.run_all(true).unwrap();
        let original = a.export_snapshot().unwrap();
        a.close();
        let cases = ["empty", "truncate", "insert", "duplicate", "reorder", "unknown",
            "missing_cell", "non_map", "missing_outcome", "wrong_bool", "wrong_code",
            "wrong_stdout", "missing_result", "source", "routing", "plan", "version",
            "missing_oracle", "duplicate_field", "stdin_type"];
        for case in cases {
            let mut snap = original.clone();
            match case {
                "empty" => snapshot_list_mut(&mut snap, "oracle_journal").clear(),
                "truncate" => { snapshot_list_mut(&mut snap, "oracle_journal").pop(); }
                "insert" | "duplicate" => {
                    let xs = snapshot_list_mut(&mut snap, "oracle_journal");
                    xs.push(xs[0].clone());
                }
                "reorder" => snapshot_list_mut(&mut snap, "oracle_journal").swap(0, 1),
                "unknown" => replace_field(&mut snapshot_list_mut(&mut snap, "oracle_journal")[0], "cell", Value::Text("unknown".into())),
                "missing_cell" => { snapshot_list_mut(&mut snap, "cells").pop(); }
                "non_map" => snapshot_list_mut(&mut snap, "oracle_journal")[0] = Value::None,
                "missing_outcome" | "missing_result" => {
                    let Value::Map(m) = &mut snapshot_list_mut(&mut snap, "oracle_journal")[0] else { panic!() };
                    m.retain(|(k, _)| k != if case == "missing_result" { "result" } else { "ok" });
                }
                "wrong_bool" | "wrong_code" | "wrong_stdout" | "stdin_type" => {
                    let key = match case { "wrong_bool" => "ok", "wrong_code" => "code", "wrong_stdout" => "stdout", _ => "stdin" };
                    replace_field(&mut snapshot_list_mut(&mut snap, "oracle_journal")[0], key, Value::None);
                }
                "source" => replace_field(&mut snapshot_list_mut(&mut snap, "cells")[0], "source", Value::Text("console.log(42)".into())),
                "routing" => replace_field(&mut snapshot_list_mut(&mut snap, "oracle_journal")[0], "kernel", Value::Text("绘图".into())),
                "plan" => snapshot_list_mut(&mut snap, "order").swap(0, 1),
                "version" => replace_field(&mut snap, "version", Value::Int(1)),
                "missing_oracle" => { let Value::Map(m) = &mut snap else { panic!() }; m.retain(|(k, _)| k != "oracle_journal"); }
                "duplicate_field" => { let Value::Map(m) = &mut snapshot_list_mut(&mut snap, "oracle_journal")[0] else { panic!() }; m.push(m[0].clone()); }
                _ => unreachable!(),
            }
            let mut b = DocumentSession::open(&path, &root).unwrap();
            let result = b.replay_check(&snap);
            assert!(result.is_err(), "{case}: {result:?}");
            assert!(b.oracle_journal.is_empty(), "{case}: executed before validation");
            assert!(b.kernel_sessions.is_empty(), "{case}: started a kernel before validation");
            println!("E1 {case}: rejected before execution: {}", result.unwrap_err());
            b.close();
        }
    }

    #[test]
    fn replay_check_uses_oracle_not_cell_summary_or_execution_journal() {
        assert!(node_available());
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("tests/notebook/shared.mq.md");
        let mut a = DocumentSession::open(&path, &root).unwrap();
        a.run_all(true).unwrap();
        // Repeat the read: a per-cell summary cannot represent this history.
        a.run_cell("读取", None).unwrap();
        let mut snap = a.export_snapshot().unwrap();
        a.close();
        snapshot_list_mut(&mut snap, "journal").clear();
        for c in snapshot_list_mut(&mut snap, "cells") {
            replace_field(c, "stdout", Value::Text("irrelevant-summary".into()));
        }
        let mut b = DocumentSession::open(&path, &root).unwrap();
        let check = b.replay_check(&snap).unwrap();
        assert_eq!(field(&check, "ok"), Some(Value::Bool(true)));
        assert_eq!(field(&check, "matched"), Some(Value::Int(4)));
        assert_eq!(field(&check, "skipped"), Some(Value::Int(0)));
        assert!(b.replay_check(&snap).unwrap_err().contains("fresh"));
        b.close();
    }

    #[test]
    fn replay_check_accepts_declared_initial_values_but_rejects_external_changes() {
        assert!(node_available());
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!("target/replay-initial-{}.mq.md", std::process::id()));
        let source = std::fs::read_to_string(root.join("tests/notebook/shared.mq.md")).unwrap()
            .replace("`会话` = > foreign.session kernels=`内核表` cells=`会话表`",
                "`初始` =\n\n| 内核 | 名称 | 值 |\n|---|---|---|\n| 分析 | seed | 41 |\n\n`会话` = > foreign.session kernels=`内核表` cells=`会话表` values=`初始`");
        std::fs::write(&path, source).unwrap();
        let mut a = DocumentSession::open(&path, &root).unwrap();
        assert!(!a.kernel_sessions.is_empty(), "fixture must initialize a live kernel");
        a.run_all(false).unwrap();
        let snap = a.export_snapshot().unwrap();
        a.close();
        let mut b = DocumentSession::open(&path, &root).unwrap();
        let check = b.replay_check(&snap).unwrap();
        assert_eq!(field(&check, "ok"), Some(Value::Bool(true)));
        b.close();
        let mut warm = DocumentSession::open(&path, &root).unwrap();
        warm.set_value("分析", "seed", Value::Int(99)).unwrap();
        assert!(warm.replay_check(&snap).unwrap_err().contains("fresh"));
        warm.close();
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn snapshot_introspection_does_not_pollute_document_observations() {
        assert!(node_available());
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("tests/notebook/shared.mq.md");
        let mut s = DocumentSession::open(&path, &root).unwrap();
        s.run_all(false).unwrap();
        let before = s.host.oracle_observations.clone();
        assert_eq!(before.len(), 3);
        s.export_snapshot().unwrap();
        s.export_snapshot().unwrap();
        assert_eq!(s.host.oracle_observations, before, "snapshot get/names are metadata, not document actions");
        // The cell trace projects to exactly the same language observations.
        let projected: Vec<Value> = s.oracle_journal.iter().enumerate().map(|(i,r)|
            Value::Map(vec![("index".into(), Value::Int(i as i64)),
                ("channel".into(), Value::Text("eval".into())), ("value".into(), r.value.clone())])).collect();
        assert_eq!(before, projected);
        s.pull_value("分析", "shared").unwrap();
        assert_eq!(s.host.oracle_observations.len(), before.len() + 1, "explicit user get must be observed");
        s.close();
    }

    #[test]
    fn replay_check_compares_failed_cell_diagnostics() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!("target/replay-failure-{}.mq.md", std::process::id()));
        let source = "# main\n\n`a` =\n```python\nraise ValueError('ordinary error')\n```\n";
        std::fs::write(&path, source).unwrap();
        let mut a = DocumentSession::open(&path, &root).unwrap();
        let record = a.run_cell("a", None).unwrap();
        assert_eq!(field(&record, "ok"), Some(Value::Bool(false)));
        let mut snap = a.export_snapshot().unwrap();
        a.close();
        let mut b = DocumentSession::open(&path, &root).unwrap();
        let initial_check = b.replay_check(&snap).unwrap();
        assert_eq!(field(&initial_check, "ok"), Some(Value::Bool(true)), "{initial_check:?}");
        b.close();
        let rows = snapshot_list_mut(&mut snap, "oracle_journal");
        let Value::Map(fields) = &mut rows[0] else { panic!() };
        let (_, value) = fields.iter_mut().find(|(k,_)| k == "value").unwrap();
        replace_field(value, "error", Value::Text("different failure".into()));
        replay::sync_expected_cells(&mut snap).unwrap();
        let mut c = DocumentSession::open(&path, &root).unwrap();
        let check = c.replay_check(&snap).unwrap();
        assert_eq!(field(&check, "ok"), Some(Value::Bool(false)));
        let Some(Value::List(checks)) = field(&check, "checks") else { panic!() };
        assert!(field(&checks[0], "diff").unwrap().as_display().contains("value"));
        c.close();
        std::fs::remove_file(path).unwrap();
    }

}
