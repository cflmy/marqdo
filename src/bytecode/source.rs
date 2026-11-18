//! Structural correspondence from a separately captured loaded AST to emission
//! witnesses. Parsing and capture of the source snapshot remain trusted inputs.
use super::{ExpressionRule, FnChunk};
use crate::ast::{Arg, CallExpr, Expr, Function, Literal, Param, Stmt};
use serde_json::{json, Value as Json};

#[derive(Debug, Clone)]
pub struct SourceFunction {
    pub declaration_path: Vec<usize>,
    pub level: u8,
    pub child_count: usize,
    pub module_root_count: usize,
    pub imports: Vec<String>,
    pub uses: Vec<(String, Vec<String>)>,
    pub name: String,
    pub line: u32,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub receiver_aliases: bool,
    pub has_contract: bool,
    pub binding_contracts: Vec<String>,
    pub contract: Option<crate::contract::Contract>,
    pub var_contracts: std::collections::HashMap<String, crate::contract::Contract>,
}
impl SourceFunction {
    pub fn from_function(fun: &Function) -> Self {
        let mut bindings = fun.var_contracts.keys().cloned().collect::<Vec<_>>();
        bindings.sort();
        Self {
            declaration_path: Vec::new(),
            level: fun.level,
            child_count: fun.children.len(),
            module_root_count: 0,
            imports: Vec::new(),
            uses: Vec::new(),
            name: fun.name.clone(),
            line: fun.span.line,
            params: fun.params.clone(),
            body: fun.body.clone(),
            receiver_aliases: crate::ast::function_uses_receiver(fun),
            has_contract: fun.contract.is_some(),
            binding_contracts: bindings,
            contract: fun.contract.clone(),
            var_contracts: fun.var_contracts.clone(),
        }
    }
    pub fn from_declaration(module: &crate::ast::Module, fun: &Function) -> Self {
        fn locate(nodes: &[Function], target: &Function, path: &mut Vec<usize>) -> bool {
            for (index, node) in nodes.iter().enumerate() {
                path.push(index);
                if std::ptr::eq(node, target) || locate(&node.children, target, path) { return true; }
                path.pop();
            }
            false
        }
        let mut result = Self::from_function(fun);
        assert!(locate(&module.functions, fun, &mut result.declaration_path), "source declaration belongs to its module");
        result.module_root_count = module.functions.len();
        result.imports = module.import_modules.keys().cloned().collect();
        result.imports.sort();
        result.uses = module.uses.iter().map(|u| (u.bind.clone(), u.path.clone())).collect();
        result
    }
    pub fn evidence(&self) -> Result<Json, String> {
        Ok(
            json!({"declaration_path":self.declaration_path,"level":self.level,"child_count":self.child_count,"module_root_count":self.module_root_count,"imports":self.imports,"uses":self.uses,
            "name":self.name,"line":self.line,"params":self.params.iter().map(|p|Ok(json!({"name":p.name,"default":p.default.as_ref().map(expression_tree).transpose()?}))).collect::<Result<Vec<_>,String>>()?,
            "body":self.body.iter().map(statement_tree).collect::<Result<Vec<_>,_>>()?,"receiver_aliases":self.receiver_aliases,"has_contract":self.has_contract,"binding_contracts":self.binding_contracts,"contracts":Self::contract_metadata(self.contract.as_ref(),&self.var_contracts)}),
        )
    }
    pub fn contract_metadata(
        contract: Option<&crate::contract::Contract>,
        bindings: &std::collections::HashMap<String, crate::contract::Contract>,
    ) -> Json {
        fn ty(ty: &crate::contract::Ty) -> Json {
            match ty {
                crate::contract::Ty::Object(name) => json!({"kind":"object","name":name}),
                other => json!({"kind":other.name()}),
            }
        }
        fn encode(c: &crate::contract::Contract) -> Json {
            json!({"params":c.params.iter().map(|p|json!({"name":p.name,"type":ty(&p.ty),"description":p.desc})).collect::<Vec<_>>(),
                "returns":c.returns.as_ref().map(|r|json!({"type":ty(&r.ty),"description":r.desc})),
                "fields":c.fields.iter().map(|f|json!({"name":f.name,"type":ty(&f.ty),"nullable":f.nullable,"description":f.desc})).collect::<Vec<_>>(),
                "start_line":c.start_line,"end_line":c.end_line,"quote":c.quote})
        }
        let bindings = bindings
            .iter()
            .map(|(name, c)| (name, encode(c)))
            .collect::<std::collections::BTreeMap<_, _>>();
        json!({"function":contract.map(encode),"bindings":bindings})
    }
}
fn normalized(call: &CallExpr) -> Result<CallExpr, String> {
    call.for_evaluation()
}
fn call_shape(call: &CallExpr) -> Json {
    json!({"callee":call.callee,"receiver":call.receiver,"path":call.path,"arguments":call.args.iter().map(|a|match a {Arg::Positional(_)=>None,Arg::Named{name,..}=>Some(name)}).collect::<Vec<_>>()})
}
fn arg_expr(arg: &Arg) -> &Expr {
    match arg {
        Arg::Positional(e) | Arg::Named { value: e, .. } => e,
    }
}

// Deliberately match the loaded AST directly, rather than invoking the emitter's
// ExpressionRule::from_source or StatementRule::from_source recorder.
fn rule(expr: &Expr) -> Json {
    use crate::value::Value;
    match expr {
        Expr::Literal(lit) => {
            let value = match lit {
                Literal::None => json!({"tag":"none"}),
                Literal::Bool(v) => json!({"tag":"bool","value":v}),
                Literal::Int(v) => json!({"tag":"int","value":v}),
                Literal::Num(v) => json!({"tag":"num","bits":v.to_bits()}),
                Literal::Text(v) => json!({"tag":"text","value":v}),
            };
            json!({"kind":"literal","value":value})
        }
        Expr::Var(name) => json!({"kind":"read","name":name}),
        Expr::Unary { op, .. } => json!({"kind":"unary","op":format!("{op:?}")}),
        Expr::Binary { op, .. } => json!({"kind":"binary","op":format!("{op:?}")}),
        Expr::Call(call) => {
            if let Err(message) = normalized(call) {
                return json!({"kind":"call_expansion_fault","message":message});
            }
            json!({"kind":"call_boundary","names":call.receiver.iter().cloned().chain(call.path.iter().filter_map(|p|p.first().cloned())).collect::<Vec<_>>()})
        }
        Expr::List(items) => json!({"kind":"list","arity":items.len()}),
        Expr::Map(items) => {
            json!({"kind":"map","keys":items.iter().map(|(name,_)|name).collect::<Vec<_>>()})
        }
        Expr::Interp(parts) => {
            json!({"kind":"interpolation_boundary","parts":parts.iter().map(super::expression::interpolation_evidence).collect::<Vec<_>>()})
        }
        Expr::Index { base, label } => {
            json!({"kind":"index_boundary","label":super::expression::index_evidence(label),"base_variable":if let Expr::Var(name)=base.as_ref(){Some(name)}else{None}})
        }
        Expr::Occluded { payload, .. } => json!({"kind":"occluded","payload":payload.is_some()}),
        Expr::Formula(value) => {
            json!({"kind":"literal","value":super::expression::value_evidence(&Value::Formula(value.clone()))})
        }
        Expr::Code(value) => {
            json!({"kind":"literal","value":{"tag":"code","lang":value.lang,"source":value.source}})
        }
    }
}
fn children(expr: &Expr) -> Vec<&Expr> {
    match expr {
        Expr::Unary { expr, .. } => vec![expr],
        Expr::Binary { left, right, .. } => vec![left, right],
        Expr::List(items) => items.iter().collect(),
        Expr::Map(items) => items.iter().map(|(_, e)| e).collect(),
        Expr::Index { base, .. } => vec![base],
        Expr::Occluded { payload, .. } => payload.iter().map(|e| e.as_ref()).collect(),
        _ => Vec::new(),
    }
}
fn expression_tree(expr: &Expr) -> Result<Json, String> {
    if matches!(expr,Expr::Call(call) if normalized(call).is_err()) {
        return Ok(json!({"rule":rule(expr),"children":[],"call":null}));
    }
    let (children, call) = if let Expr::Call(call) = expr {
        let expanded = normalized(call)?;
        (
            expanded
                .args
                .iter()
                .map(|a| expression_tree(arg_expr(a)))
                .collect::<Result<Vec<_>, _>>()?,
            Some(call_shape(&expanded)),
        )
    } else {
        (
            children(expr)
                .iter()
                .map(|e| expression_tree(e))
                .collect::<Result<Vec<_>, _>>()?,
            None,
        )
    };
    Ok(json!({"rule":rule(expr),"children":children,"call":call}))
}
fn statement_rule(stmt: &Stmt) -> Json {
    match stmt {
        Stmt::Assign { name, .. } => json!({"kind":"assign","name":name}),
        Stmt::Return { .. } => json!({"kind":"return"}),
        Stmt::Call { .. } => json!({"kind":"call"}),
        Stmt::Expr { .. } => json!({"kind":"expression"}),
        Stmt::Branch { arms, .. } => {
            json!({"kind":"branch","arms":arms.iter().map(|a|(a.condition.is_some(),a.body.len())).collect::<Vec<_>>()})
        }
        Stmt::While { body, .. } => json!({"kind":"while","body":body.len()}),
        Stmt::ForEach {
            item,
            collection,
            body,
            ..
        } => json!({"kind":"foreach","item":item,"collection":collection,"body":body.len()}),
    }
}
fn statement_children(stmt: &Stmt) -> Vec<&Stmt> {
    match stmt {
        Stmt::Branch { arms, .. } => arms.iter().flat_map(|a| a.body.iter()).collect(),
        Stmt::While { body, .. } | Stmt::ForEach { body, .. } => body.iter().collect(),
        _ => Vec::new(),
    }
}
fn statement_expressions(stmt: &Stmt) -> Vec<Expr> {
    match stmt {
        Stmt::Assign { value, .. } | Stmt::Return { value, .. } | Stmt::Expr { value, .. } => {
            vec![value.clone()]
        }
        Stmt::Call { call, .. } => vec![Expr::Call(call.clone())],
        Stmt::Branch { arms, .. } => arms.iter().filter_map(|a| a.condition.clone()).collect(),
        Stmt::While { condition, .. } => vec![condition.clone()],
        Stmt::ForEach { collection, .. } => vec![Expr::Var(collection.clone())],
    }
}
fn statement_tree(stmt: &Stmt) -> Result<Json, String> {
    Ok(
        json!({"rule":statement_rule(stmt),"expressions":statement_expressions(stmt).iter().map(expression_tree).collect::<Result<Vec<_>,_>>()?,"statements":statement_children(stmt).iter().map(|s|statement_tree(s)).collect::<Result<Vec<_>,_>>()?}),
    )
}

struct Walk<'a> {
    fun: &'a FnChunk,
    expressions: Vec<bool>,
    statements: Vec<bool>,
}
impl Walk<'_> {
    fn expression(&mut self, source: &Expr, id: usize) -> Result<(), String> {
        let node = self
            .fun
            .expressions
            .get(id)
            .ok_or("AST expression reference")?;
        if self.expressions[id] {
            return Err("AST expression has multiple owners".into());
        }
        self.expressions[id] = true;
        if node.rule.evidence() != rule(source) {
            return Err("loaded AST expression rule differs from witness".into());
        }
        if let Expr::Call(call) = source {
            let call = match normalized(call) {
                Ok(call) => call,
                Err(_) => {
                    if !node.children.is_empty()
                        || self.fun.calls.iter().any(|c| c.expression == id)
                    {
                        return Err("AST preparation fault evaluated actuals".into());
                    }
                    return Ok(());
                }
            };
            let record = self
                .fun
                .calls
                .iter()
                .find(|c| c.expression == id)
                .ok_or("AST call obligation missing")?;
            if call_shape(&call) != call_shape(&record.source)
                || node.children.len() < call.args.len()
            {
                return Err("loaded AST call differs from witness".into());
            }
            for (arg, child) in call.args.iter().zip(&node.children) {
                self.expression(arg_expr(arg), *child)?;
            }
            for &generated in &node.children[call.args.len()..] {
                let expr = self
                    .fun
                    .expressions
                    .get(generated)
                    .ok_or("generated reload reference")?;
                if self.expressions[generated]
                    || !expr.children.is_empty()
                    || !matches!(expr.rule, ExpressionRule::Read(_))
                {
                    return Err("generated reload is shared or not a read".into());
                }
                self.expressions[generated] = true;
            }
        } else {
            let expected = children(source);
            if expected.len() != node.children.len() {
                return Err("loaded AST expression child inventory differs".into());
            }
            for (source, child) in expected.iter().zip(&node.children) {
                self.expression(source, *child)?;
            }
        }
        Ok(())
    }
    fn statement(&mut self, source: &Stmt, id: usize) -> Result<(), String> {
        let node = self
            .fun
            .statements
            .get(id)
            .ok_or("AST statement reference")?;
        if self.statements[id] {
            return Err("AST statement has multiple owners".into());
        }
        self.statements[id] = true;
        if node.rule.evidence() != statement_rule(source) {
            return Err("loaded AST statement rule differs from witness".into());
        }
        let exprs = statement_expressions(source);
        let stmts = statement_children(source);
        if node.expressions.len() != exprs.len() || node.statements.len() != stmts.len() {
            return Err("loaded AST statement child inventory differs".into());
        }
        for (expr, id) in exprs.iter().zip(&node.expressions) {
            self.expression(expr, *id)?;
        }
        for (stmt, id) in stmts.iter().zip(&node.statements) {
            self.statement(stmt, *id)?;
        }
        Ok(())
    }
}
pub fn verify(fun: &FnChunk) -> Result<(), String> {
    let source = &fun.source_ast;
    if source.contract != fun.contract || source.var_contracts != fun.var_contracts {
        return Err("loaded AST contract metadata differs".into());
    }
    if source.name != fun.name
        || source.line != fun.declaration_span.line
        || source.params.iter().map(|p| &p.name).ne(fun.params.iter())
        || source.receiver_aliases != fun.receiver_aliases
        || source.has_contract != fun.contract.is_some()
        || source.binding_contracts.len() != fun.var_contracts.len()
        || source
            .binding_contracts
            .iter()
            .any(|name| !fun.var_contracts.contains_key(name))
    {
        return Err("loaded AST function metadata differs".into());
    }
    let mut walk = Walk {
        fun,
        expressions: vec![false; fun.expressions.len()],
        statements: vec![false; fun.statements.len()],
    };
    if source.params.len() != fun.default_expressions.len()
        || source.body.len() != fun.statement_roots.len()
    {
        return Err("loaded AST function root inventory differs".into());
    }
    for (param, id) in source.params.iter().zip(&fun.default_expressions) {
        match (&param.default, id) {
            (Some(expr), Some(id)) => walk.expression(expr, *id)?,
            (None, None) => {}
            _ => return Err("loaded AST default presence differs".into()),
        }
    }
    for (stmt, id) in source.body.iter().zip(&fun.statement_roots) {
        walk.statement(stmt, *id)?;
    }
    if walk.expressions.iter().any(|seen| !seen) || walk.statements.iter().any(|seen| !seen) {
        return Err("emission witness has no loaded AST owner".into());
    }
    Ok(())
}
