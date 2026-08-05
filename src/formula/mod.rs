//! High-school formula AST: parse, simplify, diff, eval, solve, plot helpers.

mod plot;

pub use plot::{plot_conic_svg, plot_function_svg, plot_points_svg, PlotStyle};

use std::collections::HashMap;
use std::fmt;

/// Expression tree for `Value::Formula`.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Var(String),
    UnaryNeg(Box<Expr>),
    Bin {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl Expr {
    pub fn num(n: f64) -> Self {
        Expr::Num(n)
    }

    pub fn var(name: impl Into<String>) -> Self {
        Expr::Var(name.into())
    }

    pub fn as_display(&self) -> String {
        self.fmt_prec(0)
    }

    fn fmt_prec(&self, parent_prec: u8) -> String {
        match self {
            Expr::Num(n) => format_num(*n),
            Expr::Var(v) => v.clone(),
            Expr::UnaryNeg(e) => {
                let inner = e.fmt_prec(3);
                let s = format!("-{inner}");
                if parent_prec > 2 {
                    format!("({s})")
                } else {
                    s
                }
            }
            Expr::Bin { op, left, right } => {
                let prec = op.prec();
                let ls = left.fmt_prec(prec);
                let rs = right.fmt_prec(if matches!(op, BinOp::Sub | BinOp::Div) {
                    prec + 1
                } else if matches!(op, BinOp::Pow) {
                    prec
                } else {
                    prec
                });
                let s = format!("{ls}{}{rs}", op.sym());
                if prec < parent_prec {
                    format!("({s})")
                } else {
                    s
                }
            }
            Expr::Call { name, args } => {
                let joined: Vec<String> = args.iter().map(|a| a.as_display()).collect();
                format!("{name}({})", joined.join(", "))
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_display())
    }
}

impl BinOp {
    fn prec(self) -> u8 {
        match self {
            BinOp::Add | BinOp::Sub => 1,
            BinOp::Mul | BinOp::Div => 2,
            BinOp::Pow => 3,
        }
    }

    fn sym(self) -> &'static str {
        match self {
            BinOp::Add => " + ",
            BinOp::Sub => " - ",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Pow => "^",
        }
    }
}

pub fn format_num(n: f64) -> String {
    if n.is_nan() {
        return "nan".into();
    }
    if n.is_infinite() {
        return if n.is_sign_positive() {
            "inf".into()
        } else {
            "-inf".into()
        };
    }
    if n.fract() == 0.0 && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    let s = format!("{n:.10}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-" {
        "0".into()
    } else {
        s.to_string()
    }
}

/// Parse ASCII formula text (`sin(x) + x^2`, optional simple `y = …` left strip).
pub fn parse(input: &str) -> Result<Expr, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("empty formula".into());
    }
    let body = if let Some((lhs, rhs)) = t.split_once('=') {
        let lhs = lhs.trim();
        if lhs.chars().all(|c| c.is_ascii_alphabetic() || c == '_') && !rhs.trim().is_empty() {
            rhs.trim()
        } else {
            t
        }
    } else {
        t
    };
    let mut p = Parser {
        src: body.chars().collect(),
        i: 0,
    };
    let e = p.parse_expr()?;
    p.skip_ws();
    if p.i < p.src.len() {
        return Err(format!(
            "unexpected trailing input near `{}`",
            p.src[p.i..].iter().collect::<String>()
        ));
    }
    Ok(e)
}

struct Parser {
    src: Vec<char>,
    i: usize,
}

impl Parser {
    fn skip_ws(&mut self) {
        while self.i < self.src.len() && self.src[self.i].is_whitespace() {
            self.i += 1;
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.skip_ws();
        self.src.get(self.i).copied()
    }

    fn bump(&mut self) -> Option<char> {
        self.skip_ws();
        let c = self.src.get(self.i).copied()?;
        self.i += 1;
        Some(c)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_add()
    }

    fn parse_add(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_mul()?;
        loop {
            match self.peek() {
                Some('+') => {
                    self.bump();
                    let right = self.parse_mul()?;
                    left = Expr::Bin {
                        op: BinOp::Add,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some('-') => {
                    self.bump();
                    let right = self.parse_mul()?;
                    left = Expr::Bin {
                        op: BinOp::Sub,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_pow()?;
        loop {
            match self.peek() {
                Some('*') => {
                    self.bump();
                    let right = self.parse_pow()?;
                    left = Expr::Bin {
                        op: BinOp::Mul,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some('/') => {
                    self.bump();
                    let right = self.parse_pow()?;
                    left = Expr::Bin {
                        op: BinOp::Div,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_pow(&mut self) -> Result<Expr, String> {
        let left = self.parse_unary()?;
        if self.peek() == Some('^') {
            self.bump();
            let right = self.parse_pow()?; // right-assoc
            Ok(Expr::Bin {
                op: BinOp::Pow,
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some('+') => {
                self.bump();
                self.parse_unary()
            }
            Some('-') => {
                self.bump();
                Ok(Expr::UnaryNeg(Box::new(self.parse_unary()?)))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some('(') => {
                self.bump();
                let e = self.parse_expr()?;
                if self.bump() != Some(')') {
                    return Err("expected `)`".into());
                }
                Ok(e)
            }
            Some(c) if c.is_ascii_digit() || c == '.' => self.parse_number(),
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                let name = self.parse_ident()?;
                if self.peek() == Some('(') {
                    self.bump();
                    let mut args = Vec::new();
                    if self.peek() != Some(')') {
                        loop {
                            args.push(self.parse_expr()?);
                            match self.peek() {
                                Some(',') => {
                                    self.bump();
                                }
                                Some(')') => break,
                                _ => return Err("expected `,` or `)` in call".into()),
                            }
                        }
                    }
                    if self.bump() != Some(')') {
                        return Err("expected `)` after call args".into());
                    }
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Some(c) => Err(format!("unexpected `{c}` in formula")),
            None => Err("unexpected end of formula".into()),
        }
    }

    fn parse_number(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        let start = self.i;
        while self.i < self.src.len()
            && (self.src[self.i].is_ascii_digit() || self.src[self.i] == '.')
        {
            self.i += 1;
        }
        let s: String = self.src[start..self.i].iter().collect();
        let n: f64 = s
            .parse()
            .map_err(|_| format!("invalid number `{s}`"))?;
        Ok(Expr::Num(n))
    }

    fn parse_ident(&mut self) -> Result<String, String> {
        self.skip_ws();
        let start = self.i;
        while self.i < self.src.len() {
            let c = self.src[self.i];
            if c.is_ascii_alphanumeric() || c == '_' {
                self.i += 1;
            } else {
                break;
            }
        }
        if start == self.i {
            return Err("expected identifier".into());
        }
        Ok(self.src[start..self.i].iter().collect())
    }
}

/// Constant-fold + light algebraic cleanup.
pub fn simplify(e: &Expr) -> Expr {
    match e {
        Expr::Num(n) => Expr::Num(*n),
        Expr::Var(v) => Expr::Var(v.clone()),
        Expr::UnaryNeg(inner) => {
            let i = simplify(inner);
            match i {
                Expr::Num(n) => Expr::Num(-n),
                Expr::UnaryNeg(x) => *x,
                other => Expr::UnaryNeg(Box::new(other)),
            }
        }
        Expr::Bin { op, left, right } => {
            let l = simplify(left);
            let r = simplify(right);
            fold_bin(*op, l, r)
        }
        Expr::Call { name, args } => {
            let args: Vec<Expr> = args.iter().map(simplify).collect();
            if args.len() == 1 {
                if let Expr::Num(n) = &args[0] {
                    if let Some(v) = eval_call_f64(name, &[*n]) {
                        return Expr::Num(v);
                    }
                }
            }
            Expr::Call {
                name: name.clone(),
                args,
            }
        }
    }
}

fn fold_bin(op: BinOp, l: Expr, r: Expr) -> Expr {
    match (op, &l, &r) {
        (BinOp::Add, Expr::Num(a), Expr::Num(b)) => Expr::Num(a + b),
        (BinOp::Sub, Expr::Num(a), Expr::Num(b)) => Expr::Num(a - b),
        (BinOp::Mul, Expr::Num(a), Expr::Num(b)) => Expr::Num(a * b),
        (BinOp::Div, Expr::Num(a), Expr::Num(b)) if *b != 0.0 => Expr::Num(a / b),
        (BinOp::Pow, Expr::Num(a), Expr::Num(b)) => Expr::Num(a.powf(*b)),
        (BinOp::Add, _, Expr::Num(0.0)) => l,
        (BinOp::Add, Expr::Num(0.0), _) => r,
        (BinOp::Sub, _, Expr::Num(0.0)) => l,
        (BinOp::Mul, _, Expr::Num(1.0)) => l,
        (BinOp::Mul, Expr::Num(1.0), _) => r,
        (BinOp::Mul, _, Expr::Num(0.0)) | (BinOp::Mul, Expr::Num(0.0), _) => Expr::Num(0.0),
        (BinOp::Pow, _, Expr::Num(1.0)) => l,
        (BinOp::Pow, _, Expr::Num(0.0)) => Expr::Num(1.0),
        (BinOp::Div, _, Expr::Num(1.0)) => l,
        _ => Expr::Bin {
            op,
            left: Box::new(l),
            right: Box::new(r),
        },
    }
}

/// Lightweight expand: distribute multiplication over addition one level.
pub fn expand(e: &Expr) -> Expr {
    let e = simplify(e);
    match e {
        Expr::Bin {
            op: BinOp::Mul,
            left,
            right,
        } => {
            let l = expand(&left);
            let r = expand(&right);
            match (l, r) {
                (
                    Expr::Bin {
                        op: BinOp::Add,
                        left: a,
                        right: b,
                    },
                    r,
                ) => simplify(&Expr::Bin {
                    op: BinOp::Add,
                    left: Box::new(Expr::Bin {
                        op: BinOp::Mul,
                        left: a,
                        right: Box::new(r.clone()),
                    }),
                    right: Box::new(Expr::Bin {
                        op: BinOp::Mul,
                        left: b,
                        right: Box::new(r),
                    }),
                }),
                (
                    l,
                    Expr::Bin {
                        op: BinOp::Add,
                        left: a,
                        right: b,
                    },
                ) => simplify(&Expr::Bin {
                    op: BinOp::Add,
                    left: Box::new(Expr::Bin {
                        op: BinOp::Mul,
                        left: Box::new(l.clone()),
                        right: a,
                    }),
                    right: Box::new(Expr::Bin {
                        op: BinOp::Mul,
                        left: Box::new(l),
                        right: b,
                    }),
                }),
                (l, r) => simplify(&Expr::Bin {
                    op: BinOp::Mul,
                    left: Box::new(l),
                    right: Box::new(r),
                }),
            }
        }
        Expr::Bin { op, left, right } => simplify(&Expr::Bin {
            op,
            left: Box::new(expand(&left)),
            right: Box::new(expand(&right)),
        }),
        Expr::UnaryNeg(x) => simplify(&Expr::UnaryNeg(Box::new(expand(&x)))),
        Expr::Call { name, args } => Expr::Call {
            name,
            args: args.iter().map(expand).collect(),
        },
        other => other,
    }
}

pub fn diff(e: &Expr, var: &str) -> Expr {
    simplify(&diff_raw(e, var))
}

fn diff_raw(e: &Expr, var: &str) -> Expr {
    match e {
        Expr::Num(_) => Expr::Num(0.0),
        Expr::Var(v) => {
            if v == var {
                Expr::Num(1.0)
            } else {
                Expr::Num(0.0)
            }
        }
        Expr::UnaryNeg(x) => Expr::UnaryNeg(Box::new(diff_raw(x, var))),
        Expr::Bin {
            op: BinOp::Add,
            left,
            right,
        } => Expr::Bin {
            op: BinOp::Add,
            left: Box::new(diff_raw(left, var)),
            right: Box::new(diff_raw(right, var)),
        },
        Expr::Bin {
            op: BinOp::Sub,
            left,
            right,
        } => Expr::Bin {
            op: BinOp::Sub,
            left: Box::new(diff_raw(left, var)),
            right: Box::new(diff_raw(right, var)),
        },
        Expr::Bin {
            op: BinOp::Mul,
            left,
            right,
        } => {
            // product rule
            Expr::Bin {
                op: BinOp::Add,
                left: Box::new(Expr::Bin {
                    op: BinOp::Mul,
                    left: Box::new(diff_raw(left, var)),
                    right: right.clone(),
                }),
                right: Box::new(Expr::Bin {
                    op: BinOp::Mul,
                    left: left.clone(),
                    right: Box::new(diff_raw(right, var)),
                }),
            }
        }
        Expr::Bin {
            op: BinOp::Div,
            left,
            right,
        } => {
            // (u'v - uv') / v^2
            let num = Expr::Bin {
                op: BinOp::Sub,
                left: Box::new(Expr::Bin {
                    op: BinOp::Mul,
                    left: Box::new(diff_raw(left, var)),
                    right: right.clone(),
                }),
                right: Box::new(Expr::Bin {
                    op: BinOp::Mul,
                    left: left.clone(),
                    right: Box::new(diff_raw(right, var)),
                }),
            };
            Expr::Bin {
                op: BinOp::Div,
                left: Box::new(num),
                right: Box::new(Expr::Bin {
                    op: BinOp::Pow,
                    left: right.clone(),
                    right: Box::new(Expr::Num(2.0)),
                }),
            }
        }
        Expr::Bin {
            op: BinOp::Pow,
            left,
            right,
        } => {
            // d/dx [u^n] when n constant: n*u^(n-1)*u'
            if let Expr::Num(n) = right.as_ref() {
                Expr::Bin {
                    op: BinOp::Mul,
                    left: Box::new(Expr::Bin {
                        op: BinOp::Mul,
                        left: Box::new(Expr::Num(*n)),
                        right: Box::new(Expr::Bin {
                            op: BinOp::Pow,
                            left: left.clone(),
                            right: Box::new(Expr::Num(n - 1.0)),
                        }),
                    }),
                    right: Box::new(diff_raw(left, var)),
                }
            } else {
                // a^u → a^u * ln(a) * u' when a constant
                if let Expr::Num(a) = left.as_ref() {
                    Expr::Bin {
                        op: BinOp::Mul,
                        left: Box::new(Expr::Bin {
                            op: BinOp::Mul,
                            left: Box::new(e.clone()),
                            right: Box::new(Expr::Call {
                                name: "ln".into(),
                                args: vec![Expr::Num(*a)],
                            }),
                        }),
                        right: Box::new(diff_raw(right, var)),
                    }
                } else {
                    // fallback: treat as exp(v*ln(u))
                    Expr::Num(0.0)
                }
            }
        }
        Expr::Call { name, args } => {
            if args.len() != 1 {
                return Expr::Num(0.0);
            }
            let u = &args[0];
            let du = diff_raw(u, var);
            let chain = |outer: Expr| {
                Expr::Bin {
                    op: BinOp::Mul,
                    left: Box::new(outer),
                    right: Box::new(du.clone()),
                }
            };
            match name.as_str() {
                "sin" => chain(Expr::Call {
                    name: "cos".into(),
                    args: vec![u.clone()],
                }),
                "cos" => chain(Expr::UnaryNeg(Box::new(Expr::Call {
                    name: "sin".into(),
                    args: vec![u.clone()],
                }))),
                "tan" => chain(Expr::Bin {
                    op: BinOp::Pow,
                    left: Box::new(Expr::Call {
                        name: "cos".into(),
                        args: vec![u.clone()],
                    }),
                    right: Box::new(Expr::Num(-2.0)),
                }),
                "sqrt" => chain(Expr::Bin {
                    op: BinOp::Div,
                    left: Box::new(Expr::Num(1.0)),
                    right: Box::new(Expr::Bin {
                        op: BinOp::Mul,
                        left: Box::new(Expr::Num(2.0)),
                        right: Box::new(Expr::Call {
                            name: "sqrt".into(),
                            args: vec![u.clone()],
                        }),
                    }),
                }),
                "ln" | "log" => chain(Expr::Bin {
                    op: BinOp::Div,
                    left: Box::new(Expr::Num(1.0)),
                    right: Box::new(u.clone()),
                }),
                "exp" => chain(Expr::Call {
                    name: "exp".into(),
                    args: vec![u.clone()],
                }),
                "abs" => {
                    // sign(u)*u' — use u/abs(u) when u≠0
                    chain(Expr::Bin {
                        op: BinOp::Div,
                        left: Box::new(u.clone()),
                        right: Box::new(Expr::Call {
                            name: "abs".into(),
                            args: vec![u.clone()],
                        }),
                    })
                }
                _ => Expr::Num(0.0),
            }
        }
    }
}

pub fn subs(e: &Expr, var: &str, value: &Expr) -> Expr {
    simplify(&subs_raw(e, var, value))
}

fn subs_raw(e: &Expr, var: &str, value: &Expr) -> Expr {
    match e {
        Expr::Num(n) => Expr::Num(*n),
        Expr::Var(v) => {
            if v == var {
                value.clone()
            } else {
                Expr::Var(v.clone())
            }
        }
        Expr::UnaryNeg(x) => Expr::UnaryNeg(Box::new(subs_raw(x, var, value))),
        Expr::Bin { op, left, right } => Expr::Bin {
            op: *op,
            left: Box::new(subs_raw(left, var, value)),
            right: Box::new(subs_raw(right, var, value)),
        },
        Expr::Call { name, args } => Expr::Call {
            name: name.clone(),
            args: args.iter().map(|a| subs_raw(a, var, value)).collect(),
        },
    }
}

pub fn eval_f64(e: &Expr, env: &HashMap<String, f64>) -> Result<f64, String> {
    match e {
        Expr::Num(n) => Ok(*n),
        Expr::Var(v) => env
            .get(v)
            .copied()
            .ok_or_else(|| format!("unbound variable `{v}`")),
        Expr::UnaryNeg(x) => Ok(-eval_f64(x, env)?),
        Expr::Bin { op, left, right } => {
            let a = eval_f64(left, env)?;
            let b = eval_f64(right, env)?;
            Ok(match op {
                BinOp::Add => a + b,
                BinOp::Sub => a - b,
                BinOp::Mul => a * b,
                BinOp::Div => {
                    if b == 0.0 {
                        return Err("division by zero".into());
                    }
                    a / b
                }
                BinOp::Pow => a.powf(b),
            })
        }
        Expr::Call { name, args } => {
            let mut xs = Vec::with_capacity(args.len());
            for a in args {
                xs.push(eval_f64(a, env)?);
            }
            eval_call_f64(name, &xs).ok_or_else(|| format!("unknown or arity-mismatch function `{name}`"))
        }
    }
}

fn eval_call_f64(name: &str, args: &[f64]) -> Option<f64> {
    match (name, args) {
        ("sin", [x]) => Some(x.sin()),
        ("cos", [x]) => Some(x.cos()),
        ("tan", [x]) => Some(x.tan()),
        ("asin", [x]) => Some(x.asin()),
        ("acos", [x]) => Some(x.acos()),
        ("atan", [x]) => Some(x.atan()),
        ("sqrt", [x]) => Some(x.sqrt()),
        ("abs", [x]) => Some(x.abs()),
        ("ln" | "log", [x]) => Some(x.ln()),
        ("exp", [x]) => Some(x.exp()),
        ("floor", [x]) => Some(x.floor()),
        ("ceil", [x]) => Some(x.ceil()),
        ("pow", [a, b]) => Some(a.powf(*b)),
        ("min", [a, b]) => Some(a.min(*b)),
        ("max", [a, b]) => Some(a.max(*b)),
        _ => None,
    }
}

/// Solve `e = 0` for `var`. Quadratic/linear closed form; else numeric scan.
pub fn solve(e: &Expr, var: &str, xmin: f64, xmax: f64) -> Result<Vec<f64>, String> {
    let e = expand(&simplify(e));
    if let Some(roots) = try_polynomial_solve(&e, var) {
        return Ok(roots);
    }
    numeric_roots(&e, var, xmin, xmax)
}

fn try_polynomial_solve(e: &Expr, var: &str) -> Option<Vec<f64>> {
    let (a, b, c) = poly_coeffs_quadratic(e, var)?;
    if a.abs() < 1e-12 {
        if b.abs() < 1e-12 {
            return if c.abs() < 1e-12 {
                None // identity
            } else {
                Some(vec![])
            };
        }
        return Some(vec![-c / b]);
    }
    let disc = b * b - 4.0 * a * c;
    if disc < -1e-12 {
        return Some(vec![]);
    }
    if disc.abs() <= 1e-12 {
        return Some(vec![-b / (2.0 * a)]);
    }
    let s = disc.sqrt();
    let mut roots = vec![(-b - s) / (2.0 * a), (-b + s) / (2.0 * a)];
    roots.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    Some(roots)
}

/// Collect ax^2+bx+c if expression is a polynomial of degree ≤ 2 in `var`.
fn poly_coeffs_quadratic(e: &Expr, var: &str) -> Option<(f64, f64, f64)> {
    let mut a = 0.0;
    let mut b = 0.0;
    let mut c = 0.0;
    if !accum_poly(e, var, 1.0, &mut a, &mut b, &mut c) {
        return None;
    }
    Some((a, b, c))
}

fn accum_poly(e: &Expr, var: &str, scale: f64, a: &mut f64, b: &mut f64, c: &mut f64) -> bool {
    match e {
        Expr::Num(n) => {
            *c += scale * n;
            true
        }
        Expr::Var(v) if v == var => {
            *b += scale;
            true
        }
        Expr::Var(_) => false,
        Expr::UnaryNeg(x) => accum_poly(x, var, -scale, a, b, c),
        Expr::Bin {
            op: BinOp::Add,
            left,
            right,
        } => accum_poly(left, var, scale, a, b, c) && accum_poly(right, var, scale, a, b, c),
        Expr::Bin {
            op: BinOp::Sub,
            left,
            right,
        } => accum_poly(left, var, scale, a, b, c) && accum_poly(right, var, -scale, a, b, c),
        Expr::Bin {
            op: BinOp::Mul,
            left,
            right,
        } => match (left.as_ref(), right.as_ref()) {
            (Expr::Num(n), r) => accum_poly(r, var, scale * n, a, b, c),
            (l, Expr::Num(n)) => accum_poly(l, var, scale * n, a, b, c),
            (Expr::Var(v), Expr::Var(w)) if v == var && w == var => {
                *a += scale;
                true
            }
            (
                Expr::Bin {
                    op: BinOp::Pow,
                    left: base,
                    right: exp,
                },
                other,
            )
            | (
                other,
                Expr::Bin {
                    op: BinOp::Pow,
                    left: base,
                    right: exp,
                },
            ) => {
                if let (Expr::Var(v), Expr::Num(2.0)) = (base.as_ref(), exp.as_ref()) {
                    if v == var {
                        if let Expr::Num(n) = other {
                            *a += scale * n;
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        },
        Expr::Bin {
            op: BinOp::Pow,
            left,
            right,
        } => match (left.as_ref(), right.as_ref()) {
            (Expr::Var(v), Expr::Num(2.0)) if v == var => {
                *a += scale;
                true
            }
            (Expr::Var(v), Expr::Num(1.0)) if v == var => {
                *b += scale;
                true
            }
            (Expr::Var(v), Expr::Num(0.0)) if v == var => {
                *c += scale;
                true
            }
            _ => false,
        },
        _ => false,
    }
}

fn numeric_roots(e: &Expr, var: &str, xmin: f64, xmax: f64) -> Result<Vec<f64>, String> {
    const STEPS: usize = 200;
    let mut roots = Vec::new();
    let mut env = HashMap::new();
    let dx = (xmax - xmin) / STEPS as f64;
    let mut x0 = xmin;
    env.insert(var.to_string(), x0);
    let mut y0 = eval_f64(e, &env)?;
    for i in 1..=STEPS {
        let x1 = xmin + dx * i as f64;
        env.insert(var.to_string(), x1);
        let y1 = eval_f64(e, &env)?;
        if y0 == 0.0 {
            push_unique(&mut roots, x0);
        } else if y0.signum() != y1.signum() && y1 != 0.0 {
            // bisection
            let mut lo = x0;
            let mut hi = x1;
            let mut flo = y0;
            for _ in 0..40 {
                let mid = 0.5 * (lo + hi);
                env.insert(var.to_string(), mid);
                let fm = eval_f64(e, &env)?;
                if fm == 0.0 || (hi - lo).abs() < 1e-10 {
                    push_unique(&mut roots, mid);
                    break;
                }
                if flo.signum() == fm.signum() {
                    lo = mid;
                    flo = fm;
                } else {
                    hi = mid;
                }
            }
        }
        if i == STEPS && y1 == 0.0 {
            push_unique(&mut roots, x1);
        }
        x0 = x1;
        y0 = y1;
    }
    Ok(roots)
}

fn push_unique(xs: &mut Vec<f64>, x: f64) {
    if xs.iter().any(|y| (y - x).abs() < 1e-6) {
        return;
    }
    xs.push(x);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_diff_quadratic() {
        let e = parse("x^2 - 2").unwrap();
        let d = diff(&e, "x");
        assert_eq!(d.as_display(), "2*x");
        let roots = solve(&e, "x", -10.0, 10.0).unwrap();
        assert_eq!(roots.len(), 2);
        assert!((roots[0] + 2f64.sqrt()).abs() < 1e-6);
        assert!((roots[1] - 2f64.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn eval_sin() {
        let e = parse("sin(x)").unwrap();
        let mut env = HashMap::new();
        env.insert("x".into(), 0.0);
        assert!((eval_f64(&e, &env).unwrap()).abs() < 1e-12);
    }
}
