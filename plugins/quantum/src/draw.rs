//! Circuit rail SVG (design §7.4 + Q8 viz style).

use serde_json::Value;

use crate::sim::{parse_ops, C, Op};

const COL_W: f64 = 80.0;
const ROW_H: f64 = 64.0;
const PAD_X: f64 = 24.0;
const PAD_Y: f64 = 36.0;
const LABEL_W: f64 = 52.0;
const GUTTER: f64 = 20.0;
const FONT_MONO: &str = "ui-monospace,Menlo,Consolas,monospace";
const GATE_W: f64 = 40.0;
const GATE_H: f64 = 40.0;

/// Legacy aliases used by older advanced-viz helpers until Q8c.
const BOX: &str = "#f7f4ef";
const STROKE: &str = "#1a1a1a";

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

#[derive(Clone, Copy)]
struct Theme {
    name: ThemeName,
    bg: &'static str,
    bg2: &'static str,
    panel: &'static str,
    wire: &'static str,
    label: &'static str,
    chip: &'static str,
    clifford_fill: &'static str,
    clifford_ink: &'static str,
    phase_fill: &'static str,
    phase_ink: &'static str,
    rotation_fill: &'static str,
    rotation_ink: &'static str,
    measure_fill: &'static str,
    measure_ink: &'static str,
    barrier: &'static str,
    ctrl: &'static str,
    accent: &'static str,
    muted: &'static str,
    axis: &'static str,
    glow: &'static str,
}

impl Theme {
    fn of(name: ThemeName) -> Self {
        match name {
            ThemeName::Dark => Self {
                name,
                bg: "#0a0e14",
                bg2: "#141c28",
                panel: "#1a2433",
                wire: "#6b7f96",
                label: "#e8eef6",
                chip: "#1e2a3a",
                clifford_fill: "#4c6fff",
                clifford_ink: "#f2f6ff",
                phase_fill: "#2de2e6",
                phase_ink: "#042024",
                rotation_fill: "#3dd6c6",
                rotation_ink: "#041a18",
                measure_fill: "#8fa3b8",
                measure_ink: "#0a1018",
                barrier: "#9aafc0",
                ctrl: "#f0f5ff",
                accent: "#5b7cfa",
                muted: "#93a4b8",
                axis: "#7d8fa3",
                glow: "#4c6fff",
            },
            ThemeName::Light => Self {
                name,
                bg: "#eef3f9",
                bg2: "#ffffff",
                panel: "#e2eaf3",
                wire: "#4a5a6a",
                label: "#15202b",
                chip: "#ffffff",
                clifford_fill: "#3d5afe",
                clifford_ink: "#ffffff",
                phase_fill: "#00838f",
                phase_ink: "#ffffff",
                rotation_fill: "#00897b",
                rotation_ink: "#ffffff",
                measure_fill: "#546e7a",
                measure_ink: "#ffffff",
                barrier: "#78909c",
                ctrl: "#1a237e",
                accent: "#3d5afe",
                muted: "#5a6a7a",
                axis: "#4a5560",
                glow: "#3d5afe",
            },
            ThemeName::Bw => Self {
                name,
                bg: "#ffffff",
                bg2: "#f7f7f7",
                panel: "#eeeeee",
                wire: "#222222",
                label: "#111111",
                chip: "#f0f0f0",
                clifford_fill: "#ffffff",
                clifford_ink: "#111111",
                phase_fill: "#eeeeee",
                phase_ink: "#111111",
                rotation_fill: "#f5f5f5",
                rotation_ink: "#111111",
                measure_fill: "#e8e8e8",
                measure_ink: "#111111",
                barrier: "#666666",
                ctrl: "#111111",
                accent: "#333333",
                muted: "#555555",
                axis: "#222222",
                glow: "#888888",
            },
        }
    }

    fn family_colors(self, family: &str) -> (&'static str, &'static str) {
        match family {
            "phase" => (self.phase_fill, self.phase_ink),
            "rotation" => (self.rotation_fill, self.rotation_ink),
            "measure" => (self.measure_fill, self.measure_ink),
            "neutral" => (self.measure_fill, self.measure_ink),
            _ => (self.clifford_fill, self.clifford_ink),
        }
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn gate_family(gate: &str) -> &'static str {
    match gate {
        "I" | "X" | "Y" | "Z" | "H" | "HADAMARD" | "CX" | "CNOT" | "CN" | "SWAP" => "clifford",
        "S" | "T" | "RZ" | "CZ" => "phase",
        "RX" | "RY" => "rotation",
        "MEASURE" | "M" => "measure",
        "BARRIER" => "barrier",
        _ => "neutral",
    }
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

fn rail_x0() -> f64 {
    PAD_X + LABEL_W + GUTTER
}

fn y_of(q: usize) -> f64 {
    PAD_Y + q as f64 * ROW_H
}

fn x_of(col: usize) -> f64 {
    rail_x0() + (col as f64 + 0.5) * COL_W
}

/// Draw a circuit diagram as standalone SVG.
pub fn circuit_svg(circuit: &Value, theme: ThemeName) -> Result<String, String> {
    let th = Theme::of(theme);
    let qubits = circuit
        .get("qubits")
        .or_else(|| circuit.get("比特数"))
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .ok_or_else(|| "circuit missing qubits".to_string())? as usize;
    let ops = parse_ops(circuit)?;
    let cols = ops.len().max(1);
    let width = (rail_x0() + cols as f64 * COL_W + PAD_X).max(320.0);
    let height = PAD_Y * 2.0 + (qubits.saturating_sub(1) as f64) * ROW_H;
    let height = height.max(PAD_Y * 2.0 + GATE_H + 8.0);

    let mut body = String::new();
    body.push_str(&svg_defs(&th));
    body.push_str(&format!(
        r##"<rect width="{w:.1}" height="{h:.1}" rx="14" fill="url(#mq-bg)"/>
<rect x="1.5" y="1.5" width="{wi:.1}" height="{hi:.1}" rx="12.5" fill="none" stroke="{panel}" stroke-width="1" opacity="0.55"/>"##,
        w = width,
        h = height,
        wi = width - 3.0,
        hi = height - 3.0,
        panel = th.panel,
    ));

    let wire_x0 = rail_x0();
    let wire_x1 = width - PAD_X;
    let label_x = PAD_X + LABEL_W - 6.0;
    for q in 0..qubits {
        let y = y_of(q);
        // dual-tone wire for depth
        body.push_str(&format!(
            r##"<line x1="{x0}" y1="{y}" x2="{x1}" y2="{y}" stroke="{wire}" stroke-width="3.2" stroke-linecap="round" opacity="0.28"/>
<line x1="{x0}" y1="{y}" x2="{x1}" y2="{y}" stroke="{wire}" stroke-width="1.8" stroke-linecap="round"/>"##,
            x0 = wire_x0,
            x1 = wire_x1,
            y = y,
            wire = th.wire,
        ));
        // label chip — wire never crosses text
        let chip_w = 38.0;
        let chip_h = 22.0;
        body.push_str(&format!(
            r##"<rect x="{rx}" y="{ry}" width="{chip_w}" height="{chip_h}" rx="7" fill="{chip}" stroke="{panel}" stroke-width="1"/>
<text x="{lx}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font}" font-size="13" font-weight="700" letter-spacing="0.02em" fill="{fill}">q{q}</text>"##,
            rx = label_x - chip_w / 2.0,
            ry = y - chip_h / 2.0,
            chip_w = chip_w,
            chip_h = chip_h,
            chip = th.chip,
            panel = th.panel,
            lx = label_x,
            y = y,
            font = FONT_MONO,
            fill = th.label,
            q = q,
        ));
    }

    for (col, op) in ops.iter().enumerate() {
        let x = x_of(col);
        match op.gate.as_str() {
            "BARRIER" => {
                draw_barrier(&mut body, x, qubits, &th);
            }
            "MEASURE" | "M" => {
                let qs = if op.qubits.is_empty() {
                    (0..qubits).collect::<Vec<_>>()
                } else {
                    op.qubits.clone()
                };
                for q in qs {
                    draw_measure(&mut body, x, q, &th);
                }
            }
            "CX" | "CNOT" | "CN" if op.qubits.len() >= 2 => {
                draw_cx(&mut body, x, op.qubits[0], op.qubits[1], &th);
            }
            "CZ" if op.qubits.len() >= 2 => {
                draw_cz(&mut body, x, op.qubits[0], op.qubits[1], &th);
            }
            "SWAP" if op.qubits.len() >= 2 => {
                draw_swap(&mut body, x, op.qubits[0], op.qubits[1], &th);
            }
            _ => {
                let q = *op.qubits.first().unwrap_or(&0);
                let fam = gate_family(&op.gate);
                draw_box(&mut body, x, q, &gate_label(op), fam, &th);
            }
        }
    }

    Ok(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" data-theme="{theme}" role="img" aria-label="quantum circuit">{body}</svg>"##,
        w = width,
        h = height,
        theme = th.name.as_str(),
        body = body,
    ))
}

fn svg_defs(th: &Theme) -> String {
    format!(
        r##"<defs>
  <linearGradient id="mq-bg" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="{bg}"/>
    <stop offset="55%" stop-color="{bg2}"/>
    <stop offset="100%" stop-color="{bg}"/>
  </linearGradient>
  <filter id="mq-glow" x="-40%" y="-40%" width="180%" height="180%">
    <feGaussianBlur stdDeviation="2.2" result="b"/>
    <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
  </filter>
  <filter id="mq-soft" x="-20%" y="-20%" width="140%" height="140%">
    <feDropShadow dx="0" dy="1.5" stdDeviation="1.8" flood-color="{glow}" flood-opacity="0.35"/>
  </filter>
</defs>"##,
        bg = th.bg,
        bg2 = th.bg2,
        glow = th.glow,
    )
}

fn draw_box(out: &mut String, x: f64, q: usize, label: &str, family: &str, th: &Theme) {
    let y = y_of(q);
    let w = GATE_W;
    let h = GATE_H;
    let (fill, ink) = th.family_colors(family);
    out.push_str(&format!(
        r##"<rect x="{rx}" y="{ry}" width="{w}" height="{h}" rx="8" fill="{fill}" stroke="{ink}" stroke-width="1.1" data-gate-family="{fam}" filter="url(#mq-soft)" opacity="0.98"/>
<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font}" font-size="14" font-weight="700" fill="{ink}">{label}</text>"##,
        rx = x - w / 2.0,
        ry = y - h / 2.0,
        w = w,
        h = h,
        fill = fill,
        ink = ink,
        fam = family,
        x = x,
        y = y,
        font = FONT_MONO,
        label = esc(label),
    ));
}

fn draw_barrier(out: &mut String, x: f64, qubits: usize, th: &Theme) {
    let y0 = y_of(0) - 20.0;
    let y1 = y_of(qubits.saturating_sub(1)) + 20.0;
    out.push_str(&format!(
        r##"<line x1="{x}" y1="{y0}" x2="{x}" y2="{y1}" stroke="{c}" stroke-width="1.75" stroke-dasharray="6 5" data-gate-family="barrier" opacity="0.85"/>"##,
        x = x,
        y0 = y0,
        y1 = y1,
        c = th.barrier,
    ));
}

fn draw_measure(out: &mut String, x: f64, q: usize, th: &Theme) {
    let y = y_of(q);
    let w = GATE_W;
    let h = GATE_H;
    let (fill, ink) = th.family_colors("measure");
    out.push_str(&format!(
        r##"<rect x="{rx}" y="{ry}" width="{w}" height="{h}" rx="8" fill="{fill}" stroke="{ink}" stroke-width="1.1" data-gate-family="measure" filter="url(#mq-soft)"/>
<path d="M {x0},{y0} A 11,11 0 0 1 {x1},{y0}" fill="none" stroke="{ink}" stroke-width="1.8"/>
<line x1="{x}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{ink}" stroke-width="1.8" stroke-linecap="round"/>"##,
        rx = x - w / 2.0,
        ry = y - h / 2.0,
        w = w,
        h = h,
        fill = fill,
        ink = ink,
        x0 = x - 11.0,
        y0 = y + 4.0,
        x1 = x + 11.0,
        x = x,
        y1 = y + 4.0,
        x2 = x + 8.0,
        y2 = y - 8.0,
    ));
}

fn draw_cx(out: &mut String, x: f64, control: usize, target: usize, th: &Theme) {
    let yc = y_of(control);
    let yt = y_of(target);
    let (fill, _) = th.family_colors("clifford");
    let r = 13.0;
    out.push_str(&format!(
        r##"<line x1="{x}" y1="{yc}" x2="{x}" y2="{yt}" stroke="{ctrl}" stroke-width="2.25" data-gate-family="clifford"/>
<circle cx="{x}" cy="{yc}" r="5.2" fill="{ctrl}" filter="url(#mq-glow)"/>
<circle cx="{x}" cy="{yt}" r="{r}" fill="{fill}" stroke="{ctrl}" stroke-width="1.6" filter="url(#mq-soft)" opacity="0.96"/>
<line x1="{x}" y1="{a}" x2="{x}" y2="{b}" stroke="{ctrl}" stroke-width="2"/>
<line x1="{a2}" y1="{yt}" x2="{b2}" y2="{yt}" stroke="{ctrl}" stroke-width="2"/>"##,
        x = x,
        yc = yc,
        yt = yt,
        ctrl = th.ctrl,
        fill = fill,
        r = r,
        a = yt - r,
        b = yt + r,
        a2 = x - r,
        b2 = x + r,
    ));
}

fn draw_cz(out: &mut String, x: f64, control: usize, target: usize, th: &Theme) {
    let yc = y_of(control);
    let yt = y_of(target);
    out.push_str(&format!(
        r##"<line x1="{x}" y1="{yc}" x2="{x}" y2="{yt}" stroke="{c}" stroke-width="2.25" data-gate-family="phase"/>
<circle cx="{x}" cy="{yc}" r="5.2" fill="{c}" filter="url(#mq-glow)"/>
<circle cx="{x}" cy="{yt}" r="5.2" fill="{c}" filter="url(#mq-glow)"/>"##,
        x = x,
        yc = yc,
        yt = yt,
        c = th.phase_fill,
    ));
}

fn draw_swap(out: &mut String, x: f64, a: usize, b: usize, th: &Theme) {
    let ya = y_of(a);
    let yb = y_of(b);
    out.push_str(&format!(
        r##"<line x1="{x}" y1="{ya}" x2="{x}" y2="{yb}" stroke="{c}" stroke-width="2.25" data-gate-family="clifford"/>"##,
        x = x,
        ya = ya,
        yb = yb,
        c = th.ctrl,
    ));
    for y in [ya, yb] {
        out.push_str(&format!(
            r##"<line x1="{x0}" y1="{y0}" x2="{x1}" y2="{y1}" stroke="{c}" stroke-width="2" stroke-linecap="round"/>
<line x1="{x0}" y1="{y1}" x2="{x1}" y2="{y0}" stroke="{c}" stroke-width="2" stroke-linecap="round"/>"##,
            x0 = x - 10.0,
            y0 = y - 10.0,
            x1 = x + 10.0,
            y1 = y + 10.0,
            c = th.ctrl,
        ));
    }
}

/// Probability histogram from basis-label → probability map.
pub fn probs_svg(probs: &serde_json::Map<String, Value>, theme: ThemeName) -> String {
    let th = Theme::of(theme);
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
    let pad_t = 24.0;
    let pad_r = 16.0;
    let width = pad_l + n as f64 * (bar_w + gap) + pad_r;
    let height = pad_t + chart_h + pad_b;
    let mut body = String::new();
    body.push_str(&svg_defs(&th));
    body.push_str(&format!(
        r##"<rect width="{w:.1}" height="{h:.1}" rx="14" fill="url(#mq-bg)"/>"##,
        w = width,
        h = height,
    ));
    body.push_str(&format!(
        r##"<line x1="{x0}" y1="{y}" x2="{x1}" y2="{y}" stroke="{axis}" stroke-width="1.2"/>"##,
        x0 = pad_l - 4.0,
        x1 = width - pad_r,
        y = pad_t + chart_h,
        axis = th.axis,
    ));
    body.push_str(&format!(
        r##"<line x1="{x}" y1="{y0}" x2="{x}" y2="{y1}" stroke="{axis}" stroke-width="1.2"/>"##,
        x = pad_l - 4.0,
        y0 = pad_t,
        y1 = pad_t + chart_h,
        axis = th.axis,
    ));
    body.push_str(&format!(
        r##"<text x="8" y="{ty}" font-family="{font}" font-size="10" fill="{lab}">1</text>"##,
        ty = pad_t + 4.0,
        font = FONT_MONO,
        lab = th.label,
    ));
    body.push_str(&format!(
        r##"<text x="8" y="{ty}" font-family="{font}" font-size="10" fill="{lab}">0</text>"##,
        ty = pad_t + chart_h + 4.0,
        font = FONT_MONO,
        lab = th.label,
    ));
    for (i, (label, p)) in entries.iter().enumerate() {
        let p = (*p).clamp(0.0, 1.0);
        let h = p * chart_h;
        let x = pad_l + i as f64 * (bar_w + gap);
        let y = pad_t + chart_h - h;
        body.push_str(&format!(
            r##"<rect x="{x}" y="{y}" width="{bar_w}" height="{h}" rx="4" fill="{fill}" stroke="{ink}" stroke-width="1" filter="url(#mq-soft)"/>"##,
            x = x,
            y = y,
            bar_w = bar_w,
            h = h.max(0.5),
            fill = th.accent,
            ink = th.ctrl,
        ));
        body.push_str(&format!(
            r##"<text x="{cx}" y="{ty}" text-anchor="middle" font-family="{font}" font-size="11" fill="{lab}">{lab_t}</text>"##,
            cx = x + bar_w / 2.0,
            ty = pad_t + chart_h + 18.0,
            font = FONT_MONO,
            lab = th.label,
            lab_t = esc(label),
        ));
        body.push_str(&format!(
            r##"<text x="{cx}" y="{ty}" text-anchor="middle" font-family="{font}" font-size="10" fill="{muted}">{pval:.2}</text>"##,
            cx = x + bar_w / 2.0,
            ty = y - 5.0,
            font = FONT_MONO,
            muted = th.muted,
            pval = p,
        ));
    }
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" data-theme="{theme}" role="img" aria-label="measurement probabilities">{body}</svg>"##,
        w = width,
        h = height,
        theme = th.name.as_str(),
        body = body,
    )
}

/// 2D Bloch sphere projection with state arrow.
pub fn bloch_svg(x: f64, y: f64, z: f64, theme: ThemeName) -> String {
    let th = Theme::of(theme);
    let cx = 130.0;
    let cy = 125.0;
    let r = 82.0;
    let width = 280.0;
    let height = 290.0;
    // Project: X right, Z up, Y into page (slight oblique).
    let px = |vx: f64, vy: f64, _vz: f64| cx + r * (vx * 0.85 + vy * 0.35);
    let py = |_vx: f64, vy: f64, vz: f64| cy - r * (vz * 0.9 - vy * 0.25);
    let mut body = String::new();
    body.push_str(&svg_defs(&th));
    body.push_str(&format!(
        r##"<rect width="{w:.1}" height="{h:.1}" rx="14" fill="url(#mq-bg)"/>"##,
        w = width,
        h = height,
    ));
    body.push_str(&format!(
        r##"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" stroke="{stroke}" stroke-width="1.5" opacity="0.28" filter="url(#mq-glow)"/>"##,
        cx = cx,
        cy = cy,
        r = r,
        fill = th.clifford_fill,
        stroke = th.wire,
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
            r##"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{axis}" stroke-width="1" stroke-dasharray="3 2"/>"##,
            x1 = x1,
            y1 = y1,
            x2 = x2,
            y2 = y2,
            axis = th.axis,
        ));
        body.push_str(&format!(
            r##"<text x="{tx}" y="{ty}" font-family="{font}" font-size="10" fill="{lab_c}">{lab}</text>"##,
            tx = x2 + 4.0,
            ty = y2 + 3.0,
            font = FONT_MONO,
            lab_c = th.label,
            lab = lab,
        ));
    }
    // state vector
    let sx = px(x, y, z);
    let sy = py(x, y, z);
    let ox = px(0.0, 0.0, 0.0);
    let oy = py(0.0, 0.0, 0.0);
    let arrow = th.phase_fill;
    body.push_str(&format!(
        r##"<line x1="{ox}" y1="{oy}" x2="{sx}" y2="{sy}" stroke="{arrow}" stroke-width="2.5" stroke-linecap="round"/>"##,
        ox = ox,
        oy = oy,
        sx = sx,
        sy = sy,
        arrow = arrow,
    ));
    body.push_str(&format!(
        r##"<circle cx="{sx}" cy="{sy}" r="4.5" fill="{arrow}" stroke="{ctrl}" stroke-width="1"/>"##,
        sx = sx,
        sy = sy,
        arrow = arrow,
        ctrl = th.ctrl,
    ));
    body.push_str(&format!(
        r##"<text x="12" y="{hy}" font-family="{font}" font-size="11" fill="{lab}">⟨X⟩={bx:.3}  ⟨Y⟩={by:.3}  ⟨Z⟩={bz:.3}</text>"##,
        hy = height - 12.0,
        font = FONT_MONO,
        lab = th.label,
        bx = x,
        by = y,
        bz = z,
    ));
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" data-theme="{theme}" role="img" aria-label="Bloch sphere">{body}</svg>"##,
        w = width,
        h = height,
        theme = th.name.as_str(),
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

fn mag(c: C) -> f64 {
    (c.re * c.re + c.im * c.im).sqrt()
}

fn heat_fill(m: f64) -> String {
    // pale → indigo by |amp|
    let t = m.clamp(0.0, 1.0);
    let r = (232.0 + (55.0 - 232.0) * t).round() as i32;
    let g = (236.0 + (66.0 - 236.0) * t).round() as i32;
    let b = (241.0 + (250.0 - 241.0) * t).round() as i32;
    format!("rgb({r},{g},{b})")
}

/// Complex matrix heatmap (magnitude fill + re/im labels for small dims).
pub fn matrix_heatmap_svg(m: &[Vec<C>], title: &str) -> String {
    let n = m.len().max(1);
    let cell = if n <= 2 {
        72.0
    } else if n <= 4 {
        56.0
    } else {
        40.0
    };
    let left = 36.0;
    let top = 28.0;
    let width = left + cell * n as f64 + 16.0;
    let height = top + cell * n as f64 + 20.0;
    let mut body = String::new();
    body.push_str(&format!(
        r#"<text x="8" y="18" font-family="ui-monospace,Menlo,monospace" font-size="12" fill="{STROKE}">{title}</text>"#,
        title = esc(title),
        STROKE = STROKE,
    ));
    for i in 0..n {
        body.push_str(&format!(
            r#"<text x="8" y="{y}" font-family="ui-monospace,Menlo,monospace" font-size="10" fill="{muted}">{i}</text>"#,
            y = top + (i as f64 + 0.55) * cell,
            i = i,
            muted = "#666",
        ));
        body.push_str(&format!(
            r#"<text x="{x}" y="{y}" text-anchor="middle" font-family="ui-monospace,Menlo,monospace" font-size="10" fill="{muted}">{i}</text>"#,
            x = left + (i as f64 + 0.5) * cell,
            y = top - 6.0,
            i = i,
            muted = "#666",
        ));
        for j in 0..n {
            let c = m
                .get(i)
                .and_then(|row| row.get(j))
                .copied()
                .unwrap_or(C { re: 0.0, im: 0.0 });
            let x = left + j as f64 * cell;
            let y = top + i as f64 * cell;
            let fill = heat_fill(mag(c));
            body.push_str(&format!(
                r#"<rect x="{x}" y="{y}" width="{cell}" height="{cell}" fill="{fill}" stroke="{STROKE}" stroke-width="0.8"/>"#,
                x = x,
                y = y,
                cell = cell,
                fill = fill,
                STROKE = STROKE,
            ));
            if n <= 4 {
                let label = if c.im.abs() < 1e-9 {
                    format!("{:.3}", c.re)
                } else if c.re.abs() < 1e-9 {
                    format!("{:.3}i", c.im)
                } else {
                    format!("{:.2}{:+.2}i", c.re, c.im)
                };
                let ink = if mag(c) > 0.55 { "#fff" } else { STROKE };
                body.push_str(&format!(
                    r#"<text x="{tx}" y="{ty}" text-anchor="middle" font-family="ui-monospace,Menlo,monospace" font-size="9" fill="{ink}">{lab}</text>"#,
                    tx = x + cell * 0.5,
                    ty = y + cell * 0.55,
                    lab = esc(&label),
                    ink = ink,
                ));
            }
        }
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" role="img" aria-label="gate matrix heatmap">{body}</svg>"#,
        w = width,
        h = height,
        body = body,
    )
}

fn phase_hue(arg: f64) -> String {
    // map [-π,π] → hue degrees
    let h = ((arg + std::f64::consts::PI) / (2.0 * std::f64::consts::PI) * 360.0) % 360.0;
    format!("hsl({h:.0},70%,45%)")
}

/// Hinton diagram for density matrix (Q7b).
pub fn hinton_svg(m: &[Vec<C>], title: &str) -> String {
    let n = m.len().max(1);
    let cell = if n <= 2 { 56.0 } else if n <= 4 { 44.0 } else { 32.0 };
    let left = 40.0;
    let top = 28.0;
    let width = left + cell * n as f64 + 16.0;
    let height = top + cell * n as f64 + 24.0;
    let mut max_a: f64 = 1e-12;
    for row in m {
        for c in row {
            max_a = max_a.max(mag(*c));
        }
    }
    let mut body = String::new();
    body.push_str(&format!(
        r#"<text x="8" y="18" font-family="ui-monospace,Menlo,monospace" font-size="12" fill="{STROKE}">{title}</text>"#,
        title = esc(title),
        STROKE = STROKE,
    ));
    body.push_str(&format!(
        r##"<rect x="{left}" y="{top}" width="{w}" height="{h}" fill="#f0eee8" stroke="{STROKE}" stroke-width="1"/>"##,
        left = left,
        top = top,
        w = cell * n as f64,
        h = cell * n as f64,
        STROKE = STROKE,
    ));
    for i in 0..n {
        for j in 0..n {
            let c = m.get(i).and_then(|r| r.get(j)).copied().unwrap_or(C::zero());
            let a = mag(c) / max_a;
            let side = (cell * 0.85 * a.sqrt()).max(1.0);
            let cx = left + (j as f64 + 0.5) * cell;
            let cy = top + (i as f64 + 0.5) * cell;
            let fill = if c.re >= 0.0 { "#f5f5f5" } else { "#1a1a1a" };
            let stroke = if c.re >= 0.0 { STROKE } else { "#f5f5f5" };
            body.push_str(&format!(
                r#"<rect x="{x}" y="{y}" width="{side}" height="{side}" fill="{fill}" stroke="{stroke}" stroke-width="0.8" data-hinton="1"/>"#,
                x = cx - side * 0.5,
                y = cy - side * 0.5,
                side = side,
                fill = fill,
                stroke = stroke,
            ));
        }
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" role="img" aria-label="hinton">{body}</svg>"#,
        w = width,
        h = height,
        body = body,
    )
}

/// Cirq-inspired density: magnitude disk + phase needle; diagonal probability bars.
pub fn density_cells_svg(m: &[Vec<C>], title: &str) -> String {
    let n = m.len().max(1);
    let cell = if n <= 2 { 64.0 } else if n <= 4 { 48.0 } else { 36.0 };
    let left = 36.0;
    let top = 28.0;
    let width = left + cell * n as f64 + 16.0;
    let height = top + cell * n as f64 + 20.0;
    let mut max_a: f64 = 1e-12;
    for row in m {
        for c in row {
            max_a = max_a.max(mag(*c));
        }
    }
    let mut body = String::new();
    body.push_str(&format!(
        r#"<text x="8" y="18" font-family="ui-monospace,Menlo,monospace" font-size="12" fill="{STROKE}">{title}</text>"#,
        title = esc(title),
        STROKE = STROKE,
    ));
    for i in 0..n {
        for j in 0..n {
            let c = m.get(i).and_then(|r| r.get(j)).copied().unwrap_or(C::zero());
            let x0 = left + j as f64 * cell;
            let y0 = top + i as f64 * cell;
            body.push_str(&format!(
                r##"<rect x="{x0}" y="{y0}" width="{cell}" height="{cell}" fill="#eeeeee" stroke="#ccc" stroke-width="0.5"/>"##,
                x0 = x0,
                y0 = y0,
                cell = cell,
            ));
            let cx = x0 + cell * 0.5;
            let cy = y0 + cell * 0.5;
            if i == n - 1 - j || i == j {
                // soft diagonal highlight for probability (i==j)
            }
            if i == j {
                let p = c.re.clamp(0.0, 1.0);
                let bw = cell * 0.7 * p;
                body.push_str(&format!(
                    r##"<rect x="{x}" y="{y}" width="{bw}" height="6" fill="#3a6ea5" data-density-diag="1"/>"##,
                    x = cx - cell * 0.35,
                    y = y0 + cell - 10.0,
                    bw = bw,
                ));
            }
            let r = cell * 0.35 * (mag(c) / max_a).sqrt();
            let arg = c.im.atan2(c.re);
            body.push_str(&format!(
                r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" fill-opacity="0.35" stroke="{fill}" stroke-width="1.2" data-density="1"/>"#,
                cx = cx,
                cy = cy,
                r = r.max(0.5),
                fill = phase_hue(arg),
            ));
            let nx = cx + r * arg.cos();
            let ny = cy - r * arg.sin();
            body.push_str(&format!(
                r#"<line x1="{cx}" y1="{cy}" x2="{nx}" y2="{ny}" stroke="{STROKE}" stroke-width="1.2"/>"#,
                cx = cx,
                cy = cy,
                nx = nx,
                ny = ny,
                STROKE = STROKE,
            ));
        }
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" role="img" aria-label="density">{body}</svg>"#,
        w = width,
        h = height,
        body = body,
    )
}

/// 2.5D cityscape: Re (left) and Im (right) bar grids.
pub fn city_svg(m: &[Vec<C>], title: &str) -> String {
    let n = m.len().max(1);
    let cell = 28.0;
    let left = 40.0;
    let top = 40.0;
    let gap = 24.0;
    let grid_w = cell * n as f64;
    let width = left + grid_w * 2.0 + gap + 24.0;
    let max_h = 80.0;
    let mut max_re: f64 = 1e-12;
    let mut max_im: f64 = 1e-12;
    for row in m {
        for c in row {
            max_re = max_re.max(c.re.abs());
            max_im = max_im.max(c.im.abs());
        }
    }
    let height = top + max_h + cell * n as f64 * 0.35 + 40.0;
    let mut body = String::new();
    body.push_str(&format!(
        r#"<text x="8" y="18" font-family="ui-monospace,Menlo,monospace" font-size="12" fill="{STROKE}">{title} (city)</text>"#,
        title = esc(title),
        STROKE = STROKE,
    ));
    body.push_str(r##"<text x="40" y="34" font-family="ui-monospace,Menlo,monospace" font-size="10" fill="#666">Re</text>"##);
    body.push_str(&format!(
        r##"<text x="{x}" y="34" font-family="ui-monospace,Menlo,monospace" font-size="10" fill="#666">Im</text>"##,
        x = left + grid_w + gap,
    ));
    let draw_city = |body: &mut String, origin_x: f64, use_re: bool| {
        for i in 0..n {
            for j in 0..n {
                let c = m.get(i).and_then(|r| r.get(j)).copied().unwrap_or(C::zero());
                let val = if use_re { c.re } else { c.im };
                let denom = if use_re { max_re } else { max_im };
                let h = (val.abs() / denom) * max_h;
                let iso_x = origin_x + (j as f64 - i as f64) * cell * 0.5;
                let iso_y = top + (i as f64 + j as f64) * cell * 0.28 + (max_h - h);
                let fill = if use_re { "#3a6ea5" } else { "#c45c26" };
                body.push_str(&format!(
                    r#"<rect x="{iso_x}" y="{iso_y}" width="{w}" height="{h}" fill="{fill}" fill-opacity="0.85" stroke="{STROKE}" stroke-width="0.4" data-city="1"/>"#,
                    iso_x = iso_x,
                    iso_y = iso_y,
                    w = cell * 0.45,
                    h = h.max(0.5),
                    fill = fill,
                    STROKE = STROKE,
                ));
            }
        }
    };
    draw_city(&mut body, left + grid_w * 0.35, true);
    draw_city(&mut body, left + grid_w + gap + grid_w * 0.35, false);
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" role="img" aria-label="city">{body}</svg>"#,
        w = width,
        h = height,
        body = body,
    )
}

/// Pauli expectation bar chart.
pub fn paulivec_svg(labels: &[String], values: &[f64], title: &str) -> String {
    let n = labels.len().max(1);
    let bar_w = 28.0;
    let gap = 8.0;
    let left = 48.0;
    let top = 28.0;
    let chart_h = 120.0;
    let width = left + n as f64 * (bar_w + gap) + 16.0;
    let height = top + chart_h + 40.0;
    let mid_y = top + chart_h * 0.5;
    let mut body = String::new();
    body.push_str(&format!(
        r#"<text x="8" y="18" font-family="ui-monospace,Menlo,monospace" font-size="12" fill="{STROKE}">{title}</text>"#,
        title = esc(title),
        STROKE = STROKE,
    ));
    body.push_str(&format!(
        r##"<line x1="{left}" y1="{mid_y}" x2="{x2}" y2="{mid_y}" stroke="#888" stroke-width="1"/>"##,
        left = left,
        mid_y = mid_y,
        x2 = width - 8.0,
    ));
    for (i, (lab, val)) in labels.iter().zip(values.iter()).enumerate() {
        let x = left + i as f64 * (bar_w + gap);
        let h = val.abs().clamp(0.0, 1.0) * (chart_h * 0.45);
        let y = if *val >= 0.0 { mid_y - h } else { mid_y };
        body.push_str(&format!(
            r##"<rect x="{x}" y="{y}" width="{bar_w}" height="{h}" fill="#2f6f4e" data-paulivec="1"/>"##,
            x = x,
            y = y,
            bar_w = bar_w,
            h = h.max(0.5),
        ));
        body.push_str(&format!(
            r#"<text x="{tx}" y="{ty}" text-anchor="middle" font-family="ui-monospace,Menlo,monospace" font-size="9" fill="{STROKE}">{lab}</text>"#,
            tx = x + bar_w * 0.5,
            ty = top + chart_h + 14.0,
            lab = esc(lab),
            STROKE = STROKE,
        ));
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" role="img" aria-label="paulivec">{body}</svg>"#,
        w = width,
        h = height,
        body = body,
    )
}

/// QSphere-style: amplitudes on a circle (phase→angle, amp→radius of marker).
pub fn qsphere_svg(amps: &[C], qubits: usize) -> String {
    let dim = amps.len();
    let cx = 160.0;
    let cy = 160.0;
    let r = 110.0;
    let width = 320.0;
    let height = 320.0;
    let mut body = String::new();
    body.push_str(&format!(
        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{STROKE}" stroke-width="1.5"/>"#,
        cx = cx,
        cy = cy,
        r = r,
        STROKE = STROKE,
    ));
    body.push_str(r##"<text x="12" y="20" font-family="ui-monospace,Menlo,monospace" font-size="12" fill="#1a1a1a">qsphere</text>"##);
    for (i, a) in amps.iter().enumerate() {
        let amp = mag(*a);
        if amp < 1e-9 {
            continue;
        }
        // latitude from Hamming weight, longitude from index
        let wt = (i as u32).count_ones() as f64;
        let lat = if qubits == 0 {
            0.0
        } else {
            std::f64::consts::PI * (wt / qubits as f64 - 0.5)
        };
        let lon = 2.0 * std::f64::consts::PI * (i as f64) / (dim as f64);
        let rr = r * lat.cos();
        let x = cx + rr * lon.cos();
        let y = cy - r * lat.sin();
        let rad = 4.0 + 14.0 * amp;
        let arg = a.im.atan2(a.re);
        body.push_str(&format!(
            r#"<circle cx="{x}" cy="{y}" r="{rad}" fill="{fill}" stroke="{STROKE}" stroke-width="1" data-qsphere="1"/>"#,
            x = x,
            y = y,
            rad = rad,
            fill = phase_hue(arg),
            STROKE = STROKE,
        ));
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" role="img" aria-label="qsphere">{body}</svg>"#,
        w = width,
        h = height,
        body = body,
    )
}

/// Side-by-side Bloch spheres for each qubit.
pub fn multibloch_svg(amps: &[C], qubits: usize) -> Result<String, String> {
    let sphere_w = 140.0;
    let width = 16.0 + qubits as f64 * sphere_w;
    let height = 160.0;
    let mut body = String::new();
    body.push_str(r##"<text x="8" y="16" font-family="ui-monospace,Menlo,monospace" font-size="12" fill="#1a1a1a">multibloch</text>"##);
    for q in 0..qubits {
        let (x, y, z) = crate::sim::bloch_vector(amps, qubits, q)?;
        // Mini sphere (Q8c will share Theme tokens fully).
        let ox = 8.0 + q as f64 * sphere_w;
        let cx = ox + 60.0;
        let cy = 90.0;
        let r = 40.0;
        body.push_str(&format!(
            r##"<text x="{ox}" y="32" font-family="ui-monospace,Menlo,monospace" font-size="11" fill="#1a1a1a">q{q}</text>"##,
            ox = ox + 8.0,
            q = q,
        ));
        body.push_str(&format!(
            r##"<ellipse cx="{cx}" cy="{cy}" rx="{r}" ry="{ry}" fill="#f7f4ef" stroke="{STROKE}" stroke-width="1.2"/>"##,
            cx = cx,
            cy = cy,
            r = r,
            ry = r * 0.55,
            STROKE = STROKE,
        ));
        body.push_str(&format!(
            r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{STROKE}" stroke-width="1.2"/>"#,
            cx = cx,
            cy = cy,
            r = r,
            STROKE = STROKE,
        ));
        let px = cx + x * r;
        let py = cy - z * r * 0.85;
        body.push_str(&format!(
            r##"<line x1="{cx}" y1="{cy}" x2="{px}" y2="{py}" stroke="#c45c26" stroke-width="2" data-multibloch="1"/>"##,
            cx = cx,
            cy = cy,
            px = px,
            py = py,
        ));
        let _ = y;
    }
    Ok(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.1} {h:.1}" role="img" aria-label="multibloch">{body}</svg>"#,
        w = width,
        h = height,
        body = body,
    ))
}
