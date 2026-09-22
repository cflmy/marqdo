//! P4 实验 (c) 契约消融（[perf-validation.md](../../doc/roadmap/perf-validation.md) §3）：
//! 契约**只在有契约处**拦截错误值（P1：无契约 = 全动态），且不误报。
//!
//! 四边界（调用实参 / 返回 / 表绑定 / 字段访问）× 三形态：
//! ① 带契约 + 错值 → 必拦（`contract.*`）；② 带契约 + 对值 → 零误报（跑通）；
//! ③ 同程序**删契约表** + 错值 → 零 `contract.*`（动态语义原样——零回归）。

mod common;

use common::{diagnose, static_diags, Verify};

/// （边界, 形态源）——每个边界的三形态共享同一程序骨架，只动契约表与值。
struct Boundary {
    name: &'static str,
    wrong: &'static str,   // ① 带契约 + 错值
    code: &'static str,    // ① 期望拦截码
    right: &'static str,   // ② 带契约 + 对值
    bare: &'static str,    // ③ 无契约 + 错值（同程序删契约表）
}

const ARG_WRONG: &str = "\
# main

**结果 = > 回显 \"7\"**
*结果*

## 回显

对于输入变量`n`,原样回显。

| 参数 | 类型 | 说明 |
|------|------|------|
| n | number | 输入值 |

*n*
";

const ARG_RIGHT: &str = "\
# main

**结果 = > 回显 3**
*结果*

## 回显

对于输入变量`n`,原样回显。

| 参数 | 类型 | 说明 |
|------|------|------|
| n | number | 输入值 |

*n*
";

const ARG_BARE: &str = "\
# main

**结果 = > 回显 \"7\"**
*结果*

## 回显

对于输入变量`n`,原样回显。

*n*
";

const RET_WRONG: &str = "\
# main

**结果 = > 整 \"hi\"**
*结果*

## 整

对于输入变量`n`,原样返回。

| 参数 | 类型 | 说明 |
|------|------|------|
| n | any | 输入值 |

| 返回 | 类型 | 说明 |
|------|------|------|
|  | number | 固定值 |

*n*
";

const RET_RIGHT: &str = "\
# main

**结果 = > 整 7**
*结果*

## 整

对于输入变量`n`,原样返回。

| 参数 | 类型 | 说明 |
|------|------|------|
| n | any | 输入值 |

| 返回 | 类型 | 说明 |
|------|------|------|
|  | number | 固定值 |

*n*
";

const RET_BARE: &str = "\
# main

**结果 = > 整 \"hi\"**
*结果*

## 整

对于输入变量`n`,原样返回。

*n*
";

const BIND_WRONG: &str = "\
# main

| 字段 | 类型 | 可空 |
|------|------|------|
| id | number | |
| name | text | 是 |

`用户` =
| id | name |
|------|------|
| abc | 小明 |

*[name](`用户`)*
";

const BIND_RIGHT: &str = "\
# main

| 字段 | 类型 | 可空 |
|------|------|------|
| id | number | |
| name | text | 是 |

`用户` =
| id | name |
|------|------|
| 1 | 小明 |

*[name](`用户`)*
";

const BIND_BARE: &str = "\
# main

`用户` =
| id | name |
|------|------|
| abc | 小明 |

*[name](`用户`)*
";

const KEY_WRONG: &str = "\
# main

| 字段 | 类型 | 可空 |
|------|------|------|
| id | number | |
| name | text | 是 |

`用户` =
| id | name |
|------|------|
| 1 | 小明 |

*[email](`用户`)*
";

const KEY_RIGHT: &str = "\
# main

| 字段 | 类型 | 可空 |
|------|------|------|
| id | number | |
| name | text | 是 |

`用户` =
| id | name |
|------|------|
| 1 | 小明 |

*[name](`用户`)*
";

const KEY_BARE: &str = "\
# main

`用户` =
| id | name |
|------|------|
| 1 | 小明 |

*[email](`用户`)*
";

fn boundaries() -> Vec<Boundary> {
    vec![
        Boundary { name: "call-arg", wrong: ARG_WRONG, code: "contract.arg_mismatch", right: ARG_RIGHT, bare: ARG_BARE },
        Boundary { name: "return", wrong: RET_WRONG, code: "contract.return_mismatch", right: RET_RIGHT, bare: RET_BARE },
        Boundary { name: "bind", wrong: BIND_WRONG, code: "contract.field_mismatch", right: BIND_RIGHT, bare: BIND_BARE },
        Boundary { name: "key-access", wrong: KEY_WRONG, code: "contract.unknown_key", right: KEY_RIGHT, bare: KEY_BARE },
    ]
}

#[test]
fn ablation_interception_rate_100() {
    let all = boundaries();
    let total = all.len();
    let mut caught = 0;
    for b in &all {
        let diags = diagnose(Verify::Run(b.code), b.wrong, &format!("abl_wrong_{}", b.name));
        assert!(
            diags.iter().any(|v| v["code"].as_str() == Some(b.code)),
            "[{}] 带契约 + 错值必须被拦截（期望 {}）: {:?}",
            b.name, b.code, diags
        );
        caught += 1;
    }
    eprintln!("契约拦截率 = {caught}/{total} = 100%");
}

#[test]
fn ablation_false_positive_rate_0() {
    let all = boundaries();
    let total = all.len();
    for b in &all {
        let run_diags = diagnose(Verify::Run(""), b.right, &format!("abl_right_{}", b.name));
        assert!(run_diags.is_empty(), "[{}] 对值被误报（运行）: {:?}", b.name, run_diags);
        let static_diags = static_diags(b.right);
        assert!(
            static_diags.is_empty(),
            "[{}] 对值被误报（静态）: {:?}",
            b.name,
            static_diags
        );
    }
    eprintln!("契约误报率 = 0/{total} = 0%");
}

#[test]
fn ablation_no_contract_zero_regression() {
    // 无契约路径：同程序删掉契约表 + 错值 ⇒ 零 contract.*（动态语义原样）。
    // key-access 的动态语义是经典 `missing map key` 运行错——也必须是非契约诊断。
    let all = boundaries();
    let total = all.len();
    for b in &all {
        let diags = diagnose(Verify::Run(""), b.bare, &format!("abl_bare_{}", b.name));
        assert!(
            diags.iter().all(|v| !v["code"].as_str().unwrap_or("").starts_with("contract.")),
            "[{}] 无契约路径出现契约诊断（P1 违例）: {:?}",
            b.name,
            diags
        );
        if b.name == "key-access" {
            // 动态基线：经典 missing map key 错误照旧（错误不是被契约吞掉或改写）。
            assert!(
                diags.iter().any(|v| v["message"]
                    .as_str()
                    .unwrap_or("")
                    .contains("missing map key")),
                "key-access 动态基线应为经典 missing map key: {diags:?}"
            );
        }
    }
    eprintln!("无契约零回归 = {total}/{total}");
}
