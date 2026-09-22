//! P4 实验 (d) 防漂移演练（[perf-validation.md](../../doc/roadmap/perf-validation.md) §4）：
//! **故意写错契约必报率 = 100%**——契约错了必被机器抓，绝不静默信任契约
//! （three-problems.md §2.8「故意写错契约的金样必红」）。
//!
//! 9 例故意漂移（类型 typo ×2 / 参数名漂移 / 错位表 / 重复表 / 四边界错值）
//! 必须全部产出 `contract.*` 诊断（静态或运行时）；
//! 2 例负对照（被绑定的数据表 / 干净契约）**零误报**。

mod common;

use common::{diagnose, static_diags, Verify};

struct Drill {
    name: &'static str,
    src: &'static str,
    /// true = 运行期诊断；false = 静态（mlsp validate = parse + check）。
    runtime: bool,
    code: &'static str,
}

/// 类型 typo（参数表 `nubmer`）→ 静态 `contract.unknown_type`。
const SRC_TYPO_PARAM: &str = "\
# main

## 加一

对于输入变量`n`,执行加一。

| 参数 | 类型 | 说明 |
|------|------|------|
| n | nubmer | 输入值 |

*n+1*
";

/// 类型 typo（字段表未知对象名 `巨龙`）→ 静态 `contract.unknown_type`。
const SRC_TYPO_FIELD: &str = "\
# main

| 字段 | 类型 | 可空 |
|------|------|------|
| id | 巨龙 | |

`用户` =
| id |
|------|
| 1 |

*[id](`用户`)*
";

/// 契约参数名 ≠ 升参名（`nn` vs `n`）→ 静态 `contract.param_drift`。
const SRC_PARAM_DRIFT: &str = "\
# main

## 加一

对于输入变量`n`,执行加一。

| 参数 | 类型 | 说明 |
|------|------|------|
| nn | number | 输入值 |

*n+1*
";

/// 错位契约表（出现在可执行语句之后）→ 静态 `contract.misplaced`。
const SRC_MISPLACED: &str = "\
# main

**x=1**

| 参数 | 类型 | 说明 |
|------|------|------|
| n | number | 输入值 |

*x*
";

/// 重复契约表（两份参数表打架）→ 装载期 `contract.duplicate`。
const SRC_DUPLICATE: &str = "\
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

/// 实参越约（参数契约 number，实参 text）→ 运行期 `contract.arg_mismatch`。
const SRC_ARG: &str = "\
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

/// 返回越约（返回契约 number，实际返回 text）→ 运行期 `contract.return_mismatch`。
const SRC_RETURN: &str = "\
# main

**结果 = > 整 \"hi\"**
*结果*

## 整

对于输入变量`n`,原样返回。

| 返回 | 类型 | 说明 |
|------|------|------|
|  | number | 固定值 |

*n*
";

/// 表绑定越约（字段 id 契约 number，单元格 abc）→ 运行期 `contract.field_mismatch`。
const SRC_FIELD: &str = "\
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

/// 缺键访问（契约有 email，集合没有）→ 运行期 `contract.unknown_key`。
const SRC_UNKNOWN_KEY: &str = "\
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

/// 负对照 1：**被绑定**的 `字段|类型|可空` 数据表（web_db_migration 式）= 数据，不是契约。
const SRC_BOUND_DATA: &str = "\
# main

`列说明` =
| 字段 | 类型 | 可空 |
|------|------|------|
| id | number | |
| name | text | 是 |

*[字段]([1](`列说明`))*
";

/// 负对照 2：干净契约 + 对值。
const SRC_OK: &str = "\
# main

**结果 = > 加一 3**
*结果*

## 加一

对于输入变量`n`,期望是数字。

| 参数 | 类型 | 说明 |
|------|------|------|
| n | number | 输入值 |

| 返回 | 类型 | 说明 |
|------|------|------|
|  | number | 加一结果 |

*n+1*
";

fn drills() -> Vec<Drill> {
    vec![
        Drill { name: "type-typo-param", src: SRC_TYPO_PARAM, runtime: false, code: "contract.unknown_type" },
        Drill { name: "type-typo-field", src: SRC_TYPO_FIELD, runtime: false, code: "contract.unknown_type" },
        Drill { name: "param-name-drift", src: SRC_PARAM_DRIFT, runtime: false, code: "contract.param_drift" },
        Drill { name: "misplaced-table", src: SRC_MISPLACED, runtime: false, code: "contract.misplaced" },
        Drill { name: "duplicate-table", src: SRC_DUPLICATE, runtime: true, code: "contract.duplicate" },
        Drill { name: "arg-mismatch", src: SRC_ARG, runtime: true, code: "contract.arg_mismatch" },
        Drill { name: "return-mismatch", src: SRC_RETURN, runtime: true, code: "contract.return_mismatch" },
        Drill { name: "field-mismatch", src: SRC_FIELD, runtime: true, code: "contract.field_mismatch" },
        Drill { name: "unknown-key", src: SRC_UNKNOWN_KEY, runtime: true, code: "contract.unknown_key" },
    ]
}

#[test]
fn drift_cases_all_reported() {
    let all = drills();
    let total = all.len();
    let mut caught = 0;
    for d in &all {
        let diags = if d.runtime {
            diagnose(Verify::Run(d.code), d.src, &format!("drift_{}", d.name))
        } else {
            static_diags(d.src)
        };
        let hit = diags.iter().any(|v| v["code"].as_str() == Some(d.code));
        assert!(
            hit,
            "[{}] 故意写错的契约必须被诊断（期望 {}）: {:?}",
            d.name, d.code, diags
        );
        caught += 1;
    }
    eprintln!("防漂移必报率 = {caught}/{total} = 100%");
}

#[test]
fn negative_controls_zero_false_positives() {
    // 1. 被绑定的 `字段|类型|可空` 数据表 = 数据（web_db_migration 式）——绝不能被当契约吃掉。
    let diags = diagnose(Verify::Run(""), SRC_BOUND_DATA, "drill_neg_bound");
    assert!(
        diags.iter().all(|v| !v["code"].as_str().unwrap_or("").starts_with("contract.")),
        "绑定数据表被误当契约: {diags:?}"
    );
    // 2. 干净契约 + 对值 ⇒ 静态零诊断、运行零诊断。
    let diags = static_diags(SRC_OK);
    assert!(diags.is_empty(), "干净契约被误报（静态）: {diags:?}");
    let diags = diagnose(Verify::Run(""), SRC_OK, "drill_neg_ok");
    assert!(diags.is_empty(), "干净契约被误报（运行）: {diags:?}");
    eprintln!("负对照误报 = 0/2 = 0%");
}
