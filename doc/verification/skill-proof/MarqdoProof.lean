/-
MarqdoProof：`marqdo prove` 生成代码的语义底座（设计 §6.2，无公理）。

原则：
- `Val` 是 Marqdo 值的忠实模型；每个 `mq*` 辅助函数对应 Marqdo 运行时语义的一个片段。
- 不出现 `axiom`、不出现 `opaque`：外部操作是 `Env := Op → Val` 参数（每个生成文件
  自带 `abbrev Env := Op → Val`，因为 `Op` 构造器集合是每文件独立的）。
- 定理陈述把"信任了什么"写清楚：对任意 `env` 成立的命题，只依赖这里给出的方程。
-/

namespace MarqdoProof

/-- Marqdo 值模型（§6.2；`text` 是发射器的真实命名，设计文档起草时的 `str` 已修订）。 -/
inductive Val where
  | none : Val
  | int : Int → Val
  | num : Float → Val
  | bool : Bool → Val
  | text : String → Val
  | list : List Val → Val
  | map : List (String × Val) → Val
  deriving Repr, BEq

/-- 定理命题包装：`Bool` 语义上的"为真"。 -/
def mqTruth (b : Bool) : Prop := b = true

/-- 映射字段读取（`结果.ok` 等）；缺失字段 → `none`（与运行时 access 一致）。
按表项递归， simp 方程友好（`mqAccess (mqResultOk v) "ok"` 直接化到 `.bool true`）。 -/
def mqAccess : Val → String → Val
  | .map [], _ => .none
  | .map ((k, v) :: rest), k' => if k == k' then v else mqAccess (.map rest) k'
  | _, _ => .none

-- ---------------------------------------------------------------- 算术

/-- 显示串：标量与解释器 `Value::as_display` 对齐（T4 差分一致性）。
容器做一层显示（值经 mqDisplayScalar），嵌套容器超出 `+` 片段——
记录为有意简化，语料不依赖嵌套容器拼接。非递归 ⇒ 无终止性负担。 -/
def mqDisplayScalar : Val → String
  | .int n => toString n
  | .num x => toString x
  | .bool b => if b then "True" else "False"
  | .text s => s
  | _ => "…"

def mqDisplay : Val → String
  | .int n => toString n
  | .num x => toString x
  | .bool b => if b then "True" else "False"
  | .text s => s
  | .list xs => "[" ++ String.intercalate ", " (xs.map mqDisplayScalar) ++ "]"
  | .map ps => "{" ++ String.intercalate ", " (ps.map (fun p => p.1 ++ ": " ++ mqDisplayScalar p.2)) ++ "}"
  | _ => "None"

/-- `+`：int 算术 + 文本拼接 + num 混合，与解释器 `eval_binary(Add)` 一致。
偏差：解释器 i64 溢出报运行时错误；Lean `Int` 无界（语料只测界内值）。 -/
def mqAdd : Val → Val → Val
  | .int a, .int b => .int (a + b)
  | .num a, .num b => .num (a + b)
  | .int a, .num b => .num (Float.ofInt a + b)
  | .num a, .int b => .num (a + Float.ofInt b)
  | .text a, .text b => .text (a ++ b)
  | .text a, b => .text (a ++ mqDisplay b)
  | a, .text b => .text (mqDisplay a ++ b)
  | _, _ => .none

def mqSub : Val → Val → Val
  | .int a, .int b => .int (a - b)
  | _, _ => .none

def mqMul : Val → Val → Val
  | .int a, .int b => .int (a * b)
  | _, _ => .none

/-- 整除；除以 0 在 Marqdo 运行时是运行时错误，此处 P1 模型按 Int 语义取 0。 -/
def mqDiv : Val → Val → Val
  | .int a, .int b => .int (a / b)
  | _, _ => .none

def mqNeg : Val → Val
  | .int a => .int (-a)
  | _ => .none

-- ---------------------------------------------------------------- 比较

def mqLt : Val → Val → Bool
  | .int a, .int b => a < b
  | _, _ => false

def mqLe : Val → Val → Bool
  | .int a, .int b => a ≤ b
  | _, _ => false

def mqGt : Val → Val → Bool
  | .int a, .int b => a > b
  | _, _ => false

def mqGe : Val → Val → Bool
  | .int a, .int b => a ≥ b
  | _, _ => false

/-- 结构化相等（生成定理的 simp 用方程逐构造器化简）。 -/
def mqEq : Val → Val → Bool
  | .none, .none => true
  | .int a, .int b => a == b
  | .num a, .num b => a == b
  | .bool a, .bool b => a == b
  | .text a, .text b => a == b
  | .list a, .list b => a == b
  | .map a, .map b => a == b
  | _, _ => false

@[simp] theorem mqGt_int (a b : Int) : mqGt (Val.int a) (Val.int b) = decide (a > b) := rfl
@[simp] theorem mqGe_int (a b : Int) : mqGe (Val.int a) (Val.int b) = decide (a ≥ b) := rfl
@[simp] theorem mqLt_int (a b : Int) : mqLt (Val.int a) (Val.int b) = decide (a < b) := rfl
@[simp] theorem mqLe_int (a b : Int) : mqLe (Val.int a) (Val.int b) = decide (a ≤ b) := rfl
@[simp] theorem mqEq_int (a b : Int) : mqEq (Val.int a) (Val.int b) = decide (a = b) := rfl
@[simp] theorem mqEq_bool (a b : Bool) : mqEq (Val.bool a) (Val.bool b) = (a == b) := rfl
@[simp] theorem mqEq_text (a b : String) : mqEq (Val.text a) (Val.text b) = (a == b) := rfl

-- ---------------------------------------------------------------- 分支矛盾

/-- 同一值对同一阈值既 `> n` 又 `≤ n`：分支矛盾（simp_all 消掉互斥分支）。 -/
@[simp] theorem mqGt_mqLe_false {v : Val} {n : Int}
    (h1 : mqGt v (Val.int n) = true) (h2 : mqLe v (Val.int n) = true) : False := by
  cases v with
  | int x =>
    simp only [mqGt, mqLe, decide_eq_true_eq] at h1 h2
    omega
  | _ => simp [mqGt] at h1

@[simp] theorem mqLe_mqGt_false {v : Val} {n : Int}
    (h1 : mqLe v (Val.int n) = true) (h2 : mqGt v (Val.int n) = true) : False := by
  cases v with
  | int x =>
    simp only [mqLe, mqGt, decide_eq_true_eq] at h1 h2
    omega
  | _ => simp [mqGt] at h2

@[simp] theorem mqLt_mqGe_false {v : Val} {n : Int}
    (h1 : mqLt v (Val.int n) = true) (h2 : mqGe v (Val.int n) = true) : False := by
  cases v with
  | int x =>
    simp only [mqLt, mqGe, decide_eq_true_eq] at h1 h2
    omega
  | _ => simp [mqLt] at h1

@[simp] theorem mqGe_mqLt_false {v : Val} {n : Int}
    (h1 : mqGe v (Val.int n) = true) (h2 : mqLt v (Val.int n) = true) : False := by
  cases v with
  | int x =>
    simp only [mqGe, mqLt, decide_eq_true_eq] at h1 h2
    omega
  | _ => simp [mqLt] at h2

-- ---------------------------------------------------------------- 布尔

def mqAnd (a b : Bool) : Bool := a && b
def mqOr (a b : Bool) : Bool := a || b
def mqNot (b : Bool) : Bool := !b

-- ---------------------------------------------------------------- 列表

/-- Val 列表的 `包含`（谓词；轨迹上的包含由生成文件按 `Op.name` 内联）。 -/
def mqContains (xs v : Val) : Bool :=
  match xs with
  | .list ys => ys.contains v
  | _ => false

/-- Val 列表的长度（`Val.int` 面）。 -/
def mqLen : Val → Val
  | .list ys => .int ys.length
  | _ => .none

/-- 从 `Val.int` 取 `Nat`（供 `长度` 比较等）。 -/
def mqNatOf : Val → Nat
  | .int n => n.toNat
  | _ => 0

/-- Val 列表拼接。 -/
def mqAppend : Val → Val → Val
  | .list a, .list b => .list (a ++ b)
  | _, _ => .none

-- ---------------------------------------------------------------- lib/result

/-- `result.ok(v)`：Val 字面映射构造（§12 偏差 2）。 -/
def mqResultOk (v : Val) : Val :=
  .map [("ok", .bool true), ("value", v), ("error", .none)]

/-- `result.error(code, message, operation)`。 -/
def mqResultError (code message operation : Val) : Val :=
  .map [("ok", .bool false), ("value", .none),
        ("error", .map [("code", code), ("message", message), ("operation", operation)])]

/-- `result.value_or(r, dflt)`。 -/
def mqValueOr (r dflt : Val) : Val :=
  match mqAccess r "ok" with
  | .bool true => mqAccess r "value"
  | _ => dflt

end MarqdoProof
