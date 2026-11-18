//! 渐进式契约（Phase 2 · T2.1–T2.5）：契约提取、四边界运行时校验、`check` 静态互查。
//!
//! 验收对照 `doc/roadmap/three-problems-plan.md` 的完成定义：
//! P1 零回归（绑定表/普通文档表不吃）、P2 锚点诊断、契约漂移必产诊断。
//! 升参推断 = 散文声明（`` `名` ``）+ 体读取；契约提取在其**之前**执行。

use std::path::PathBuf;

use marqdo::ast::{Expr, Function, Module, Stmt};
use marqdo::check::check_module;
use marqdo::contract::Ty;
use marqdo::diagnostics::Diagnostic;
use marqdo::load::load_module_from_source;
use marqdo::{run_file, RunOptions};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/contracts")
        .join(name)
}

fn find_fn<'a>(module: &'a Module, name: &str) -> &'a Function {
    fn walk<'a>(funs: &'a [Function], name: &str) -> Option<&'a Function> {
        for f in funs {
            if f.name == name {
                return Some(f);
            }
            if let Some(c) = walk(&f.children, name) {
                return Some(c);
            }
        }
        None
    }
    walk(&module.functions, name).unwrap_or_else(|| panic!("function `{name}` not found"))
}

/// anyhow 错误链 → 结构化 Diagnostic（load 的 context 包裹也不丢）。
fn structured(err: anyhow::Error) -> Diagnostic {
    match err.downcast::<Diagnostic>() {
        Ok(d) => d,
        Err(e) => Diagnostic::find(&e)
            .cloned()
            .expect("错误必须是结构化 Diagnostic（AI/MLSP 面）"),
    }
}

/// 运行夹具并断言失败的错误是结构化 Diagnostic。
fn run_err_code(name: &str) -> Diagnostic {
    let err = run_file(&fixture(name), &RunOptions::default())
        .expect_err(&format!("{name}: 期望契约报错，实际运行通过"));
    structured(err)
}

// ---------- T2.1 提取 ----------

#[test]
fn extracts_param_and_return_contract() {
    let src = "\
# main

## 加一

对于输入变量`n`,执行加一。

| 参数 | 类型 | 说明 |
|------|------|------|
| n | number | 输入值 |

| 返回 | 类型 | 说明 |
|------|------|------|
|  | number | 加一结果 |

*n+1*
";
    let module = load_module_from_source(src).unwrap();
    let fun = find_fn(&module, "加一");
    let c = fun.contract.as_ref().expect("应提取到函数契约");
    assert_eq!(c.params.len(), 1);
    assert_eq!(c.params[0].name, "n");
    assert_eq!(c.params[0].ty, Ty::Number);
    assert_eq!(c.params[0].desc, "输入值");
    assert_eq!(c.returns.as_ref().unwrap().ty, Ty::Number);
    // 契约表从 body 移除（元数据永不执行）
    assert!(
        !fun.body
            .iter()
            .any(|s| matches!(s, Stmt::Expr { value: Expr::Map(_), .. })),
        "契约表应从 body 移除，剩余 body: {:?}", fun.body
    );
    // 升参（散文声明 `n` + 体读取）恰好覆盖契约
    assert_eq!(
        fun.params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        ["n"]
    );
    // T2.4：锚点 + 原文行
    assert_eq!(c.start_line, 7);
    assert!(c.quote.iter().any(|q| q.contains("| 参数 | 类型 | 说明 |")));
}

#[test]
fn contract_cells_do_not_pollute_inference() {
    // 契约说明列里的 `` `x` `` 只是文档提及——提取先于升参推断，`x` 不得被升参。
    let src = "\
# main

## 加一

对于输入变量`n`,临时量`x`。

| 参数 | 类型 | 说明 |
|------|------|------|
| n | number | 与 `x` 相加 |

*n+1*
";
    let module = load_module_from_source(src).unwrap();
    let fun = find_fn(&module, "加一");
    assert_eq!(fun.contract.as_ref().unwrap().params[0].name, "n");
    assert_eq!(
        fun.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
        ["n"],
        "契约单元格的名不得被升参（提取先于推断）"
    );
}

#[test]
fn bound_data_table_never_contract() {
    // P1 零回归反例：**绑定**的 `字段|类型|可空` 数据表（web_db_migration 风格）不吃。
    let module = load_module_from_source(
        &std::fs::read_to_string(fixture("bound_data_table.mq.md")).unwrap(),
    )
    .unwrap();
    let fun = find_fn(&module, "main");
    assert!(fun.contract.is_none());
    assert!(fun.var_contracts.is_empty());
    run_file(&fixture("bound_data_table.mq.md"), &RunOptions::default())
        .expect("绑定数据表必须照常运行");
}

#[test]
fn duplicate_contract_tables_diagnosed() {
    // 两份契约打架 ⇒ 必产诊断（防漂移的一部分）。
    let src = "\
# main

## 加一

对于输入变量`n`,执行加一。

| 参数 | 类型 |
|------|------|
| n | number |

| 参数 | 类型 |
|------|------|
| n | number |

*n+1*
";
    let err = load_module_from_source(src).expect_err("重复契约必须报错");
    let d = structured(err);
    assert_eq!(d.to_json()["code"], "contract.duplicate");
}

// ---------- T2.2 四边界运行时校验 ----------

#[test]
fn call_arg_mismatch_reports_contract_diagnostic() {
    let d = run_err_code("arg_mismatch.mq.md");
    let v = d.to_json();
    assert_eq!(v["code"], "contract.arg_mismatch");
    assert!(
        v["message"].as_str().unwrap().contains("参数 `n` 期望 number，实际 text"),
        "{v}"
    );
    assert_eq!(v["severity"], "error");
    // T2.4：锚点能回溯到出错表，且带原文行
    assert!(v["doc_anchor"].as_str().unwrap().contains("#L"));
    assert!(
        v["doc_quote"].as_str().unwrap().contains("| 参数 | 类型 | 说明 |"),
        "doc_quote 应含契约表原文: {v}"
    );
    // contract_ref 指向声明
    assert!(
        v["contract_ref"]["declared"].as_str().unwrap().contains("number"),
        "{v}"
    );
}

#[test]
fn return_mismatch_reports_contract_diagnostic() {
    let d = run_err_code("return_mismatch.mq.md");
    let v = d.to_json();
    assert_eq!(v["code"], "contract.return_mismatch");
    assert!(
        v["message"].as_str().unwrap().contains("返回值期望 number，实际 text"),
        "{v}"
    );
    assert!(v["doc_anchor"].as_str().unwrap().contains("#L"));
}

#[test]
fn field_mismatch_at_binding_reports_contract_diagnostic() {
    // 字段表紧邻绑定之前 ⇒ 集合契约（T2.1）；绑定时校验（边界 3）。
    let module = load_module_from_source(
        &std::fs::read_to_string(fixture("field_mismatch.mq.md")).unwrap(),
    )
    .unwrap();
    let fun = find_fn(&module, "main");
    let c = fun.var_contracts.get("用户").expect("应提取到集合契约");
    assert_eq!(c.fields.len(), 2);
    assert_eq!(c.fields[0].name, "id");
    assert_eq!(c.fields[0].ty, Ty::Number);
    assert!(!c.fields[0].nullable);
    assert!(c.fields[1].nullable);

    let d = run_err_code("field_mismatch.mq.md");
    let v = d.to_json();
    assert_eq!(v["code"], "contract.field_mismatch");
    assert!(
        v["doc_quote"].as_str().unwrap().contains("| 字段 | 类型 | 可空 |"),
        "{v}"
    );
}

#[test]
fn unknown_key_at_index_reports_contract_diagnostic() {
    let d = run_err_code("unknown_key.mq.md");
    let v = d.to_json();
    assert_eq!(v["code"], "contract.unknown_key");
    assert!(v["message"].as_str().unwrap().contains("email"), "{v}");
}

#[test]
fn ok_contract_runs_unchanged() {
    run_file(&fixture("ok_contract.mq.md"), &RunOptions::default())
        .expect("合法契约必须照常运行（P1/P2：不改变正常执行）");
}

// ---------- T2.3/T2.5 `check` 静态互查 ----------

#[test]
fn check_catches_param_drift() {
    let src = "\
# main

## 加一

对于输入变量`n`,执行加一。

| 参数 | 类型 | 说明 |
|------|------|------|
| nn | number | 输入值 |

*n+1*
";
    let module = load_module_from_source(src).unwrap();
    let diags = check_module(&module, None);
    let jsons: Vec<serde_json::Value> = diags.iter().map(|d| d.to_json()).collect();
    assert!(
        jsons.iter().any(|v| v["code"] == "contract.param_drift" && v["severity"] == "error"),
        "契约行名字错位 ⇒ Error: {jsons:?}"
    );
    assert!(
        jsons.iter().any(|v| v["code"] == "contract.param_drift" && v["severity"] == "warning"),
        "新升参未被契约覆盖 ⇒ Warning: {jsons:?}"
    );
}

#[test]
fn check_catches_unknown_type_with_suggestion() {
    let src = "\
# main

## 加一

对于输入变量`n`,执行加一。

| 参数 | 类型 | 说明 |
|------|------|------|
| n | nubmer | 输入值 |

*n+1*
";
    let module = load_module_from_source(src).unwrap();
    let diags = check_module(&module, None);
    let d = diags
        .iter()
        .find(|d| d.to_json()["code"] == "contract.unknown_type")
        .expect("未知类型名必产诊断（防 nubmer 拼错）");
    let v = d.to_json();
    assert!(
        v["suggestion"].as_str().unwrap().contains("number"),
        "应给出最近建议: {v}"
    );
    assert!(v["doc_anchor"].as_str().unwrap().contains("#L"));
}

#[test]
fn check_catches_misplaced_contract_table() {
    let src = "\
# main

**x=1**

| 参数 | 类型 | 说明 |
|------|------|------|
| n | number | 输入值 |

*x*
";
    let module = load_module_from_source(src).unwrap();
    let diags = check_module(&module, None);
    assert!(
        diags.iter().any(|d| d.to_json()["code"] == "contract.misplaced"),
        "错位契约必产诊断: {:?}",
        diags.iter().map(|d| d.to_json()).collect::<Vec<_>>()
    );
}
