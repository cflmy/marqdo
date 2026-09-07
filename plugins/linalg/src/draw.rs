//! Structure SVG for factorizations (teaching layouts).

use serde_json::Value;

/// Draw a factor object or matrix as structure SVG.
pub fn draw_factor(factor: &Value, kind: Option<&str>, path: Option<&str>) -> Result<String, String> {
    let kind = kind
        .or_else(|| factor.get("kind").and_then(|k| k.as_str()))
        .unwrap_or("structure");
    let svg = match kind {
        "eig" => draw_eig(factor),
        "svd" => draw_svd(factor),
        "qr" => draw_qr(factor),
        "lu" | "ge" => draw_lu(factor),
        "chol" => draw_chol(factor),
        "structure" => draw_structure(factor),
        "heatmap" | "hinton" => draw_heatmap(factor, kind == "hinton"),
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

fn box_svg(x: f64, y: f64, w: f64, h: f64, label: &str, fill: &str) -> String {
    format!(
        "<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"{fill}\" stroke=\"#333\" stroke-width=\"1.5\" rx=\"4\"/>\
<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"16\" font-family=\"serif\">{}</text>",
        x + w / 2.0,
        y + h / 2.0,
        escape(label)
    )
}

fn draw_eig(factor: &Value) -> String {
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
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"520\" height=\"140\" data-linalg=\"eig\">\
<text x=\"16\" y=\"28\" font-size=\"18\" font-family=\"serif\">{}</text>\
{}\
{}\
{}\
<text x=\"16\" y=\"120\" font-size=\"13\" font-family=\"sans-serif\" fill=\"#444\">λ = [{}]</text>\
</svg>",
        escape(latex),
        box_svg(40.0, 48.0, 70.0, 50.0, "P", "#e8f1ff"),
        box_svg(130.0, 48.0, 70.0, 50.0, "D", "#fff3d6"),
        box_svg(220.0, 48.0, 90.0, 50.0, "P⁻¹", "#e8f1ff"),
        escape(&evals)
    )
}

fn draw_svd(factor: &Value) -> String {
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
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"560\" height=\"140\" data-linalg=\"svd\">\
<text x=\"16\" y=\"28\" font-size=\"18\" font-family=\"serif\">A = U Σ V⊤</text>\
{}\
{}\
{}\
<text x=\"16\" y=\"120\" font-size=\"13\" font-family=\"sans-serif\" fill=\"#444\">σ = [{}]</text>\
</svg>",
        box_svg(40.0, 48.0, 70.0, 50.0, "U", "#e8f1ff"),
        box_svg(130.0, 48.0, 70.0, 50.0, "Σ", "#fff3d6"),
        box_svg(220.0, 48.0, 80.0, 50.0, "V⊤", "#e8f1ff"),
        escape(&s)
    )
}

fn draw_qr(factor: &Value) -> String {
    let _ = factor;
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"420\" height=\"120\" data-linalg=\"qr\">\
<text x=\"16\" y=\"28\" font-size=\"18\" font-family=\"serif\">A = Q R</text>\
{}\
{}\
</svg>",
        box_svg(40.0, 48.0, 70.0, 50.0, "Q", "#e8f1ff"),
        box_svg(130.0, 48.0, 70.0, 50.0, "R", "#ffe8e8"),
    )
}

fn draw_lu(factor: &Value) -> String {
    let _ = factor;
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"420\" height=\"120\" data-linalg=\"lu\">\
<text x=\"16\" y=\"28\" font-size=\"18\" font-family=\"serif\">A = L U</text>\
{}\
{}\
</svg>",
        box_svg(40.0, 48.0, 70.0, 50.0, "L", "#e8ffe8"),
        box_svg(130.0, 48.0, 70.0, 50.0, "U", "#ffe8e8"),
    )
}

fn draw_chol(factor: &Value) -> String {
    let _ = factor;
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"420\" height=\"120\" data-linalg=\"chol\">\
<text x=\"16\" y=\"28\" font-size=\"18\" font-family=\"serif\">A = L L⊤</text>\
{}\
{}\
</svg>",
        box_svg(40.0, 48.0, 70.0, 50.0, "L", "#e8ffe8"),
        box_svg(130.0, 48.0, 80.0, 50.0, "L⊤", "#e8ffe8"),
    )
}

fn draw_structure(factor: &Value) -> String {
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
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"280\" height=\"120\" data-linalg=\"structure\">\
{}\
</svg>",
        box_svg(40.0, 30.0, 160.0, 60.0, &label, "#f4f4f4")
    )
}

fn draw_heatmap(factor: &Value, hinton: bool) -> String {
    let data = factor
        .get("data")
        .or_else(|| {
            factor
                .get("eigenvectors")
                .and_then(|v| v.get("data"))
        })
        .and_then(|d| d.as_array());
    let Some(rows) = data else {
        return draw_structure(factor);
    };
    let n = rows.len().min(16);
    let m = rows
        .first()
        .and_then(|r| r.as_array())
        .map(|r| r.len())
        .unwrap_or(0)
        .min(16);
    if n == 0 || m == 0 {
        return draw_structure(factor);
    }
    let mut max_abs: f64 = 1e-12;
    for row in rows.iter().take(n) {
        if let Some(cells) = row.as_array() {
            for c in cells.iter().take(m) {
                if let Some(v) = c.as_f64() {
                    max_abs = max_abs.max(v.abs());
                }
            }
        }
    }
    let cell = 18.0;
    let w = 40.0 + m as f64 * cell;
    let h = 40.0 + n as f64 * cell;
    let mut parts = vec![format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" data-linalg=\"{}\">",
        if hinton { "hinton" } else { "heatmap" }
    )];
    for (i, row) in rows.iter().take(n).enumerate() {
        let Some(cells) = row.as_array() else { continue };
        for (j, c) in cells.iter().take(m).enumerate() {
            let v = c.as_f64().unwrap_or(0.0);
            let x = 20.0 + j as f64 * cell;
            let y = 20.0 + i as f64 * cell;
            if hinton {
                let s = (v.abs() / max_abs).sqrt() * (cell - 4.0);
                let fill = if v >= 0.0 { "#2266aa" } else { "#cc3333" };
                parts.push(format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{s}\" height=\"{s}\" fill=\"{fill}\"/>",
                    x + (cell - s) / 2.0,
                    y + (cell - s) / 2.0
                ));
            } else {
                let t = ((v / max_abs) + 1.0) * 0.5;
                let r = (255.0 * (1.0 - t)) as u8;
                let b = (255.0 * t) as u8;
                parts.push(format!(
                    "<rect x=\"{x}\" y=\"{y}\" width=\"{cell}\" height=\"{cell}\" fill=\"rgb({r},80,{b})\" stroke=\"#fff\" stroke-width=\"0.5\"/>"
                ));
            }
        }
    }
    parts.push("</svg>".into());
    parts.join("")
}
