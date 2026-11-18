//! Complete interactive operation history. Wire values are tagged so invalid
//! boundary attempts can be replayed without serializing an opaque capability.
use super::*;

fn map(fields: Vec<(&str, Value)>) -> Value {
    Value::Map(fields.into_iter().map(|(k, v)| (k.into(), v)).collect())
}
fn object(v: &Value) -> Result<&[(String, Value)], String> {
    let Value::Map(xs) = v else {
        return Err("history: expected record".into());
    };
    let mut seen = HashSet::new();
    if xs.iter().any(|(k, _)| !seen.insert(k)) {
        return Err("history: duplicate field".into());
    }
    Ok(xs)
}
fn field<'a>(v: &'a Value, key: &str) -> Result<&'a Value, String> {
    object(v)?
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or_else(|| format!("history: missing {key}"))
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    match field(v, key)? {
        Value::Text(s) => Ok(s),
        _ => Err(format!("history: {key} must be text")),
    }
}
fn list<'a>(v: &'a Value, key: &str) -> Result<&'a [Value], String> {
    match field(v, key)? {
        Value::List(xs) => Ok(xs),
        _ => Err(format!("history: {key} must be a list")),
    }
}
fn boolean(v: &Value, key: &str) -> Result<bool, String> {
    match field(v, key)? {
        Value::Bool(x) => Ok(*x),
        _ => Err(format!("history: {key} must be boolean")),
    }
}
fn optional_text<'a>(v: &'a Value, key: &str) -> Result<Option<&'a str>, String> {
    match field(v, key)? {
        Value::None => Ok(None),
        Value::Text(x) => Ok(Some(x)),
        _ => Err(format!("history: {key} must be text or None")),
    }
}
fn exact(v: &Value, keys: &[&str]) -> Result<(), String> {
    let xs = object(v)?;
    if xs.len() != keys.len() || xs.iter().any(|(k, _)| !keys.contains(&k.as_str())) {
        return Err("history: unexpected or missing fields".into());
    }
    Ok(())
}
// Internal metadata is not itself a foreign column table. Compare it without
// applying a payload-domain rule to a dictionary of separate host bindings.
fn control_canonical(v: &Value) -> Value {
    match v {
        Value::Map(fields) => {
            let mut fields: Vec<_> = fields
                .iter()
                .map(|(k, v)| (k.clone(), control_canonical(v)))
                .collect();
            fields.sort_by(|a, b| a.0.cmp(&b.0));
            Value::Map(fields)
        }
        Value::List(values) => Value::List(values.iter().map(control_canonical).collect()),
        Value::Num(n) if *n == 0.0 => Value::Num(0.0),
        value => value.clone(),
    }
}
fn same(a: &Value, b: &Value) -> Result<bool, String> {
    Ok(encode(&control_canonical(a)) == encode(&control_canonical(b)))
}
pub(super) fn validate_control(v: &Value) -> Result<(), String> {
    match v {
        Value::Map(fields) => {
            object(v)?;
            for (_, v) in fields {
                validate_control(v)?;
            }
            Ok(())
        }
        Value::List(values) => {
            for v in values {
                validate_control(v)?;
            }
            Ok(())
        }
        Value::None | Value::Bool(_) | Value::Int(_) | Value::Num(_) | Value::Text(_) => {
            super::super::oracle::admit(v)
        }
        _ => Err("history: control result contains a non-data value".into()),
    }
}

fn encode(v: &Value) -> Value {
    let (tag, payload) = match v {
        Value::None => ("null", Value::None),
        Value::Bool(b) => ("bool", Value::Bool(*b)),
        Value::Int(n) => ("int", Value::Text(n.to_string())),
        Value::Num(n) => ("num", Value::Text(format!("{:016x}", n.to_bits()))),
        Value::Text(s) => ("text", Value::Text(s.clone())),
        Value::List(xs) => ("list", Value::List(xs.iter().map(encode).collect())),
        Value::Map(xs) => (
            "map",
            Value::List(
                xs.iter()
                    .map(|(k, v)| Value::List(vec![Value::Text(k.clone()), encode(v)]))
                    .collect(),
            ),
        ),
        Value::Code(c) => (
            "code",
            map(vec![
                ("lang", Value::Text(c.lang.clone())),
                ("source", Value::Text(c.source.clone())),
            ]),
        ),
        Value::Handle(_) => ("invalid-handle", Value::None),
        Value::Secret(_) => ("invalid-secret", Value::None),
        Value::Formula(_) => ("invalid-formula", Value::None),
    };
    Value::List(vec![Value::Text(tag.into()), payload])
}
fn decode(v: &Value) -> Result<Value, String> {
    let Value::List(pair) = v else {
        return Err("history: expected tagged value".into());
    };
    if pair.len() != 2 {
        return Err("history: invalid tagged value".into());
    }
    let Value::Text(tag) = &pair[0] else {
        return Err("history: missing value tag".into());
    };
    let p = &pair[1];
    Ok(match (tag.as_str(), p) {
        ("null", Value::None) => Value::None,
        ("bool", Value::Bool(b)) => Value::Bool(*b),
        ("text", Value::Text(s)) => Value::Text(s.clone()),
        ("int", Value::Text(s)) => Value::Int(s.parse().map_err(|_| "history: invalid integer")?),
        ("num", Value::Text(s)) => Value::Num(f64::from_bits(
            u64::from_str_radix(s, 16).map_err(|_| "history: invalid float bits")?,
        )),
        ("list", Value::List(xs)) => Value::List(xs.iter().map(decode).collect::<Result<_, _>>()?),
        ("map", Value::List(xs)) => {
            let mut entries = Vec::new();
            for item in xs {
                let Value::List(kv) = item else {
                    return Err("history: invalid map entry".into());
                };
                if kv.len() != 2 {
                    return Err("history: invalid map entry".into());
                }
                let Value::Text(key) = &kv[0] else {
                    return Err("history: invalid map key".into());
                };
                entries.push((key.clone(), decode(&kv[1])?));
            }
            Value::Map(entries)
        }
        ("code", _) => {
            exact(p, &["lang", "source"])?;
            Value::Code(crate::value::CodeBlock {
                lang: text(p, "lang")?.into(),
                source: text(p, "source")?.into(),
            })
        }
        ("invalid-handle", Value::None) => Value::Handle("replay-invalid-payload".into()),
        ("invalid-secret", Value::None) => Value::Secret(String::new()),
        ("invalid-formula", Value::None) => Value::Formula(crate::formula::Expr::Num(0.0)),
        _ => return Err("history: unknown tag or incorrect payload type".into()),
    })
}

#[derive(Clone, Debug)]
pub(super) struct HistoryEntry {
    controls: Vec<super::control::ControlEvent>,
    kind: String,
    args: Value,
    outcome: Value,
    protocol: Vec<Value>,
    observations: Vec<Value>,
    cells: Vec<Value>,
    state: Value,
}

pub(super) struct ReplayBoundary {
    cells: std::collections::VecDeque<Value>,
    records: Option<std::collections::VecDeque<Value>>,
    error: Option<String>,
}
impl HistoryEntry {
    fn wire(&self, index: usize) -> Value {
        let mut fields = vec![
            ("index", Value::Int(index as i64)),
            ("kind", Value::Text(self.kind.clone())),
            ("args", encode(&self.args)),
            ("outcome", encode(&self.outcome)),
            ("protocol", Value::List(self.protocol.clone())),
            (
                "observations",
                encode(&Value::List(self.observations.clone())),
            ),
            ("cells", Value::List(self.cells.clone())),
            ("state", encode(&self.state)),
        ];
        if !self.controls.is_empty() {
            fields.push((
                "controls",
                Value::List(self.controls.iter().map(|event| event.value()).collect()),
            ));
        }
        map(fields)
    }
    fn parse(v: &Value, index: usize) -> Result<Self, String> {
        let mut schema = vec![
            "index",
            "kind",
            "args",
            "outcome",
            "protocol",
            "observations",
            "cells",
            "state",
        ];
        let has_controls = object(v)?.iter().any(|(key, _)| key == "controls");
        if has_controls {
            schema.push("controls");
        }
        exact(v, &schema)?;
        let mut controls = Vec::new();
        if has_controls {
            let mut previous = 0;
            for row in list(v, "controls")? {
                exact(row, &["after_us", "results"])?;
                let Value::Int(after) = field(row, "after_us")? else {
                    return Err("history: control time must be integer".into());
                };
                if *after < 0 || (*after as u64) < previous {
                    return Err("history: invalid control ordering".into());
                }
                previous = *after as u64;
                let mut names = HashSet::new();
                let mut results = Vec::new();
                for result in list(row, "results")? {
                    exact(result, &["kernel", "accepted", "active", "error"])?;
                    let name = text(result, "kernel")?;
                    if name.is_empty() || !names.insert(name) {
                        return Err("history: duplicate/empty control kernel".into());
                    }
                    let accepted = boolean(result, "accepted")?;
                    let active = boolean(result, "active")?;
                    let error = text(result, "error")?;
                    if (!accepted && active)
                        || (accepted && !error.is_empty())
                        || (!accepted && error.is_empty())
                    {
                        return Err("history: inconsistent interrupt receipt".into());
                    }
                    results.push((
                        name.into(),
                        if accepted {
                            Ok(active)
                        } else {
                            Err(error.into())
                        },
                    ));
                }
                controls.push(super::control::ControlEvent {
                    after_us: *after as u64,
                    results,
                });
            }
        }
        if field(v, "index")? != &Value::Int(index as i64) {
            return Err(format!("history: action {index} index differs"));
        }
        let observations = decode(field(v, "observations")?)?;
        let Value::List(observations) = observations else {
            return Err("history: observations must be a list".into());
        };
        for value in &observations {
            super::super::oracle::admit(value)?;
        }
        Ok(Self {
            controls,
            kind: text(v, "kind")?.into(),
            args: decode(field(v, "args")?)?,
            outcome: decode(field(v, "outcome")?)?,
            protocol: list(v, "protocol")?.to_vec(),
            observations,
            cells: list(v, "cells")?.to_vec(),
            state: decode(field(v, "state")?)?,
        })
    }
}

impl DocumentSession {
    pub(super) fn initialize_values(&mut self) -> Result<(), String> {
        let mut initialized = HashSet::new();
        for (kernel, _, _) in self.initial_values.clone() {
            if initialized.insert(kernel.clone()) {
                self.ensure_kernel_session(&kernel)?;
            }
        }
        Ok(())
    }
    pub(super) fn plan_image(&self) -> Value {
        map(vec![
            (
                "source",
                Value::Text(self.host.entry_source.clone().unwrap_or_default()),
            ),
            ("source_hash", Value::Text(self.source_hash.clone())),
            ("persistent", Value::Bool(self.persistent)),
            (
                "kernels",
                Value::List(
                    self.kernels
                        .iter()
                        .map(|k| {
                            map(vec![
                                ("name", Value::Text(k.name.clone())),
                                ("lang", Value::Text(k.lang.clone())),
                                ("cmd", k.cmd.clone().map(Value::Text).unwrap_or(Value::None)),
                                ("implicit", Value::Bool(k.implicit)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "cells",
                Value::List(
                    self.cells
                        .iter()
                        .map(|c| {
                            map(vec![
                                ("name", Value::Text(c.name.clone())),
                                ("lang", Value::Text(c.lang.clone())),
                                ("source", Value::Text(c.source.clone())),
                                ("source_hash", Value::Text(c.source_hash.clone())),
                                ("kernel", Value::Text(c.kernel.clone())),
                                ("listed", Value::Bool(c.listed)),
                                ("stdin", Value::Text(c.stdin.clone())),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "order",
                Value::List(self.order.iter().cloned().map(Value::Text).collect()),
            ),
            (
                "initial_values",
                Value::List(
                    self.initial_values
                        .iter()
                        .map(|(k, n, v)| {
                            map(vec![
                                ("kernel", Value::Text(k.clone())),
                                ("name", Value::Text(n.clone())),
                                ("value", v.clone()),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])
    }
    fn from_image(
        path: &Path,
        root: &Path,
        image: &Value,
        initialize: bool,
    ) -> Result<Self, String> {
        exact(
            image,
            &[
                "source",
                "source_hash",
                "persistent",
                "kernels",
                "cells",
                "order",
                "initial_values",
            ],
        )?;
        let source = text(image, "source")?;
        let source_hash = hash_source(source);
        if text(image, "source_hash")? != source_hash {
            return Err("history: source hash differs".into());
        }
        let ctx = crate::binding::BindingContext::from_binds(
            &[],
            crate::binding::cwd_for_path(Some(path)),
        );
        let module = crate::load::load_module_with_source(path, source, &ctx)
            .map_err(|e| format!("history source: {e}"))?;
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
            .ok_or("history: missing entry")?;
        let mut parsed = Vec::new();
        collect_cells(&entry.body, &mut parsed, &mut HashSet::new())?;
        let persistent = boolean(image, "persistent")?;
        let mut kernels = Vec::new();
        let mut names = HashSet::new();
        for row in list(image, "kernels")? {
            exact(row, &["name", "lang", "cmd", "implicit"])?;
            let name = text(row, "name")?.to_string();
            if !names.insert(name.clone()) {
                return Err("history: duplicate kernel".into());
            }
            kernels.push(KernelDecl {
                name,
                lang: text(row, "lang")?.into(),
                cmd: optional_text(row, "cmd")?.map(str::to_string),
                implicit: boolean(row, "implicit")?,
            });
        }
        let mut cells = Vec::new();
        let mut cellnames = HashSet::new();
        for row in list(image, "cells")? {
            exact(
                row,
                &[
                    "name",
                    "lang",
                    "source",
                    "source_hash",
                    "kernel",
                    "listed",
                    "stdin",
                ],
            )?;
            let name = text(row, "name")?;
            if !cellnames.insert(name.to_string()) {
                return Err("history: duplicate cell".into());
            }
            let mut cell = parsed
                .iter()
                .find(|c| c.name == name)
                .cloned()
                .ok_or("history: cell absent from source")?;
            if cell.source != text(row, "source")?
                || cell.source_hash != text(row, "source_hash")?
                || cell.lang != text(row, "lang")?
            {
                return Err("history: cell source differs".into());
            }
            cell.kernel = text(row, "kernel")?.into();
            cell.listed = boolean(row, "listed")?;
            cell.stdin = text(row, "stdin")?.into();
            if persistent {
                if cell.listed
                    && !kernels.iter().any(|k| {
                        k.name == cell.kernel
                            && registry::canonical(&k.lang) == registry::canonical(&cell.lang)
                    })
                {
                    return Err("history: invalid cell routing".into());
                }
            } else if cell.kernel != "one-shot" {
                return Err("history: invalid one-shot routing".into());
            }
            if !cell.listed {
                cell.status = CellStatus::Unlisted;
            }
            cells.push(cell);
        }
        if cells.len() != parsed.len() {
            return Err("history: incomplete source cells".into());
        }
        let mut order = Vec::new();
        let mut seen = HashSet::new();
        for item in list(image, "order")? {
            let Value::Text(name) = item else {
                return Err("history: order must contain names".into());
            };
            if !seen.insert(name.clone()) || !cells.iter().any(|c| &c.name == name && c.listed) {
                return Err("history: invalid order".into());
            }
            order.push(name.clone());
        }
        if cells.iter().filter(|c| c.listed).count() != order.len() {
            return Err("history: incomplete order".into());
        }
        let mut initial_values = Vec::new();
        for row in list(image, "initial_values")? {
            exact(row, &["kernel", "name", "value"])?;
            let kernel = text(row, "kernel")?;
            let name = text(row, "name")?;
            let value = field(row, "value")?.clone();
            if !kernels.iter().any(|k| k.name == kernel) || !foreign_session::valid_name(name) {
                return Err("history: invalid initial binding".into());
            }
            super::super::oracle::admit(&value)?;
            initial_values.push((kernel.into(), name.into(), value));
        }
        let mut host = HostContext::default();
        host.cwd = path.parent().unwrap_or_else(|| Path::new(".")).into();
        host.fs_root = Some(root.into());
        host.entry_path = Some(path.into());
        host.entry_source = Some(source.into());
        host.entry_metadata = module.metadata;
        let mut session = Self {
            control: super::DocumentControl::default(),
            replay_controls: Vec::new(),
            replay_boundary: None,
            source_hash,
            replay_fresh: true,
            initial_values,
            initial_image: image.clone(),
            action_history: Vec::new(),
            action_depth: 0,
            path: path.into(),
            root: root.into(),
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
        if initialize {
            session.initialize_values()?;
            session.replay_fresh = true;
        }
        Ok(session)
    }
    fn history_state(&self) -> Value {
        map(vec![
            ("plan", self.plan_image()),
            ("cursor", Value::Int(self.cursor as i64)),
            ("generation", Value::Int(self.generation as i64)),
            (
                "live_kernels",
                Value::List({
                    let mut names: Vec<_> = self
                        .kernel_sessions
                        .iter()
                        .filter(|(_, id)| self.host.foreign_sessions.contains_key(*id))
                        .map(|(name, _)| t(name))
                        .collect();
                    names.sort_by_key(|v| v.as_display());
                    names
                }),
            ),
            (
                "values",
                Value::List({
                    let mut bindings: Vec<_> = self.value_store.iter().collect();
                    bindings.sort_by_key(|(name, _)| *name);
                    bindings
                        .into_iter()
                        .map(|(name, value)| map(vec![("name", t(name)), ("value", value.clone())]))
                        .collect()
                }),
            ),
            (
                "cells",
                Value::List(
                    self.cells
                        .iter()
                        .map(|c| {
                            map(vec![
                                ("name", Value::Text(c.name.clone())),
                                ("runs", Value::Int(c.runs as i64)),
                                ("status", Value::Text(c.status.as_str().into())),
                                ("ok", Value::Bool(c.last_ok)),
                                ("stdout", Value::Text(c.last_stdout.clone())),
                                ("result", c.last_result.clone()),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])
    }
}

fn t(s: &str) -> Value {
    Value::Text(s.into())
}
fn ot(s: Option<&str>) -> Value {
    s.map(t).unwrap_or(Value::None)
}
impl DocumentSession {
    pub(super) fn check_replay_cell_boundary(&mut self) {
        let Some(boundary) = &mut self.replay_boundary else {
            return;
        };
        let actual = self
            .oracle_journal
            .last()
            .expect("record just appended")
            .to_value();
        let expected = boundary.cells.pop_front();
        if !expected
            .as_ref()
            .is_some_and(|expected| same(&actual, expected).unwrap_or(false))
        {
            boundary.error =
                Some("replay: cell observation differs at its execution boundary".into());
        }
    }

    pub(super) fn replay_batch_can_continue(&mut self) -> bool {
        if self.host.foreign_replay_divergence.is_some() {
            return false;
        }
        let Some(boundary) = &mut self.replay_boundary else {
            return true;
        };
        if boundary.error.is_some() {
            return false;
        }
        if boundary
            .records
            .as_ref()
            .is_some_and(|records| records.is_empty())
        {
            boundary.error = Some("replay: unexpected extra batch unit".into());
            return false;
        }
        true
    }

    pub(super) fn check_replay_batch_record(&mut self, record: &Value, seq_before: u64) {
        let Some(boundary) = &mut self.replay_boundary else {
            return;
        };
        if let Some(records) = &mut boundary.records {
            let actual = eval_result(record);
            let expected = records.pop_front();
            if !matches!((actual, expected), (Ok(actual), Some(expected)) if same(&actual, &expected).unwrap_or(false))
            {
                boundary.error =
                    Some("replay: batch result differs at its execution boundary".into());
            }
        }
        // Run-all's order is unique (validated in both plan readers). A refused
        // unit must not silently consume an expected admitted observation.
        if self.seq == seq_before {
            if let (Ok(name), Some(expected)) = (text(record, "cell"), boundary.cells.front()) {
                if text(expected, "cell").ok() == Some(name) {
                    boundary.error =
                        Some("replay: missing cell observation at execution boundary".into());
                }
            }
        }
    }

    pub fn run_cell_with_stdin(
        &mut self,
        cell_name: &str,
        kernel_override: Option<&str>,
        stdin: Option<&str>,
    ) -> Result<Value, String> {
        self.tracked(
            "cell",
            map(vec![
                ("cell", t(cell_name)),
                ("kernel", ot(kernel_override)),
                ("stdin", ot(stdin)),
            ]),
            |session| session.run_cell_with_stdin_inner(cell_name, kernel_override, stdin),
        )
    }
    pub fn step_with_stdin(&mut self, stdin: Option<&str>) -> Result<Value, String> {
        self.tracked("step", map(vec![("stdin", ot(stdin))]), |session| {
            session.step_with_stdin_inner(stdin)
        })
    }
    pub fn run_all_with_inputs(
        &mut self,
        restart: bool,
        stdin: Option<&str>,
        inputs: Option<&HashMap<String, String>>,
    ) -> Result<Value, String> {
        self.tracked(
            "run_all",
            map(vec![
                ("restart", Value::Bool(restart)),
                ("stdin", ot(stdin)),
                (
                    "inputs",
                    inputs
                        .map(|m| Value::Map(m.iter().map(|(k, v)| (k.clone(), t(v))).collect()))
                        .unwrap_or(Value::None),
                ),
            ]),
            |session| session.run_all_with_inputs_inner(restart, stdin, inputs),
        )
    }
    pub fn interrupt(&mut self) -> Result<Value, String> {
        self.tracked("interrupt", map(vec![]), |session| {
            session.interrupt_inner()
        })
    }
    pub fn restart_kernel(&mut self, name: &str) -> Result<Value, String> {
        self.tracked("restart", map(vec![("kernel", t(name))]), |session| {
            session.restart_kernel_inner(name)
        })
    }
    pub fn restart_all(&mut self) -> Result<Value, String> {
        self.tracked("restart_all", map(vec![]), |session| {
            session.restart_all_inner()
        })
    }
    pub fn assign_kernel(&mut self, cell: &str, kernel: &str) -> Result<Value, String> {
        self.tracked(
            "assign",
            map(vec![("cell", t(cell)), ("kernel", t(kernel))]),
            |session| session.assign_kernel_inner(cell, kernel),
        )
    }
    pub fn add_cell(
        &mut self,
        cell: &str,
        lang: &str,
        source: &str,
        kernel: &str,
    ) -> Result<Value, String> {
        self.tracked(
            "add",
            map(vec![
                ("cell", t(cell)),
                ("lang", t(lang)),
                ("source", t(source)),
                ("kernel", t(kernel)),
            ]),
            |session| session.add_cell_inner(cell, lang, source, kernel),
        )
    }
    pub fn remove_cell(&mut self, cell: &str) -> Result<Value, String> {
        self.tracked("remove", map(vec![("cell", t(cell))]), |session| {
            session.remove_cell_inner(cell)
        })
    }
    pub fn set_value(&mut self, kernel: &str, name: &str, value: Value) -> Result<Value, String> {
        self.tracked(
            "set",
            map(vec![
                ("kernel", t(kernel)),
                ("name", t(name)),
                ("value", value.clone()),
            ]),
            |session| session.set_value_inner(kernel, name, value),
        )
    }
    pub fn push_value(&mut self, kernel: &str, name: &str, value: Value) -> Result<Value, String> {
        self.tracked(
            "push",
            map(vec![
                ("kernel", t(kernel)),
                ("name", t(name)),
                ("value", value.clone()),
            ]),
            |session| session.push_value_inner(kernel, name, value),
        )
    }
    pub fn pull_value(&mut self, kernel: &str, name: &str) -> Result<Value, String> {
        self.tracked(
            "get",
            map(vec![("kernel", t(kernel)), ("name", t(name))]),
            |session| session.pull_value_inner(kernel, name),
        )
    }
    pub fn state_list(&mut self) -> Result<Value, String> {
        self.tracked("state", map(vec![]), |session| session.state_list_inner())
    }
    pub fn clear_cell(&mut self, cell: &str) -> Result<Value, String> {
        self.tracked("clear_cell", map(vec![("cell", t(cell))]), |session| {
            session.clear_cell_inner(cell)
        })
    }
    pub fn import_snapshot(&mut self, snapshot: &Value) -> Result<Value, String> {
        self.tracked(
            "import",
            map(vec![("snapshot", snapshot.clone())]),
            |session| session.import_snapshot_inner(snapshot),
        )
    }
    pub fn replay_snapshot(&mut self, snapshot: &Value) -> Result<Value, String> {
        self.tracked(
            "rerun",
            map(vec![("snapshot", snapshot.clone())]),
            |session| session.replay_snapshot_inner(snapshot),
        )
    }
    pub fn persist_cell(&mut self, cell: &str, key: &str) -> Result<Value, String> {
        self.tracked(
            "persist",
            map(vec![("cell", t(cell)), ("key", t(key))]),
            |session| session.persist_cell_inner(cell, key),
        )
    }
    pub(super) fn inspect_kernels(&mut self) -> Result<Value, String> {
        let mut kernels = Vec::new();
        for decl in self.kernels.clone() {
            let mut values = Vec::new();
            if let Some(id) = self.kernel_sessions.get(&decl.name).cloned() {
                if let Ok(Value::List(names)) =
                    foreign_session::names(&mut self.host, &Value::Handle(id.clone()))
                {
                    for name in names {
                        if let Value::Text(n) = &name {
                            if let Ok(v) = foreign_session::get(
                                &mut self.host,
                                &Value::Handle(id.clone()),
                                &Value::Text(n.clone()),
                            ) {
                                values.push(Value::Map(vec![
                                    ("name".into(), Value::Text(n.clone())),
                                    ("value".into(), v),
                                ]));
                            }
                        }
                    }
                }
            }
            kernels.push(Value::Map(vec![
                ("name".into(), Value::Text(decl.name.clone())),
                ("lang".into(), Value::Text(decl.lang.clone())),
                (
                    "cmd".into(),
                    Value::Text(decl.cmd.clone().unwrap_or_default()),
                ),
                ("values".into(), Value::List(values)),
            ]));
        }
        Ok(Value::List(kernels))
    }
    pub fn clear_outputs(&mut self) -> Value {
        self.tracked("clear", map(vec![]), |s| Ok(s.clear_outputs_inner()))
            .expect("clearing outputs cannot fail")
    }
    pub fn close(&mut self) {
        let _ = self.tracked("close", map(vec![]), |s| {
            s.close_inner();
            Ok(Value::None)
        });
    }
    pub fn reload(&mut self, root: &Path) -> Result<(), String> {
        match std::fs::read_to_string(&self.path) {
            Ok(source) => self
                .tracked(
                    "reload",
                    map(vec![
                        ("source", t(&source)),
                        ("image", Value::None),
                        ("input_error", Value::None),
                    ]),
                    |s| s.reload_source(&source, root).map(|_| Value::None),
                )
                .map(|_| ()),
            Err(error) => {
                let error = format!("read {}: {error}", self.path.display());
                self.tracked(
                    "reload",
                    map(vec![
                        ("source", Value::None),
                        ("image", Value::None),
                        ("input_error", t(&error)),
                    ]),
                    |_| Err(error),
                )
                .map(|_| ())
            }
        }
    }
}

fn eval_result(value: &Value) -> Result<Value, String> {
    let xs = object(value)?;
    if xs.iter().any(|(k, _)| k == "ok") {
        let mut fields = vec![
            ("value", super::super::oracle::eval_observation(value)?),
            ("ok", field(value, "ok")?.clone()),
            ("code", field(value, "code")?.clone()),
            ("stdout", field(value, "stdout")?.clone()),
        ];
        for key in ["cursor", "generation"] {
            if let Some((_, v)) = xs.iter().find(|(k, _)| k == key) {
                fields.push((key, v.clone()));
            }
        }
        Ok(map(fields))
    } else {
        Ok(value.clone())
    }
}
fn outcome(kind: &str, result: &Result<Value, String>) -> Result<Value, String> {
    Ok(match result {
        Err(error) => map(vec![("ok", Value::Bool(false)), ("error", t(error))]),
        Ok(value) => {
            let projected = match kind {
                "cell" | "step" => eval_result(value)?,
                "run_all" | "rerun" => {
                    let mut fields = Vec::new();
                    for key in ["ok", "ran", "failed", "replayed", "cancelled", "remaining"] {
                        if let Some((_, v)) = object(value)?.iter().find(|(k, _)| k == key) {
                            fields.push((key, v.clone()));
                        }
                    }
                    fields.push((
                        "records",
                        Value::List(
                            list(value, "records")?
                                .iter()
                                .map(eval_result)
                                .collect::<Result<_, _>>()?,
                        ),
                    ));
                    map(fields)
                }
                "clear" | "clear_cell" => Value::None,
                _ => value.clone(),
            };
            map(vec![("ok", Value::Bool(true)), ("value", projected)])
        }
    })
}
impl DocumentSession {
    pub(super) fn tracked(
        &mut self,
        kind: &str,
        mut args: Value,
        f: impl FnOnce(&mut Self) -> Result<Value, String>,
    ) -> Result<Value, String> {
        if self.action_depth != 0 {
            return f(self);
        }
        let seq = self.seq;
        let observed = self.host.oracle_observations.len();
        let requests = self.host.foreign_protocol_steps.len();
        let scheduler = self
            .control
            .begin(std::mem::take(&mut self.replay_controls));
        self.action_depth += 1;
        let result = f(self);
        self.action_depth -= 1;
        let controls = self.control.finish(scheduler);
        self.replay_fresh = false;
        if kind == "reload" && result.is_ok() {
            let Value::Map(fields) = &mut args else {
                unreachable!()
            };
            fields.iter_mut().find(|(k, _)| k == "image").unwrap().1 = self.plan_image();
        }
        let entry = HistoryEntry {
            controls,
            kind: kind.into(),
            args,
            outcome: outcome(kind, &result)?,
            protocol: self.host.foreign_protocol_steps[requests..]
                .iter()
                .map(|step| {
                    map(vec![
                        ("session", t(&step.session)),
                        ("request", t(&step.request.to_string())),
                        (
                            "response",
                            match &step.response {
                                Ok(value) => map(vec![
                                    ("ok", Value::Bool(true)),
                                    ("json", t(&value.to_string())),
                                ]),
                                Err(error) => {
                                    map(vec![("ok", Value::Bool(false)), ("error", t(error))])
                                }
                            },
                        ),
                    ])
                })
                .collect(),
            observations: self.host.oracle_observations[observed..].to_vec(),
            cells: self
                .oracle_journal
                .iter()
                .filter(|j| j.seq > seq)
                .map(OracleRecord::to_value)
                .collect(),
            state: self.history_state(),
        };
        self.action_history.push(entry);
        result
    }
    pub(super) fn action_history_json(&self) -> Value {
        Value::List(
            self.action_history
                .iter()
                .enumerate()
                .map(|(i, e)| e.wire(i))
                .collect(),
        )
    }
    pub(super) fn complete_oracle_rows(&self) -> Vec<Value> {
        self.action_history
            .iter()
            .flat_map(|e| e.cells.clone())
            .collect()
    }

    fn check_action(entry: &HistoryEntry) -> Result<(), String> {
        let a = &entry.args;
        let schema: &[&str] = match entry.kind.as_str() {
            "cell" => &["cell", "kernel", "stdin"],
            "step" => &["stdin"],
            "run_all" => &["restart", "stdin", "inputs"],
            "restart" => &["kernel"],
            "assign" => &["cell", "kernel"],
            "add" => &["cell", "lang", "source", "kernel"],
            "remove" | "clear_cell" => &["cell"],
            "set" | "push" => &["kernel", "name", "value"],
            "get" => &["kernel", "name"],
            "import" | "rerun" => &["snapshot"],
            "persist" => &["cell", "key"],
            "reload" => &["source", "image", "input_error"],
            "restart_all" | "interrupt" | "state" | "inspect" | "clear" | "close" => &[],
            _ => return Err("history: unknown action kind".into()),
        };
        exact(a, schema)?;
        for key in schema {
            match *key {
                "value" | "snapshot" | "image" => {}
                "restart" => {
                    boolean(a, key)?;
                }
                "input_error" => {
                    optional_text(a, key)?;
                }
                "source" if entry.kind == "reload" => {
                    optional_text(a, key)?;
                }
                "stdin" => {
                    optional_text(a, key)?;
                }
                "kernel" if entry.kind == "cell" => {
                    optional_text(a, key)?;
                }
                "inputs" => {
                    if field(a, key)? != &Value::None {
                        for (_, v) in object(field(a, key)?)? {
                            if !matches!(v, Value::Text(_)) {
                                return Err("history: inputs must map names to text".into());
                            }
                        }
                    }
                }
                _ => {
                    text(a, key)?;
                }
            }
        }
        for step in &entry.protocol {
            exact(step, &["session", "request", "response"])?;
            text(step, "session")?;
            serde_json::from_str::<serde_json::Value>(text(step, "request")?)
                .map_err(|_| "history: malformed protocol request")?;
            let response = field(step, "response")?;
            if boolean(response, "ok")? {
                exact(response, &["ok", "json"])?;
                serde_json::from_str::<serde_json::Value>(text(response, "json")?)
                    .map_err(|_| "history: malformed protocol response")?;
            } else {
                exact(response, &["ok", "error"])?;
                text(response, "error")?;
            }
        }
        validate_control(&entry.outcome)?;
        validate_control(&entry.state)?;
        if boolean(&entry.outcome, "ok")? {
            exact(&entry.outcome, &["ok", "value"])?;
        } else {
            exact(&entry.outcome, &["ok", "error"])?;
            text(&entry.outcome, "error")?;
        }
        Ok(())
    }
    fn execute_action(&mut self, entry: &HistoryEntry) -> Result<Value, String> {
        let a = &entry.args;
        match entry.kind.as_str() {
            "cell" => self.run_cell_with_stdin(
                text(a, "cell")?,
                optional_text(a, "kernel")?,
                optional_text(a, "stdin")?,
            ),
            "step" => self.step_with_stdin(optional_text(a, "stdin")?),
            "run_all" => {
                let inputs = match field(a, "inputs")? {
                    Value::None => None,
                    v => Some(
                        object(v)?
                            .iter()
                            .map(|(k, v)| match v {
                                Value::Text(s) => Ok((k.clone(), s.clone())),
                                _ => Err("history: input type".into()),
                            })
                            .collect::<Result<HashMap<_, _>, String>>()?,
                    ),
                };
                self.run_all_with_inputs(
                    boolean(a, "restart")?,
                    optional_text(a, "stdin")?,
                    inputs.as_ref(),
                )
            }
            "restart" => self.restart_kernel(text(a, "kernel")?),
            "restart_all" => self.restart_all(),
            "assign" => self.assign_kernel(text(a, "cell")?, text(a, "kernel")?),
            "add" => self.add_cell(
                text(a, "cell")?,
                text(a, "lang")?,
                text(a, "source")?,
                text(a, "kernel")?,
            ),
            "remove" => self.remove_cell(text(a, "cell")?),
            "set" => self.set_value(
                text(a, "kernel")?,
                text(a, "name")?,
                field(a, "value")?.clone(),
            ),
            "push" => self.push_value(
                text(a, "kernel")?,
                text(a, "name")?,
                field(a, "value")?.clone(),
            ),
            "get" => self.pull_value(text(a, "kernel")?, text(a, "name")?),
            "state" => self.state_list(),
            "inspect" => self.tracked("inspect", a.clone(), |s| s.inspect_kernels()),
            "import" => self.import_snapshot(field(a, "snapshot")?),
            "rerun" => self.replay_snapshot(field(a, "snapshot")?),
            "persist" => self.persist_cell(text(a, "cell")?, text(a, "key")?),
            "clear_cell" => self.clear_cell(text(a, "cell")?),
            "clear" => Ok(self.clear_outputs()),
            "interrupt" => self.interrupt(),
            "close" => {
                self.close();
                Ok(Value::None)
            }
            "reload" => {
                let args = a.clone();
                let image = field(a, "image")?.clone();
                let source = optional_text(a, "source")?.map(str::to_string);
                let input_error = optional_text(a, "input_error")?.map(str::to_string);
                let root = self.root.clone();
                self.tracked("reload", args, |s| {
                    if let Some(error) = input_error {
                        return Err(error);
                    }
                    if image == Value::None {
                        s.reload_source(
                            source.as_deref().ok_or("history: reload source missing")?,
                            &root,
                        )?;
                    } else {
                        let fresh = Self::from_image(&s.path, &root, &image, false)?;
                        s.install_reload(fresh)?;
                    }
                    Ok(Value::None)
                })
            }
            _ => Err("history: unsupported action".into()),
        }
    }
    // Validate configuration transitions without querying any foreign kernel.
    fn simulate_action(&mut self, entry: &HistoryEntry) -> Result<(), String> {
        Self::check_action(entry)?;
        let a = &entry.args;
        let success = boolean(&entry.outcome, "ok")?;
        match entry.kind.as_str() {
            "reload" if success => {
                let image = field(a, "image")?;
                if optional_text(a, "input_error")?.is_some()
                    || Some(text(image, "source")?) != optional_text(a, "source")?
                {
                    return Err("history: reload source differs".into());
                }
                let fresh = Self::from_image(&self.path, &self.root, image, false)?;
                self.install_reload(fresh)?;
            }
            "reload" => {
                if let Some(error) = optional_text(a, "input_error")? {
                    if optional_text(a, "source")?.is_some()
                        || text(&entry.outcome, "error")? != error
                    {
                        return Err("history: invalid reload input failure".into());
                    }
                } else if optional_text(a, "source")?.is_none() {
                    return Err("history: missing reload source".into());
                }
                if field(a, "image")? != &Value::None {
                    return Err("history: failed reload has a plan image".into());
                }
            }
            "assign" | "add" | "remove" => {
                let result = match entry.kind.as_str() {
                    "assign" => self.assign_kernel_inner(text(a, "cell")?, text(a, "kernel")?),
                    "add" => self.add_cell_inner(
                        text(a, "cell")?,
                        text(a, "lang")?,
                        text(a, "source")?,
                        text(a, "kernel")?,
                    ),
                    _ => self.remove_cell_inner(text(a, "cell")?),
                };
                if result.is_ok() != success {
                    return Err("history: impossible plan edit outcome".into());
                }
            }
            "cell" => {
                if let Some(kernel) = optional_text(a, "kernel")? {
                    if let Some(ci) = self.cell_index(text(a, "cell")?) {
                        if self.kernel(kernel).map(|k| registry::canonical(&k.lang))
                            == Some(registry::canonical(&self.cells[ci].lang))
                        {
                            self.cells[ci].kernel = kernel.into();
                            self.cells[ci].listed = true;
                        }
                    }
                }
            }
            "step" if success => {
                if self.cursor < self.order.len() {
                    self.cursor += 1;
                }
            }
            "run_all" if success => {
                let Value::Int(ran) = field(field(&entry.outcome, "value")?, "ran")? else {
                    return Err("history: invalid run count".into());
                };
                if *ran < 0
                    || *ran as usize > self.order.len()
                    || (entry.controls.is_empty() && *ran as usize != self.order.len())
                {
                    return Err("history: impossible run count".into());
                }
                self.cursor = *ran as usize;
            }
            "restart_all" | "clear" => {
                self.cursor = 0;
            }
            "set" | "import" | "persist" if success => {
                self.generation += 1;
            }
            _ => {}
        }
        match entry.kind.as_str() {
            "cell" | "step" | "run_all" | "rerun" => {
                if entry.observations.len() != entry.cells.len() {
                    return Err("history: cell/observation projection length differs".into());
                }
                for (row, obs) in entry.cells.iter().zip(&entry.observations) {
                    if text(obs, "channel")? != "eval"
                        || !same(field(obs, "value")?, field(row, "value")?)?
                    {
                        return Err("history: cell observation value differs".into());
                    }
                }
                if entry.kind == "cell" && entry.cells.len() != if success { 1 } else { 0 } {
                    return Err("history: impossible cell outcome".into());
                }
            }
            "get" => {
                if !entry.cells.is_empty()
                    || entry.observations.len() != if success { 1 } else { 0 }
                {
                    return Err("history: get observation count differs".into());
                }
                if success
                    && (text(&entry.observations[0], "channel")? != "get"
                        || !same(
                            field(&entry.observations[0], "value")?,
                            field(&entry.outcome, "value")?,
                        )?)
                {
                    return Err("history: get observation differs".into());
                }
            }
            _ => {
                if !entry.cells.is_empty() || !entry.observations.is_empty() {
                    return Err("history: control action has foreign observations".into());
                }
            }
        }
        if success && ["set", "push", "get"].contains(&entry.kind.as_str()) {
            let kernel = text(a, "kernel")?;
            let name = text(a, "name")?;
            if self.kernel(kernel).is_none() || !foreign_session::valid_name(name) {
                return Err("history: impossible binding operation".into());
            }
            let value = if entry.kind == "get" {
                field(&entry.outcome, "value")?
            } else {
                field(a, "value")?
            };
            super::super::oracle::admit(value)?;
            self.value_store.insert(name.to_string(), value.clone());
        }
        if !same(
            field(&self.history_state(), "values")?,
            field(&entry.state, "values")?,
        )? {
            return Err("history: inconsistent host bindings".into());
        }
        for row in &entry.cells {
            let name = text(row, "cell")?;
            let ci = self
                .cell_index(name)
                .ok_or("history: observation refers to missing cell")?;
            let cell = &self.cells[ci];
            if text(row, "source_hash")? != cell.source_hash
                || text(row, "lang")? != cell.lang
                || text(row, "kernel")? != cell.kernel
            {
                return Err("history: observation source or route differs".into());
            }
            text(row, "stdin")?;
            text(row, "stdout")?;
            boolean(row, "ok")?;
            if !matches!(field(row, "code")?, Value::Int(_)) {
                return Err("history: invalid observation code".into());
            }
            field(row, "result")?;
            super::super::oracle::admit(field(row, "value")?)?;
        }
        if !same(&self.plan_image(), field(&entry.state, "plan")?)? {
            return Err("history: inconsistent plan transition".into());
        }
        if field(&entry.state, "cursor")? != &Value::Int(self.cursor as i64)
            || field(&entry.state, "generation")? != &Value::Int(self.generation as i64)
        {
            return Err("history: inconsistent cursor or generation".into());
        }
        Ok(())
    }
    pub fn replay_check(&mut self, snapshot: &Value) -> Result<Value, String> {
        let expanded = if super::packing::is_packed(snapshot) {
            Some(super::packing::unpack(snapshot)?)
        } else {
            None
        };
        let snapshot = expanded.as_ref().unwrap_or(snapshot);
        super::super::oracle::admit(snapshot)?;
        let version = field(snapshot, "version")?;
        if version != &Value::Int(4) && version != &Value::Int(5) {
            return Err("replay: version 4/5 complete action history required".into());
        }
        if !self.replay_fresh || self.seq != 0 || !self.action_history.is_empty() {
            return Err("replay: fresh document session required".into());
        }
        if text(snapshot, "document_hash")? != self.source_hash {
            return Err("replay: document hash differs".into());
        }
        let rows = list(snapshot, "actions")?;
        if field(snapshot, "action_count")? != &Value::Int(rows.len() as i64) {
            return Err("history: action count differs".into());
        }
        let entries = rows
            .iter()
            .enumerate()
            .map(|(i, row)| HistoryEntry::parse(row, i))
            .collect::<Result<Vec<_>, _>>()?;
        if version == &Value::Int(4)
            && rows.iter().any(|row| {
                object(row).is_ok_and(|fields| fields.iter().any(|(key, _)| key == "controls"))
            })
        {
            return Err("history: concurrent controls require version 5".into());
        }
        let image = field(snapshot, "initial_image")?;
        let mut simulation = Self::from_image(&self.path, &self.root, image, false)?;
        let mut cells = Vec::new();
        let mut observations = Vec::new();
        for entry in &entries {
            simulation.simulate_action(entry)?;
            cells.extend(entry.cells.clone());
            observations.extend(entry.observations.clone());
        }
        for (i, row) in cells.iter().enumerate() {
            if field(row, "seq")? != &Value::Int((i + 1) as i64) {
                return Err("history: noncontiguous oracle sequence".into());
            }
        }
        if field(snapshot, "oracle_count")? != &Value::Int(cells.len() as i64)
            || !same(&Value::List(cells), field(snapshot, "oracle_journal")?)?
        {
            return Err("history: oracle projection differs".into());
        }
        let observed = Value::List(observations);
        super::super::oracle::replay_check(&observed, &observed)?;
        if !same(&observed, field(snapshot, "observation_journal")?)? {
            return Err("history: document observation projection differs".into());
        }
        if text(snapshot, "document_hash")? != simulation.source_hash
            || boolean(snapshot, "persistent")? != simulation.persistent
        {
            return Err("history: final source or mode differs".into());
        }
        let expected_order =
            Value::List(simulation.order.iter().cloned().map(Value::Text).collect());
        if !same(field(snapshot, "order")?, &expected_order)? {
            return Err("history: final plan order differs".into());
        }
        let final_cells = list(snapshot, "cells")?;
        if final_cells.len() != simulation.cells.len() {
            return Err("history: final cell manifest differs".into());
        }
        for (row, cell) in final_cells.iter().zip(&simulation.cells) {
            for (key, expected) in [
                ("name", &cell.name),
                ("source", &cell.source),
                ("source_hash", &cell.source_hash),
                ("kernel", &cell.kernel),
                ("lang", &cell.lang),
            ] {
                if text(row, key)? != expected {
                    return Err("history: final cell identity differs".into());
                }
            }
        }
        if field(snapshot, "generation")? != &Value::Int(simulation.generation as i64) {
            return Err("history: final generation differs".into());
        }
        let last = entries
            .last()
            .ok_or("history: snapshot lacks state inspection")?;
        if last.kind != "inspect"
            || !boolean(&last.outcome, "ok")?
            || !same(field(&last.outcome, "value")?, field(snapshot, "kernels")?)?
        {
            return Err("history: final state inspection differs".into());
        }
        // All validation above is pure with respect to external kernels.
        let mut fresh = Self::from_image(&self.path, &self.root, image, true)?;
        self.close_inner();
        std::mem::swap(self, &mut fresh);
        let mut checks = Vec::new();
        let mut action_checks = Vec::new();
        let mut matched_observations = 0i64;
        let mut mismatched = 0i64;
        let mut skipped = 0i64;
        for (index, expected) in entries.iter().enumerate() {
            let protocol = expected
                .protocol
                .iter()
                .map(|step| {
                    let response = field(step, "response")?;
                    Ok(super::foreign_session::ProtocolStep {
                        session: text(step, "session")?.into(),
                        request: serde_json::from_str(text(step, "request")?)
                            .map_err(|e| e.to_string())?,
                        response: if boolean(response, "ok")? {
                            Ok(serde_json::from_str(text(response, "json")?)
                                .map_err(|e| e.to_string())?)
                        } else {
                            Err(text(response, "error")?.into())
                        },
                    })
                })
                .collect::<Result<std::collections::VecDeque<_>, String>>()?;
            self.host.foreign_replay_expected = Some(protocol.into());
            self.host.foreign_replay_divergence = None;
            let records = if ["run_all", "rerun"].contains(&expected.kind.as_str())
                && boolean(&expected.outcome, "ok")?
            {
                Some(
                    list(field(&expected.outcome, "value")?, "records")?
                        .to_vec()
                        .into(),
                )
            } else if expected.kind == "run_all" {
                Some(std::collections::VecDeque::new())
            } else {
                None
            };
            self.replay_boundary = Some(ReplayBoundary {
                cells: expected.cells.clone().into(),
                records,
                error: None,
            });
            self.replay_controls = expected.controls.clone();
            let _result = self.execute_action(expected);
            let boundary = self.replay_boundary.take().unwrap();
            let mut guarded_error = self
                .host
                .foreign_replay_divergence
                .take()
                .or(boundary.error);
            if self
                .host
                .foreign_replay_expected
                .take()
                .is_some_and(|cursor| cursor.finish().is_err())
                || !boundary.cells.is_empty()
                || boundary.records.is_some_and(|records| !records.is_empty())
            {
                guarded_error
                    .get_or_insert("replay: recorded execution prefix was not consumed".into());
            }
            let actual = self
                .action_history
                .last()
                .ok_or("history: action produced no record")?;
            let mut bad_cells = 0i64;
            for n in 0..expected.cells.len().max(actual.cells.len()) {
                let exp = expected.cells.get(n);
                let act = actual.cells.get(n);
                if act.is_none() {
                    // The causal guard deliberately did not execute this suffix.
                    // It is unobserved, not a second observed mismatch.
                    skipped += 1;
                    checks.push(map(vec![
                        (
                            "cell",
                            t(exp
                                .map(|r| text(r, "cell"))
                                .transpose()?
                                .unwrap_or("missing")),
                        ),
                        ("action", Value::Int(index as i64)),
                        ("step", Value::Int(n as i64)),
                        ("match", Value::Bool(false)),
                        ("compared", Value::Bool(false)),
                        ("diff", t("not executed after replay divergence")),
                    ]));
                    continue;
                }
                let mut diff = String::new();
                match (exp, act) {
                    (Some(exp), Some(act)) => {
                        for key in [
                            "seq",
                            "cell",
                            "kernel",
                            "lang",
                            "source_hash",
                            "stdin",
                            "ok",
                            "code",
                            "stdout",
                            "result",
                            "value",
                        ] {
                            if !same(field(exp, key)?, field(act, key)?)? {
                                diff = format!(
                                    "action {index} observation {n} {key}: expected {}, got {}",
                                    field(exp, key)?.as_display(),
                                    field(act, key)?.as_display()
                                );
                                break;
                            }
                        }
                    }
                    _ => diff = format!("action {index}: oracle observation length differs"),
                }
                let matched = diff.is_empty();
                if !matched {
                    bad_cells += 1;
                }
                let cell = exp
                    .or(act)
                    .map(|r| text(r, "cell"))
                    .transpose()?
                    .unwrap_or("missing");
                checks.push(map(vec![
                    ("cell", t(cell)),
                    ("action", Value::Int(index as i64)),
                    ("step", Value::Int(n as i64)),
                    ("match", Value::Bool(matched)),
                    ("compared", Value::Bool(true)),
                    ("diff", t(&diff)),
                ]));
            }
            let mut diff = String::new();
            for (key, a, b) in [
                (
                    "controls",
                    Value::List(actual.controls.iter().map(|e| e.value()).collect()),
                    Value::List(expected.controls.iter().map(|e| e.value()).collect()),
                ),
                ("outcome", actual.outcome.clone(), expected.outcome.clone()),
                (
                    "protocol",
                    Value::List(actual.protocol.clone()),
                    Value::List(expected.protocol.clone()),
                ),
                (
                    "observations",
                    Value::List(actual.observations.clone()),
                    Value::List(expected.observations.clone()),
                ),
                ("state", actual.state.clone(), expected.state.clone()),
            ] {
                if !same(&a, &b)? {
                    diff = format!(
                        "action {index} {key}: expected {}, got {}",
                        b.as_display(),
                        a.as_display()
                    );
                    break;
                }
            }
            if let Some(error) = guarded_error {
                diff = error;
            }
            let matched = diff.is_empty() && bad_cells == 0;
            action_checks.push(map(vec![
                ("action", Value::Int(index as i64)),
                ("kind", t(&expected.kind)),
                ("match", Value::Bool(matched)),
                ("diff", t(&diff)),
            ]));
            if bad_cells > 0 {
                mismatched += bad_cells;
                matched_observations += (actual.cells.len() as i64 - bad_cells).max(0);
            } else if !matched {
                mismatched += 1;
                checks.push(map(vec![
                    ("cell", t(&expected.kind)),
                    ("action", Value::Int(index as i64)),
                    ("step", Value::Int(index as i64)),
                    ("match", Value::Bool(false)),
                    ("compared", Value::Bool(true)),
                    ("diff", t(&diff)),
                ]));
            } else {
                matched_observations += expected.observations.len() as i64;
            }
            // A mismatched prefix cannot justify executing its dependent suffix.
            if !matched {
                break;
            }
        }
        let executed = action_checks.len();
        Ok(map(vec![
            ("ok", Value::Bool(mismatched == 0)),
            ("matched", Value::Int(matched_observations)),
            (
                "matched_actions",
                Value::Int(executed as i64 - if mismatched > 0 { 1 } else { 0 }),
            ),
            ("executed_actions", Value::Int(executed as i64)),
            (
                "remaining_actions",
                Value::Int((entries.len() - executed) as i64),
            ),
            ("mismatched", Value::Int(mismatched)),
            ("skipped", Value::Int(skipped)),
            ("checks", Value::List(checks)),
            ("action_checks", Value::List(action_checks)),
        ]))
    }
}

#[cfg(test)]
pub(super) fn sync_expected_cells(snapshot: &mut Value) -> Result<(), String> {
    let originals = list(snapshot, "oracle_journal")?.to_vec();
    let mut entries = list(snapshot, "actions")?
        .iter()
        .enumerate()
        .map(|(i, v)| HistoryEntry::parse(v, i))
        .collect::<Result<Vec<_>, _>>()?;
    for entry in &mut entries {
        for cell in &mut entry.cells {
            if let Some(updated) = originals
                .iter()
                .find(|r| field(r, "seq").ok() == field(cell, "seq").ok())
            {
                let old_value = field(cell, "value")?.clone();
                let new_value = field(updated, "value")?.clone();
                let old_stdout = field(cell, "stdout")?.clone();
                let new_stdout = field(updated, "stdout")?.clone();
                if !same(&old_value, &new_value)? {
                    for obs in &mut entry.observations {
                        if field(obs, "channel")? == &t("eval")
                            && same(field(obs, "value")?, &old_value)?
                        {
                            set_field(obs, "value", new_value.clone())?;
                        }
                    }
                }
                fn update_projection(
                    v: &mut Value,
                    old_value: &Value,
                    new_value: &Value,
                    old_stdout: &Value,
                    new_stdout: &Value,
                ) -> Result<(), String> {
                    if let Value::Map(fields) = v {
                        let has_eval = fields.iter().any(|(k, _)| k == "stdout")
                            && fields.iter().any(|(k, _)| k == "code");
                        if has_eval {
                            for (k, value) in fields.iter_mut() {
                                if k == "value" && same(value, old_value)? {
                                    *value = new_value.clone();
                                }
                                if k == "stdout" && same(value, old_stdout)? {
                                    *value = new_stdout.clone();
                                }
                            }
                        }
                        for (_, value) in fields {
                            update_projection(value, old_value, new_value, old_stdout, new_stdout)?;
                        }
                    } else if let Value::List(values) = v {
                        for value in values {
                            update_projection(value, old_value, new_value, old_stdout, new_stdout)?;
                        }
                    }
                    Ok(())
                }
                update_projection(
                    &mut entry.outcome,
                    &old_value,
                    &new_value,
                    &old_stdout,
                    &new_stdout,
                )?;
                *cell = updated.clone();
            }
        }
    }
    let observations = entries
        .iter()
        .flat_map(|e| e.observations.clone())
        .collect();
    let actions = entries.iter().enumerate().map(|(i, e)| e.wire(i)).collect();
    set_field(snapshot, "actions", Value::List(actions))?;
    set_field(snapshot, "observation_journal", Value::List(observations))?;
    Ok(())
}
#[cfg(test)]
fn set_field(record: &mut Value, key: &str, value: Value) -> Result<(), String> {
    let Value::Map(fields) = record else {
        return Err("test: expected map".into());
    };
    fields
        .iter_mut()
        .find(|(k, _)| k == key)
        .ok_or("test: missing field")?
        .1 = value;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aggregate_replay_checks_each_unit_before_dependent_side_effects() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for language in ["python", "node"] {
            for persistent in [false, true] {
                let nonce = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let path = root.join(format!(
                    "target/batch-boundary-{}-{nonce}.mq.md",
                    std::process::id()
                ));
                let flag = path.with_extension("input");
                let started = path.with_extension("started");
                let suffix = path.with_extension("suffix");
                let literal = |p: &Path| serde_json::to_string(&p.display().to_string()).unwrap();
                let (first, next) = if language == "python" {
                    (format!("from pathlib import Path\nPath({}).write_text('first')\nprint(1 if Path({}).exists() else 2)", literal(&started), literal(&flag)),
                     format!("open({}, 'w').write('suffix')\nprint(7)", literal(&suffix)))
                } else {
                    (format!("require('fs').writeFileSync({}, 'first'); console.log(require('fs').existsSync({}) ? 1 : 2);", literal(&started), literal(&flag)),
                     format!("require('fs').writeFileSync({}, 'suffix'); console.log(7);", literal(&suffix)))
                };
                let header = if persistent {
                    "---\nimport:\n  foreign: lib/foreign.mq.md\n---\n# main\n\n`会话` = > foreign.session\n"
                } else {
                    "# main\n"
                };
                std::fs::write(&path, format!("{header}\n`first` =\n```{language}\n{first}\n```\n\n`next` =\n```{language}\n{next}\n```\n")).unwrap();
                std::fs::write(&flag, "present").unwrap();
                let mut recording = DocumentSession::open(&path, &root).unwrap();
                recording.run_all(false).unwrap();
                let snapshot = recording.export_snapshot().unwrap();
                recording.close();
                let good =
                    DocumentSession::check_snapshot_isolated(&path, &root, &snapshot).unwrap();
                assert_eq!(
                    field(&good, "ok").unwrap(),
                    &Value::Bool(true),
                    "{language} persistent={persistent}: {}",
                    good.as_display()
                );
                std::fs::remove_file(&flag).unwrap();
                for compact in [false, true] {
                    let _ = std::fs::remove_file(&started);
                    let _ = std::fs::remove_file(&suffix);
                    let input = if compact {
                        super::super::packing::pack(&snapshot).unwrap()
                    } else {
                        snapshot.clone()
                    };
                    let checked =
                        DocumentSession::check_snapshot_isolated(&path, &root, &input).unwrap();
                    assert_eq!(field(&checked, "ok").unwrap(), &Value::Bool(false));
                    assert!(
                        started.exists(),
                        "first primitive must execute to observe the changed response"
                    );
                    assert!(
                        !suffix.exists(),
                        "{language} persistent={persistent}: mismatching batch executed its suffix"
                    );
                }
                if persistent {
                    let _ = std::fs::remove_file(&started);
                    let mut wrong_request = snapshot.clone();
                    let Value::Map(fields) = &mut wrong_request else {
                        panic!()
                    };
                    let (_, Value::List(actions)) =
                        fields.iter_mut().find(|(k, _)| k == "actions").unwrap()
                    else {
                        panic!()
                    };
                    let mut entry = HistoryEntry::parse(&actions[0], 0).unwrap();
                    let Value::Map(step) = &mut entry.protocol[0] else {
                        panic!()
                    };
                    let (_, Value::Text(json)) =
                        step.iter_mut().find(|(k, _)| k == "request").unwrap()
                    else {
                        panic!()
                    };
                    let mut request: serde_json::Value = serde_json::from_str(json).unwrap();
                    request["code"] = "different code".into();
                    *json = request.to_string();
                    actions[0] = entry.wire(0);
                    let checked =
                        DocumentSession::check_snapshot_isolated(&path, &root, &wrong_request)
                            .unwrap();
                    assert_eq!(field(&checked, "ok").unwrap(), &Value::Bool(false));
                    assert!(
                        !started.exists(),
                        "request divergence was detected after dispatch"
                    );
                    assert!(!suffix.exists());
                }
                for file in [path, started] {
                    let _ = std::fs::remove_file(file);
                }
                println!("E12 {language} persistent={persistent}: matching replay passes; changed response stops inside batch; compact replay has same boundary");
            }
        }
    }
    fn fixture() -> (PathBuf, PathBuf, String) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join(format!(
            "target/history-{}-{}.mq.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = r#"---
import:
  foreign: lib/foreign.mq.md
---
# main

`kernels` =
| 名称 | 语言 | 命令 |
|---|---|---|
| py | python | |

`units` =
| 单元 | 内核 |
|---|---|
| inc | py |
| bad | py |
| read | py |

`initial` =
| 内核 | 名称 | 值 |
|---|---|---|
| py | n | 1 |

`会话` = > foreign.session kernels=`kernels` cells=`units` values=`initial`

`inc` =
```python
n += 1
n
```

`bad` =
```python
n = 99
2**53
```

`read` =
```python
n
```
"#
        .to_string();
        std::fs::write(&path, &source).unwrap();
        (root, path, source)
    }
    fn assert_replay(b: &mut DocumentSession, snap: &Value) {
        let result = b.replay_check(snap).unwrap();
        assert_eq!(
            field(&result, "ok").unwrap(),
            &Value::Bool(true),
            "{}",
            result.as_display()
        );
        assert_eq!(field(&result, "remaining_actions").unwrap(), &Value::Int(0));
    }
    #[test]
    fn history_replays_inputs_fault_recovery_restart_edits_and_clear() {
        let (root, path, source) = fixture();
        let mut a = DocumentSession::open(&path, &root).unwrap();
        assert_eq!(a.pull_value("py", "n").unwrap(), Value::Int(1));
        a.set_value("py", "n", Value::Int(4)).unwrap();
        a.run_cell("inc", None).unwrap();
        assert!(a
            .run_cell("bad", None)
            .unwrap_err()
            .contains("outside V_safe"));
        assert_eq!(a.pull_value("py", "n").unwrap(), Value::Int(99));
        a.restart_kernel("py").unwrap();
        assert_eq!(a.pull_value("py", "n").unwrap(), Value::Int(1));
        // Invalid outbound requests must be represented without making them legal wire payloads.
        assert!(a.set_value("py", "n", Value::Int(i64::MAX)).is_err());
        assert!(a
            .set_value("py", "n", Value::Handle("must-not-escape".into()))
            .is_err());
        assert!(a
            .set_value("py", "n", Value::Secret("must-not-be-recorded".into()))
            .is_err());
        a.set_value("py", "n", Value::Int(8)).unwrap();
        a.step().unwrap();
        let result = a.run_all(false).unwrap();
        assert_eq!(field(&result, "failed").unwrap(), &Value::Int(1));
        a.clear_outputs();
        assert_eq!(a.pull_value("py", "n").unwrap(), Value::Int(99));
        a.add_cell("extra", "python", "n += 2\nn", "py").unwrap();
        a.run_cell("extra", None).unwrap();
        a.clear_cell("extra").unwrap();
        a.remove_cell("extra").unwrap();
        a.state_list().unwrap();
        a.interrupt().unwrap();
        assert_eq!(
            a.pull_value("py", "n").unwrap(),
            Value::Int(101),
            "idle interrupt preserves kernel state"
        );
        let snap = a.export_snapshot().unwrap();
        let wire = super::super::super::json::stringify(&snap, None).unwrap();
        let Value::Text(ref serialized) = wire else {
            panic!()
        };
        assert!(!serialized.contains("must-not-escape"));
        assert!(!serialized.contains("must-not-be-recorded"));
        let snap = super::super::super::json::parse(&wire).unwrap();
        a.close();
        let mut b = DocumentSession::open(&path, &root).unwrap();
        assert_replay(&mut b, &snap);
        assert_eq!(b.pull_value("py", "n").unwrap(), Value::Int(101));
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            source,
            "replay must not rewrite source"
        );
        b.close();
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn history_replays_reload_versions_without_rewriting_the_document() {
        let (root, path, source) = fixture();
        let mut a = DocumentSession::open(&path, &root).unwrap();
        a.set_value("py", "n", Value::Int(5)).unwrap();
        a.run_cell("inc", None).unwrap();
        let second = source
            .replace("n += 1\nn", "n += 10\nn")
            .replace("| py | n | 1 |", "| py | n | 2 |");
        std::fs::write(&path, &second).unwrap();
        a.reload(&root).unwrap();
        a.run_cell("inc", None).unwrap();
        assert_eq!(a.pull_value("py", "n").unwrap(), Value::Int(16));
        a.restart_all().unwrap();
        assert_eq!(a.pull_value("py", "n").unwrap(), Value::Int(2));
        let third = second
            .replace("| py | python | |", "| py | python | \"python3 -u\" |")
            .replace("| py | n | 2 |", "| py | n | 3 |");
        std::fs::write(&path, &third).unwrap();
        a.reload(&root).unwrap();
        assert_eq!(
            a.pull_value("py", "n").unwrap(),
            Value::Int(3),
            "changed kernel command must retire the old process"
        );
        assert_eq!(a.host.entry_source.as_ref().unwrap(), &third);
        let snap = a.export_snapshot().unwrap();
        a.close();
        let mut b = DocumentSession::open(&path, &root).unwrap();
        assert_replay(&mut b, &snap);
        assert_eq!(b.host.entry_source.as_ref().unwrap(), &third);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), third);
        b.close();
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn malformed_action_suffix_is_rejected_before_any_kernel_request() {
        let (root, path, _) = fixture();
        let mut a = DocumentSession::open(&path, &root).unwrap();
        a.run_cell("inc", None).unwrap();
        a.pull_value("py", "n").unwrap();
        let original = a.export_snapshot().unwrap();
        a.close();
        for kind in [
            "unknown",
            "count",
            "index",
            "args",
            "image",
            "projection",
            "truncated",
            "state",
            "outcome",
        ] {
            let mut snap = original.clone();
            match kind {
                "count" => set_field(&mut snap, "action_count", Value::Int(0)).unwrap(),
                "image" => {
                    let Value::Map(fields) = &mut snap else {
                        panic!()
                    };
                    let img = &mut fields
                        .iter_mut()
                        .find(|(k, _)| k == "initial_image")
                        .unwrap()
                        .1;
                    set_field(img, "source_hash", t("wrong")).unwrap();
                }
                "projection" => {
                    set_field(&mut snap, "observation_journal", Value::List(vec![])).unwrap()
                }
                _ => {
                    let Value::Map(fields) = &mut snap else {
                        panic!()
                    };
                    let (_, Value::List(actions)) =
                        fields.iter_mut().find(|(k, _)| k == "actions").unwrap()
                    else {
                        panic!()
                    };
                    if kind == "truncated" {
                        actions.pop();
                    } else {
                        let last = actions.last_mut().unwrap();
                        match kind {
                            "unknown" => set_field(last, "kind", t("not-an-operation")).unwrap(),
                            "index" => set_field(last, "index", Value::Int(0)).unwrap(),
                            "args" => set_field(last, "args", encode(&Value::Int(1))).unwrap(),
                            "state" => set_field(last, "state", encode(&Value::None)).unwrap(),
                            "outcome" => {
                                set_field(last, "outcome", encode(&Value::Handle("bad".into())))
                                    .unwrap()
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
            let mut b = DocumentSession::open(&path, &root).unwrap();
            let requests = b.host.foreign_protocol_steps.len();
            assert!(
                b.replay_check(&snap).is_err(),
                "{kind}: malformed history accepted"
            );
            assert_eq!(
                b.host.foreign_protocol_steps.len(),
                requests,
                "{kind}: executed before suffix validation"
            );
            assert!(b.action_history.is_empty());
            b.close();
        }
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn generated_history_matches_independent_state_and_survives_json_roundtrip() {
        // The experiment driver supplies larger budgets; ordinary CI retains the short corpus.
        let steps: usize = std::env::var("MARQDO_HISTORY_STEPS")
            .map(|s| s.parse().expect("positive history length"))
            .unwrap_or(32);
        assert!(steps > 0);
        let seeds: Vec<u64> = std::env::var("MARQDO_HISTORY_SEEDS")
            .map(|s| s.split(',').map(|s| s.parse().expect("u64 seed")).collect())
            .unwrap_or_else(|_| vec![3, 11, 29, 71]);
        assert!(!seeds.is_empty());
        for lang in ["python", "node"] {
            for &seed in &seeds {
                let (root, path, source) = fixture();
                let source = source.replace("python", lang);
                std::fs::write(&path, &source).unwrap();
                let mut a = DocumentSession::open(&path, &root).unwrap();
                let mut expected = 1i64;
                let mut cursor = 0usize;
                let mut rng = seed;
                let mut operations = [0usize; 13];
                let record_start = std::time::Instant::now();
                for step in 0..steps {
                    rng = rng
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let operation = ((rng >> 32) % 13) as usize;
                    operations[operation] += 1;
                    match operation {
                        0 => {
                            expected = ((rng >> 40) % 1000) as i64;
                            a.set_value("py", "n", Value::Int(expected)).unwrap();
                        }
                        1 => {
                            a.run_cell("inc", None).unwrap();
                            expected += 1;
                        }
                        2 => {
                            assert!(a.run_cell("bad", None).is_err());
                            expected = 99;
                        }
                        3 => {
                            a.restart_kernel("py").unwrap();
                            expected = 1;
                        }
                        4 => {
                            assert_eq!(a.pull_value("py", "n").unwrap(), Value::Int(expected));
                        }
                        5 => {
                            a.clear_outputs();
                            cursor = 0;
                        }
                        6 => {
                            assert!(a.set_value("py", "n", Value::Int(i64::MAX)).is_err());
                        }
                        7 => {
                            a.run_all(false).unwrap();
                            expected = 99;
                            cursor = 3;
                        }
                        8 => {
                            let result = a.step();
                            match cursor {
                                0 => {
                                    result.unwrap();
                                    expected += 1;
                                    cursor = 1;
                                }
                                1 => {
                                    assert!(result.is_err());
                                    expected = 99;
                                }
                                2 => {
                                    result.unwrap();
                                    cursor = 3;
                                }
                                _ => {
                                    result.unwrap();
                                }
                            }
                        }
                        9 => {
                            a.restart_all().unwrap();
                            expected = 1;
                            cursor = 0;
                        }
                        10 => {
                            a.interrupt().unwrap();
                        }
                        11 => {
                            a.state_list().unwrap();
                        }
                        _ => {
                            assert!(a.run_cell("missing", None).is_err());
                        }
                    }
                    assert_eq!(
                        a.pull_value("py", "n").unwrap(),
                        Value::Int(expected),
                        "{lang} seed {seed} step {step}"
                    );
                    assert_eq!(a.cursor, cursor, "{lang} seed {seed} step {step} cursor");
                }
                let record_ns = record_start.elapsed().as_nanos();
                let snapshot_start = std::time::Instant::now();
                let snap = a.export_snapshot().unwrap();
                let snapshot_ns = snapshot_start.elapsed().as_nanos();
                let action_count = a.action_history.len();
                let protocol_count = a.host.foreign_protocol_steps.len();
                let observation_count = a.host.oracle_observations.len();
                let encode_start = std::time::Instant::now();
                let wire = super::super::super::json::stringify(&snap, None).unwrap();
                let encode_ns = encode_start.elapsed().as_nanos();
                let decode_start = std::time::Instant::now();
                let decoded_raw = super::super::super::json::parse(&wire).unwrap();
                let decode_ns = decode_start.elapsed().as_nanos();
                drop(decoded_raw);
                let pack_start = std::time::Instant::now();
                let packed = super::super::packing::pack(&snap).unwrap();
                let pack_ns = pack_start.elapsed().as_nanos();
                let compact_nodes = list(&packed, "nodes").unwrap().len();
                let compact_encode_start = std::time::Instant::now();
                let compact_wire = super::super::super::json::stringify(&packed, None).unwrap();
                let compact_encode_ns = compact_encode_start.elapsed().as_nanos();
                let compact_decode_start = std::time::Instant::now();
                let packed = super::super::super::json::parse(&compact_wire).unwrap();
                let compact_decode_ns = compact_decode_start.elapsed().as_nanos();
                let unpack_start = std::time::Instant::now();
                let unpacked = super::super::packing::unpack(&packed).unwrap();
                let unpack_ns = unpack_start.elapsed().as_nanos();
                assert_eq!(
                    unpacked, snap,
                    "lossless packing changed the complete history"
                );
                a.close();
                let mut b = DocumentSession::open(&path, &root).unwrap();
                let replay_start = std::time::Instant::now();
                assert_replay(&mut b, &packed);
                let replay_ns = replay_start.elapsed().as_nanos();
                assert_eq!(b.pull_value("py", "n").unwrap(), Value::Int(expected));
                assert_eq!(b.cursor, cursor);
                b.close();
                std::fs::remove_file(path).unwrap();
                if let Ok(directory) = std::env::var("MARQDO_HISTORY_SNAPSHOT_DIR") {
                    let directory = PathBuf::from(directory);
                    std::fs::create_dir_all(&directory).unwrap();
                    let Value::Text(raw) = &wire else {
                        panic!("snapshot text")
                    };
                    let Value::Text(compact) = &compact_wire else {
                        panic!("snapshot text")
                    };
                    std::fs::write(
                        directory.join(format!("raw-{lang}-{seed}-{steps}.json")),
                        raw,
                    )
                    .unwrap();
                    std::fs::write(
                        directory.join(format!("compact-{lang}-{seed}-{steps}.json")),
                        compact,
                    )
                    .unwrap();
                }
                println!(
                    "HISTORY_METRIC:{}",
                    serde_json::json!({
                        "language": lang, "seed": seed, "steps": steps,
                        "operations": operations, "actions": action_count,
                        "protocol_steps": protocol_count, "observations": observation_count,
                        "snapshot_bytes": match &wire { Value::Text(text) => text.len(), _ => panic!("stringify must return text") }, "record_ns": record_ns,
                        "snapshot_ns": snapshot_ns, "encode_ns": encode_ns,
                        "compact_snapshot_bytes": match &compact_wire { Value::Text(text) => text.len(), _ => panic!("stringify must return text") },
                        "compact_nodes": compact_nodes, "pack_ns": pack_ns, "unpack_ns": unpack_ns,
                        "compact_encode_ns": compact_encode_ns, "compact_decode_ns": compact_decode_ns,
                        "decode_ns": decode_ns, "replay_ns": replay_ns,
                        "expected_final_value": expected, "expected_cursor": cursor,
                        "correct": true
                    })
                );
            }
        }
    }

    #[test]
    fn mismatch_stops_before_dependent_external_effect() {
        let (root, path, _) = fixture();
        let marker = path.with_extension("marker");
        let literal = serde_json::to_string(&marker.display().to_string()).unwrap();
        let mut a = DocumentSession::open(&path, &root).unwrap();
        a.pull_value("py", "n").unwrap();
        a.add_cell(
            "touch",
            "python",
            &format!("with open({literal}, 'w') as out:\n out.write('effect')\nn"),
            "py",
        )
        .unwrap();
        a.run_cell("touch", None).unwrap();
        assert!(marker.exists());
        let mut snap = a.export_snapshot().unwrap();
        a.close();
        std::fs::remove_file(&marker).unwrap();
        let mut entries = list(&snap, "actions")
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, v)| HistoryEntry::parse(v, i).unwrap())
            .collect::<Vec<_>>();
        set_field(&mut entries[0].outcome, "value", Value::Int(2)).unwrap();
        set_field(&mut entries[0].observations[0], "value", Value::Int(2)).unwrap();
        for entry in &mut entries {
            let Value::Map(fields) = &mut entry.state else {
                panic!()
            };
            let (_, Value::List(bindings)) =
                fields.iter_mut().find(|(k, _)| k == "values").unwrap()
            else {
                panic!()
            };
            for binding in bindings {
                if text(binding, "name").unwrap() == "n" {
                    set_field(binding, "value", Value::Int(2)).unwrap();
                }
            }
        }

        set_field(
            &mut snap,
            "actions",
            Value::List(entries.iter().enumerate().map(|(i, e)| e.wire(i)).collect()),
        )
        .unwrap();
        set_field(
            &mut snap,
            "observation_journal",
            Value::List(
                entries
                    .iter()
                    .flat_map(|e| e.observations.clone())
                    .collect(),
            ),
        )
        .unwrap();
        let mut b = DocumentSession::open(&path, &root).unwrap();
        let check = b.replay_check(&snap).unwrap();
        assert_eq!(field(&check, "ok").unwrap(), &Value::Bool(false));
        assert_eq!(field(&check, "executed_actions").unwrap(), &Value::Int(1));
        assert!(
            !marker.exists(),
            "dependent code ran after a mismatched prefix"
        );
        b.close();
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn separate_array_bindings_are_not_a_ragged_metadata_table() {
        let (root, path, _) = fixture();
        let mut a = DocumentSession::open(&path, &root).unwrap();
        a.push_value("py", "left", Value::List(vec![Value::Int(1)]))
            .unwrap();
        a.push_value(
            "py",
            "right",
            Value::List(vec![Value::Int(2), Value::Int(3)]),
        )
        .unwrap();
        a.state_list().unwrap();
        let snap = a.export_snapshot().unwrap();
        a.close();
        let mut b = DocumentSession::open(&path, &root).unwrap();
        assert_replay(&mut b, &snap);
        b.close();
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn invalid_import_payload_is_checked_before_any_binding_write() {
        let (root, path, _) = fixture();
        let mut a = DocumentSession::open(&path, &root).unwrap();
        let mut invalid = a.export_snapshot().unwrap();
        let Value::Map(fields) = &mut invalid else {
            panic!()
        };
        let (_, Value::List(kernels)) = fields.iter_mut().find(|(k, _)| k == "kernels").unwrap()
        else {
            panic!()
        };
        set_field(
            &mut kernels[0],
            "values",
            Value::List(vec![
                map(vec![("name", t("n")), ("value", Value::Int(999))]),
                map(vec![
                    ("name", t("bad")),
                    (
                        "value",
                        Value::Code(crate::value::CodeBlock {
                            lang: "python".into(),
                            source: "1".into(),
                        }),
                    ),
                ]),
            ]),
        )
        .unwrap();
        let requests = a.host.foreign_protocol_steps.len();
        assert!(a.import_snapshot(&invalid).is_err());
        assert_eq!(
            a.host.foreign_protocol_steps.len(),
            requests,
            "invalid imported code caused a partial foreign write"
        );
        assert_eq!(a.pull_value("py", "n").unwrap(), Value::Int(1));
        let snap = a.export_snapshot().unwrap();
        a.close();
        let mut b = DocumentSession::open(&path, &root).unwrap();
        assert_replay(&mut b, &snap);
        b.close();
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn failed_reload_input_is_an_explicit_historical_input() {
        let (root, path, source) = fixture();
        let mut a = DocumentSession::open(&path, &root).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert!(a.reload(&root).is_err());
        std::fs::write(&path, &source).unwrap();
        a.run_cell("inc", None).unwrap();
        let snap = a.export_snapshot().unwrap();
        a.close();
        let mut b = DocumentSession::open(&path, &root).unwrap();
        assert_replay(&mut b, &snap);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
        b.close();
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn failed_introspection_replies_are_not_silently_ignored() {
        let (root, path, _) = fixture();
        let mut a = DocumentSession::open(&path, &root).unwrap();
        a.add_cell(
            "function",
            "python",
            "def unsupported(): return 1\nunsupported",
            "py",
        )
        .unwrap();
        assert!(a.run_cell("function", None).is_err());
        let mut snap = a.export_snapshot().unwrap();
        a.close();
        let mut entries = list(&snap, "actions")
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, row)| HistoryEntry::parse(row, i).unwrap())
            .collect::<Vec<_>>();
        let inspect = entries.last_mut().unwrap();
        let mut changed = false;
        for step in &mut inspect.protocol {
            let response = field(step, "response").unwrap();
            if boolean(response, "ok").unwrap() {
                let mut raw: serde_json::Value =
                    serde_json::from_str(text(response, "json").unwrap()).unwrap();
                if raw.get("domain_error").and_then(|v| v.as_bool()) == Some(true) {
                    raw["error"] =
                        serde_json::Value::String("not JSON-safe: changed diagnostic".into());
                    let Value::Map(fields) = step else { panic!() };
                    let (_, response) = fields.iter_mut().find(|(k, _)| k == "response").unwrap();
                    set_field(response, "json", t(&raw.to_string())).unwrap();
                    changed = true;
                    break;
                }
            }
        }
        assert!(
            changed,
            "fixture must contain a refused introspection response"
        );
        set_field(
            &mut snap,
            "actions",
            Value::List(entries.iter().enumerate().map(|(i, e)| e.wire(i)).collect()),
        )
        .unwrap();
        let mut b = DocumentSession::open(&path, &root).unwrap();
        let check = b.replay_check(&snap).unwrap();
        assert_eq!(field(&check, "ok").unwrap(), &Value::Bool(false));
        assert!(field(&check, "action_checks")
            .unwrap()
            .as_display()
            .contains("protocol"));
        b.close();
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn repeated_process_death_history_has_stable_failure_observations() {
        for lang in ["python", "node"] {
            for _ in 0..4 {
                let (root, path, source) = fixture();
                let source = source.replace("python", lang).replace(
                    "n = 99\n2**53",
                    if lang == "python" {
                        "import os\nos._exit(7)"
                    } else {
                        "process.exit(7)"
                    },
                );
                std::fs::write(&path, &source).unwrap();
                let mut a = DocumentSession::open(&path, &root).unwrap();
                let record = a.run_cell("bad", None).unwrap();
                assert_eq!(field(&record, "ok").unwrap(), &Value::Bool(false));
                assert_eq!(a.pull_value("py", "n").unwrap(), Value::Int(1));
                let snap = a.export_snapshot().unwrap();
                a.close();
                let mut b = DocumentSession::open(&path, &root).unwrap();
                assert_replay(&mut b, &snap);
                b.close();
                std::fs::remove_file(path).unwrap();
            }
        }
    }
}

impl DocumentSession {
    /// Validate and compare a recording without touching an existing live view.
    /// No declarations or seed kernels run before the history preflight.
    pub(crate) fn check_snapshot_isolated(
        path: &Path,
        root: &Path,
        snapshot: &Value,
    ) -> Result<Value, String> {
        let source =
            std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let mut host = HostContext::default();
        host.cwd = path.parent().unwrap_or_else(|| Path::new(".")).into();
        host.fs_root = Some(root.into());
        host.entry_path = Some(path.into());
        host.entry_source = Some(source.clone());
        let mut target = Self {
            control: super::DocumentControl::default(),
            replay_controls: Vec::new(),
            replay_boundary: None,
            source_hash: hash_source(&source),
            initial_values: Vec::new(),
            initial_image: Value::None,
            action_history: Vec::new(),
            action_depth: 0,
            replay_fresh: true,
            path: path.into(),
            root: root.into(),
            persistent: false,
            cells: Vec::new(),
            kernels: Vec::new(),
            order: Vec::new(),
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
        target.replay_check(snapshot)
    }
}
