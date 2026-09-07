//! Symbolic matrix expressions (MatExpr) for formula-first linear algebra.
//!
//! Trees stay abstract until `explicit` (L2). Simplify applies R1–R5.

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum Dim {
    Int(usize),
    Sym(String),
}

impl Dim {
    pub fn to_json(&self) -> Value {
        match self {
            Dim::Int(n) => json!(*n),
            Dim::Sym(s) => json!(s),
        }
    }

    pub fn from_json(v: &Value) -> Result<Self, String> {
        match v {
            Value::Number(n) => {
                let u = n
                    .as_u64()
                    .or_else(|| n.as_i64().map(|i| i as u64))
                    .ok_or_else(|| "bad dim number".to_string())?;
                Ok(Dim::Int(u as usize))
            }
            Value::String(s) => {
                let t = s.trim();
                if let Ok(n) = t.parse::<usize>() {
                    Ok(Dim::Int(n))
                } else {
                    Ok(Dim::Sym(t.to_string()))
                }
            }
            _ => Err("dim must be int or string".into()),
        }
    }

    fn ascii(&self) -> String {
        match self {
            Dim::Int(n) => n.to_string(),
            Dim::Sym(s) => s.clone(),
        }
    }

    fn latex(&self) -> String {
        self.ascii()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Symbol {
        name: String,
        rows: Dim,
        cols: Dim,
    },
    Eye {
        n: Dim,
    },
    Zero {
        rows: Dim,
        cols: Dim,
    },
    Dense {
        data: Vec<Vec<f64>>,
    },
    Mul {
        factors: Vec<Expr>,
    },
    Add {
        terms: Vec<Expr>,
    },
    Sub {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Transpose {
        arg: Box<Expr>,
    },
    Inv {
        arg: Box<Expr>,
    },
    /// Block matrix grid (rows of block rows).
    Block {
        blocks: Vec<Vec<Expr>>,
    },
    /// Kronecker product (lazy; do not expand by default).
    Kron {
        factors: Vec<Expr>,
    },
}

impl Expr {
    pub fn shape(&self) -> Result<(Dim, Dim), String> {
        match self {
            Expr::Symbol { rows, cols, .. } => Ok((rows.clone(), cols.clone())),
            Expr::Eye { n } => Ok((n.clone(), n.clone())),
            Expr::Zero { rows, cols } => Ok((rows.clone(), cols.clone())),
            Expr::Dense { data } => {
                let r = data.len();
                let c = data.first().map(|row| row.len()).unwrap_or(0);
                if data.iter().any(|row| row.len() != c) {
                    return Err("dense matrix rows must have equal length".into());
                }
                Ok((Dim::Int(r), Dim::Int(c)))
            }
            Expr::Mul { factors } => {
                if factors.is_empty() {
                    return Err("empty MatMul".into());
                }
                let (rows, mut cols) = factors[0].shape()?;
                for f in &factors[1..] {
                    let (r, c) = f.shape()?;
                    if !dims_eq(&cols, &r) {
                        return Err(format!(
                            "incompatible shapes for mul: …×{} with {}×{}",
                            cols.ascii(),
                            r.ascii(),
                            c.ascii()
                        ));
                    }
                    cols = c;
                }
                Ok((rows, cols))
            }
            Expr::Add { terms } => {
                if terms.is_empty() {
                    return Err("empty MatAdd".into());
                }
                let (r0, c0) = terms[0].shape()?;
                for t in &terms[1..] {
                    let (r, c) = t.shape()?;
                    if !dims_eq(&r0, &r) || !dims_eq(&c0, &c) {
                        return Err(format!(
                            "incompatible shapes for add: {}×{} vs {}×{}",
                            r0.ascii(),
                            c0.ascii(),
                            r.ascii(),
                            c.ascii()
                        ));
                    }
                }
                Ok((r0, c0))
            }
            Expr::Sub { left, right } => {
                let (r0, c0) = left.shape()?;
                let (r1, c1) = right.shape()?;
                if !dims_eq(&r0, &r1) || !dims_eq(&c0, &c1) {
                    return Err(format!(
                        "incompatible shapes for sub: {}×{} vs {}×{}",
                        r0.ascii(),
                        c0.ascii(),
                        r1.ascii(),
                        c1.ascii()
                    ));
                }
                Ok((r0, c0))
            }
            Expr::Transpose { arg } => {
                let (r, c) = arg.shape()?;
                Ok((c, r))
            }
            Expr::Inv { arg } => {
                let (r, c) = arg.shape()?;
                if !dims_eq(&r, &c) {
                    return Err(format!(
                        "inverse needs square matrix, got {}×{}",
                        r.ascii(),
                        c.ascii()
                    ));
                }
                Ok((r, c))
            }
            Expr::Block { blocks } => block_shape(blocks),
            Expr::Kron { factors } => {
                if factors.is_empty() {
                    return Err("empty Kronecker product".into());
                }
                let (mut rows, mut cols) = factors[0].shape()?;
                for f in &factors[1..] {
                    let (r, c) = f.shape()?;
                    rows = dim_mul(&rows, &r)?;
                    cols = dim_mul(&cols, &c)?;
                }
                Ok((rows, cols))
            }
        }
    }

    pub fn ascii(&self) -> String {
        match self {
            Expr::Symbol { name, .. } => name.clone(),
            Expr::Eye { n } => format!("I({})", n.ascii()),
            Expr::Zero { rows, cols } => format!("0({}×{})", rows.ascii(), cols.ascii()),
            Expr::Dense { data } => format_dense_ascii(data),
            Expr::Mul { factors } => factors
                .iter()
                .map(|f| {
                    let s = f.ascii();
                    if matches!(f, Expr::Add { .. } | Expr::Sub { .. }) {
                        format!("({s})")
                    } else {
                        s
                    }
                })
                .collect::<Vec<_>>()
                .join("*"),
            Expr::Add { terms } => terms
                .iter()
                .map(|t| t.ascii())
                .collect::<Vec<_>>()
                .join(" + "),
            Expr::Sub { left, right } => format!("{} - {}", left.ascii(), right.ascii()),
            Expr::Transpose { arg } => {
                let inner = arg.ascii();
                if matches!(
                    arg.as_ref(),
                    Expr::Symbol { .. } | Expr::Eye { .. } | Expr::Zero { .. } | Expr::Dense { .. }
                ) {
                    format!("{inner}^T")
                } else {
                    format!("({inner})^T")
                }
            }
            Expr::Inv { arg } => {
                let inner = arg.ascii();
                if matches!(
                    arg.as_ref(),
                    Expr::Symbol { .. } | Expr::Eye { .. } | Expr::Zero { .. } | Expr::Dense { .. }
                ) {
                    format!("{inner}^-1")
                } else {
                    format!("({inner})^-1")
                }
            }
            Expr::Block { blocks } => {
                let rows: Vec<String> = blocks
                    .iter()
                    .map(|row| {
                        let cells: Vec<String> = row.iter().map(|b| b.ascii()).collect();
                        format!("[{}]", cells.join(", "))
                    })
                    .collect();
                format!("Block[{}]", rows.join("; "))
            }
            Expr::Kron { factors } => factors
                .iter()
                .map(|f| {
                    let s = f.ascii();
                    if matches!(
                        f,
                        Expr::Add { .. } | Expr::Sub { .. } | Expr::Mul { .. } | Expr::Block { .. }
                    ) {
                        format!("({s})")
                    } else {
                        s
                    }
                })
                .collect::<Vec<_>>()
                .join("⊗"),
        }
    }

    pub fn latex(&self) -> String {
        match self {
            Expr::Symbol { name, .. } => name.clone(),
            Expr::Eye { n } => format!("I_{{{}}}", n.latex()),
            Expr::Zero { rows, cols } => {
                format!("0_{{{}\\times {}}}", rows.latex(), cols.latex())
            }
            Expr::Dense { data } => format_dense_latex(data),
            Expr::Mul { factors } => factors
                .iter()
                .map(|f| {
                    let s = f.latex();
                    if matches!(f, Expr::Add { .. } | Expr::Sub { .. }) {
                        format!("\\left({s}\\right)")
                    } else {
                        s
                    }
                })
                .collect::<Vec<_>>()
                .join(""),
            Expr::Add { terms } => terms
                .iter()
                .map(|t| t.latex())
                .collect::<Vec<_>>()
                .join(" + "),
            Expr::Sub { left, right } => format!("{} - {}", left.latex(), right.latex()),
            Expr::Transpose { arg } => {
                let inner = arg.latex();
                if matches!(
                    arg.as_ref(),
                    Expr::Symbol { .. } | Expr::Eye { .. } | Expr::Zero { .. } | Expr::Dense { .. }
                ) {
                    format!("{inner}^{{\\top}}")
                } else {
                    format!("\\left({inner}\\right)^{{\\top}}")
                }
            }
            Expr::Inv { arg } => {
                let inner = arg.latex();
                if matches!(
                    arg.as_ref(),
                    Expr::Symbol { .. } | Expr::Eye { .. } | Expr::Zero { .. } | Expr::Dense { .. }
                ) {
                    format!("{inner}^{{-1}}")
                } else {
                    format!("\\left({inner}\\right)^{{-1}}")
                }
            }
            Expr::Block { blocks } => {
                let body: Vec<String> = blocks
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|b| b.latex())
                            .collect::<Vec<_>>()
                            .join(" & ")
                    })
                    .collect();
                format!(
                    "\\begin{{bmatrix}}{}\\end{{bmatrix}}",
                    body.join(" \\\\ ")
                )
            }
            Expr::Kron { factors } => factors
                .iter()
                .map(|f| {
                    let s = f.latex();
                    if matches!(
                        f,
                        Expr::Add { .. } | Expr::Sub { .. } | Expr::Mul { .. } | Expr::Block { .. }
                    ) {
                        format!("\\left({s}\\right)")
                    } else {
                        s
                    }
                })
                .collect::<Vec<_>>()
                .join(" \\otimes "),
        }
    }

    pub fn to_value(&self) -> Result<Value, String> {
        let (rows, cols) = self.shape()?;
        let mut obj = serde_json::Map::new();
        obj.insert("_type".into(), json!("linalg_expr"));
        obj.insert("shape".into(), json!([rows.to_json(), cols.to_json()]));
        obj.insert("ascii".into(), json!(self.ascii()));
        obj.insert("latex".into(), json!(self.latex()));
        match self {
            Expr::Symbol { name, rows, cols } => {
                obj.insert("kind".into(), json!("symbol"));
                obj.insert("name".into(), json!(name));
                obj.insert("rows".into(), rows.to_json());
                obj.insert("cols".into(), cols.to_json());
            }
            Expr::Eye { n } => {
                obj.insert("kind".into(), json!("eye"));
                obj.insert("n".into(), n.to_json());
            }
            Expr::Zero { rows, cols } => {
                obj.insert("kind".into(), json!("zero"));
                obj.insert("rows".into(), rows.to_json());
                obj.insert("cols".into(), cols.to_json());
            }
            Expr::Dense { data } => {
                obj.insert("kind".into(), json!("dense"));
                obj.insert("data".into(), json!(data));
                obj.insert("dtype".into(), json!("real"));
            }
            Expr::Mul { factors } => {
                obj.insert("kind".into(), json!("mul"));
                let kids: Result<Vec<_>, _> = factors.iter().map(|f| f.to_value()).collect();
                obj.insert("factors".into(), Value::Array(kids?));
            }
            Expr::Add { terms } => {
                obj.insert("kind".into(), json!("add"));
                let kids: Result<Vec<_>, _> = terms.iter().map(|t| t.to_value()).collect();
                obj.insert("terms".into(), Value::Array(kids?));
            }
            Expr::Sub { left, right } => {
                obj.insert("kind".into(), json!("sub"));
                obj.insert("left".into(), left.to_value()?);
                obj.insert("right".into(), right.to_value()?);
            }
            Expr::Transpose { arg } => {
                obj.insert("kind".into(), json!("transpose"));
                obj.insert("arg".into(), arg.to_value()?);
            }
            Expr::Inv { arg } => {
                obj.insert("kind".into(), json!("inv"));
                obj.insert("arg".into(), arg.to_value()?);
            }
            Expr::Block { blocks } => {
                obj.insert("kind".into(), json!("block"));
                let mut grid = Vec::new();
                for row in blocks {
                    let mut r = Vec::new();
                    for b in row {
                        r.push(b.to_value()?);
                    }
                    grid.push(Value::Array(r));
                }
                obj.insert("blocks".into(), Value::Array(grid));
            }
            Expr::Kron { factors } => {
                obj.insert("kind".into(), json!("kron"));
                let kids: Result<Vec<_>, _> = factors.iter().map(|f| f.to_value()).collect();
                obj.insert("factors".into(), Value::Array(kids?));
            }
        }
        Ok(Value::Object(obj))
    }

    pub fn from_value(v: &Value) -> Result<Self, String> {
        let obj = v
            .as_object()
            .ok_or_else(|| "linalg_expr must be a map".to_string())?;
        let kind = obj
            .get("kind")
            .and_then(|x| x.as_str())
            .ok_or_else(|| "missing kind".to_string())?;
        match kind {
            "symbol" => Ok(Expr::Symbol {
                name: obj
                    .get("name")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| "symbol missing name")?
                    .to_string(),
                rows: Dim::from_json(obj.get("rows").ok_or("symbol missing rows")?)?,
                cols: Dim::from_json(obj.get("cols").ok_or("symbol missing cols")?)?,
            }),
            "eye" => Ok(Expr::Eye {
                n: Dim::from_json(obj.get("n").ok_or("eye missing n")?)?,
            }),
            "zero" => Ok(Expr::Zero {
                rows: Dim::from_json(obj.get("rows").ok_or("zero missing rows")?)?,
                cols: Dim::from_json(obj.get("cols").ok_or("zero missing cols")?)?,
            }),
            "dense" => {
                let data = parse_dense(obj.get("data").ok_or("dense missing data")?)?;
                Ok(Expr::Dense { data })
            }
            "mul" => {
                let factors = obj
                    .get("factors")
                    .and_then(|x| x.as_array())
                    .ok_or("mul missing factors")?;
                let factors: Result<Vec<_>, _> =
                    factors.iter().map(Expr::from_value).collect();
                Ok(Expr::Mul {
                    factors: factors?,
                })
            }
            "add" => {
                let terms = obj
                    .get("terms")
                    .and_then(|x| x.as_array())
                    .ok_or("add missing terms")?;
                let terms: Result<Vec<_>, _> = terms.iter().map(Expr::from_value).collect();
                Ok(Expr::Add { terms: terms? })
            }
            "sub" => Ok(Expr::Sub {
                left: Box::new(Expr::from_value(
                    obj.get("left").ok_or("sub missing left")?,
                )?),
                right: Box::new(Expr::from_value(
                    obj.get("right").ok_or("sub missing right")?,
                )?),
            }),
            "transpose" => Ok(Expr::Transpose {
                arg: Box::new(Expr::from_value(
                    obj.get("arg").ok_or("transpose missing arg")?,
                )?),
            }),
            "inv" => Ok(Expr::Inv {
                arg: Box::new(Expr::from_value(obj.get("arg").ok_or("inv missing arg")?)?),
            }),
            "block" => {
                let grid = obj
                    .get("blocks")
                    .and_then(|x| x.as_array())
                    .ok_or("block missing blocks")?;
                let mut blocks = Vec::new();
                for row in grid {
                    let cells = row
                        .as_array()
                        .ok_or("block row must be a list")?;
                    let row: Result<Vec<_>, _> =
                        cells.iter().map(Expr::from_value).collect();
                    blocks.push(row?);
                }
                let expr = Expr::Block { blocks };
                let _ = expr.shape()?;
                Ok(expr)
            }
            "kron" => {
                let factors = obj
                    .get("factors")
                    .and_then(|x| x.as_array())
                    .ok_or("kron missing factors")?;
                let factors: Result<Vec<_>, _> =
                    factors.iter().map(Expr::from_value).collect();
                Ok(Expr::Kron {
                    factors: flatten_kron(factors?),
                })
            },
            other => Err(format!("unknown linalg_expr kind `{other}`")),
        }
    }
}

fn dims_eq(a: &Dim, b: &Dim) -> bool {
    match (a, b) {
        (Dim::Int(x), Dim::Int(y)) => x == y,
        (Dim::Sym(x), Dim::Sym(y)) => x == y,
        // symbolic vs int: allow only if we cannot prove mismatch — treat as compatible
        // for teaching formulas like n×n with concrete later; strict mode would reject.
        (Dim::Int(_), Dim::Sym(_)) | (Dim::Sym(_), Dim::Int(_)) => true,
    }
}

fn dim_add(a: &Dim, b: &Dim) -> Result<Dim, String> {
    match (a, b) {
        (Dim::Int(x), Dim::Int(y)) => Ok(Dim::Int(x + y)),
        (Dim::Sym(x), Dim::Sym(y)) => Ok(Dim::Sym(format!("({x}+{y})"))),
        (Dim::Int(x), Dim::Sym(y)) => Ok(Dim::Sym(format!("({x}+{y})"))),
        (Dim::Sym(x), Dim::Int(y)) => Ok(Dim::Sym(format!("({x}+{y})"))),
    }
}

fn dim_mul(a: &Dim, b: &Dim) -> Result<Dim, String> {
    match (a, b) {
        (Dim::Int(x), Dim::Int(y)) => Ok(Dim::Int(x.saturating_mul(*y))),
        (Dim::Sym(x), Dim::Sym(y)) => Ok(Dim::Sym(format!("({x}*{y})"))),
        (Dim::Int(x), Dim::Sym(y)) => Ok(Dim::Sym(format!("({x}*{y})"))),
        (Dim::Sym(x), Dim::Int(y)) => Ok(Dim::Sym(format!("({x}*{y})"))),
    }
}

fn block_shape(blocks: &[Vec<Expr>]) -> Result<(Dim, Dim), String> {
    if blocks.is_empty() || blocks[0].is_empty() {
        return Err("empty block matrix".into());
    }
    let br = blocks.len();
    let bc = blocks[0].len();
    if blocks.iter().any(|row| row.len() != bc) {
        return Err("block matrix rows must have equal width".into());
    }
    // Row heights: from column 0
    let mut row_heights = Vec::with_capacity(br);
    for i in 0..br {
        let (h, _) = blocks[i][0].shape()?;
        for j in 1..bc {
            let (h2, _) = blocks[i][j].shape()?;
            if !dims_eq(&h, &h2) {
                return Err(format!(
                    "block row {i}: incompatible heights {} vs {}",
                    h.ascii(),
                    h2.ascii()
                ));
            }
        }
        row_heights.push(h);
    }
    // Col widths: from row 0
    let mut col_widths = Vec::with_capacity(bc);
    for j in 0..bc {
        let (_, w) = blocks[0][j].shape()?;
        for i in 1..br {
            let (_, w2) = blocks[i][j].shape()?;
            if !dims_eq(&w, &w2) {
                return Err(format!(
                    "block col {j}: incompatible widths {} vs {}",
                    w.ascii(),
                    w2.ascii()
                ));
            }
        }
        col_widths.push(w);
    }
    let mut rows = row_heights[0].clone();
    for h in &row_heights[1..] {
        rows = dim_add(&rows, h)?;
    }
    let mut cols = col_widths[0].clone();
    for w in &col_widths[1..] {
        cols = dim_add(&cols, w)?;
    }
    Ok((rows, cols))
}

fn parse_dense(v: &Value) -> Result<Vec<Vec<f64>>, String> {
    let rows = v
        .as_array()
        .ok_or_else(|| "dense data must be list of rows".to_string())?;
    let mut out = Vec::new();
    let mut width = None;
    for row in rows {
        let cells = row
            .as_array()
            .ok_or_else(|| "dense row must be a list".to_string())?;
        let mut r = Vec::new();
        for c in cells {
            let n = c
                .as_f64()
                .or_else(|| c.as_i64().map(|i| i as f64))
                .or_else(|| c.as_u64().map(|u| u as f64))
                .or_else(|| c.as_str().and_then(|s| s.trim().parse().ok()))
                .ok_or_else(|| "dense cell must be a number".to_string())?;
            r.push(n);
        }
        if let Some(w) = width {
            if r.len() != w {
                return Err("dense matrix rows must have equal length".into());
            }
        } else {
            width = Some(r.len());
        }
        out.push(r);
    }
    if out.is_empty() {
        return Err("empty dense matrix".into());
    }
    Ok(out)
}

fn format_dense_ascii(data: &[Vec<f64>]) -> String {
    let rows: Vec<String> = data
        .iter()
        .map(|row| {
            let cells: Vec<String> = row.iter().map(|x| format_num(*x)).collect();
            format!("[{}]", cells.join(","))
        })
        .collect();
    format!("[{}]", rows.join(","))
}

fn format_dense_latex(data: &[Vec<f64>]) -> String {
    let body: Vec<String> = data
        .iter()
        .map(|row| {
            row.iter()
                .map(|x| format_num(*x))
                .collect::<Vec<_>>()
                .join(" & ")
        })
        .collect();
    format!(
        "\\begin{{bmatrix}}{}\\end{{bmatrix}}",
        body.join(" \\\\ ")
    )
}

fn format_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// Flatten nested Mul.
fn flatten_mul(factors: Vec<Expr>) -> Vec<Expr> {
    let mut out = Vec::new();
    for f in factors {
        match f {
            Expr::Mul { factors: inner } => out.extend(flatten_mul(inner)),
            other => out.push(other),
        }
    }
    out
}

fn is_eye(e: &Expr) -> bool {
    matches!(e, Expr::Eye { .. })
}

fn is_zero(e: &Expr) -> bool {
    matches!(e, Expr::Zero { .. })
}

/// Apply R1–R5 once bottom-up, then iterate to fixpoint (small bound).
pub fn simplify(expr: Expr) -> Result<Expr, String> {
    let mut cur = expr;
    for _ in 0..32 {
        let next = simplify_once(cur.clone())?;
        // shape check keeps trees valid
        let _ = next.shape()?;
        if next == cur {
            return Ok(cur);
        }
        cur = next;
    }
    Ok(cur)
}

fn simplify_once(expr: Expr) -> Result<Expr, String> {
    match expr {
        Expr::Mul { factors } => {
            let factors: Result<Vec<_>, _> =
                flatten_mul(factors).into_iter().map(simplify_once).collect();
            let mut factors = factors?;
            let eye_n = factors.iter().find_map(|f| match f {
                Expr::Eye { n } => Some(n.clone()),
                _ => None,
            });
            // R1: drop Eye
            factors.retain(|f| !is_eye(f));
            if factors.is_empty() {
                return Ok(Expr::Eye {
                    n: eye_n.unwrap_or(Dim::Sym("n".into())),
                });
            }
            if factors.len() == 1 {
                return Ok(factors.pop().unwrap());
            }
            // R1 zero absorb: …*0*… → 0 with product shape
            if let Some(z) = factors.iter().find(|f| is_zero(f)) {
                let (r, _) = factors.first().unwrap().shape()?;
                let (_, c) = factors.last().unwrap().shape()?;
                let _ = z;
                return Ok(Expr::Zero { rows: r, cols: c });
            }
            Ok(Expr::Mul { factors })
        }
        Expr::Add { terms } => {
            let terms: Result<Vec<_>, _> = terms.into_iter().map(simplify_once).collect();
            let mut terms = terms?;
            let zero_shape = terms.iter().find_map(|t| match t {
                Expr::Zero { rows, cols } => Some((rows.clone(), cols.clone())),
                _ => None,
            });
            terms.retain(|t| !is_zero(t));
            if terms.is_empty() {
                let (rows, cols) = zero_shape.unwrap_or((Dim::Int(0), Dim::Int(0)));
                return Ok(Expr::Zero { rows, cols });
            }
            if terms.len() == 1 {
                return Ok(terms.pop().unwrap());
            }
            Ok(Expr::Add { terms })
        }
        Expr::Sub { left, right } => {
            let left = simplify_once(*left)?;
            let right = simplify_once(*right)?;
            if is_zero(&right) {
                return Ok(left);
            }
            Ok(Expr::Sub {
                left: Box::new(left),
                right: Box::new(right),
            })
        }
        Expr::Transpose { arg } => {
            let arg = simplify_once(*arg)?;
            match arg {
                // R3: (AB…)^T → …^T B^T A^T
                Expr::Mul { factors } => {
                    let factors: Vec<Expr> = factors
                        .into_iter()
                        .rev()
                        .map(|f| Expr::Transpose {
                            arg: Box::new(f),
                        })
                        .collect();
                    simplify_once(Expr::Mul { factors })
                }
                // R3: (A^T)^T → A
                Expr::Transpose { arg: inner } => Ok(*inner),
                // (A+B)^T → A^T+B^T
                Expr::Add { terms } => {
                    let terms = terms
                        .into_iter()
                        .map(|t| Expr::Transpose {
                            arg: Box::new(t),
                        })
                        .collect();
                    simplify_once(Expr::Add { terms })
                }
                Expr::Sub { left, right } => simplify_once(Expr::Sub {
                    left: Box::new(Expr::Transpose { arg: left }),
                    right: Box::new(Expr::Transpose { arg: right }),
                }),
                // R4 partial: (A^-1)^T → (A^T)^-1
                Expr::Inv { arg: inner } => Ok(Expr::Inv {
                    arg: Box::new(Expr::Transpose { arg: inner }),
                }),
                Expr::Eye { n } => Ok(Expr::Eye { n }),
                Expr::Zero { rows, cols } => Ok(Expr::Zero {
                    rows: cols,
                    cols: rows,
                }),
                // Block^T → block of transposed tiles, swapped indices
                Expr::Block { blocks } => {
                    if blocks.is_empty() {
                        return Ok(Expr::Block { blocks });
                    }
                    let br = blocks.len();
                    let bc = blocks[0].len();
                    let mut out = vec![vec![]; bc];
                    for j in 0..bc {
                        out[j] = Vec::with_capacity(br);
                        for i in 0..br {
                            out[j].push(Expr::Transpose {
                                arg: Box::new(blocks[i][j].clone()),
                            });
                        }
                    }
                    simplify_once(Expr::Block { blocks: out })
                }
                // (A⊗B)^T → A^T ⊗ B^T
                Expr::Kron { factors } => {
                    let factors = factors
                        .into_iter()
                        .map(|f| Expr::Transpose {
                            arg: Box::new(f),
                        })
                        .collect();
                    simplify_once(Expr::Kron { factors })
                }
                other => Ok(Expr::Transpose {
                    arg: Box::new(other),
                }),
            }
        }
        Expr::Inv { arg } => {
            let arg = simplify_once(*arg)?;
            match arg {
                // R2: (A^-1)^-1 → A
                Expr::Inv { arg: inner } => Ok(*inner),
                // R4: (A^T)^-1 stays; (A^-1)^T handled above
                Expr::Eye { n } => Ok(Expr::Eye { n }),
                other => Ok(Expr::Inv {
                    arg: Box::new(other),
                }),
            }
        }
        Expr::Block { blocks } => {
            let mut out = Vec::new();
            for row in blocks {
                let mut r = Vec::new();
                for b in row {
                    r.push(simplify_once(b)?);
                }
                out.push(r);
            }
            Ok(Expr::Block { blocks: out })
        }
        Expr::Kron { factors } => {
            // R7: flatten nested Kronecker
            let factors: Result<Vec<_>, _> = flatten_kron(factors)
                .into_iter()
                .map(simplify_once)
                .collect();
            let mut factors = factors?;
            if factors.len() == 1 {
                return Ok(factors.pop().unwrap());
            }
            Ok(Expr::Kron { factors })
        }
        Expr::Symbol { .. } | Expr::Eye { .. } | Expr::Zero { .. } | Expr::Dense { .. } => {
            Ok(expr)
        }
    }
}

fn flatten_kron(factors: Vec<Expr>) -> Vec<Expr> {
    let mut out = Vec::new();
    for f in factors {
        match f {
            Expr::Kron { factors: inner } => out.extend(flatten_kron(inner)),
            other => out.push(other),
        }
    }
    out
}

/// Block-level collapse (R6 subset): `Block * Block` → block of products.
pub fn collapse(expr: Expr) -> Result<Expr, String> {
    let expr = simplify(expr)?;
    collapse_once(expr)
}

fn collapse_once(expr: Expr) -> Result<Expr, String> {
    match expr {
        Expr::Mul { factors } => {
            let factors: Result<Vec<_>, _> =
                flatten_mul(factors).into_iter().map(collapse_once).collect();
            let factors = factors?;
            let mut acc = factors[0].clone();
            for f in factors.into_iter().skip(1) {
                acc = collapse_mul_pair(acc, f)?;
            }
            Ok(acc)
        }
        Expr::Block { blocks } => {
            let mut out = Vec::new();
            for row in blocks {
                let mut r = Vec::new();
                for b in row {
                    r.push(collapse_once(b)?);
                }
                out.push(r);
            }
            Ok(Expr::Block { blocks: out })
        }
        Expr::Add { terms } => {
            let terms: Result<Vec<_>, _> = terms.into_iter().map(collapse_once).collect();
            Ok(Expr::Add { terms: terms? })
        }
        Expr::Sub { left, right } => Ok(Expr::Sub {
            left: Box::new(collapse_once(*left)?),
            right: Box::new(collapse_once(*right)?),
        }),
        Expr::Transpose { arg } => Ok(Expr::Transpose {
            arg: Box::new(collapse_once(*arg)?),
        }),
        Expr::Inv { arg } => Ok(Expr::Inv {
            arg: Box::new(collapse_once(*arg)?),
        }),
        Expr::Kron { factors } => {
            let factors: Result<Vec<_>, _> = factors.into_iter().map(collapse_once).collect();
            Ok(Expr::Kron {
                factors: flatten_kron(factors?),
            })
        }
        other => Ok(other),
    }
}

fn collapse_mul_pair(a: Expr, b: Expr) -> Result<Expr, String> {
    match (a, b) {
        (Expr::Block { blocks: left }, Expr::Block { blocks: right }) => {
            let lr = left.len();
            let lc = left[0].len();
            let rr = right.len();
            let rc = right[0].len();
            if lc != rr {
                return Err(format!(
                    "block mul: left cols {lc} != right rows {rr}"
                ));
            }
            let mut out = Vec::with_capacity(lr);
            for i in 0..lr {
                let mut row = Vec::with_capacity(rc);
                for j in 0..rc {
                    let mut term = mul(left[i][0].clone(), right[0][j].clone())?;
                    for k in 1..lc {
                        let p = mul(left[i][k].clone(), right[k][j].clone())?;
                        term = add(term, p)?;
                    }
                    row.push(simplify(term)?);
                }
                out.push(row);
            }
            let expr = Expr::Block { blocks: out };
            let _ = expr.shape()?;
            Ok(expr)
        }
        (a, b) => {
            let expr = Expr::Mul {
                factors: flatten_mul(vec![a, b]),
            };
            let _ = expr.shape()?;
            Ok(expr)
        }
    }
}

pub fn mul(a: Expr, b: Expr) -> Result<Expr, String> {
    let expr = Expr::Mul {
        factors: flatten_mul(vec![a, b]),
    };
    let _ = expr.shape()?;
    Ok(expr)
}

pub fn add(a: Expr, b: Expr) -> Result<Expr, String> {
    let expr = Expr::Add {
        terms: vec![a, b],
    };
    let _ = expr.shape()?;
    Ok(expr)
}

pub fn sub(a: Expr, b: Expr) -> Result<Expr, String> {
    let expr = Expr::Sub {
        left: Box::new(a),
        right: Box::new(b),
    };
    let _ = expr.shape()?;
    Ok(expr)
}

pub fn transpose(a: Expr) -> Result<Expr, String> {
    let expr = Expr::Transpose {
        arg: Box::new(a),
    };
    let _ = expr.shape()?;
    Ok(expr)
}

pub fn inv(a: Expr) -> Result<Expr, String> {
    let expr = Expr::Inv {
        arg: Box::new(a),
    };
    let _ = expr.shape()?;
    Ok(expr)
}

pub fn symbol(name: &str, rows: Dim, cols: Dim) -> Expr {
    Expr::Symbol {
        name: name.to_string(),
        rows,
        cols,
    }
}

pub fn eye(n: Dim) -> Expr {
    Expr::Eye { n }
}

pub fn zeros(rows: Dim, cols: Dim) -> Expr {
    Expr::Zero { rows, cols }
}

pub fn from_list(data: Vec<Vec<f64>>) -> Result<Expr, String> {
    if data.is_empty() {
        return Err("empty matrix".into());
    }
    let w = data[0].len();
    if w == 0 || data.iter().any(|r| r.len() != w) {
        return Err("dense matrix rows must have equal non-zero length".into());
    }
    Ok(Expr::Dense { data })
}

pub fn block(blocks: Vec<Vec<Expr>>) -> Result<Expr, String> {
    let expr = Expr::Block { blocks };
    let _ = expr.shape()?;
    Ok(expr)
}

pub fn kron(a: Expr, b: Expr) -> Result<Expr, String> {
    let expr = Expr::Kron {
        factors: flatten_kron(vec![a, b]),
    };
    let _ = expr.shape()?;
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transpose_of_product() {
        let a = symbol("A", Dim::Int(2), Dim::Int(3));
        let b = symbol("B", Dim::Int(3), Dim::Int(2));
        let p = mul(a, b).unwrap();
        let t = transpose(p).unwrap();
        let s = simplify(t).unwrap();
        assert_eq!(s.ascii(), "B^T*A^T");
    }

    #[test]
    fn double_inv() {
        let a = symbol("A", Dim::Sym("n".into()), Dim::Sym("n".into()));
        let i = inv(inv(a).unwrap()).unwrap();
        let s = simplify(i).unwrap();
        assert_eq!(s.ascii(), "A");
    }

    #[test]
    fn shape_mismatch() {
        let a = symbol("A", Dim::Int(2), Dim::Int(2));
        let b = symbol("B", Dim::Int(3), Dim::Int(3));
        assert!(mul(a, b).is_err());
    }

    #[test]
    fn kron_lazy_ascii() {
        let a = symbol("A", Dim::Int(2), Dim::Int(2));
        let b = symbol("B", Dim::Int(2), Dim::Int(2));
        let k = kron(a, b).unwrap();
        assert_eq!(k.ascii(), "A⊗B");
        let (r, c) = k.shape().unwrap();
        assert_eq!(r, Dim::Int(4));
        assert_eq!(c, Dim::Int(4));
    }

    #[test]
    fn block_collapse_mul() {
        let a = symbol("A", Dim::Int(2), Dim::Int(2));
        let b = symbol("B", Dim::Int(2), Dim::Int(2));
        let z = zeros(Dim::Int(2), Dim::Int(2));
        let left = block(vec![
            vec![a.clone(), b.clone()],
            vec![z.clone(), a.clone()],
        ])
        .unwrap();
        let right = block(vec![
            vec![a.clone(), z.clone()],
            vec![z.clone(), b.clone()],
        ])
        .unwrap();
        let p = mul(left, right).unwrap();
        let c = collapse(p).unwrap();
        // [[A,B],[0,A]] * [[A,0],[0,B]] → [[A*A, B*B],[0, A*B]]
        assert!(matches!(c, Expr::Block { .. }));
        assert_eq!(c.ascii(), "Block[[A*A, B*B]; [0(2×2), A*B]]");
    }

    #[test]
    fn block_collapse_same_blocks() {
        let a = symbol("A", Dim::Int(2), Dim::Int(2));
        let b = symbol("B", Dim::Int(2), Dim::Int(2));
        let z = zeros(Dim::Int(2), Dim::Int(2));
        let m = block(vec![
            vec![a.clone(), b.clone()],
            vec![z.clone(), a.clone()],
        ])
        .unwrap();
        let p = mul(m.clone(), m).unwrap();
        let c = collapse(p).unwrap();
        assert_eq!(
            c.ascii(),
            "Block[[A*A, A*B + B*A]; [0(2×2), A*A]]"
        );
    }
}
