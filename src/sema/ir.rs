//! Versioned semantic artifacts for executable knowledge.
//!
//! This is deliberately a *projection* of the existing syntax AST.  It gives
//! every consumer one deterministic place to obtain identity, contracts,
//! effects, provenance and hashes without changing the classic runtime AST.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{json, Value};
use sha2::{Digest as _, Sha256};

use crate::ast::{Arg, CallExpr, Expr, Function, Stmt};
use crate::contract::Ty;
use crate::diagnostics::{Diagnostic, Severity, Span};
use crate::project::ProjectRoot;

use super::{CallableKind, ModuleId, ProjectIndex, SymbolDecl, SymbolId, SymbolKind};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtifactId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    Off,
    Check,
    Strict,
}

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub project_root: PathBuf,
    pub entry: Option<PathBuf>,
    pub lock_mode: LockMode,
}

impl CompileOptions {
    pub fn for_path(path: &Path) -> Result<Self> {
        let project = if path.is_dir() {
            ProjectRoot::new(path)?
        } else {
            ProjectRoot::discover(path)?
        };
        Ok(Self {
            project_root: project.path().to_path_buf(),
            entry: path.is_file().then(|| path.to_path_buf()),
            lock_mode: LockMode::Off,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectSet(BTreeSet<String>);

impl EffectSet {
    pub fn pure() -> Self {
        Self(BTreeSet::from(["pure".to_string()]))
    }

    pub fn add(&mut self, effect: impl Into<String>) {
        let effect = effect.into();
        if effect != "pure" {
            self.0.remove("pure");
        }
        self.0.insert(effect);
    }

    pub fn extend(&mut self, other: &Self) {
        for effect in &other.0 {
            self.add(effect.clone());
        }
    }

    pub fn is_pure(&self) -> bool {
        self.0.len() == 1 && self.0.contains("pure")
    }

    pub fn values(&self) -> Vec<String> {
        self.0.iter().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone)]
pub struct CanonicalParam {
    pub name: String,
    pub ty: String,
    pub required: bool,
    pub default_digest: Option<Digest>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CanonicalSignature {
    pub receiver: String,
    pub params: Vec<CanonicalParam>,
    pub returns: String,
    pub effects: EffectSet,
}

#[derive(Debug, Clone)]
pub struct ContractSet {
    pub params: Vec<String>,
    pub returns: Option<String>,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SourceRef {
    pub uri: String,
    pub locator: Option<String>,
    pub content_digest: Option<Digest>,
    pub relation: String,
}

#[derive(Debug, Clone)]
pub struct VerificationRecord {
    pub parse: String,
    pub resolve: String,
    pub contract: String,
    pub execution: String,
    pub tests: String,
    pub evidence: String,
    pub domain: String,
    pub methods: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArtifactHashes {
    pub interface: Digest,
    pub implementation: Digest,
    pub evidence: Digest,
    pub artifact: Digest,
}

#[derive(Debug, Clone)]
pub struct SemanticArtifact {
    pub artifact_id: ArtifactId,
    pub symbol_id: Option<SymbolId>,
    pub display_name: String,
    pub kind: String,
    pub callable: Option<String>,
    pub source: SourceLocation,
    pub signature: Option<CanonicalSignature>,
    pub contract: ContractSet,
    pub effects: EffectSet,
    pub dependencies: Vec<ArtifactId>,
    pub provenance: Vec<SourceRef>,
    pub applicability: String,
    pub hashes: ArtifactHashes,
    pub verification: VerificationRecord,
    pub lifecycle: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgeRef {
    pub id: String,
    pub label: String,
    pub context: String,
    pub source: SourceLocation,
    pub resolution: String,
    pub target: Option<SymbolId>,
}

#[derive(Debug, Clone)]
pub struct KnowledgeStep {
    pub id: ArtifactId,
    pub reference: String,
    pub kind: String,
    pub calls: Vec<SymbolId>,
    pub effects: EffectSet,
    pub source: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct SemanticProject {
    pub schema_version: u32,
    pub project_id: String,
    pub root: PathBuf,
    pub artifacts: Vec<SemanticArtifact>,
    pub refs: Vec<KnowledgeRef>,
    pub steps: Vec<KnowledgeStep>,
}

#[derive(Debug, Clone)]
pub struct Compilation {
    pub index: ProjectIndex,
    pub semantics: SemanticProject,
    pub diagnostics: Vec<Diagnostic>,
}

/// Compile every project source once into stable semantic facts.  This is
/// side-effect free: it never loads plugins, evaluates expressions or fetches
/// sources.
pub fn compile_project(options: &CompileOptions) -> Result<Compilation> {
    let root = ProjectRoot::new(&options.project_root)?;
    let index = ProjectIndex::build(root.clone())?;
    let mut files = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for file in root.source_files()? {
        let text = fs::read_to_string(&file)
            .with_context(|| format!("read semantic source {}", file.display()))?;
        match crate::load::parse_source_auto(&text) {
            Ok(module) => {
                let module_id = index
                    .modules()
                    .iter()
                    .find_map(|(id, path)| (path == &file).then(|| id.clone()));
                if let Some(module_id) = module_id {
                    files.insert(module_id, (file, text, module));
                }
            }
            Err(error) => {
                diagnostics.push(
                    Diagnostic::new(Some(&file), Span::new(1, 1), error.to_string())
                        .with_code("parse.semantic")
                        .with_severity(Severity::Error),
                );
            }
        }
    }

    let project_id = index.project_id().to_string();
    let mut artifacts = Vec::new();
    let mut refs = Vec::new();
    let mut steps = Vec::new();
    let mut local_by_name: BTreeMap<(ModuleId, String), SymbolId> = BTreeMap::new();
    for symbol in index.symbols() {
        local_by_name.insert((symbol.module.clone(), symbol.name.clone()), symbol.id.clone());
    }

    for (module_id, (path, text, module)) in &files {
        artifacts.push(module_artifact(&index, module_id, path, text));
        let mut by_path = BTreeMap::new();
        collect_functions(&module.functions, &mut Vec::new(), &mut by_path);
        for symbol in index.symbols().iter().filter(|s| &s.module == module_id) {
            let function = by_path.get(&symbol.declaration_path).expect("symbol collected from module");
            let (artifact, mut artifact_refs, mut artifact_steps) = artifact_for_function(
                &index,
                module_id,
                path,
                function,
                symbol,
                &local_by_name,
            );
            refs.append(&mut artifact_refs);
            steps.append(&mut artifact_steps);
            artifacts.push(artifact);
        }
    }
    artifacts.sort_by(|a, b| a.artifact_id.0.cmp(&b.artifact_id.0));
    refs.sort_by(|a, b| a.id.cmp(&b.id));
    steps.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    Ok(Compilation {
        index,
        semantics: SemanticProject {
            schema_version: SCHEMA_VERSION,
            project_id,
            root: root.path().to_path_buf(),
            artifacts,
            refs,
            steps,
        },
        diagnostics,
    })
}

fn module_artifact(index: &ProjectIndex, module: &ModuleId, path: &Path, text: &str) -> SemanticArtifact {
    let implementation = digest(normalize_source(text));
    let interface = digest(module.as_str());
    let evidence = digest("");
    let artifact_id = ArtifactId(format!("mq://{}/{}/module", percent(index.project_id()), percent(module.as_str())));
    semantic_artifact(
        artifact_id,
        None,
        module.as_str().to_string(),
        "module".to_string(),
        None,
        source_location(index.root().path(), path, 1, text.lines().count() as u32),
        None,
        ContractSet { params: vec![], returns: None, fields: vec![] },
        EffectSet::pure(),
        vec![],
        vec![SourceRef { uri: rel_uri(index.root().path(), path), locator: None, content_digest: Some(implementation.clone()), relation: "defines".into() }],
        "project-local module".into(),
        interface,
        implementation,
        evidence,
    )
}

fn artifact_for_function(
    index: &ProjectIndex,
    module: &ModuleId,
    path: &Path,
    function: &Function,
    symbol: &SymbolDecl,
    local_by_name: &BTreeMap<(ModuleId, String), SymbolId>,
) -> (SemanticArtifact, Vec<KnowledgeRef>, Vec<KnowledgeStep>) {
    let calls = collect_call_sites(function);
    let mut effects = direct_effects(&calls);
    let mut dependencies = Vec::new();
    let mut refs = Vec::new();
    let mut steps = Vec::new();
    for (n, (_, call, line)) in calls.iter().enumerate() {
        let target = resolve_local_call(module, call, local_by_name);
        if let Some(target) = &target {
            dependencies.push(ArtifactId(target.as_str().to_string()));
        }
        let ref_id = format!("{}#ref-{}", symbol.id.as_str(), n + 1);
        refs.push(KnowledgeRef {
            id: ref_id.clone(),
            label: call_name(call),
            context: "executable".into(),
            source: source_location(index.root().path(), path, *line, *line),
            resolution: target.as_ref().map(|_| "symbol".to_string()).unwrap_or_else(|| "external_or_dynamic".to_string()),
            target: target.clone(),
        });
        steps.push(KnowledgeStep {
            id: ArtifactId(format!("{}#step-{}", symbol.id.as_str(), n + 1)),
            reference: ref_id,
            kind: "call".into(),
            calls: target.into_iter().collect(),
            effects: effects_for_call(call),
            source: source_location(index.root().path(), path, *line, *line),
        });
    }
    if effects.0.is_empty() {
        effects = EffectSet::pure();
    }
    let (signature, contract) = signature(function, symbol.callable, &effects);
    let interface = digest(signature_json(signature.as_ref()));
    let implementation = digest(format!("{:?}", function.body));
    let provenance = vec![SourceRef {
        uri: rel_uri(index.root().path(), path),
        locator: Some(format!("L{}", function.span.line)),
        content_digest: Some(implementation.clone()),
        relation: "defines".into(),
    }];
    let evidence = digest(contract_summary(&contract));
    let kind = match symbol.kind {
        SymbolKind::Type => "type",
        SymbolKind::Function => "function",
    }.to_string();
    let artifact = semantic_artifact(
        ArtifactId(symbol.id.as_str().to_string()),
        Some(symbol.id.clone()),
        symbol.declaration_path.join("."),
        kind,
        symbol.callable.map(|kind| kind.as_str().to_string()),
        source_location(index.root().path(), path, function.span.line, function.span.line),
        signature,
        contract,
        effects,
        dependencies,
        provenance,
        "no explicit applicability constraints".into(),
        interface,
        implementation,
        evidence,
    );
    (artifact, refs, steps)
}

#[allow(clippy::too_many_arguments)]
fn semantic_artifact(
    artifact_id: ArtifactId, symbol_id: Option<SymbolId>, display_name: String, kind: String,
    callable: Option<String>, source: SourceLocation, signature: Option<CanonicalSignature>,
    contract: ContractSet, effects: EffectSet, mut dependencies: Vec<ArtifactId>,
    provenance: Vec<SourceRef>, applicability: String, interface: Digest, implementation: Digest,
    evidence: Digest,
) -> SemanticArtifact {
    dependencies.sort_by(|a, b| a.0.cmp(&b.0));
    dependencies.dedup();
    let artifact = digest(format!("{}:{}:{}", interface.0, implementation.0, evidence.0));
    SemanticArtifact {
        artifact_id, symbol_id, display_name, kind, callable, source, signature, contract,
        effects, dependencies, provenance, applicability, hashes: ArtifactHashes {
            interface, implementation, evidence, artifact,
        },
        verification: VerificationRecord {
            parse: "passed".into(), resolve: "passed".into(), contract: "unchecked".into(),
            execution: "not_run".into(), tests: "unknown".into(), evidence: "unverified".into(),
            domain: "unreviewed".into(), methods: vec!["static_project_compile".into()],
        },
        lifecycle: "draft".into(),
    }
}

fn signature(function: &Function, callable: Option<CallableKind>, effects: &EffectSet) -> (Option<CanonicalSignature>, ContractSet) {
    if function.is_object() {
        let fields = function.contract.as_ref().map(|c| c.fields.iter().map(|f| format!("{}:{}", f.name, ty_name(&f.ty))).collect()).unwrap_or_default();
        return (None, ContractSet { params: vec![], returns: None, fields });
    }
    let contract = function.contract.as_ref();
    let params = function.params.iter().map(|param| {
        let cp = contract.and_then(|c| c.params.iter().find(|item| item.name == param.name));
        CanonicalParam {
            name: param.name.clone(), ty: cp.map(|p| ty_name(&p.ty)).unwrap_or_else(|| "any".into()),
            required: param.default.is_none(), default_digest: param.default.as_ref().map(|value| digest(format!("{:?}", value))), unit: None,
        }
    }).collect::<Vec<_>>();
    let set = ContractSet {
        params: contract.map(|c| c.params.iter().map(|p| format!("{}:{}", p.name, ty_name(&p.ty))).collect()).unwrap_or_default(),
        returns: contract.and_then(|c| c.returns.as_ref().map(|r| ty_name(&r.ty))),
        fields: contract.map(|c| c.fields.iter().map(|f| format!("{}:{}", f.name, ty_name(&f.ty))).collect()).unwrap_or_default(),
    };
    let receiver = match callable {
        Some(CallableKind::InstanceMethod) => "instance",
        Some(CallableKind::AssociatedFunction) => "none",
        _ => "none",
    }.to_string();
    (Some(CanonicalSignature {
        receiver, params, returns: set.returns.clone().unwrap_or_else(|| "any".into()), effects: effects.clone(),
    }), set)
}

fn ty_name(ty: &Ty) -> String { ty.name() }

fn collect_functions<'a>(functions: &'a [Function], path: &mut Vec<String>, out: &mut BTreeMap<Vec<String>, &'a Function>) {
    for function in functions {
        path.push(function.name.clone());
        out.insert(path.clone(), function);
        collect_functions(&function.children, path, out);
        path.pop();
    }
}

fn collect_call_sites(function: &Function) -> Vec<(usize, CallExpr, u32)> {
    let mut calls = Vec::new();
    fn expr(value: &Expr, line: u32, calls: &mut Vec<(usize, CallExpr, u32)>) {
        match value {
            Expr::Call(call) => {
                calls.push((calls.len(), call.clone(), line));
                for arg in &call.args { match arg { Arg::Positional(value) | Arg::Named { value, .. } => expr(value, line, calls) } }
            }
            Expr::Unary { expr: value, .. } => expr(value, line, calls),
            Expr::Binary { left, right, .. } => { expr(left, line, calls); expr(right, line, calls); }
            Expr::List(items) => for item in items { expr(item, line, calls); },
            Expr::Map(items) => for (_, item) in items { expr(item, line, calls); },
            Expr::Index { base, .. } => expr(base, line, calls),
            Expr::Occluded { payload: Some(payload), .. } => expr(payload, line, calls),
            _ => {}
        }
    }
    fn statements(items: &[Stmt], calls: &mut Vec<(usize, CallExpr, u32)>) {
        for item in items {
            match item {
                Stmt::Call { call, span } => { calls.push((calls.len(), call.clone(), span.line)); for arg in &call.args { match arg { Arg::Positional(value) | Arg::Named { value, .. } => expr(value, span.line, calls) } } }
                Stmt::Assign { value, span, .. } | Stmt::Return { value, span } | Stmt::Expr { value, span } => expr(value, span.line, calls),
                Stmt::Branch { arms, .. } => for arm in arms { if let Some(condition) = &arm.condition { expr(condition, 0, calls) }; statements(&arm.body, calls); },
                Stmt::While { condition, body, .. } => { expr(condition, 0, calls); statements(body, calls); }
                Stmt::ForEach { body, .. } => statements(body, calls),
            }
        }
    }
    statements(&function.body, &mut calls);
    calls
}

fn call_name(call: &CallExpr) -> String {
    call.path.as_ref().map(|path| path.join(".")).or_else(|| call.receiver.as_ref().map(|receiver| format!("{receiver}.{}", call.callee))).unwrap_or_else(|| call.callee.clone())
}

fn resolve_local_call(module: &ModuleId, call: &CallExpr, symbols: &BTreeMap<(ModuleId, String), SymbolId>) -> Option<SymbolId> {
    if call.path.is_none() && call.receiver.is_none() { symbols.get(&(module.clone(), call.callee.clone())).cloned() } else { None }
}

fn effects_for_call(call: &CallExpr) -> EffectSet {
    let name = call_name(call).to_ascii_lowercase();
    let mut effects = EffectSet::default();
    let effect = if name.starts_with("fs.") || name.contains("writeback") { if name.contains("write") || name.contains("remove") || name.contains("move") || name.contains("mkdir") { "fs.write" } else { "fs.read" } }
    else if name.starts_with("http.") || name.starts_with("net.") { if name.contains("post") || name.contains("put") || name.contains("delete") { "network.write" } else { "network.read" } }
    else if name.starts_with("foreign.") || name.starts_with("subtask.") || name == "exec" { "process.exec" }
    else if name.starts_with("plugin.") { "plugin.call" }
    else if name.starts_with("llm.") || name.starts_with("openai.") { "llm.call" }
    else if name.starts_with("time.") || name.starts_with("datetime.") { "time.read" }
    else if name.starts_with("random.") || name.starts_with("uuid.") { "random.read" }
    else if name.starts_with("input") { "user.input" }
    else if name.starts_with("print") || name.starts_with("log.") { "output.write" }
    else { return EffectSet::pure() };
    effects.add(effect);
    effects
}

fn direct_effects(calls: &[(usize, CallExpr, u32)]) -> EffectSet {
    let mut effects = EffectSet::default();
    for (_, call, _) in calls { effects.extend(&effects_for_call(call)); }
    effects
}

fn source_location(root: &Path, path: &Path, line: u32, end_line: u32) -> SourceLocation {
    SourceLocation { file: rel_uri(root, path), line, end_line }
}

fn rel_uri(root: &Path, path: &Path) -> String { path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/") }
fn digest(value: impl AsRef<[u8]>) -> Digest { let mut hash = Sha256::new(); hash.update(value.as_ref()); Digest(format!("sha256:{:x}", hash.finalize())) }
fn normalize_source(source: &str) -> String { source.lines().map(str::trim).filter(|line| !line.is_empty()).collect::<Vec<_>>().join("\n") }
fn percent(value: &str) -> String { value.bytes().map(|byte| match byte { b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (byte as char).to_string(), _ => format!("%{byte:02X}") }).collect() }
fn contract_summary(contract: &ContractSet) -> String { format!("{}|{:?}|{}", contract.params.join(","), contract.returns, contract.fields.join(",")) }

fn signature_json(signature: Option<&CanonicalSignature>) -> String {
    match signature { Some(signature) => format!("{}|{}|{}|{}", signature.receiver, signature.params.iter().map(|p| format!("{}:{}:{}", p.name, p.ty, p.required)).collect::<Vec<_>>().join(","), signature.returns, signature.effects.values().join(",")), None => "type".into() }
}

impl SemanticProject {
    pub fn to_json(&self) -> Value {
        json!({
            "schema_version": self.schema_version, "project": {"id": self.project_id, "root": self.root},
            "artifacts": self.artifacts.iter().map(artifact_json).collect::<Vec<_>>(),
            "refs": self.refs.iter().map(ref_json).collect::<Vec<_>>(),
            "steps": self.steps.iter().map(step_json).collect::<Vec<_>>(),
        })
    }
}

pub fn artifact_json(artifact: &SemanticArtifact) -> Value {
    json!({"schema_version": SCHEMA_VERSION, "artifact_id": artifact.artifact_id.0, "symbol_id": artifact.symbol_id.as_ref().map(SymbolId::as_str), "display_name": artifact.display_name, "kind": artifact.kind, "callable": artifact.callable, "source": {"file": artifact.source.file, "line": artifact.source.line, "end_line": artifact.source.end_line}, "signature": artifact.signature.as_ref().map(|s| json!({"receiver":s.receiver,"params":s.params.iter().map(|p|json!({"name":p.name,"type":p.ty,"required":p.required,"default_digest":p.default_digest.as_ref().map(|d|&d.0),"unit":p.unit})).collect::<Vec<_>>(),"returns":s.returns,"effects":s.effects.values()})), "contract":{"params":artifact.contract.params,"returns":artifact.contract.returns,"fields":artifact.contract.fields}, "effects":artifact.effects.values(), "dependencies":artifact.dependencies.iter().map(|d|&d.0).collect::<Vec<_>>(), "provenance":artifact.provenance.iter().map(|p|json!({"uri":p.uri,"locator":p.locator,"content_digest":p.content_digest.as_ref().map(|d|&d.0),"relation":p.relation})).collect::<Vec<_>>(), "applicability":artifact.applicability, "hashes":{"interface":artifact.hashes.interface.0,"implementation":artifact.hashes.implementation.0,"evidence":artifact.hashes.evidence.0,"artifact":artifact.hashes.artifact.0}, "verification":{"parse":artifact.verification.parse,"resolve":artifact.verification.resolve,"contract":artifact.verification.contract,"execution":artifact.verification.execution,"tests":artifact.verification.tests,"evidence":artifact.verification.evidence,"domain":artifact.verification.domain,"methods":artifact.verification.methods}, "lifecycle":artifact.lifecycle})
}
fn ref_json(reference: &KnowledgeRef) -> Value { json!({"id":reference.id,"label":reference.label,"context":reference.context,"source":{"file":reference.source.file,"line":reference.source.line,"end_line":reference.source.end_line},"resolution":reference.resolution,"target":reference.target.as_ref().map(SymbolId::as_str)}) }
fn step_json(step: &KnowledgeStep) -> Value { json!({"id":step.id.0,"reference":step.reference,"kind":step.kind,"calls":step.calls.iter().map(SymbolId::as_str).collect::<Vec<_>>(),"effects":step.effects.values(),"source":{"file":step.source.file,"line":step.source.line,"end_line":step.source.end_line}}) }

/// Atomically write the semantic portion of `.marqdo/`.  A reader therefore
/// sees either the previous graph or the complete next graph, never a mix.
pub fn write_semantic_projections(out_dir: &Path, compilation: &Compilation) -> Result<()> {
    let stage = out_dir.with_extension("tmp-sema");
    if stage.exists() { fs::remove_dir_all(&stage).with_context(|| format!("clear {}", stage.display()))?; }
    fs::create_dir_all(stage.join("artifacts"))?;
    fs::create_dir_all(stage.join("graph"))?;
    fs::create_dir_all(stage.join("verification"))?;
    let project = compilation.semantics.to_json();
    write_json(&stage.join("project.json"), &json!({"schema_version":SCHEMA_VERSION,"project":project["project"],"artifact_count":compilation.semantics.artifacts.len()}))?;
    write_json(&stage.join("symbols.json"), &json!({"schema_version":SCHEMA_VERSION,"symbols":compilation.index.symbols().iter().map(|s|json!({"id":s.id.as_str(),"module":s.module.as_str(),"kind":s.kind.as_str(),"callable":s.callable.map(|c|c.as_str()),"name":s.name,"declaration_path":s.declaration_path,"source":{"file":rel_uri(compilation.index.root().path(),&s.source),"line":s.line}})).collect::<Vec<_>>() }))?;
    write_json(&stage.join("refs.json"), &json!({"schema_version":SCHEMA_VERSION,"refs":project["refs"]}))?;
    write_json(&stage.join("graph/nodes.json"), &json!({"schema_version":SCHEMA_VERSION,"nodes":project["artifacts"]}))?;
    write_json(&stage.join("graph/edges.json"), &json!({"schema_version":SCHEMA_VERSION,"edges":compilation.semantics.artifacts.iter().flat_map(|a|a.dependencies.iter().map(move |d|json!({"from":a.artifact_id.0,"to":d.0,"kind":"calls"}))).collect::<Vec<_>>() }))?;
    for artifact in &compilation.semantics.artifacts {
        let name = hex_name(&artifact.artifact_id.0);
        write_json(&stage.join("artifacts").join(format!("{name}.json")), &artifact_json(artifact))?;
        write_json(&stage.join("verification").join(format!("{name}.json")), &artifact_json(artifact)["verification"])?;
    }
    if out_dir.exists() {
        for name in ["project.json", "symbols.json", "refs.json", "artifacts", "graph", "verification"] { let target = out_dir.join(name); if target.exists() { if target.is_dir() { fs::remove_dir_all(&target)?; } else { fs::remove_file(&target)?; } } }
    } else { fs::create_dir_all(out_dir)?; }
    for name in ["project.json", "symbols.json", "refs.json", "artifacts", "graph", "verification"] { fs::rename(stage.join(name), out_dir.join(name))?; }
    fs::remove_dir_all(stage)?;
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<()> { fs::write(path, serde_json::to_vec_pretty(value)?).with_context(|| format!("write {}", path.display())) }
fn hex_name(value: &str) -> String { digest(value).0.trim_start_matches("sha256:").to_string() }

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root() -> PathBuf { let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(); let root = std::env::temp_dir().join(format!("marqdo-sema-ir-{id}")); fs::create_dir_all(&root).unwrap(); root }

    #[test]
    fn compilation_is_deterministic_and_preserves_contract_effects() {
        let root = root();
        fs::write(root.join("main.mq.md"), "# main\n\n| 参数 | 类型 |\n| --- | --- |\n| path | text |\n\n## save\n\n> fs.write_text `path` ok\n\n*1*").unwrap();
        let options = CompileOptions::for_path(&root).unwrap();
        let first = compile_project(&options).unwrap();
        let second = compile_project(&options).unwrap();
        assert_eq!(first.semantics.to_json(), second.semantics.to_json());
        let save = first.semantics.artifacts.iter().find(|a| a.display_name.ends_with("save")).unwrap();
        assert!(save.effects.values().contains(&"fs.write".to_string()));
        assert_eq!(save.hashes.implementation.0, first.semantics.artifacts.iter().find(|a| a.display_name.ends_with("save")).unwrap().hashes.implementation.0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn semantic_files_have_schema_versions() {
        let root = root(); fs::write(root.join("a.mq.md"), "# a\n\n*1*").unwrap();
        let out = root.join(".marqdo"); let compilation = compile_project(&CompileOptions::for_path(&root).unwrap()).unwrap();
        write_semantic_projections(&out, &compilation).unwrap();
        let project: Value = serde_json::from_str(&fs::read_to_string(out.join("project.json")).unwrap()).unwrap();
        assert_eq!(project["schema_version"], SCHEMA_VERSION);
        assert!(out.join("artifacts").read_dir().unwrap().next().is_some());
        fs::remove_dir_all(root).unwrap();
    }
}
