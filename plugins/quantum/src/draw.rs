//! Circuit rail SVG (design §7.4).

use serde_json::Value;

use crate::sim::{parse_ops, Op};

const COL_W: f64 = 56.0;
const ROW_H: f64 = 40.0;
const PAD_X: f64 = 48.0;
const PAD_Y: f64 = 24.0;
const WIRE: &str = "#1a1a1a";
const BOX: &str = "#f7f4ef";
const STROKE: &str = "#1a1a1a";

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn gate_label(op: &Op) -> String {
    match op.gate.as_str() {
        "RX" => op
            .theta
            .map(|t| format!("Rx({t:.2})"))
            .unwrap_or_else(|| "Rx".into()),
        "RY" => op
            .theta
            .map(|t| format!("Ry({t:.2})"))
            .unwrap_or_else(|| "Ry".into()),
        "RZ" => op
            .theta
            .map(|t| format!("Rz({t:.2})"))
            .unwrap_or_else(|| "Rz".into()),
        "HADAMARD" => "H".into(),
        "CNOT" | "CN" => "CX".into(),
        other => other.to_string(),
    }
}

fn y_of(q: usize) -> f64 {
    PAD_Y + q as f64 * ROW_H
}

fn x_of(col: usize) -> f64 {
    PAD_X + col as f64 * COL_W
}

/// Draw a circuit diagram as standalone SVG.
pub fn circuit_svg(circuit: &Value) -> Result<String, String> {
    let qubits = circuit
        .get("qubits")
        .or_else(|| circuit.get("比特数"))
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .ok_or_else(|| "circuit missing qubits".to_string())? as usize;
    let ops = parse_ops(circuit)?;
    let cols = ops.len().max(1);
    let width = PAD_X + cols as f64 * COL_W + 32.0;
    let height = PAD_Y * 2.0 + (qubits.saturating_sub(1).max(0) as f64) * ROW_H + 8.0;
    let height = height.max(PAD_Y * 2.0 + 24.0);

    let mut body = String::new();
    // wires
    for q in 0..qubits {
        let y = y_of(q);
        body.push_str(&format!(
            r#"<line x1="{x0}" y1="{y}" x2="{x1}" y2="{y}" stroke="{WIRE}" stroke-width="1.5"/>"#,
            x0 = 8.0,
            x1 = width - 8.0,
            y = y,
        ));
        body.push_str(&format!(
            r#"<text x="4" y="{ty}" font-family="ui-monospace,Menlo,monospace" font-size="11" fill="{WIRE}">q{q}</text>"#,
            ty = y + 4.0,
        ));
    }

    for (col, op) in ops.iter().enumerate() {
        let x = x_of(col);
        match op.gate.as_str() {
            "CX" | "CNOT" | "CN" if op.qubits.len() >= 2 => {
                draw_cx(&mut body, x, op.qubits[0], op.qubits[1]);
            }
            "CZ" if op.qubits.len() >= 2 => {
                draw_cz(&mut body, x, op.qubits[0], op.qubits[1]);
            }
            "SWAP" if op.qubits.len() >= 2 => {
                draw_swap(&mut body, x, op.qubits[0], op.qubits[1]);
            }
            _ => {
                let q = *op.qubits.first().unwrap_or(&0);
                draw_box(&mut body, x, q, &gate_label(op));
            }
        }
    }

    Ok(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" role="img" aria-label="quantum circuit">{body}</svg>"#,
        w = width,
        h = height,
        body = body,
    ))
}

fn draw_box(out: &mut String, x: f64, q: usize, label: &str) {
    let y = y_of(q);
    let w = 28.0;
    let h = 28.0;
    out.push_str(&format!(
        r#"<rect x="{rx}" y="{ry}" width="{w}" height="{h}" rx="3" fill="{BOX}" stroke="{STROKE}" stroke-width="1.5"/>"#,
        rx = x - w / 2.0,
        ry = y - h / 2.0,
    ));
    out.push_str(&format!(
        r#"<text x="{x}" y="{ty}" text-anchor="middle" font-family="ui-monospace,Menlo,monospace" font-size="11" fill="{STROKE}">{label}</text>"#,
        ty = y + 4.0,
        label = esc(label),
    ));
}

fn draw_cx(out: &mut String, x: f64, control: usize, target: usize) {
    let yc = y_of(control);
    let yt = y_of(target);
    out.push_str(&format!(
        r#"<line x1="{x}" y1="{yc}" x2="{x}" y2="{yt}" stroke="{STROKE}" stroke-width="1.5"/>"#
    ));
    out.push_str(&format!(
        r#"<circle cx="{x}" cy="{yc}" r="4" fill="{STROKE}"/>"#
    ));
    out.push_str(&format!(
        r#"<circle cx="{x}" cy="{yt}" r="10" fill="none" stroke="{STROKE}" stroke-width="1.5"/>"#
    ));
    out.push_str(&format!(
        r#"<line x1="{x}" y1="{a}" x2="{x}" y2="{b}" stroke="{STROKE}" stroke-width="1.5"/>"#,
        a = yt - 10.0,
        b = yt + 10.0,
    ));
    out.push_str(&format!(
        r#"<line x1="{a}" y1="{yt}" x2="{b}" y2="{yt}" stroke="{STROKE}" stroke-width="1.5"/>"#,
        a = x - 10.0,
        b = x + 10.0,
    ));
}

fn draw_cz(out: &mut String, x: f64, control: usize, target: usize) {
    let yc = y_of(control);
    let yt = y_of(target);
    out.push_str(&format!(
        r#"<line x1="{x}" y1="{yc}" x2="{x}" y2="{yt}" stroke="{STROKE}" stroke-width="1.5"/>"#
    ));
    out.push_str(&format!(
        r#"<circle cx="{x}" cy="{yc}" r="4" fill="{STROKE}"/>"#
    ));
    out.push_str(&format!(
        r#"<circle cx="{x}" cy="{yt}" r="4" fill="{STROKE}"/>"#
    ));
}

fn draw_swap(out: &mut String, x: f64, a: usize, b: usize) {
    let ya = y_of(a);
    let yb = y_of(b);
    out.push_str(&format!(
        r#"<line x1="{x}" y1="{ya}" x2="{x}" y2="{yb}" stroke="{STROKE}" stroke-width="1.5"/>"#
    ));
    for y in [ya, yb] {
        out.push_str(&format!(
            r#"<line x1="{x0}" y1="{y0}" x2="{x1}" y2="{y1}" stroke="{STROKE}" stroke-width="1.5"/>"#,
            x0 = x - 8.0,
            y0 = y - 8.0,
            x1 = x + 8.0,
            y1 = y + 8.0,
        ));
        out.push_str(&format!(
            r#"<line x1="{x0}" y1="{y0}" x2="{x1}" y2="{y1}" stroke="{STROKE}" stroke-width="1.5"/>"#,
            x0 = x - 8.0,
            y0 = y + 8.0,
            x1 = x + 8.0,
            y1 = y - 8.0,
        ));
    }
}

/// Probability histogram from basis-label → probability map.
pub fn probs_svg(probs: &serde_json::Map<String, Value>) -> String {
    let mut entries: Vec<(String, f64)> = probs
        .iter()
        .filter_map(|(k, v)| v.as_f64().map(|p| (k.clone(), p)))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    if entries.is_empty() {
        entries.push(("∅".into(), 0.0));
    }
    let n = entries.len().max(1);
    let bar_w = 36.0;
    let gap = 12.0;
    let chart_h = 120.0;
    let pad_l = 40.0;
    let pad_b = 36.0;
    let pad_t = 20.0;
    let pad_r = 16.0;
    let width = pad_l + n as f64 * (bar_w + gap) + pad_r;
    let height = pad_t + chart_h + pad_b;
    let mut body = String::new();
    // axis
    body.push_str(&format!(
        r#"<line x1="{x0}" y1="{y}" x2="{x1}" y2="{y}" stroke="{STROKE}" stroke-width="1.2"/>"#,
        x0 = pad_l - 4.0,
        x1 = width - pad_r,
        y = pad_t + chart_h,
    ));
    body.push_str(&format!(
        r#"<line x1="{x}" y1="{y0}" x2="{x}" y2="{y1}" stroke="{STROKE}" stroke-width="1.2"/>"#,
        x = pad_l - 4.0,
        y0 = pad_t,
        y1 = pad_t + chart_h,
    ));
    body.push_str(&format!(
        r#"<text x="6" y="{ty}" font-family="ui-monospace,Menlo,monospace" font-size="10" fill="{STROKE}">1</text>"#,
        ty = pad_t + 4.0,
    ));
    body.push_str(&format!(
        r#"<text x="6" y="{ty}" font-family="ui-monospace,Menlo,monospace" font-size="10" fill="{STROKE}">0</text>"#,
        ty = pad_t + chart_h + 4.0,
    ));
    for (i, (label, p)) in entries.iter().enumerate() {
        let p = (*p).clamp(0.0, 1.0);
        let h = p * chart_h;
        let x = pad_l + i as f64 * (bar_w + gap);
        let y = pad_t + chart_h - h;
        let bar_fill = "#4a6fa5";
        let muted = "#555";
        body.push_str(&format!(
            r#"<rect x="{x}" y="{y}" width="{bar_w}" height="{h}" fill="{bar_fill}" stroke="{STROKE}" stroke-width="1"/>"#,
            x = x,
            y = y,
            bar_w = bar_w,
            h = h,
            bar_fill = bar_fill,
            STROKE = STROKE,
        ));
        body.push_str(&format!(
            r#"<text x="{cx}" y="{ty}" text-anchor="middle" font-family="ui-monospace,Menlo,monospace" font-size="10" fill="{STROKE}">{lab}</text>"#,
            cx = x + bar_w / 2.0,
            ty = pad_t + chart_h + 16.0,
            lab = esc(label),
            STROKE = STROKE,
        ));
        body.push_str(&format!(
            r#"<text x="{cx}" y="{ty}" text-anchor="middle" font-family="ui-monospace,Menlo,monospace" font-size="9" fill="{muted}">{pval:.2}</text>"#,
            cx = x + bar_w / 2.0,
            ty = y - 4.0,
            pval = p,
            muted = muted,
        ));
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" role="img" aria-label="measurement probabilities">{body}</svg>"#,
        w = width,
        h = height,
        body = body,
    )
}

/// 2D Bloch sphere projection with state arrow.
pub fn bloch_svg(x: f64, y: f64, z: f64) -> String {
    let cx = 110.0;
    let cy = 110.0;
    let r = 70.0;
    let width = 220.0;
    let height = 240.0;
    // Project: X right, Z up, Y into page (slight oblique).
    let px = |vx: f64, vy: f64, _vz: f64| cx + r * (vx * 0.85 + vy * 0.35);
    let py = |_vx: f64, vy: f64, vz: f64| cy - r * (vz * 0.9 - vy * 0.25);
    let mut body = String::new();
    body.push_str(&format!(
        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" stroke="{STROKE}" stroke-width="1.5"/>"#,
        cx = cx,
        cy = cy,
        r = r,
        fill = BOX,
        STROKE = STROKE,
    ));
    // axes
    let axes = [
        ("+X", 1.0, 0.0, 0.0),
        ("-X", -1.0, 0.0, 0.0),
        ("+Y", 0.0, 1.0, 0.0),
        ("+Z", 0.0, 0.0, 1.0),
        ("-Z", 0.0, 0.0, -1.0),
    ];
    for (lab, vx, vy, vz) in axes {
        let x1 = px(0.0, 0.0, 0.0);
        let y1 = py(0.0, 0.0, 0.0);
        let x2 = px(vx, vy, vz);
        let y2 = py(vx, vy, vz);
        body.push_str(&format!(
            r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{axis}" stroke-width="1" stroke-dasharray="3 2"/>"#,
            x1 = x1,
            y1 = y1,
            x2 = x2,
            y2 = y2,
            axis = "#888",
        ));
        body.push_str(&format!(
            r#"<text x="{tx}" y="{ty}" font-family="ui-monospace,Menlo,monospace" font-size="10" fill="{STROKE}">{lab}</text>"#,
            tx = x2 + 4.0,
            ty = y2 + 3.0,
            lab = lab,
            STROKE = STROKE,
        ));
    }
    // state vector
    let sx = px(x, y, z);
    let sy = py(x, y, z);
    let ox = px(0.0, 0.0, 0.0);
    let oy = py(0.0, 0.0, 0.0);
    let arrow = "#c0392b";
    body.push_str(&format!(
        r#"<line x1="{ox}" y1="{oy}" x2="{sx}" y2="{sy}" stroke="{arrow}" stroke-width="2.5"/>"#,
        ox = ox,
        oy = oy,
        sx = sx,
        sy = sy,
        arrow = arrow,
    ));
    body.push_str(&format!(
        r#"<circle cx="{sx}" cy="{sy}" r="4" fill="{arrow}"/>"#,
        sx = sx,
        sy = sy,
        arrow = arrow,
    ));
    body.push_str(&format!(
        r#"<text x="12" y="{hy}" font-family="ui-monospace,Menlo,monospace" font-size="11" fill="{STROKE}">⟨X⟩={bx:.3}  ⟨Y⟩={by:.3}  ⟨Z⟩={bz:.3}</text>"#,
        hy = height - 12.0,
        bx = x,
        by = y,
        bz = z,
        STROKE = STROKE,
    ));
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" role="img" aria-label="Bloch sphere">{body}</svg>"#,
        w = width,
        h = height,
        body = body,
    )
}

/// Minimal single-gate glyph for `# gate`.draw.
pub fn gate_svg(name: &str) -> String {
    let label = esc(name);
    let width = 64.0;
    let height = 48.0;
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" role="img" aria-label="gate {label}"><rect x="12" y="8" width="40" height="32" rx="4" fill="{BOX}" stroke="{STROKE}" stroke-width="1.5"/><text x="32" y="29" text-anchor="middle" font-family="ui-monospace,Menlo,monospace" font-size="14" fill="{STROKE}">{label}</text></svg>"#,
        w = width,
        h = height,
        label = label,
        BOX = BOX,
        STROKE = STROKE,
    )
}
