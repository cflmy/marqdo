//! Document session: cells, kernels, execution journal, state store.
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
    pub listed: bool,
    pub status: CellStatus,
    pub runs: u64,
    pub last_ms: u64,
    pub last_ok: bool,
    pub last_error: String,
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

pub struct DocumentSession {
    pub path: PathBuf,
    root: PathBuf,
    pub persistent: bool,
    pub cells: Vec<Cell>,
    pub kernels: Vec<KernelDecl>,
    pub order: Vec<String>,
    pub journal: Vec<JournalRecord>,
    pub value_store: HashMap<String, Value>,
    pub cursor: usize,
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
) -> Result<(Option<Vec<HashMap<String, Value>>>, Option<Vec<HashMap<String, Value>>>), String> {
    let Some(plan) = bindings.get("会话") else {
        return Ok((None, None));
    };
    let Value::Map(fields) = plan else {
        // `会话` is also a common low-level session id binding (e.g. from
        // foreign.session_open). It is not a view plan; leave cells stateless.
        return Ok((None, None));
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
    Ok((kernels, cells))
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
                    listed: false,
                    status: CellStatus::Never,
                    runs: 0,
                    last_ms: 0,
                    last_ok: false,
                    last_error: String::new(),
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

impl DocumentSession {
    pub fn open(path: &Path, root: &Path) -> Result<Self, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        let bind_ctx = crate::binding::BindingContext::from_binds(
            &[],
            crate::binding::cwd_for_path(Some(path)),
        );
        let module = crate::load::load_module_with_ctx(path, &bind_ctx)
            .map_err(|e| format!("load {}: {e}", path.display()))?;
        let entry = module
            .functions
            .iter()
            .find(|f| f.name == "main" && f.is_object())
            .or_else(|| {
                let objs: Vec<_> = module.functions.iter().filter(|f| f.is_object()).collect();
                if objs.len() == 1 {
                    Some(objs[0])
                } else {
                    None
                }
            })
            .ok_or_else(|| "notebook: no `# main` (or sole object) to index".to_string())?;
        let mut cells = Vec::new();
        let mut seen = HashSet::new();
        collect_cells(&entry.body, &mut cells, &mut seen)?;
        let declarations = capture_bindings(path, root, &module, &source)?;

        let (plan_kernels, plan_cells) = plan_tables(&declarations)?;
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
        host.entry_source = Some(source);

        Ok(Self {
            path: path.to_path_buf(),
            root: root.to_path_buf(),
            persistent,
            cells,
            kernels,
            order,
            journal: Vec::new(),
            value_store: HashMap::new(),
            cursor: 0,
            opened_ms: now_ms(),
            host,
            kernel_sessions: HashMap::new(),
            seq: 0,
        })
    }

    /// Re-read the document, preserving kernels/state/journal. Changed cells become dirty.
    pub fn reload(&mut self, root: &Path) -> Result<(), String> {
        let mut fresh = Self::open(&self.path, root)?;
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
        fresh.kernel_sessions = std::mem::take(&mut self.kernel_sessions);
        fresh.value_store = std::mem::take(&mut self.value_store);
        fresh.journal = std::mem::take(&mut self.journal);
        fresh.seq = self.seq;
        fresh.cursor = self.cursor.min(fresh.order.len());
        fresh.opened_ms = self.opened_ms;
        let known: HashSet<String> = fresh.kernels.iter().map(|k| k.name.clone()).collect();
        let orphans: Vec<String> = fresh
            .kernel_sessions
            .keys()
            .filter(|k| !known.contains(*k))
            .cloned()
            .collect();
        for k in orphans {
            if let Some(id) = fresh.kernel_sessions.remove(&k) {
                let _ = foreign_session::close(&mut fresh.host, &Value::Text(id));
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
            Value::Text(s) => s,
            _ => return Err("session_open returned non-text id".into()),
        };
        self.kernel_sessions
            .insert(kernel_name.to_string(), id.clone());
        Ok(id)
    }

    fn record_result(&mut self, cell: &str, kernel: &str, record: &Value) -> Result<(), String> {
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
        let lang = self
            .cell_index(cell)
            .map(|i| self.cells[i].lang.clone())
            .unwrap_or_default();
        if let Some(i) = self.cell_index(cell) {
            let c = &mut self.cells[i];
            c.runs += 1;
            c.last_ms = ms;
            c.last_ok = ok;
            c.last_error = error.clone();
            c.status = if ok { CellStatus::Clean } else { CellStatus::Error };
        }
        self.seq += 1;
        self.journal.push(JournalRecord {
            seq: self.seq,
            cell: cell.to_string(),
            kernel: kernel.to_string(),
            lang,
            ok,
            code,
            ms,
            timestamp_ms: now_ms(),
            error,
        });
        Ok(())
    }

    pub fn run_cell(&mut self, cell_name: &str, kernel_override: Option<&str>) -> Result<Value, String> {
        self.run_cell_with_stdin(cell_name, kernel_override, None)
    }

    pub fn run_cell_with_stdin(
        &mut self,
        cell_name: &str,
        kernel_override: Option<&str>,
        stdin: Option<&str>,
    ) -> Result<Value, String> {
        let ci = self
            .cell_index(cell_name)
            .ok_or_else(|| format!("unknown cell `{cell_name}`"))?;
        if !self.cells[ci].listed && kernel_override.is_none() {
            return Err(format!("cell `{cell_name}` is not listed in the foreign session"));
        }
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
                    self.record_result(cell_name, "one-shot", &record)?;
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
            let source = Value::Text(self.cells[ci].source.clone());
            let record = foreign_session::eval(
                &mut self.host,
                &Value::Text(id),
                None,
                Some(&source),
                stdin.map(|s| Value::Text(s.to_string())).as_ref(),
                None,
            )?;
            self.record_result(cell_name, &kernel, &record)?;
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

    pub fn step_with_stdin(&mut self, stdin: Option<&str>) -> Result<Value, String> {
        if self.cursor >= self.order.len() {
            return Ok(Value::Map(vec![
                ("done".into(), Value::Bool(true)),
                ("cursor".into(), Value::Int(self.cursor as i64)),
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
        if restart {
            self.restart_all()?;
        }
        self.cursor = 0;
        let mut records = Vec::new();
        let mut failed = 0i64;
        for name in self.order.clone() {
            let record = match self.run_cell_with_stdin(&name, None, stdin) {
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
            records.push(Value::Map(m));
            self.cursor += 1;
        }
        Ok(Value::Map(vec![
            ("ok".into(), Value::Bool(failed == 0)),
            ("ran".into(), Value::Int(self.order.len() as i64)),
            ("failed".into(), Value::Int(failed)),
            ("records".into(), Value::List(records)),
        ]))
    }

    pub fn restart_kernel(&mut self, name: &str) -> Result<Value, String> {
        if let Some(id) = self.kernel_sessions.remove(name) {
            let _ = foreign_session::close(&mut self.host, &Value::Text(id));
        }
        for c in &mut self.cells {
            if c.kernel == name {
                c.status = CellStatus::Never;
                c.last_error.clear();
            }
        }
        Ok(Value::Bool(true))
    }

    pub fn restart_all(&mut self) -> Result<Value, String> {
        let names: Vec<String> = self.kernel_sessions.keys().cloned().collect();
        for name in names {
            self.restart_kernel(&name)?;
        }
        self.cursor = 0;
        for c in &mut self.cells {
            c.status = if c.listed { CellStatus::Never } else { CellStatus::Unlisted };
        }
        Ok(Value::Bool(true))
    }

    pub fn assign_kernel(&mut self, cell: &str, kernel: &str) -> Result<Value, String> {
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
        Ok(Value::None)
    }

    pub fn state_list(&mut self) -> Result<Value, String> {
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
                    if let Ok(names) = foreign_session::names(&mut self.host, &Value::Text(id)) {
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

    pub fn push_value(&mut self, kernel: &str, name: &str, value: Value) -> Result<Value, String> {
        let id = self.ensure_kernel_session(kernel)?;
        foreign_session::set(
            &mut self.host,
            &Value::Text(id),
            &Value::Text(name.to_string()),
            &value,
        )?;
        self.value_store.insert(name.to_string(), value);
        Ok(Value::None)
    }

    pub fn pull_value(&mut self, kernel: &str, name: &str) -> Result<Value, String> {
        let id = self.ensure_kernel_session(kernel)?;
        let value = foreign_session::get(
            &mut self.host,
            &Value::Text(id),
            &Value::Text(name.to_string()),
        )?;
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
    pub fn clear_cell(&mut self, cell: &str) -> Result<Value, String> {
        let i = self
            .cell_index(cell)
            .ok_or_else(|| format!("unknown cell `{cell}`"))?;
        self.cells[i].runs = 0;
        self.cells[i].last_ms = 0;
        self.cells[i].last_ok = false;
        self.cells[i].last_error.clear();
        if self.cells[i].listed {
            self.cells[i].status = CellStatus::Never;
        }
        self.journal.retain(|j| j.cell != cell);
        Ok(self.summary())
    }

    /// Clear displayed outputs and per-cell records; kernels and live state survive.
    pub fn clear_outputs(&mut self) -> Value {
        for cell in &mut self.cells {
            cell.runs = 0;
            cell.last_ms = 0;
            cell.last_ok = false;
            cell.last_error.clear();
            if cell.listed {
                cell.status = CellStatus::Never;
            }
        }
        self.journal.clear();
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

    pub fn close(&mut self) {
        let ids: Vec<String> = self.kernel_sessions.values().cloned().collect();
        for id in ids {
            let _ = foreign_session::close(&mut self.host, &Value::Text(id));
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
import foreign:lib/foreign.mq.md
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
import foreign:lib/foreign.mq.md
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
import foreign:lib/foreign.mq.md
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
        let src2 = "# main\n\n`a` =\n```node\nconsole.log(2);\n```\n";
        std::fs::write(&path, src2).unwrap();
        s.reload(&root).unwrap();
        assert_eq!(s.cells[0].status, CellStatus::Dirty);
        assert!(s.cells[0].source.contains("console.log(2)"));
        s.close();
        let _ = std::fs::remove_file(&path);
    }
}
