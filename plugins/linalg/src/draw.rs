//! Structure SVG for factorizations (teaching layouts) with Q8-aligned themes.

use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeName {
    Dark,
    Light,
    Bw,
}

impl ThemeName {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "dark" | "暗" => Ok(Self::Dark),
            "light" | "亮" | "浅" => Ok(Self::Light),
            "bw" | "mono" | "黑白" => Ok(Self::Bw),
            other => Err(format!("unknown theme `{other}` (dark|light|bw)")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Bw => "bw",
        }
    }
}

struct Theme {
    name: ThemeName,
    bg: &'static str,
    ink: &'static str,
    muted: &'static str,
    stroke: &'static str,
    box_a: &'static str,
    box_b: &'static str,
    box_c: &'static str,
    heat_neg: &'static str,
    heat_pos: &'static str,
}

impl Theme {
    fn of(name: ThemeName) -> Self {
        match name {
            ThemeName::Dark => Self {
                name,
                bg: "#0a0e14",
                ink: "#e8eef6",
                muted: "#93a4b8",
                stroke: "#6b7f96",
                box_a: "#1e3a5f",
                box_b: "#3d2e14",
                box_c: "#1e3f2e",
                heat_neg: "#cc5555",
                heat_pos: "#4c8fff",
            },
            ThemeName::Light => Self {
                name,
                bg: "#eef3f9",
                ink: "#15202b",
                muted: "#5a6a7a",
                stroke: "#4a5a6a",
                box_a: "#e8f1ff",
                box_b: "#fff3d6",
                box_c: "#e8ffe8",
                heat_neg: "#cc3333",
                heat_pos: "#2266aa",
            },
            ThemeName::Bw => Self {
                name,
                bg: "#ffffff",
                ink: "#111111",
                muted: "#555555",
                stroke: "#222222",
                box_a: "#f0f0f0",
                box_b: "#e0e0e0",
                box_c: "#d0d0d0",
                heat_neg: "#000000",
                heat_pos: "#666666",
            },
        }
    }
}

/// Draw a factor object or matrix as structure SVG.
pub fn draw_factor(
    factor: &Value,
    kind: Option<&str>,
    path: Option<&str>,
    theme: ThemeName,
) -> Result<String, String> {
    let kind = kind
        .or_else(|| factor.get("kind").and_then(|k| k.as_str()))
        .unwrap_or("structure");
    let th = Theme::of(theme);
    let svg = match kind {
        "eig" => draw_eig(factor, &th),
        "svd" => draw_svd(factor, &th),
        "qr" => draw_qr(factor, &th),
        "lu" | "ge" => draw_lu(factor, &th),
        "chol" => draw_chol(factor, &th),
        "structure" => draw_structure(factor, &th),
        "heatmap" | "hinton" => draw_heatmap(factor, kind == "hinton", &th),
        other => {
            return Err(format!(
                "unknown draw kind `{other}` (want eig|svd|qr|lu|ge|chol|structure|heatmap|hinton)"
            ))
        }
    };
    let _ = path;
    Ok(svg)
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn box_svg(x: f64, y: f64, w: f64, h: f64, label: &str, fill: &str, th: &Theme) -> String {
    format!(
        "<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"{fill}\" stroke=\"{}\" stroke-width=\"1.5\" rx=\"4\"/>\
<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"16\" font-family=\"serif\" fill=\"{}\">{}</text>",
        th.stroke,
        x + w / 2.0,
        y + h / 2.0,
        th.ink,
        escape(label)
    )
}

fn root(w: f64, h: f64, kind: &str, th: &Theme, body: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" data-linalg=\"{kind}\" data-theme=\"{}\" style=\"background:{}\">\
{body}\
</svg>",
        th.name.as_str(),
        th.bg
    )
}

fn draw_eig(factor: &Value, th: &Theme) -> String {
    let latex = factor
        .get("latex")
        .and_then(|x| x.as_str())
        .unwrap_or("A = P D P^{-1}");
    let evals = factor
        .get("eigenvalues")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_f64())
                .map(|v| format!("{v:.4}"))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let body = format!(
        "<text x=\"16\" y=\"28\" font-size=\"18\" font-family=\"serif\" fill=\"{}\">{}</text>\
{}\
{}\
{}\
<text x=\"16\" y=\"120\" font-size=\"13\" font-family=\"sans-serif\" fill=\"{}\">λ = [{}]</text>",
        th.ink,
        escape(latex),
        box_svg(40.0, 48.0, 70.0, 50.0, "P", th.box_a, th),
        box_svg(130.0, 48.0, 70.0, 50.0, "D", th.box_b, th),
        box_svg(220.0, 48.0, 90.0, 50.0, "P⁻¹", th.box_a, th),
        th.muted,
        escape(&evals)
    );
    root(520.0, 140.0, "eig", th, &body)
}

fn draw_svd(factor: &Value, th: &Theme) -> String {
    let s = factor
        .get("S")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_f64())
                .map(|v| format!("{v:.4}"))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let body = format!(
        "<text x=\"16\" y=\"28\" font-size=\"18\" font-family=\"serif\" fill=\"{}\">A = U Σ V⊤</text>\
{}\
{}\
{}\
<text x=\"16\" y=\"120\" font-size=\"13\" font-family=\"sans-serif\" fill=\"{}\">σ = [{}]</text>",
        th.ink,
        box_svg(40.0, 48.0, 70.0, 50.0, "U", th.box_a, th),
        box_svg(130.0, 48.0, 70.0, 50.0, "Σ", th.box_b, th),
        box_svg(220.0, 48.0, 80.0, 50.0, "V⊤", th.box_a, th),
        th.muted,
        escape(&s)
    );
    root(560.0, 140.0, "svd", th, &body)
}

fn draw_qr(_factor: &Value, th: &Theme) -> String {
    let body = format!(
        "<text x=\"16\" y=\"28\" font-size=\"18\" font-family=\"serif\" fill=\"{}\">A = Q R</text>\
{}\
{}",
        th.ink,
        box_svg(40.0, 48.0, 70.0, 50.0, "Q", th.box_a, th),
        box_svg(130.0, 48.0, 70.0, 50.0, "R", th.box_b, th),
    );
    root(420.0, 120.0, "qr", th, &body)
}

fn draw_lu(_factor: &Value, th: &Theme) -> String {
    let body = format!(
        "<text x=\"16\" y=\"28\" font-size=\"18\" font-family=\"serif\" fill=\"{}\">A = L U</text>\
{}\
{}",
        th.ink,
        box_svg(40.0, 48.0, 70.0, 50.0, "L", th.box_c, th),
        box_svg(130.0, 48.0, 70.0, 50.0, "U", th.box_b, th),
    );
    root(420.0, 120.0, "lu", th, &body)
}

fn draw_chol(_factor: &Value, th: &Theme) -> String {
    let body = format!(
        "<text x=\"16\" y=\"28\" font-size=\"18\" font-family=\"serif\" fill=\"{}\">A = L L⊤</text>\
{}\
{}",
        th.ink,
        box_svg(40.0, 48.0, 70.0, 50.0, "L", th.box_c, th),
        box_svg(130.0, 48.0, 80.0, 50.0, "L⊤", th.box_c, th),
    );
    root(420.0, 120.0, "chol", th, &body)
}

fn draw_structure(factor: &Value, th: &Theme) -> String {
    let label = if let (Some(r), Some(c)) = (factor.get("rows"), factor.get("cols")) {
        format!("{}×{}", r, c)
    } else if let Some(Value::Array(a)) = factor.get("shape") {
        if a.len() >= 2 {
            format!("{}×{}", a[0], a[1])
        } else {
            factor
                .get("kind")
                .and_then(|k| k.as_str())
                .unwrap_or("matrix")
                .to_string()
        }
    } else {
        factor
            .get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or("matrix")
            .to_string()
    };
    let body = box_svg(40.0, 30.0, 160.0, 60.0, &label, th.box_a, th);
    root(280.0, 120.0, "structure", th, &body)
}

fn draw_heatmap(factor: &Value, hinton: bool, th: &Theme) -> String {
    let data = factor
        .get("data")
        .or_else(|| factor.get("eigenvectors").and_then(|v| v.get("data")))
        .and_then(|d| d.as_array());
    let Some(rows) = data else {
        return draw_structure(factor, th);
    };
    let n = rows.len().min(16);
    let m = rows
        .first()
        .and_then(|r| r.as_array())
        .map(|r| r.len())
        .unwrap_or(0)
        .min(16);
    if n == 0 || m == 0 {
        return draw_structure(factor, th);
    }
    let mut max_abs: f64 = 1e-12;
    for row in rows.iter().take(n) {
        if let Some(cells) = row.as_array() {
            for c in cells.iter().take(m) {
                let v = cell_mag(c);
                max_abs = max_abs.max(v);
            }
        }
    }
    let cell = 18.0;
    let w = 40.0 + m as f64 * cell;
    let h = 40.0 + n as f64 * cell;
    let tag = if hinton { "hinton" } else { "heatmap" };
    let mut parts = vec![format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" data-linalg=\"{tag}\" data-theme=\"{}\" style=\"background:{}\">",
        th.name.as_str(),
        th.bg
    )];
    for (i, row) in rows.iter().take(n).enumerate() {
        let Some(cells) = row.as_array() else { continue };
        for (j, c) in cells.iter().take(m).enumerate() {
            let v = cell_signed(c);
            let x = 20.0 + j as f64 * cell;
            let y = 20.0 + i as f64 * cell;
            if hinton {
                let s = (v.abs() / max_abs).sqrt() * (cell - 4.0);
                let fill = if v >= 0.0 { th.heat_pos } else { th.heat_neg };
                parts.push(format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{s}\" height=\"{s}\" fill=\"{fill}\"/>",
                    x + (cell - s) / 2.0,
                    y + (cell - s) / 2.0
                ));
            } else {
                let t = ((v / max_abs) + 1.0) * 0.5;
                let r = (255.0 * (1.0 - t)) as u8;
                let b = (255.0 * t) as u8;
                let fill = if th.name == ThemeName::Bw {
                    let g = (255.0 * (1.0 - v.abs() / max_abs)) as u8;
                    format!("rgb({g},{g},{g})")
                } else {
                    format!("rgb({r},80,{b})")
                };
                parts.push(format!(
                    "<rect x=\"{x}\" y=\"{y}\" width=\"{cell}\" height=\"{cell}\" fill=\"{fill}\" stroke=\"{}\" stroke-width=\"0.5\"/>",
                    th.bg
                ));
            }
        }
    }
    parts.push("</svg>".into());
    parts.join("")
}

fn cell_mag(c: &Value) -> f64 {
    if let Some(v) = c.as_f64() {
        return v.abs();
    }
    if let Some(a) = c.as_array() {
        if a.len() >= 2 {
            let re = a[0].as_f64().unwrap_or(0.0);
            let im = a[1].as_f64().unwrap_or(0.0);
            return (re * re + im * im).sqrt();
        }
    }
    0.0
}

fn cell_signed(c: &Value) -> f64 {
    if let Some(v) = c.as_f64() {
        return v;
    }
    // complex: use real part for signed heatmap, magnitude for abs
    if let Some(a) = c.as_array() {
        if a.len() >= 2 {
            return a[0].as_f64().unwrap_or(0.0);
        }
    }
    0.0
}
