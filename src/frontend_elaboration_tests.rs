//! Source-level obligations for the real elaborator and both execution paths.
use crate::{
    bytecode::{compile_module, Vm},
    interp::Interpreter,
    load::load_module,
    value::Value,
};
use std::path::PathBuf;

// Evidence describes local machine values, not a foreign copied payload.
fn local_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Int(value) => serde_json::json!(value),
        _ => crate::host::json::value_to_json(value).unwrap(),
    }
}

fn document(id: &str, source: &str) -> (PathBuf, crate::ast::Module) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/frontend-elaboration")
        .join(format!("{id}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.mq.md");
    std::fs::write(&path, source).unwrap();
    let module = load_module(&path).unwrap();
    (path, module)
}

#[test]
fn named_arguments_preserve_authored_effect_order() {
    let source = "# main\n**answer = combine(b=mark(2), a=mark(1))**\n*answer*\n\n## combine\n    + `a`\n    + `b`\n**n = a * 10 + b**\n*n*\n\n## mark\n    + `value`\n> print text=`value`\n*value*\n";
    both("argument-order", source, Value::Int(12), "2\n1\n");
}

#[test]
fn defaults_see_target_lexical_scope_and_earlier_parameters() {
    let source = "# main\n**answer = > choose a=7**\n*answer*\n\n## choose\n    + `a`\n    + `b`=`a`\n    + `c`=> helper\n**n = b * 10 + c**\n\n### helper\n*3*\n\n*n*\n\n## helper\n*9*\n";
    both("default-scope", source, Value::Int(73), "");
    let (_, module) = document("default-scope", source);
    let choose = module.functions[0]
        .children
        .iter()
        .find(|f| f.name == "choose")
        .unwrap();
    assert!(choose.children.iter().any(|f| f.name == "helper"));
    assert_eq!(
        module.functions[0]
            .children
            .iter()
            .filter(|f| f.name == "helper")
            .count(),
        1
    );
}

fn both(id: &str, source: &str, value: Value, stdout: &str) {
    let (path, module) = document(id, source);
    let mut tree = Interpreter::with_capture(Some(&path), false);
    let tree_value = tree.run_module(&module).unwrap();
    assert_eq!(tree_value, value, "{id}: tree");
    assert_eq!(tree.captured_stdout, stdout, "{id}: tree stdout");
    let program = compile_module(Some(&path), &module).unwrap();
    let mut vm = Vm::with_capture(Some(&path));
    let vm_value = vm.run(&program).unwrap();
    assert_eq!(vm_value, value, "{id}: bytecode");
    assert_eq!(vm.captured_stdout, stdout, "{id}: bytecode stdout");
    #[cfg(feature = "exec-host")]
    let boundary = {
        let encode = |(journal, protocol, live): (
            Vec<Value>,
            Vec<crate::host::foreign_session::ProtocolStep>,
            usize,
        )| {
            serde_json::json!({
            "journal":crate::host::json::value_to_json(&Value::List(journal)).unwrap(),
            "protocol":protocol.iter().map(|step|serde_json::json!({"session":step.session,"request":step.request,"response":step.response})).collect::<Vec<_>>(),"live":live})
        };
        serde_json::json!({"tree":encode(tree.application_evidence()),"bytecode":encode(vm.application_evidence())})
    };
    #[cfg(not(feature = "exec-host"))]
    let boundary = serde_json::Value::Null;
    if let Some(out) = std::env::var_os("MARQDO_FRONTEND_DIR") {
        use serde_json::json;
        fn rows(
            module: &crate::ast::Module,
            namespace: &[String],
            out: &mut Vec<serde_json::Value>,
            modules: &mut Vec<serde_json::Value>,
        ) {
            fn functions(
                module: &crate::ast::Module,
                namespace: &[String],
                lexical: &[String],
                parent: Option<u32>,
                items: &[crate::ast::Function],
                out: &mut Vec<serde_json::Value>,
            ) {
                for fun in items {
                    let mut path = lexical.to_vec();
                    path.push(fun.name.clone());
                    out.push(json!({"name":fun.name,"line":fun.span.line,"path":module.source.path,"sha256":module.source.sha256,
                        "module_path":namespace,"lexical_path":path,"parent_line":parent,"level":fun.level,"base":fun.base,
                        "params":fun.params.iter().map(|p|json!({"name":p.name,"default":p.default.is_some(),"line":p.span.line})).collect::<Vec<_>>(),
                        "source_ast":crate::bytecode::SourceFunction::from_declaration(module,fun).evidence().unwrap(),"body_count":fun.body.len(),"uses_receiver":crate::ast::function_uses_receiver(fun)}));
                    functions(
                        module,
                        namespace,
                        &path,
                        Some(fun.span.line),
                        &fun.children,
                        out,
                    );
                }
            }
            let text = module
                .source
                .path
                .as_ref()
                .and_then(|path| std::fs::read_to_string(path).ok());
            modules.push(json!({"module_path":namespace,"path":module.source.path,"sha256":module.source.sha256,"text":text,
                "imports":({let mut names=module.import_modules.keys().cloned().collect::<Vec<_>>();names.sort();names}),
                "uses":module.uses.iter().map(|binding|json!({"bind":binding.bind,"path":binding.path})).collect::<Vec<_>>()}));
            functions(module, namespace, &[], None, &module.functions, out);
            let mut imports = module.import_modules.iter().collect::<Vec<_>>();
            imports.sort_by_key(|(name, _)| *name);
            for (bind, imported) in imports {
                let mut child = namespace.to_vec();
                child.push(bind.clone());
                rows(imported, &child, out, modules);
            }
        }
        let mut ast = Vec::new();
        let mut modules = Vec::new();
        rows(&module, &[], &mut ast, &mut modules);
        let certificates = program.verify().unwrap();
        let evidence = json!({"version":1,"id":id,"source":source,"path":path,"sha256":module.source.sha256,
            "expected":{"value":local_json(&value),"stdout":stdout},
            "tree":{"value":local_json(&tree_value),"stdout":tree.captured_stdout},
            "bytecode":{"value":local_json(&vm_value),"stdout":vm.captured_stdout},
            "boundary":boundary,"ast":ast,"modules":modules,"objects":program.objects,"object_bases":program.object_bases,"methods":program.methods,"parents":program.parents,"children":program.children,
            "entry":program.entry,"entry_metadata":program.entry_metadata.iter().map(|(name,value)|json!([name,local_json(value)])).collect::<Vec<_>>(),"functions":program.functions.iter().enumerate().map(|(fi,f)|{let mut row=json!({
                "stack_heights":certificates[fi].heights,
                "body_start":f.body_start,"default_expressions":f.default_expressions,"statement_roots":f.statement_roots,
                "statements":f.statements.iter().map(|s|json!({"rule":s.rule.evidence(),"start":s.start,"end":s.end,"statements":s.statements,"expressions":s.expressions,"auxiliary":s.auxiliary})).collect::<Vec<_>>(),
                "constants":f.constants.iter().map(crate::bytecode::expression_value_evidence).collect::<Vec<_>>(),
                "expressions":f.expressions.iter().map(|e|json!({"rule":e.rule.evidence(),"start":e.start,"end":e.end,"children":e.children})).collect::<Vec<_>>(),
                "name":f.name,"line":f.declaration_span.line,"path":f.source.path,"sha256":f.source.sha256,
                "module_path":f.module_path,"spans":f.spans.iter().map(|span|span.line).collect::<Vec<_>>(),
                "params":f.params,"receiver_aliases":f.receiver_aliases,"defaults":f.defaults,"scope":f.module_scope,"locals":f.locals,"call_layouts":f.call_layouts,"constant_count":f.constants.len(),
                "code":f.code.iter().map(|op|format!("{op:?}")).collect::<Vec<_>>()
            });row["source_ast"]=f.source_ast.evidence().unwrap();row["contracts"]=crate::bytecode::SourceFunction::contract_metadata(f.contract.as_ref(),&f.var_contracts);row["has_contract"]=json!(f.contract.is_some());row["binding_contracts"]=json!(f.var_contracts.keys().collect::<Vec<_>>());row["source_local_count"]=json!(f.source_local_count);row["calls"]=json!(f.calls.iter().map(|c|{let mut row=c.evidence();row["binding_plan"]=c.binding_evidence(&program);row}).collect::<Vec<_>>());row}).collect::<Vec<_>>()});
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(
            out.join(format!("{id}.json")),
            serde_json::to_vec_pretty(&evidence).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn method_signature_belongs_to_the_actual_receiver() {
    both("receiver-signature", "# A\n## m\n    + `x`\n*x*\n\n# B\n## m\n    + `y`\n    + `z`=3\n**r = y * 10 + z**\n*r*\n\n# main\n**b = > B**\n**answer = > `b`.m y=7**\n*answer*\n", Value::Int(73), "");
}

#[test]
fn defaults_distinguish_omission_from_explicit_none() {
    both(
        "explicit-none",
        "# main\n**answer = > f value=None**\n*answer*\n\n## f\n    + `value`=7\n*value*\n",
        Value::None,
        "",
    );
}

#[test]
fn default_effects_execute_once_and_in_parameter_order() {
    both("default-order", "# main\n**answer = f(c=mark(9))**\n*answer*\n\n## f\n    + `a`=> mark value=1\n    + `b`=> mark value=2\n    + `c`=0\n**r = a * 100 + b * 10 + c**\n*r*\n\n## mark\n    + `value`\n> print text=`value`\n*value*\n", Value::Int(129), "9\n1\n2\n");
}

#[test]
fn supplied_later_parameter_is_available_to_a_default() {
    both(
        "provided-later",
        "# main\n**answer = > f b=8**\n*answer*\n\n## f\n    + `a`=`b`\n    + `b`=7\n*a*\n",
        Value::Int(8),
        "",
    );
}

#[test]
fn imported_defaults_keep_the_definition_module() {
    let (path, _) = document("import-default", "# main\n*0*\n");
    let library = "## choose\n    + `value`=> helper\n*value*\n\n## helper\n*3*\n";
    std::fs::write(path.parent().unwrap().join("lib.mq.md"), library).unwrap();
    both("import-default", "---\nimport:\n  lib: lib.mq.md\n---\n# main\n**answer = > lib.choose**\n*answer*\n\n## helper\n*9*\n", Value::Int(3), "");
}

#[test]
fn nested_calls_do_not_overwrite_pending_actuals_or_leak_temporary_slots() {
    let mut source = String::from("# main\n");
    for _ in 0..400 {
        source.push_str("**answer = pair(b=pair(2, 3), a=pair(4, 5))**\n");
    }
    source.push_str("*answer*\n\n## pair\n    + `a`\n    + `b`\n**r = a * 10 + b**\n*r*\n");
    both("temporary-lifetimes", &source, Value::Int(473), "");
}

#[test]
fn unused_extra_named_arguments_still_execute_in_authored_order() {
    both("extra-effects", "# main\n**answer = f(extra=mark(2), x=mark(1))**\n*answer*\n\n## f\n    + `x`\n*x*\n\n## mark\n    + `value`\n> print text=`value`\n*value*\n", Value::Int(1), "2\n1\n");
}

#[test]
fn builtin_argument_permutation_happens_after_authored_evaluation() {
    both("builtin-order", "# main\n**answer = split(sep=mark(\"S\"), value=mark(\"X\"))**\n*answer*\n\n## mark\n    + `value`\n> print text=`value`\n*value*\n", Value::List(vec![Value::Text("X".into())]), "S\nX\n");
}

#[test]
fn imported_object_identity_survives_cross_module_calls() {
    let (path, _) = document("qualified-objects", "# main\n*0*\n");
    std::fs::write(
        path.parent().unwrap().join("a.mq.md"),
        "# Counter\n## read\n*3*\n",
    )
    .unwrap();
    std::fs::write(
        path.parent().unwrap().join("b.mq.md"),
        "## through\n    + `value`\n**r = > `value`.read**\n*r*\n\n# Counter\n## read\n*7*\n",
    )
    .unwrap();
    both("qualified-objects", "---\nimport:\n  a: a.mq.md\n  b: b.mq.md\n---\n# main\n**x = > a.Counter**\n**y = > b.Counter**\n**p = > `x`.read**\n**q = > `y`.read**\n**r = > b.through value=`x`**\n**answer = p * 100 + q * 10 + r**\n*answer*\n", Value::Int(373), "");
}

#[test]
fn inherited_methods_resolve_bases_in_the_defining_module() {
    let (path, _) = document("qualified-inheritance", "# main\n*0*\n");
    for (name, value) in [("a", 3), ("b", 7)] {
        std::fs::write(
            path.parent().unwrap().join(format!("{name}.mq.md")),
            format!("# Parent\n## read\n*{value}*\n\n# Child = > Parent\n"),
        )
        .unwrap();
    }
    both("qualified-inheritance", "---\nimport:\n  a: a.mq.md\n  b: b.mq.md\n---\n# main\n**x = > a.Child**\n**y = > b.Child**\n**p = > `x`.read**\n**q = > `y`.read**\n**answer = p * 10 + q**\n*answer*\n", Value::Int(37), "");
}

#[test]
fn imported_type_aliases_disambiguate_inheritance() {
    let (path, _) = document("aliased-base", "# main\n*0*\n");
    for (name, value) in [("a", 3), ("b", 7)] {
        std::fs::write(
            path.parent().unwrap().join(format!("{name}.mq.md")),
            format!("# Parent\n## read\n*{value}*\n"),
        )
        .unwrap();
    }
    both("aliased-base","---\nimport:\n  a: a.mq.md\n  b: b.mq.md\n  Base: a.Parent\n---\n# Child = > Base\n\n# main\n**x = > Child**\n**answer = > `x`.read**\n*answer*\n",Value::Int(3),"");
}

#[test]
fn default_object_arguments_keep_root_module_context() {
    let (path, _) = document("external-object-default", "# main\n*0*\n");
    std::fs::write(
        path.parent().unwrap().join("a.mq.md"),
        "# Counter\n## read\n*3*\n",
    )
    .unwrap();
    std::fs::write(
        path.parent().unwrap().join("b.mq.md"),
        "## through\n    + `value`\n**r = > `value`.read**\n*r*\n\n# Counter\n## read\n*7*\n",
    )
    .unwrap();
    let source=format!("---\nimport:\n  a: a.mq.md\n  b: b.mq.md\n  json: {}\n---\n## choose\n    + `value`=> a.Counter\n**r = > json.get value=`value` key=\"_type_module\"**\n*r*\n",PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib/json.mq.md").display());
    std::fs::write(&path, &source).unwrap();
    for backend in [crate::Backend::Tree, crate::Backend::Bytecode] {
        assert_eq!(
            crate::call::call_file(&path, "choose", &[], backend).unwrap(),
            Value::List(vec![Value::Text("a".into())])
        );
    }
}

#[test]
fn entry_defaults_use_metadata_as_supplied_arguments() {
    both("entry-metadata-default","---\nvalue: 7\n---\n# main\n    + `value`=> mark\n*value*\n\n## mark\n> print text=DEFAULT\n*99*\n",Value::Int(7),"");
    both(
        "entry-omitted-default",
        "# main\n    + `value`=> mark\n*value*\n\n## mark\n> print text=DEFAULT\n*99*\n",
        Value::Int(99),
        "DEFAULT\n",
    );
}

#[test]
fn named_calls_to_main_do_not_inject_entry_metadata() {
    let (path, module) = document(
        "named-main",
        "---\nvalue: 7\n---\n# main\n    + `value`=3\n*value*\n",
    );
    let fun = module.functions.iter().find(|f| f.name == "main").unwrap();
    let args = vec![("value".into(), Value::Int(42))];
    let mut tree = Interpreter::with_capture(Some(&path), false);
    assert_eq!(
        tree.invoke_bound_function(&module, fun, &args).unwrap(),
        Value::Int(42)
    );
    let program = crate::bytecode::compile_for_call(Some(&path), &module).unwrap();
    let mut vm = Vm::with_capture(Some(&path));
    assert_eq!(
        vm.call_function(&program, program.entry, &args).unwrap(),
        Value::Int(42)
    );
}

#[cfg(feature = "exec-host")]
#[test]
fn entry_session_defaults_execute_in_the_owning_backend_context() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for language in ["python", "node"] {
        let source=format!("---\nimport:\n  foreign: {}\n---\n# main\n    + `s`=> foreign.session_open lang=\"{language}\"\n**> foreign.session_set id=`s` name=\"n\" value=7**\n**answer = > foreign.session_get id=`s` name=\"n\"**\n**> foreign.session_close id=`s`**\n*answer*\n",root.join("lib/foreign.mq.md").display());
        both(
            &format!("entry-session-{language}"),
            &source,
            Value::Int(7),
            "",
        );
    }
}

#[test]
fn declarations_read_only_by_defaults_are_inferred_inputs() {
    let source="# main\n**answer = > choose b=7**\n*answer*\n\n## choose\n    + `a`=`b`\n请求参数`b`。\n*a*\n";
    both("default-only-input", source, Value::Int(7), "");
    let (path, module) = document("default-only-input", source);
    let choose = &module.functions[0].children[0];
    assert!(choose.params.iter().any(|p| p.name == "b" && p.inferred));
    for backend in [crate::Backend::Tree, crate::Backend::Bytecode] {
        assert_eq!(
            crate::call::call_file(
                &path,
                "main.choose",
                &[("b".into(), Value::Int(7))],
                backend
            )
            .unwrap(),
            Value::Int(7)
        );
    }
}

#[test]
fn explicit_and_prose_defaults_keep_declaration_order() {
    let source="# main\n**answer = > choose**\n*answer*\n\n## choose\n    + `a`=> mark value=1\n参数`b`=mark(2)\n**r = a * 10 + b**\n*r*\n\n## mark\n    + `value`\n> print text=`value`\n*value*\n";
    both("prose-default-order", source, Value::Int(12), "1\n2\n");
}

#[test]
fn default_faults_use_the_parameter_source_line() {
    let source = "# main\n**answer = > bad**\n*answer*\n\n## bad\n    + `value`=1 / 0\n*value*\n";
    let (path, module) = document("default-fault-line", source);
    let expected = source
        .lines()
        .position(|line| line.contains("+ `value`"))
        .unwrap() as u32
        + 1;
    let mut tree = Interpreter::with_capture(Some(&path), false);
    let program = compile_module(Some(&path), &module).unwrap();
    let mut vm = Vm::with_capture(Some(&path));
    for result in [tree.run_module(&module), vm.run(&program)] {
        let error = result.unwrap_err();
        let diagnostic = crate::diagnostics::Diagnostic::find(&error).unwrap();
        assert!(diagnostic.message.contains("division by zero"));
        assert_eq!(diagnostic.span, crate::diagnostics::Span::new(expected, 1));
        assert_eq!(diagnostic.path.as_deref(), Some(path.as_path()));
    }
}

#[test]
fn duplicate_parameters_are_frontend_errors() {
    let (path, _) = document("duplicate-names", "# main\n*0*\n");
    for (source, expected) in [(
        "# main\n    + `value`\n    + `value`=7\n*value*\n",
        "duplicate parameter",
    )] {
        std::fs::write(&path, source).unwrap();
        let error = load_module(&path).unwrap_err();
        assert!(error.to_string().contains(expected), "{error:#}");
    }
}

#[test]
fn qualified_paths_keep_the_first_source_declaration() {
    let (path, _) = document("first-qualified-declaration", "# main\n*0*\n");
    std::fs::write(
        path.parent().unwrap().join("a.mq.md"),
        "# A\n## read\n*1*\n\n## read\n*2*\n",
    )
    .unwrap();
    both(
        "first-qualified-declaration",
        "---\nimport:\n  a: a.mq.md\n---\n# main\n**answer = > a.A.read**\n*answer*\n",
        Value::Int(1),
        "",
    );
}

#[test]
fn scratch_slots_do_not_alias_authored_names() {
    let source="# main\n**ignored = mark(1)**\n**`\0call.arg.1.0` = 7**\n**ignored = mark(2)**\n*`\0call.arg.1.0`*\n\n## mark\n    + `value`\n> print text=`value`\n*value*\n";
    both("scratch-freshness", source, Value::Int(7), "1\n2\n");
}

#[test]
fn default_expressions_can_read_extra_named_bindings() {
    both(
        "default-extra-binding",
        "# main\n**answer = > choose b=7**\n*answer*\n\n## choose\n    + `a`=`b`\n*a*\n",
        Value::Int(7),
        "",
    );
}

#[test]
fn metadata_initialization_matches_first_binding_semantics() {
    let (path, mut module) = document("metadata-first-binding", "# main\n*value*\n");
    module.metadata = vec![
        ("value".into(), Value::Int(7)),
        ("value".into(), Value::Int(8)),
    ];
    let mut tree = Interpreter::with_capture(Some(&path), false);
    assert_eq!(tree.run_module(&module).unwrap(), Value::Int(7));
    let program = compile_module(Some(&path), &module).unwrap();
    let mut vm = Vm::with_capture(Some(&path));
    assert_eq!(vm.run(&program).unwrap(), Value::Int(7));
}

#[cfg(feature = "exec-host")]
#[test]
fn copied_object_module_tags_remain_data_across_foreign_boundaries() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for language in ["python", "node"] {
        let id = format!("copied-object-{language}");
        let (path, _) = document(&id, "# main\n*0*\n");
        for (name, value) in [("a", 3), ("b", 7)] {
            std::fs::write(
                path.parent().unwrap().join(format!("{name}.mq.md")),
                format!("# Counter\n## read\n*{value}*\n"),
            )
            .unwrap();
        }
        let source=format!("---\nimport:\n  a: a.mq.md\n  b: b.mq.md\n  foreign: {}\n---\n# main\n**x = > a.Counter**\n**s = > foreign.session_open lang=\"{language}\"**\n**> foreign.session_set id=`s` name=\"object\" value=`x`**\n**copy = > foreign.session_get id=`s` name=\"object\"**\n**> foreign.session_close id=`s`**\n**p = > `copy`.read**\n**y = > b.Counter**\n**q = > `y`.read**\n**answer = p * 10 + q**\n*answer*\n",root.join("lib/foreign.mq.md").display());
        both(&id, &source, Value::Int(37), "");
    }
}

#[test]
fn local_constant_and_jump_indices_do_not_truncate() {
    let mut locals = String::from("# main\n");
    for i in 0..700 {
        locals.push_str(&format!("**x{i} = {i}**\n"));
    }
    locals.push_str("**answer = x0 + x699**\n*answer*\n");
    both("wide-locals", &locals, Value::Int(699), "");
    let mut constants = String::from("# main\n");
    for i in 0..70000 {
        constants.push_str(&format!("**answer = {i}**\n"));
    }
    constants.push_str("*answer*\n");
    both("wide-constants", &constants, Value::Int(69999), "");
    let mut jump = String::from("# main\n**flag = False**\n1. `flag`\n");
    for _ in 0..25000 {
        jump.push_str("  **answer = 0**\n");
    }
    jump.push_str("2. *\n  **answer = 8**\n\n*answer*\n");
    both("wide-jumps", &jump, Value::Int(8), "");
}

#[test]
fn receiver_used_only_by_a_default_is_still_an_instance_method() {
    let source = format!("---\nimport:\n  json: {}\n---\n# B\n**state = > json.parse text={{\"n\":7}}**\n*state*\n\n## m\n    + `value`=> json.get value=`self` key=\"n\"\n*value*\n\n# main\n**b = > B**\n**answer = > `b`.m**\n*answer*\n", PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib/json.mq.md").display());
    both("receiver-default", &source, Value::Int(7), "");
    let (_, module) = document("receiver-default", &source);
    assert!(crate::ast::function_uses_receiver(
        &module.functions[0].children[0]
    ));
}

#[test]
fn pure_call_checks_transitive_default_effects_before_executing_them() {
    let source = "## choose\n    + `value`=> mark\n*value*\n\n## mark\n> print text=effect\n*7*\n";
    let (path, _) = document("default-effects-policy", source);
    for backend in [crate::Backend::Tree, crate::Backend::Bytecode] {
        let error = crate::call::call_file(&path, "choose", &[], backend).unwrap_err();
        let diagnostic = crate::diagnostics::Diagnostic::find(&error).unwrap();
        assert_eq!(diagnostic.code, "artifact.effect_denied", "{error:#}");
    }
}

#[test]
fn an_imported_main_cannot_replace_the_entry_module() {
    let (path, _) = document("entry-scope", "# main\n*0*\n");
    std::fs::write(
        path.parent().unwrap().join("lib.mq.md"),
        "# main\n> print text=WRONG-ENTRY\n*99*\n",
    )
    .unwrap();
    both(
        "entry-scope",
        "---\nimport:\n  lib: lib.mq.md\n---\n# main\n*7*\n",
        Value::Int(7),
        "",
    );
}

#[test]
fn omitted_later_defaults_are_unbound_and_none_is_not_a_missing_slot() {
    let source = "# main\n**answer = > f**\n*answer*\n\n## f\n    + `a`=`b`\n    + `b`=7\n*a*\n";
    let (path, module) = document("unbound-default", source);
    let mut tree = Interpreter::with_capture(Some(&path), false);
    assert!(tree
        .run_module(&module)
        .unwrap_err()
        .to_string()
        .contains("undefined variable `b`"));
    let program = compile_module(Some(&path), &module).unwrap();
    let mut vm = Vm::with_capture(Some(&path));
    assert!(vm
        .run(&program)
        .unwrap_err()
        .to_string()
        .contains("undefined variable `b`"));
}

#[test]
fn signed_integer_literals_include_the_negative_endpoint() {
    both(
        "minimum-int",
        "# main\n**answer = -9223372036854775808**\n*answer*\n",
        Value::Int(i64::MIN),
        "",
    );
}

#[cfg(feature = "exec-host")]
#[test]
fn nested_actuals_and_lexical_defaults_preserve_real_foreign_transitions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for language in ["python", "node"] {
        let code = "n = n * 10 + marqdo_input; n";
        let source = format!(
            r#"---
import:
  foreign: {}
  json: {}
---
# main
**h = > foreign.session_open lang="{language}"**
**> foreign.session_set id=`h` name="n" value=0**
**answer = combine(b=mark(id=h, value=2), a=mark(id=h, value=1))**
**n = > foreign.session_get id=`h` name="n"**
**> foreign.session_close id=`h`**
**r = answer * 100 + n**
*r*

## mark
    + `id`
    + `value`
**record = > foreign.session_eval id=`id` source={code:?} input=`value`**
**answer = > json.get value=`record` key="result"**
*answer*

## combine
    + `a`
    + `b`
    + `c`=> helper
**r = a * 1000 + b * 10 + c**

### helper
*3*

*r*

## helper
*9*
"#,
            root.join("lib/foreign.mq.md").display(),
            root.join("lib/json.mq.md").display()
        );
        both(
            &format!("foreign-arguments-{language}"),
            &source,
            Value::Int(2102321),
            "",
        );
    }
}

#[test]
fn machine_arithmetic_matches_wide_reference_and_reports_language_faults() {
    let values = [
        i64::MIN,
        i64::MIN + 1,
        -3037000500,
        -7,
        -1,
        0,
        1,
        7,
        3037000500,
        i64::MAX - 1,
        i64::MAX,
    ];
    let mut records = Vec::new();
    for left in values {
        for right in values {
            for op in ["+", "-", "*", "/"] {
                let wide = if op == "/" && right == 0 {
                    None
                } else {
                    Some(match op {
                        "+" => left as i128 + right as i128,
                        "-" => left as i128 - right as i128,
                        "*" => left as i128 * right as i128,
                        "/" => left as i128 / right as i128,
                        _ => unreachable!(),
                    })
                };
                let expected = wide.and_then(|n| i64::try_from(n).ok());
                let fault = if op == "/" && right == 0 {
                    "division by zero"
                } else {
                    "integer overflow"
                };
                let source = format!("# main\n**answer = ({left}) {op} ({right})**\n*answer*\n");
                let (path, module) = document("machine-ints", &source);
                let mut tree = Interpreter::with_capture(Some(&path), false);
                let result = tree.run_module(&module);
                let program = compile_module(Some(&path), &module).unwrap();
                let mut vm = Vm::with_capture(Some(&path));
                let result_vm = vm.run(&program);
                let encode = |result: anyhow::Result<Value>| match result {
                    Ok(value) => {
                        assert_eq!(Some(value.clone()), expected.map(Value::Int), "{source}");
                        serde_json::json!({"value":local_json(&value)})
                    }
                    Err(error) => {
                        assert!(expected.is_none(), "{source}: {error}");
                        assert!(error.to_string().contains(fault), "{source}: {error}");
                        serde_json::json!({"error":error.to_string()})
                    }
                };
                records.push(serde_json::json!({"left":left,"right":right,"op":op,"tree":encode(result),"bytecode":encode(result_vm)}));
            }
        }
    }
    for value in values {
        let source = format!("# main\n**answer = -({value})**\n*answer*\n");
        let (path, module) = document("machine-negation", &source);
        let mut tree = Interpreter::with_capture(Some(&path), false);
        let program = compile_module(Some(&path), &module).unwrap();
        let mut vm = Vm::with_capture(Some(&path));
        for result in [tree.run_module(&module), vm.run(&program)] {
            if value == i64::MIN {
                assert!(result.unwrap_err().to_string().contains("integer overflow"));
            } else {
                assert_eq!(result.unwrap(), Value::Int((-(value as i128)) as i64));
            }
        }
    }
    for value in [
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        9223372036854775808.0,
    ] {
        assert!(crate::builtin::builtin_int(&Value::Num(value))
            .unwrap_err()
            .contains("64-bit"));
    }
    assert_eq!(
        crate::builtin::builtin_int(&Value::Num(-9223372036854775808.0)).unwrap(),
        i64::MIN
    );
    if let Some(out) = std::env::var_os("MARQDO_FRONTEND_DIR") {
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(
            out.join("machine-arithmetic.json"),
            serde_json::to_vec_pretty(&serde_json::json!({"version":1,"cases":records})).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn source_bound_calls_preserve_the_selected_declaration() {
    let source = "# A\n## read\n*1*\n\n## read\n*2*\n";
    let (path, module) = document("source-bound-duplicates", source);
    assert_eq!(module.functions[0].children.len(), 2);
    let mut rows = Vec::new();
    for (index, fun) in module.functions[0].children.iter().enumerate() {
        let expected = Value::Int(index as i64 + 1);
        let mut observed = Vec::new();
        for backend in [crate::Backend::Tree, crate::Backend::Bytecode] {
            let (value, stdout) = crate::call::execute_source_function(
                &path,
                fun.span.line,
                "A.read",
                &[],
                crate::host::HostCaps::default(),
                backend,
                false,
            )
            .unwrap();
            assert_eq!(value, expected);
            assert!(stdout.is_empty());
            observed.push(local_json(&value));
        }
        rows.push(
            serde_json::json!({"line":fun.span.line,"tree":observed[0],"bytecode":observed[1]}),
        );
    }
    write_extra_evidence(
        "source-bound-duplicates",
        serde_json::json!({"source":source,"calls":rows}),
    );
}

fn write_extra_evidence(name: &str, value: serde_json::Value) {
    if let Some(out) = std::env::var_os("MARQDO_FRONTEND_DIR") {
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(
            out.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn all_four_contract_boundaries_preserve_diagnostics() {
    let mut rows = Vec::new();
    for (fixture, code) in [
        ("arg_mismatch", "contract.arg_mismatch"),
        ("return_mismatch", "contract.return_mismatch"),
        ("field_mismatch", "contract.field_mismatch"),
        ("unknown_key", "contract.unknown_key"),
    ] {
        let source = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/contracts")
                .join(format!("{fixture}.mq.md")),
        )
        .unwrap();
        let (path, module) = document(fixture, &source);
        let mut tree = Interpreter::with_capture(Some(&path), false);
        let tree_error = tree.run_module(&module).unwrap_err();
        let program = compile_module(Some(&path), &module).unwrap();
        let mut vm = Vm::with_capture(Some(&path));
        let vm_error = vm.run(&program).unwrap_err();
        let t = crate::diagnostics::Diagnostic::find(&tree_error)
            .unwrap()
            .to_json();
        let v = crate::diagnostics::Diagnostic::find(&vm_error)
            .unwrap()
            .to_json();
        assert_eq!(t["code"], code);
        assert_eq!(v["code"], code);
        for key in ["span", "doc_anchor", "doc_quote", "contract_ref"] {
            assert_eq!(t[key], v[key], "{fixture}: {key}");
        }
        assert!(tree.captured_stdout.is_empty());
        assert!(vm.captured_stdout.is_empty());
        rows.push(serde_json::json!({"fixture":fixture,"source":source,"tree":t,"bytecode":v,"tree_stdout":tree.captured_stdout,"bytecode_stdout":vm.captured_stdout}));
    }
    // Invalid defaults must be checked before any authored body effect.
    let source="# main\n**answer = > choose**\n*answer*\n\n## choose\n    + `n`=\"bad\"\n\n| 参数 | 类型 | 说明 |\n|------|------|------|\n| n | number | 输入 |\n\n> print text=\"BODY\"\n*n*\n";
    let (path, module) = document("default-contract", source);
    for bytecode in [false, true] {
        let (error, stdout) = if bytecode {
            let program = compile_module(Some(&path), &module).unwrap();
            let mut vm = Vm::with_capture(Some(&path));
            (vm.run(&program).unwrap_err(), vm.captured_stdout)
        } else {
            let mut tree = Interpreter::with_capture(Some(&path), false);
            (tree.run_module(&module).unwrap_err(), tree.captured_stdout)
        };
        assert_eq!(
            crate::diagnostics::Diagnostic::find(&error)
                .unwrap()
                .to_json()["code"],
            "contract.arg_mismatch"
        );
        assert!(stdout.is_empty());
    }
    write_extra_evidence("contract-boundaries", serde_json::json!({"cases":rows}));
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/contracts/ok_contract.mq.md"),
    )
    .unwrap();
    both("valid-contract", &source, Value::Int(4), "");
}

#[test]
fn duplicate_actuals_fault_after_evaluation_and_only_when_reached() {
    both(
        "unreached-duplicate",
        "# main\n*7*\n\n## unused\n> print text=\"a\" text=\"b\"\n",
        Value::Int(7),
        "",
    );
    let source="# main\n**answer = combine(a=mark(1), a=mark(2), b=mark(3))**\n> print text=\"SUFFIX\"\n*answer*\n\n## combine\n    + `a`\n    + `b`\n*a*\n\n## mark\n    + `value`\n> print text=`value`\n*value*\n";
    let (path, module) = document("duplicate-arguments", source);
    let mut tree = Interpreter::with_capture(Some(&path), false);
    let t = tree.run_module(&module).unwrap_err();
    let program = compile_module(Some(&path), &module).unwrap();
    let mut vm = Vm::with_capture(Some(&path));
    let v = vm.run(&program).unwrap_err();
    assert!(t.to_string().contains("duplicate named argument `a`"));
    assert!(v.to_string().contains("duplicate named argument `a`"));
    assert_eq!(tree.captured_stdout, "1\n2\n3\n");
    assert_eq!(vm.captured_stdout, tree.captured_stdout);
    write_extra_evidence(
        "duplicate-arguments",
        serde_json::json!({"source":source,"tree_error":t.to_string(),"bytecode_error":v.to_string(),"tree_stdout":tree.captured_stdout,"bytecode_stdout":vm.captured_stdout}),
    );
}

#[test]
fn imported_contract_blame_separates_call_site_and_declaration() {
    let mut rows = Vec::new();
    for omitted in [false, true] {
        let (path, _) = document(
            if omitted {
                "import-contract-default"
            } else {
                "import-contract-supplied"
            },
            "# main\n*0*\n",
        );
        let library="## check\n    + `n`=\"bad\"\n\n| 参数 | 类型 | 说明 |\n|------|------|------|\n| n | number | 输入 |\n\n> print text=\"BODY\"\n*n*\n";
        let library_path = path.parent().unwrap().join("api.mq.md");
        std::fs::write(&library_path, library).unwrap();
        let source = format!(
            "---\nimport:\n  api: api.mq.md\n---\n# main\n**answer = > api.check{}**\n*answer*\n",
            if omitted { "" } else { " n=\"bad\"" }
        );
        std::fs::write(&path, &source).unwrap();
        let module = load_module(&path).unwrap();
        let mut tree = Interpreter::with_capture(Some(&path), false);
        let te = tree.run_module(&module).unwrap_err();
        let program = compile_module(Some(&path), &module).unwrap();
        let mut vm = Vm::with_capture(Some(&path));
        let ve = vm.run(&program).unwrap_err();
        let t = crate::diagnostics::Diagnostic::find(&te).unwrap().to_json();
        let v = crate::diagnostics::Diagnostic::find(&ve).unwrap().to_json();
        for diag in [&t, &v] {
            assert_eq!(diag["code"], "contract.arg_mismatch");
            assert_eq!(diag["span"]["line"], 6);
            assert_eq!(diag["span"]["file"], path.to_str().unwrap());
            assert_eq!(
                diag["doc_anchor"],
                format!("{}#L4-L6", library_path.display())
            );
            assert!(diag["suggestion"]
                .as_str()
                .unwrap()
                .contains(&format!("{}#L4-L6", library_path.display())));
        }
        assert!(tree.captured_stdout.is_empty() && vm.captured_stdout.is_empty());
        rows.push(serde_json::json!({"omitted":omitted,"source":source,"library":library,"tree":t,"bytecode":v,"tree_stdout":tree.captured_stdout,"bytecode_stdout":vm.captured_stdout}));
    }
    write_extra_evidence("imported-contracts", serde_json::json!({"cases":rows}));
}

#[test]
fn generated_default_masks_and_declaration_orders_preserve_effects() {
    let names = ["a", "b", "c"];
    let mut rows = Vec::new();
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        for mask in 0..27usize {
            let states = [mask % 3, (mask / 3) % 3, (mask / 9) % 3];
            let args = (0..3)
                .filter_map(|i| match states[i] {
                    0 => None,
                    1 => Some(format!("{}={}", names[i], (i + 1) * 10)),
                    _ => Some(format!("{}=None", names[i])),
                })
                .collect::<Vec<_>>()
                .join(" ");
            let params = order
                .iter()
                .map(|i| {
                    format!(
                        "    + `{}`=> mark value={}\n",
                        names[*i],
                        match i {
                            0 => "1",
                            1 => "`a`",
                            _ => "`b`",
                        }
                    )
                })
                .collect::<String>();
            let source=format!("# main\n**answer = > choose {args}**\n*answer*\n\n## choose\n{params}\n**out = str(a) + \"|\" + str(b) + \"|\" + str(c)**\n*out*\n\n## mark\n    + `value`\n> print text=`value`\n*value*\n");
            let (path, module) = document("generated-defaults", &source);
            let mut tree = Interpreter::with_capture(Some(&path), false);
            let tr = tree.run_module(&module);
            let program = compile_module(Some(&path), &module).unwrap();
            let mut vm = Vm::with_capture(Some(&path));
            let vr = vm.run(&program);
            let encode = |r: anyhow::Result<Value>, stdout: String| match r {
                Ok(value) => serde_json::json!({"value":local_json(&value),"stdout":stdout}),
                Err(error) => serde_json::json!({"error":error.to_string(),"stdout":stdout}),
            };
            rows.push(serde_json::json!({"order":order,"states":states,"source":source,"tree":encode(tr,tree.captured_stdout),"bytecode":encode(vr,vm.captured_stdout)}));
        }
    }
    write_extra_evidence("generated-defaults", serde_json::json!({"cases":rows}));
}

#[test]
fn flat_bytecode_validation_rejects_corruption_before_effects() {
    use crate::bytecode::Op;
    let (path, module) = document(
        "flat-corruptions",
        "# main\n> print text=\"BEFORE\"\n*7*\n\n## f\n    + `n`=3\n*n*\n",
    );
    let program = compile_module(Some(&path), &module).unwrap();
    let certs = program.verify().unwrap();
    assert_eq!(certs.len(), program.functions.len());
    let main = program.entry;
    let mutations: Vec<(&str, Box<dyn Fn(&mut crate::bytecode::Program)>)> = vec![
        (
            "constant",
            Box::new(move |p| p.functions[main].code[0] = Op::Constant(usize::MAX)),
        ),
        (
            "local",
            Box::new(move |p| p.functions[main].code[0] = Op::GetLocal(usize::MAX)),
        ),
        (
            "jump",
            Box::new(move |p| p.functions[main].code[0] = Op::Jump(usize::MAX)),
        ),
        (
            "underflow",
            Box::new(move |p| p.functions[main].code[0] = Op::Add),
        ),
        (
            "callee",
            Box::new(move |p| p.functions[main].code[0] = Op::Call(usize::MAX, 0)),
        ),
        (
            "layout",
            Box::new(move |p| p.functions[main].code[0] = Op::CallArgs(main, usize::MAX)),
        ),
        (
            "presence",
            Box::new(move |p| p.functions[main].code[0] = Op::JumpIfProvided(usize::MAX, 1)),
        ),
        (
            "span",
            Box::new(move |p| {
                p.functions[main].spans.pop();
            }),
        ),
        (
            "duplicate-slot",
            Box::new(move |p| {
                p.functions[main].locals = vec!["x".into(), "x".into()];
            }),
        ),
        (
            "parameter-prefix",
            Box::new(move |p| {
                let f = p.functions.iter_mut().find(|f| f.name == "f").unwrap();
                f.locals[0] = "wrong".into();
            }),
        ),
    ];
    let mut cycle = program.clone();
    let child = cycle.functions.iter().position(|f| f.name == "f").unwrap();
    cycle.parents[main] = Some(child);
    cycle.children[child].push(main);
    assert!(cycle.verify().unwrap_err().contains("cyclic"));
    let mut rows = Vec::new();
    for (name, mutate) in mutations {
        let mut corrupted = program.clone();
        mutate(&mut corrupted);
        let error = corrupted.verify().unwrap_err();
        let mut vm = Vm::with_capture(Some(&path));
        let runtime = vm.run(&corrupted).unwrap_err();
        assert!(vm.captured_stdout.is_empty(), "{name}");
        assert!(
            runtime.to_string().contains("bytecode") || runtime.to_string().contains("invalid")
        );
        rows.push(serde_json::json!({"mutation":name,"error":error,"stdout":vm.captured_stdout}));
    }
    write_extra_evidence("flat-corruptions", serde_json::json!({"cases":rows}));
}

#[test]
fn bare_calls_use_the_source_resolver_declaration_identity() {
    both(
        "first-bare-declaration",
        "## pick\n*1*\n\n## pick\n*2*\n\n# main\n**answer = > pick**\n*answer*\n",
        Value::Int(1),
        "",
    );
}

#[test]
fn semantic_emission_checks_reject_stack_preserving_corruption() {
    both("short-circuit-effects","# main\n**a = False and mark()**\n**b = True or mark()**\n**c = True and mark()**\n**d = False or mark()**\n*b and c and d*\n\n## mark\n> print text=\"RHS\"\n*True*\n",Value::Bool(true),"RHS\nRHS\n");
    use crate::bytecode::Op;
    let (path, module) = document(
        "semantic-corruptions",
        "# main\n**a = 7**\n**b = 3**\n**answer = a + b**\n> print text=\"AFTER\"\n*answer*\n",
    );
    let original = compile_module(Some(&path), &module).unwrap();
    let mut rows = Vec::new();
    for mutation in ["operator", "constant", "variable", "operand_order"] {
        let mut p = original.clone();
        let f = &mut p.functions[p.entry];
        match mutation {
            "operator" => *f.code.iter_mut().find(|op| matches!(op, Op::Add)).unwrap() = Op::Sub,
            "constant" => {
                *f.constants
                    .iter_mut()
                    .find(|v| matches!(v, Value::Int(7)))
                    .unwrap() = Value::Int(8)
            }
            "variable" => {
                let a = f.locals.iter().position(|n| n == "a").unwrap();
                let b = f.locals.iter().position(|n| n == "b").unwrap();
                *f.code
                    .iter_mut()
                    .find(|op| **op == Op::GetLocal(a))
                    .unwrap() = Op::GetLocal(b);
            }
            _ => {
                let e = f
                    .expressions
                    .iter_mut()
                    .find(|e| {
                        matches!(
                            e.rule,
                            crate::bytecode::ExpressionRule::Binary(crate::ast::BinaryOp::Add)
                        )
                    })
                    .unwrap();
                e.children.swap(0, 1);
            }
        }
        let error = p.verify().unwrap_err();
        let mut vm = Vm::with_capture(Some(&path));
        assert!(vm.run(&p).is_err());
        assert!(vm.captured_stdout.is_empty());
        rows.push(
            serde_json::json!({"mutation":mutation,"error":error,"stdout":vm.captured_stdout}),
        );
    }
    for (name, source) in [
        ("unary", "# main\n**a = 7**\n**answer = -a**\n*answer*\n"),
        (
            "short_circuit",
            "# main\n**answer = False and True**\n*answer*\n",
        ),
    ] {
        let (path, module) = document(name, source);
        let mut p = compile_module(Some(&path), &module).unwrap();
        let f = &mut p.functions[p.entry];
        if name == "unary" {
            *f.code
                .iter_mut()
                .find(|op| matches!(op, Op::Negate))
                .unwrap() = Op::Not;
        } else {
            let op = f
                .code
                .iter_mut()
                .find(|op| matches!(op, Op::JumpIfFalse(_)))
                .unwrap();
            if let Op::JumpIfFalse(t) = op {
                *t -= 1;
            }
        }
        let error = p.verify().unwrap_err();
        rows.push(serde_json::json!({"mutation":name,"error":error,"stdout":""}));
    }
    write_extra_evidence("semantic-corruptions", serde_json::json!({"cases":rows}));
}

#[test]
fn scalar_addition_and_interpolation_preserve_source_types() {
    let mut rows = Vec::new();
    for expression in ["7 + True", "None + 7", "7 + False"] {
        let source = format!("# main\n**answer = {expression}**\n*answer*\n");
        let (path, module) = document("scalar-source-types", &source);
        let mut tree = Interpreter::with_capture(Some(&path), false);
        let tr = tree.run_module(&module).unwrap_err();
        let program = compile_module(Some(&path), &module).unwrap();
        let mut vm = Vm::with_capture(Some(&path));
        let vr = vm.run(&program).unwrap_err();
        assert!(tr.to_string().contains("`+` needs ints, nums, or text"));
        assert!(vr.to_string().contains("`+` needs ints, nums, or text"));
        rows.push(serde_json::json!({"expression":expression,"source":source,"tree_error":tr.to_string(),"bytecode_error":vr.to_string()}));
    }
    let (path, mut module) = document("adjacent-interpolation-ast", "# main\n*0*\n");
    module.metadata = vec![("a".into(), Value::Int(7)), ("b".into(), Value::Int(3))];
    module.functions[0].body[0] = crate::ast::Stmt::Return {
        value: crate::ast::Expr::Interp(vec![
            crate::ast::InterpPart::Var("a".into()),
            crate::ast::InterpPart::Var("b".into()),
        ]),
        span: crate::diagnostics::Span::new(2, 1),
    };
    let mut tree = Interpreter::with_capture(Some(&path), false);
    let tr = tree.run_module(&module).unwrap();
    let program = compile_module(Some(&path), &module).unwrap();
    let mut vm = Vm::with_capture(Some(&path));
    let vr = vm.run(&program).unwrap();
    assert_eq!(tr, Value::Text("73".into()));
    assert_eq!(vr, tr);
    write_extra_evidence(
        "primitive-regressions",
        serde_json::json!({"cases":rows,"interpolation_ast":{"tree":local_json(&tr),"bytecode":local_json(&vr)}}),
    );
}

#[test]
fn nested_foreach_uses_fresh_slots_and_preserves_authored_names() {
    let source="# main\n`xs` =\n| value |\n|---|\n| 1 |\n| 2 |\n\n**__i_xs = 99**\n**total = 0**\n- [x](遍历 xs 为 x)\n  - [y](遍历 xs 为 y)\n    **total = total + 1**\n\n**answer = __i_xs * 10 + total**\n*answer*\n";
    both("nested-loop-hygiene", source, Value::Int(994), "");
}

#[test]
fn statement_and_prologue_changes_are_rejected_before_effects() {
    use crate::bytecode::Op;
    let source="# main\n**answer = > choose**\n> print text=\"AFTER\"\n*answer*\n\n## choose\n    + `n`=7\n**value = n + 1**\n*value*\n";
    let (path, module) = document("statement-corruptions", source);
    let p = compile_module(Some(&path), &module).unwrap();
    let fid = p.functions.iter().position(|f| f.name == "choose").unwrap();
    let mut rows = Vec::new();
    for mutation in [
        "wrong_default_target",
        "missing_fill",
        "wrong_assignment",
        "wrong_return",
        "extra_instruction",
    ] {
        let mut bad = p.clone();
        let f = &mut bad.functions[fid];
        match mutation {
            "wrong_default_target" => {
                if let Op::JumpIfProvided(_, target) = &mut f.code[0] {
                    *target = f.body_start;
                }
            }
            "missing_fill" => {
                let pc = f
                    .code
                    .iter()
                    .position(|op| matches!(op, Op::FillMissingParameters))
                    .unwrap();
                f.code[pc] = Op::CheckArguments;
            }
            "wrong_assignment" => {
                let value = f.locals.iter().position(|n| n == "value").unwrap();
                let n = f.locals.iter().position(|n| n == "n").unwrap();
                *f.code
                    .iter_mut()
                    .find(|op| **op == Op::SetLocal(value))
                    .unwrap() = Op::SetLocal(n);
            }
            "wrong_return" => {
                let pc = f
                    .code
                    .iter()
                    .position(|op| matches!(op, Op::Return))
                    .unwrap();
                f.code[pc] = Op::Pop;
            }
            _ => {
                f.code.push(Op::None_);
                f.spans.push(f.declaration_span);
            }
        }
        let error = bad.verify().unwrap_err();
        let mut vm = Vm::with_capture(Some(&path));
        assert!(vm.run(&bad).is_err());
        assert!(vm.captured_stdout.is_empty());
        rows.push(
            serde_json::json!({"mutation":mutation,"error":error,"stdout":vm.captured_stdout}),
        );
    }
    write_extra_evidence("statement-corruptions", serde_json::json!({"cases":rows}));
}

#[test]
fn while_iteration_limit_is_the_same_language_fault_on_both_backends() {
    both("while-condition-effects", "# main\n**n = 2**\n- test(value=`n`)\n  > print text=\"BODY\"\n  **n = n - 1**\n*n*\n\n## test\n    + `value`\n> print text=`value`\n**answer = value > 0**\n*answer*\n",Value::Int(0),"2\nBODY\n1\nBODY\n0\n");
    both(
        "while-early-return",
        "# main\n- True\n  *7*\n> print text=\"UNREACHABLE\"\n*0*\n",
        Value::Int(7),
        "",
    );
    both(
        "while-normal",
        "# main\n**n = 3**\n- `n` > 0\n  **n = n - 1**\n*n*\n",
        Value::Int(0),
        "",
    );
    let (path, mut module) = document("while-limit-ast", "# main\n*0*\n");
    module.functions[0].body = vec![crate::ast::Stmt::While {
        condition: crate::ast::Expr::Literal(crate::ast::Literal::Bool(true)),
        body: Vec::new(),
        span: crate::diagnostics::Span::new(2, 1),
    }];
    let mut tree = Interpreter::new(Some(&path), false);
    let tr = tree.run_module(&module).unwrap_err();
    let program = compile_module(Some(&path), &module).unwrap();
    let mut vm = Vm::new(Some(&path));
    let vr = vm.run(&program).unwrap_err();
    assert!(tr
        .to_string()
        .contains("while loop exceeded iteration limit"));
    assert!(vr
        .to_string()
        .contains("while loop exceeded iteration limit"));
    for error in [&tr, &vr] {
        assert_eq!(
            crate::diagnostics::Diagnostic::find(error)
                .unwrap()
                .span
                .line,
            2
        );
    }
    write_extra_evidence(
        "while-limit",
        serde_json::json!({"limit":1000000,"tree_error":tr.to_string(),"bytecode_error":vr.to_string(),"line":2,"input":"direct AST with literal-True empty while"}),
    );
}

#[test]
fn dynamic_calls_keep_the_calling_module_scope() {
    let (path, _) = document("dynamic-local-scope", "# main\n*0*\n");
    std::fs::write(path.parent().unwrap().join("a.mq.md"), "## hidden\n*99*\n").unwrap();
    let source="---\nimport:\n  a: a.mq.md\n---\n## hidden\n*7*\n\n# main\n**answer = > call_fn name=\"hidden\"**\n*answer*\n";
    both("dynamic-local-scope", source, Value::Int(7), "");
    let denied="---\nimport:\n  a: a.mq.md\n---\n# main\n**answer = > call_fn name=\"hidden\"**\n*answer*\n";
    std::fs::write(&path, denied).unwrap();
    let module = load_module(&path).unwrap();
    let mut tree = Interpreter::with_capture(Some(&path), false);
    let tr = tree.run_module(&module).unwrap_err();
    let program = compile_module(Some(&path), &module).unwrap();
    let mut vm = Vm::with_capture(Some(&path));
    let vr = vm.run(&program).unwrap_err();
    assert!(tr.to_string().contains("hidden"));
    assert!(vr.to_string().contains("hidden"));
    assert!(tree.captured_stdout.is_empty() && vm.captured_stdout.is_empty());
    write_extra_evidence(
        "dynamic-import-refused",
        serde_json::json!({"source":denied,"tree_error":tr.to_string(),"bytecode_error":vr.to_string(),"tree_stdout":tree.captured_stdout,"bytecode_stdout":vm.captured_stdout}),
    );
}

#[test]
fn receiver_alias_defaults_use_logical_presence_and_restore_the_body_receiver() {
    let method="## mark\n> print text=\"DEFAULT\"\n*9*\n\n# Box\n## m\n    + `self`=> mark\n    + `x`=`self`\n**answer = type(value=self) + \"|\" + type(value=x)**\n*answer*\n\n# main\n**box = > Box**\n";
    for (id, args, value, stdout) in [
        ("receiver-alias-omitted", "", "map|int", "DEFAULT\n"),
        ("receiver-alias-none", " self=None", "map|map", ""),
    ] {
        let source = format!("{method}**answer = > `box`.m{args}**\n*answer*\n");
        both(id, &source, Value::Text(value.into()), stdout);
    }
    let mut cases = Vec::new();
    for (argument, expect_call_site) in
        [("", false), (" self=None", false), (" self=\"bad\"", true)]
    {
        let source=format!("## mark\n*9*\n\n# Box\n## m\n    + `self`=> mark\n    + `x`=`self`\n\n| 参数 | 类型 |\n|---|---|\n| self | number |\n\n*x*\n\n# main\n**box = > Box**\n**answer = > `box`.m{argument}**\n*answer*\n");
        let (path, module) = document("receiver-alias-contract", &source);
        let mut tree = Interpreter::with_capture(Some(&path), false);
        let te = tree.run_module(&module).unwrap_err();
        let program = compile_module(Some(&path), &module).unwrap();
        let mut vm = Vm::with_capture(Some(&path));
        let ve = vm.run(&program).unwrap_err();
        let t = crate::diagnostics::Diagnostic::find(&te).unwrap().to_json();
        let v = crate::diagnostics::Diagnostic::find(&ve).unwrap().to_json();
        assert_eq!(t["code"], "contract.arg_mismatch");
        assert_eq!(v["code"], t["code"]);
        assert_eq!(t["span"], v["span"]);
        assert_eq!(t["doc_anchor"], v["doc_anchor"]);
        assert_eq!(t["source_sha256"], v["source_sha256"]);
        assert_eq!(t["frames"], v["frames"]);
        let line = if expect_call_site { 17 } else { 5 };
        assert_eq!(t["span"]["line"], line);
        cases.push(serde_json::json!({"argument":argument,"source":source,"tree":t,"bytecode":v,"expected_line":line}));
    }
    write_extra_evidence(
        "receiver-alias-contracts",
        serde_json::json!({"cases":cases}),
    );
}

#[test]
fn call_staging_checks_reject_layout_target_and_scratch_corruption() {
    both("unreached-plugin", "# main\n*7*\n\n## unused_named\n> unknown_plugin z=7 a=3\n\n## unused_positional\n> unknown_plugin 9 3\n> input\n",Value::Int(7),"");
    use crate::bytecode::{CallDispatch, Op};
    let source="# main\n> print text=\"PROBE\"\n**answer = combine(b=mark(2), a=mark(1))**\n*answer*\n\n## combine\n    + `a`\n    + `b`\n**n = a * 10 + b**\n*n*\n\n## mark\n    + `value`\n*value*\n";
    let (path, module) = document("call-corruption", source);
    let original = compile_module(Some(&path), &module).unwrap();
    let entry = original.entry;
    let call = original.functions[entry]
        .calls
        .iter()
        .position(|c| c.source.callee == "combine")
        .unwrap();
    let target = original.functions[entry]
        .calls
        .iter()
        .find_map(|c| {
            if c.source.callee == "mark" {
                match c.dispatch {
                    Some(CallDispatch::User { target, .. }) => Some(target),
                    _ => None,
                }
            } else {
                None
            }
        })
        .unwrap();
    let mut cases = Vec::new();
    for mutation in [
        "target",
        "layout",
        "reload",
        "scratch_store",
        "source_alias",
        "nested_alias",
        "actual_order",
        "uncovered_call",
    ] {
        let mut bad = original.clone();
        let f = &mut bad.functions[entry];
        let record = f.calls[call].clone();
        let node = &f.expressions[record.expression];
        match mutation {
            "target" => {
                let pc = (node.start..node.end)
                    .rev()
                    .find(|i| matches!(f.code[*i], Op::CallArgs(_, _)))
                    .unwrap();
                let Op::CallArgs(_, layout) = f.code[pc] else {
                    unreachable!()
                };
                f.code[pc] = Op::CallArgs(target, layout);
            }
            "layout" => {
                let Op::CallArgs(_, layout) = f.code[node.end - 1] else {
                    unreachable!()
                };
                f.call_layouts[layout].swap(0, 1);
            }
            "reload" => {
                let id = node.children[record.stages.len()];
                let pc = f.expressions[id].start;
                let other = record.stages[1].slot;
                f.code[pc] = Op::GetLocal(other);
                f.expressions[id].rule =
                    crate::bytecode::ExpressionRule::Read(f.locals[other].clone());
            }
            "scratch_store" => {
                let pc = f.expressions[record.stages[0].expression].end;
                f.code[pc] = Op::SetLocal(record.stages[1].slot);
            }
            "source_alias" => {
                f.calls[call].stages[0].slot = 0;
            }
            "nested_alias" => {
                let nested = f
                    .calls
                    .iter()
                    .position(|c| c.source.callee == "mark")
                    .unwrap();
                f.calls[nested].stages[0].slot = record.stages[0].slot;
            }
            "actual_order" => {
                f.calls[call].stages.swap(0, 1);
            }
            "uncovered_call" => {
                f.calls.remove(call);
            }
            _ => unreachable!(),
        }
        let mut vm = Vm::with_capture(Some(&path));
        let error = vm.run(&bad).unwrap_err().to_string();
        assert!(
            error.contains("bytecode verification"),
            "{mutation}: {error}"
        );
        assert_eq!(vm.captured_stdout, "");
        cases.push(
            serde_json::json!({"mutation":mutation,"error":error,"stdout":vm.captured_stdout}),
        );
    }
    for (mutation,source) in [
        ("host_identity","# main\n> print text=\"PROBE\"\n**answer = host_exists(path=\"unused\")**\n*answer*\n"),
        ("method_name","# Box\n## m\n*7*\n\n# main\n> print text=\"PROBE\"\n**box = > Box**\n**answer = > `box`.m**\n*answer*\n"),
        ("plugin_name","# main\n> print text=\"PROBE\"\n**answer = > unknown_plugin x=7**\n*answer*\n"),
    ] {
        let (path,module)=document(mutation,source);let mut bad=compile_module(Some(&path),&module).unwrap();let f=&mut bad.functions[bad.entry];
        for op in &mut f.code {
            match op {
                Op::HostCallArgs(id,_) if mutation=="host_identity"=>{*id=5;break;},
                Op::MethodCallArgs(id,_) if mutation=="method_name"=>{let index=f.constants.len();f.constants.push(Value::Text("other_method".into()));*id=index;break;},
                Op::PluginCallArgs(id,_) if mutation=="plugin_name"=>{let index=f.constants.len();f.constants.push(Value::Text("@other_plugin".into()));*id=index;break;},
                _=>{},
            }
        }
        let mut vm=Vm::with_capture(Some(&path));let error=vm.run(&bad).unwrap_err().to_string();assert!(error.contains("bytecode verification"),"{mutation}: {error}");assert_eq!(vm.captured_stdout,"");
        cases.push(serde_json::json!({"mutation":mutation,"error":error,"stdout":vm.captured_stdout}));
    }
    write_extra_evidence("call-corruptions", serde_json::json!({"cases":cases}));
}

#[test]
fn loaded_ast_refuses_coherently_forged_emission_witnesses() {
    use crate::bytecode::{CallDispatch, ExpressionRule, Op, StatementRule};
    let (path, module) = document(
        "source-witness-corruption",
        "# main\n> print text=\"PROBE\"\n**n = 7**\n**m = 3**\n**result = n + m**\n*result*\n",
    );
    let original = compile_module(Some(&path), &module).unwrap();
    let mut cases = Vec::new();
    for mutation in [
        "operator_and_rule",
        "literal_and_rule",
        "variable_and_rule",
        "assignment_and_rule",
        "orphan_expression",
    ] {
        let mut bad = original.clone();
        let f = &mut bad.functions[bad.entry];
        match mutation {
            "operator_and_rule" => {
                let e = f
                    .expressions
                    .iter_mut()
                    .find(|e| matches!(e.rule, ExpressionRule::Binary(crate::ast::BinaryOp::Add)))
                    .unwrap();
                e.rule = ExpressionRule::Binary(crate::ast::BinaryOp::Sub);
                f.code[e.end - 1] = Op::Sub;
            }
            "literal_and_rule" => {
                let e = f
                    .expressions
                    .iter_mut()
                    .find(|e| matches!(e.rule, ExpressionRule::Literal(Value::Int(7))))
                    .unwrap();
                let Op::Constant(index) = f.code[e.start] else {
                    unreachable!()
                };
                f.constants[index] = Value::Int(9);
                e.rule = ExpressionRule::Literal(Value::Int(9));
            }
            "variable_and_rule" => {
                let e = f
                    .expressions
                    .iter_mut()
                    .find(|e| matches!(&e.rule,ExpressionRule::Read(name) if name=="result"))
                    .unwrap();
                let slot = f.locals.iter().position(|name| name == "m").unwrap();
                e.rule = ExpressionRule::Read("m".into());
                f.code[e.start] = Op::GetLocal(slot);
            }
            "assignment_and_rule" => {
                let s = f
                    .statements
                    .iter_mut()
                    .find(|s| matches!(&s.rule,StatementRule::Assign(name) if name=="n"))
                    .unwrap();
                let slot = f.locals.iter().position(|name| name == "m").unwrap();
                s.rule = StatementRule::Assign("m".into());
                f.code[s.end - 2] = Op::SetLocal(slot);
            }
            "orphan_expression" => {
                f.expressions.push(
                    f.expressions
                        .iter()
                        .find(|e| matches!(e.rule, ExpressionRule::Literal(Value::Int(7))))
                        .unwrap()
                        .clone(),
                );
            }
            _ => unreachable!(),
        }
        let mut vm = Vm::with_capture(Some(&path));
        let error = vm.run(&bad).unwrap_err().to_string();
        assert!(error.contains("AST"), "{mutation}: {error}");
        assert_eq!(vm.captured_stdout, "");
        cases.push(
            serde_json::json!({"mutation":mutation,"error":error,"stdout":vm.captured_stdout}),
        );
    }
    let (path,module)=document("source-call-witness-corruption","# main\n> print text=\"PROBE\"\n**answer = f(value=7)**\n*answer*\n\n## f\n    + `value`\n*7*\n\n## g\n    + `value`\n*3*\n");
    let mut bad = compile_module(Some(&path), &module).unwrap();
    let target = bad.functions.iter().position(|f| f.name == "g").unwrap();
    let f = &mut bad.functions[bad.entry];
    let call = f.calls.iter_mut().find(|c| c.source.callee == "f").unwrap();
    call.source.callee = "g".into();
    call.dispatch = Some(CallDispatch::User {
        target,
        tag_instance: false,
    });
    let node = &f.expressions[call.expression];
    let Op::CallArgs(_, layout) = f.code[node.end - 1] else {
        unreachable!()
    };
    f.code[node.end - 1] = Op::CallArgs(target, layout);
    let mut vm = Vm::with_capture(Some(&path));
    let error = vm.run(&bad).unwrap_err().to_string();
    assert!(error.contains("AST"), "{error}");
    assert_eq!(vm.captured_stdout, "");
    cases.push(serde_json::json!({"mutation":"callee_and_dispatch_witness","error":error,"stdout":vm.captured_stdout}));
    for mutation in [
        "formula_bits_and_rule",
        "index_key_and_rule",
        "interpolation_and_rule",
    ] {
        use crate::ast::{Expr, IndexKey, InterpPart, Literal, Stmt};
        let (path, mut module) = document(mutation, "# main\n*0*\n");
        let span = crate::diagnostics::Span::new(2, 1);
        let (prefix, value) = match mutation {
            "formula_bits_and_rule" => (
                Vec::new(),
                Expr::Formula(crate::formula::Expr::Num(f64::from_bits(
                    0x7ff8_0000_0000_0001,
                ))),
            ),
            "index_key_and_rule" => (
                vec![Stmt::Assign {
                    name: "m".into(),
                    value: Expr::Map(vec![
                        ("a".into(), Expr::Literal(Literal::Int(7))),
                        ("b".into(), Expr::Literal(Literal::Int(3))),
                    ]),
                    span,
                    end_line: 2,
                }],
                Expr::Index {
                    base: Box::new(Expr::Var("m".into())),
                    label: IndexKey::Lit("a".into()),
                },
            ),
            _ => (
                vec![Stmt::Assign {
                    name: "n".into(),
                    value: Expr::Literal(Literal::Int(7)),
                    span,
                    end_line: 2,
                }],
                Expr::Interp(vec![
                    InterpPart::Lit("ORIGINAL-".into()),
                    InterpPart::Var("n".into()),
                ]),
            ),
        };
        module.functions[0].body = prefix;
        module.functions[0].body.push(Stmt::Return { value, span });
        let mut bad = compile_module(Some(&path), &module).unwrap();
        let f = &mut bad.functions[bad.entry];
        match mutation {
            "formula_bits_and_rule" => {
                let e = f
                    .expressions
                    .iter_mut()
                    .find(|e| matches!(e.rule, ExpressionRule::Literal(Value::Formula(_))))
                    .unwrap();
                let value = Value::Formula(crate::formula::Expr::Num(f64::from_bits(
                    0x7ff8_0000_0000_0002,
                )));
                let Op::Constant(index) = f.code[e.start] else {
                    unreachable!()
                };
                f.constants[index] = value.clone();
                e.rule = ExpressionRule::Literal(value);
            }
            "index_key_and_rule" => {
                let id = f
                    .expressions
                    .iter()
                    .position(|e| matches!(e.rule, ExpressionRule::Index { .. }))
                    .unwrap();
                let pc = f.expressions[f.expressions[id].children[0]].end;
                let index = f.constants.len();
                f.constants.push(Value::Text("b".into()));
                f.code[pc] = Op::Constant(index);
                let ExpressionRule::Index { label, .. } = &mut f.expressions[id].rule else {
                    unreachable!()
                };
                *label = IndexKey::Lit("b".into());
            }
            _ => {
                let e = f
                    .expressions
                    .iter_mut()
                    .find(|e| matches!(e.rule, ExpressionRule::Interpolation(_)))
                    .unwrap();
                let ExpressionRule::Interpolation(parts) = &mut e.rule else {
                    unreachable!()
                };
                parts[0] = InterpPart::Lit("CHANGED-".into());
                let Op::Constant(index) = f.code[e.start] else {
                    unreachable!()
                };
                f.constants[index] = Value::Text("CHANGED-".into());
            }
        }
        let mut vm = Vm::with_capture(Some(&path));
        let error = vm.run(&bad).unwrap_err().to_string();
        assert!(error.contains("AST"), "{mutation}: {error}");
        assert_eq!(vm.captured_stdout, "");
        cases.push(serde_json::json!({"mutation":mutation,"error":error,"stdout":vm.captured_stdout,"input":"direct AST; formula NaN payload bits, static index and interpolation component identity"}));
    }
    let (path, mut module) = document("source-contract-witness-corruption", "# main\n*7*\n");
    module.functions[0].contract = Some(crate::contract::Contract {
        params: Vec::new(),
        returns: Some(crate::contract::ReturnContract {
            ty: crate::contract::Ty::Number,
            desc: "must remain numeric".into(),
        }),
        fields: Vec::new(),
        start_line: 1,
        end_line: 1,
        quote: vec!["return number".into()],
    });
    let mut bad = compile_module(Some(&path), &module).unwrap();
    bad.functions[bad.entry]
        .contract
        .as_mut()
        .unwrap()
        .returns
        .as_mut()
        .unwrap()
        .ty = crate::contract::Ty::Any;
    let mut vm = Vm::with_capture(Some(&path));
    let error = vm.run(&bad).unwrap_err().to_string();
    assert!(error.contains("AST"), "{error}");
    assert_eq!(vm.captured_stdout, "");
    cases.push(serde_json::json!({"mutation":"contract_metadata","error":error,"stdout":vm.captured_stdout,"input":"direct AST contract; preserve contract presence but weaken Number to Any"}));
    write_extra_evidence(
        "source-witness-corruptions",
        serde_json::json!({"cases":cases}),
    );
}

#[test]
fn modifier_conflicts_are_runtime_faults_before_actual_effects() {
    both(
        "unreached-modifier-conflict",
        "# main\n*7*\n\n## unused\n> 礼貌 [f] 礼貌=False\n\n## f\n*3*\n",
        Value::Int(7),
        "",
    );
    both(
        "modifier-valid",
        "# main\n**answer = 礼貌 [f]**\n*answer*\n\n## f\n    + `礼貌`\n*礼貌*\n",
        Value::Bool(true),
        "",
    );
    both(
        "modifier-alias-order",
        "# main\n> 内容 [打印] 内容=False\n*7*\n",
        Value::Int(7),
        "False\n",
    );
    let defaults =
        "## choose\n    + `value`=> 礼貌 [f] 礼貌=False\n*value*\n\n## f\n*3*\n\n# main\n";
    both(
        "modifier-default-skipped",
        &format!("{defaults}**answer = > choose value=7**\n*answer*\n"),
        Value::Int(7),
        "",
    );
    both(
        "modifier-default-none",
        &format!("{defaults}**answer = > choose value=None**\n*answer*\n"),
        Value::None,
        "",
    );
    let mut cases = Vec::new();
    for (id,source,wanted_stdout,line) in [
        ("modifier-reached","# main\n> print text=\"PREFIX\"\n**answer = 礼貌 [f] 礼貌=([mark] 2)**\n> print text=\"SUFFIX\"\n\n## mark\n    + `value`\n> print text=\"ACTUAL\"\n*value*\n\n## f\n*3*\n".to_string(),"PREFIX\n",3),
        ("modifier-default-omitted",format!("{defaults}**answer = > choose**\n*answer*\n"),"",2),
    ] {
        let (path,module)=document(id,&source);
        if id=="modifier-reached" {
            let crate::ast::Stmt::Assign{value:crate::ast::Expr::Call(call),..}=&module.functions[0].body[1] else{panic!("not a source call")};
            assert!(matches!(&call.args[0],crate::ast::Arg::Named{value:crate::ast::Expr::Call(_),..}),"probe must contain an actual effectful argument: {:?}",call.args);
        }
        let mut tree=Interpreter::with_capture(Some(&path),false);let te=tree.run_module(&module).unwrap_err();
        let program=compile_module(Some(&path),&module).unwrap();let mut vm=Vm::with_capture(Some(&path));let ve=vm.run(&program).unwrap_err();
        assert!(te.to_string().contains("pre-bracket modifier"));assert!(ve.to_string().contains("pre-bracket modifier"));
        assert_eq!(tree.captured_stdout,wanted_stdout);assert_eq!(vm.captured_stdout,wanted_stdout);
        let t=crate::diagnostics::Diagnostic::find(&te).unwrap().to_json();let v=crate::diagnostics::Diagnostic::find(&ve).unwrap().to_json();
        for key in ["code","message","span","source_sha256","frames"]{assert_eq!(t[key],v[key],"{id}: {key}");}
        assert_eq!(t["span"]["line"],line);
        cases.push(serde_json::json!({"id":id,"source":source,"tree":t,"bytecode":v,"stdout":wanted_stdout,"tree_stdout":tree.captured_stdout,"bytecode_stdout":vm.captured_stdout,"line":line}));
    }
    write_extra_evidence("modifier-faults", serde_json::json!({"cases":cases}));
}

#[test]
fn native_call_binding_faults_preserve_effects_and_diagnostic_frames() {
    let mut rows = Vec::new();
    for (id, call, trace, message) in [
        (
            "len-missing",
            "len()",
            "",
            "missing argument for parameter `value`",
        ),
        (
            "len-extra",
            "len(mark(1), mark(2))",
            "1\n2\n",
            "too many positional arguments (1 extra)",
        ),
        (
            "len-unknown-named",
            "len(other=mark(1))",
            "1\n",
            "missing argument for parameter `value`",
        ),
        (
            "str-missing",
            "str()",
            "",
            "missing argument for parameter `value`",
        ),
        (
            "print-missing",
            "print()",
            "",
            "missing argument for parameter `text`",
        ),
        (
            "split-missing",
            "split(mark(1))",
            "1\n",
            "missing argument for parameter `sep`",
        ),
        (
            "split-extra",
            "split(mark(1), mark(2), mark(3))",
            "1\n2\n3\n",
            "too many positional arguments (1 extra)",
        ),
        (
            "at-missing",
            "at(mark(1))",
            "1\n",
            "missing argument for parameter `index`",
        ),
        (
            "call-fn-missing",
            "call_fn()",
            "",
            "missing argument for parameter `name`",
        ),
        (
            "host-missing",
            "host_exists(extra=mark(1))",
            "1\n",
            "host_exists requires `path`",
        ),
        (
            "host-extra",
            "host_now_unix(mark(1))",
            "1\n",
            "too many positional arguments (1 extra)",
        ),
        (
            "plugin-mixed",
            "missing_plugin(mark(1), extra=mark(2))",
            "1\n2\n",
            "unknown function `missing_plugin`",
        ),
        (
            "unknown-path",
            "missing_lib.f(value=mark(1))",
            "1\n",
            "unresolved dynamic call `missing_lib.f`",
        ),
        (
            "unknown-deep-path",
            "missing_lib.Box.f(value=mark(1))",
            "1\n",
            "unknown `missing_lib.Box.f`",
        ),
        (
            "duplicate-builtin",
            "len(value=mark(1), value=mark(2))",
            "1\n2\n",
            "duplicate named argument `value`",
        ),
        (
            "duplicate-host",
            "host_now_unix(extra=mark(1), extra=mark(2))",
            "1\n2\n",
            "duplicate named argument `extra`",
        ),
        (
            "duplicate-unknown-plugin",
            "missing_plugin(a=mark(1), a=mark(2))",
            "1\n2\n",
            "unknown function `missing_plugin`",
        ),
        (
            "duplicate-unknown-path",
            "missing_lib.f(a=mark(1), a=mark(2))",
            "1\n2\n",
            "unresolved dynamic call `missing_lib.f`",
        ),
    ] {
        let body = format!(
            "> print text=\"PREFIX\"\n**answer = {call}**\n> print text=\"SUFFIX\"\n*answer*\n"
        );
        let mark = "\n## mark\n    + `value`\n> print text=`value`\n*value*\n";
        let unreached = format!("# main\n*7*\n\n## unused\n{body}{mark}");
        both(
            &format!("call-fault-{id}-unreached"),
            &unreached,
            Value::Int(7),
            "",
        );
        let source = format!("# main\n{body}{mark}");
        let (path, module) = document(&format!("call-fault-{id}"), &source);
        let mut tree = Interpreter::with_capture(Some(&path), false);
        let te = tree.run_module(&module).unwrap_err();
        let program = compile_module(Some(&path), &module).unwrap();
        let mut vm = Vm::with_capture(Some(&path));
        let ve = vm.run(&program).unwrap_err();
        let t = crate::diagnostics::Diagnostic::find(&te).unwrap().to_json();
        let v = crate::diagnostics::Diagnostic::find(&ve).unwrap().to_json();
        let wanted_stdout = format!("PREFIX\n{trace}");
        assert_eq!(t["message"], message, "{id}: source error");
        assert_eq!(tree.captured_stdout, wanted_stdout, "{id}: source trace");
        assert_eq!(vm.captured_stdout, wanted_stdout, "{id}: VM trace");
        for key in ["code", "message", "span", "source_sha256", "frames"] {
            assert_eq!(t[key], v[key], "{id}: {key}");
        }
        assert_eq!(t["span"]["line"], 3);
        rows.push(serde_json::json!({"id":id,"source":source,"expected_message":message,"expected_stdout":wanted_stdout,"tree":t,"bytecode":v,"tree_stdout":tree.captured_stdout,"bytecode_stdout":vm.captured_stdout}));
    }
    write_extra_evidence("call-binding-faults", serde_json::json!({"cases":rows}));
}

#[test]
fn path_dispatch_uses_runtime_presence_before_static_fallback() {
    let definitions = "# Box\n## read\n*1*\n\n# Other\n## read\n*7*\n\n";
    both(
        "path-bound-receiver",
        &format!("{definitions}# main\n**Box = > Other**\n**answer = > Box.read**\n*answer*\n"),
        Value::Int(7),
        "",
    );
    both(
        "path-unbound-fallback",
        &format!("{definitions}# main\n**answer = > Box.read**\n*answer*\n**Box = > Other**\n"),
        Value::Int(1),
        "",
    );
}

#[cfg(feature = "plugin-host")]
#[test]
fn registered_plugins_bind_mixed_arguments_after_authored_effects() {
    let source="# main\n**answer = binding_probe(mark(2), b=mark(3), ignored=mark(9))**\n*answer*\n\n## mark\n    + `value`\n> print text=`value`\n*value*\n";
    let (path, module) = document("plugin-native-binding", source);
    let host = || {
        let mut h = crate::host::HostContext::for_capture(Some(&path), Default::default());
        h.plugins.register_binding_probe();
        h
    };
    let expected = Value::Map(vec![
        ("a".into(), Value::Int(2)),
        ("b".into(), Value::Int(3)),
        ("ignored".into(), Value::Int(9)),
    ]);
    let mut tree = Interpreter::with_capture(Some(&path), false).with_host(host());
    let t = tree.run_module(&module).unwrap();
    assert_eq!(t, expected);
    assert_eq!(tree.captured_stdout, "2\n3\n9\n");
    let program = compile_module(Some(&path), &module).unwrap();
    let mut vm = Vm::with_capture(Some(&path)).with_host(host());
    let v = vm.run(&program).unwrap();
    assert_eq!(v, expected);
    assert_eq!(vm.captured_stdout, "2\n3\n9\n");
    write_extra_evidence(
        "registered-plugin-binding",
        serde_json::json!({"source":source,"expected":local_json(&expected),"tree":local_json(&t),"bytecode":local_json(&v),"tree_stdout":tree.captured_stdout,"bytecode_stdout":vm.captured_stdout,"abi":"native-json-c"}),
    );
}

#[test]
fn source_declaration_graph_rejects_coherent_target_and_order_corruption() {
    use crate::bytecode::{CallDispatch, Op, PathFallback};
    let source="# main\n> print text=\"PROBE\"\n**answer = choose()**\n*answer*\n\n## choose\n*1*\n\n## other\n*2*\n";
    both("same-signature-targets", source, Value::Int(1), "PROBE\n");
    let (path, module) = document("name-graph-corruption", source);
    let original = compile_module(Some(&path), &module).unwrap();
    let entry = original.entry;
    let other = original
        .functions
        .iter()
        .position(|f| f.name == "other")
        .unwrap();
    let mut rows = Vec::new();
    for mutation in [
        "coherent-target",
        "child-order",
        "declaration-identity",
        "scope",
        "module-header",
        "missing-root",
        "object-name",
        "method-name",
    ] {
        let mut bad = original.clone();
        match mutation {
            "coherent-target" => {
                let f = &mut bad.functions[entry];
                let c = f
                    .calls
                    .iter_mut()
                    .find(|c| c.source.callee == "choose")
                    .unwrap();
                c.dispatch = Some(CallDispatch::User {
                    target: other,
                    tag_instance: false,
                });
                let e = &f.expressions[c.expression];
                let Op::CallArgs(_, layout) = f.code[e.end - 1] else {
                    panic!("native call")
                };
                f.code[e.end - 1] = Op::CallArgs(other, layout);
            }
            "child-order" => bad.children[entry].reverse(),
            "declaration-identity" => {
                bad.functions[other].source_ast.declaration_path =
                    bad.functions[entry].source_ast.declaration_path.clone()
            }
            "scope" => bad.functions[other].module_scope = 99,
            "module-header" => bad.functions[other]
                .source_ast
                .imports
                .push("invented".into()),
            "missing-root" => bad.functions[entry].source_ast.module_root_count += 1,
            "object-name" => bad.objects[0].0 = "invented".into(),
            "method-name" => bad.methods[0].1 = "invented".into(),
            _ => unreachable!(),
        }
        let rejection = bad.verify().unwrap_err();
        let mut vm = Vm::with_capture(Some(&path));
        let error = vm.run(&bad).unwrap_err().to_string();
        assert!(error.contains(&rejection), "{mutation}: {error}");
        assert_eq!(vm.captured_stdout, "");
        rows.push(
            serde_json::json!({"mutation":mutation,"error":error,"stdout":vm.captured_stdout}),
        );
    }
    let (path,module)=document("path-fallback-corruption","# Box\n## read\n*1*\n\n## other\n*2*\n\n# main\n> print text=\"PROBE\"\n**answer = > Box.read**\n*answer*\n");
    let mut bad = compile_module(Some(&path), &module).unwrap();
    let other = bad
        .functions
        .iter()
        .position(|f| f.name == "other")
        .unwrap();
    let call = bad.functions[bad.entry]
        .calls
        .iter_mut()
        .find(|c| c.source.path.is_some())
        .unwrap();
    let Some(CallDispatch::Path { fallback, .. }) = &mut call.dispatch else {
        panic!("path call")
    };
    *fallback = PathFallback::User {
        target: other,
        tag_instance: false,
    };
    let mut vm = Vm::with_capture(Some(&path));
    let error = vm.run(&bad).unwrap_err().to_string();
    assert!(error.contains("bytecode verification"));
    assert_eq!(vm.captured_stdout, "");
    rows.push(serde_json::json!({"mutation":"coherent-path-fallback","error":error,"stdout":vm.captured_stdout}));
    write_extra_evidence("name-graph-corruptions", serde_json::json!({"cases":rows}));
}

#[test]
fn user_parameter_masks_preserve_fault_priority_and_actual_effects() {
    let definitions="\n## choose\n    + `a`=> mark value=9\n    + `b`\n**answer = a * 10 + b**\n*answer*\n\n## mark\n    + `value`\n> print text=`value`\n*value*\n";
    both(
        "masked-default-binding",
        &format!("# main\n**answer = choose(b=mark(2))**\n*answer*\n{definitions}"),
        Value::Int(92),
        "2\n9\n",
    );
    both(
        "named-before-positional",
        &format!("# main\n**answer = choose(mark(1), a=mark(2))**\n*answer*\n{definitions}"),
        Value::Int(21),
        "1\n2\n",
    );
    let mut rows = Vec::new();
    for (id, call, trace, message) in [
        (
            "user-surplus",
            "choose(mark(1), mark(2), mark(3))",
            "1\n2\n3\n",
            "too many positional arguments (1 extra)",
        ),
        (
            "user-required-before-default",
            "choose()",
            "",
            "missing argument for parameter `b`",
        ),
        (
            "user-duplicate",
            "choose(a=mark(1), a=mark(2))",
            "1\n2\n",
            "duplicate named argument `a`",
        ),
    ] {
        let body = format!(
            "> print text=\"PREFIX\"\n**answer = {call}**\n> print text=\"SUFFIX\"\n*answer*\n"
        );
        both(
            &format!("{id}-unreached"),
            &format!("# main\n*7*\n\n## unused\n{body}{definitions}"),
            Value::Int(7),
            "",
        );
        let source = format!("# main\n{body}{definitions}");
        let (path, module) = document(id, &source);
        let mut tree = Interpreter::with_capture(Some(&path), false);
        let te = tree.run_module(&module).unwrap_err();
        let program = compile_module(Some(&path), &module).unwrap();
        let mut vm = Vm::with_capture(Some(&path));
        let ve = vm.run(&program).unwrap_err();
        let t = crate::diagnostics::Diagnostic::find(&te).unwrap().to_json();
        let v = crate::diagnostics::Diagnostic::find(&ve).unwrap().to_json();
        let wanted_stdout = format!("PREFIX\n{trace}");
        assert_eq!(t["message"], message);
        assert_eq!(tree.captured_stdout, wanted_stdout);
        assert_eq!(vm.captured_stdout, wanted_stdout);
        for key in ["message", "code", "span", "source_sha256", "frames"] {
            assert_eq!(t[key], v[key], "{id}: {key}");
        }
        rows.push(serde_json::json!({"id":id,"source":source,"expected_message":message,"expected_stdout":wanted_stdout,"tree":t,"bytecode":v,"tree_stdout":tree.captured_stdout,"bytecode_stdout":vm.captured_stdout}));
    }
    write_extra_evidence("user-binding-faults", serde_json::json!({"cases":rows}));
}

#[test]
fn argument_binding_plans_cover_parameter_masks_and_labels() {
    let alphabet = [
        None,
        Some("a".to_string()),
        Some("b".to_string()),
        Some("extra".to_string()),
    ];
    let mut cases = Vec::new();
    for parameters in [
        vec![],
        vec!["a"],
        vec!["a", "b"],
        vec!["a", "a"],
        vec!["a", "b", "c"],
    ] {
        let params = parameters.iter().map(|p| p.to_string()).collect::<Vec<_>>();
        for mask in 0..(1usize << params.len()) {
            let omissions = (0..params.len())
                .map(|i| mask & (1 << i) != 0)
                .collect::<Vec<_>>();
            for size in 0..=4 {
                for code in 0..4usize.pow(size as u32) {
                    let mut digits = code;
                    let mut names = Vec::new();
                    for _ in 0..size {
                        names.push(alphabet[digits % 4].clone());
                        digits /= 4;
                    }
                    cases.push(crate::arguments::plan_evidence(&params, &names, &omissions));
                }
            }
        }
    }
    assert_eq!(cases.len(), 6479, "complete mask/label corpus");
    write_extra_evidence("argument-binding-plans", serde_json::json!({"cases":cases}));
}
