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
