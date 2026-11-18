//! SVG plot helpers for high-school graphs.

use std::collections::HashMap;

use super::{eval_f64, Expr};

const MAX_STEPS: usize = 2000;
const DEFAULT_STEPS: usize = 200;
const SVG_W: f64 = 520.0;
const SVG_H: f64 = 360.0;
const PAD: f64 = 36.0;

#[derive(Debug, Clone, Copy)]
pub struct PlotStyle {
    pub grid: bool,
}

impl Default for PlotStyle {
    fn default() -> Self {
        Self { grid: true }
    }
}

pub fn plot_function_svg(
    expr: &Expr,
    var: &str,
    xmin: f64,
    xmax: f64,
    steps: usize,
    style: PlotStyle,
) -> Result<String, String> {
    if !(xmin.is_finite() && xmax.is_finite()) || xmax <= xmin {
        return Err("plot needs finite min < max".into());
    }
    let steps = steps.clamp(2, MAX_STEPS);
    let mut env = HashMap::new();
    let mut pts: Vec<(f64, f64)> = Vec::with_capacity(steps + 1);
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;
    for i in 0..=steps {
        let x = xmin + (xmax - xmin) * (i as f64) / (steps as f64);
        env.insert(var.to_string(), x);
        match eval_f64(expr, &env) {
            Ok(y) if y.is_finite() => {
                ymin = ymin.min(y);
                ymax = ymax.max(y);
                pts.push((x, y));
            }
            _ => {}
        }
    }
    if pts.is_empty() {
        return Err("plot produced no finite points".into());
    }
    if (ymax - ymin).abs() < 1e-12 {
        ymin -= 1.0;
        ymax += 1.0;
    }
    let pad_y = 0.05 * (ymax - ymin);
    ymin -= pad_y;
    ymax += pad_y;

    let mut path = String::new();
    for (i, (x, y)) in pts.iter().enumerate() {
        let (sx, sy) = map_xy(*x, *y, xmin, xmax, ymin, ymax);
        if i == 0 {
            path.push_str(&format!("M {sx:.2} {sy:.2}"));
        } else {
            path.push_str(&format!(" L {sx:.2} {sy:.2}"));
        }
    }

    let frame = draw_frame(xmin, xmax, ymin, ymax, style);
    Ok(svg_wrap(
        &format!(r#"{frame}<path d="{path}" class="curve"/>"#),
        "function plot",
    ))
}

pub fn plot_points_svg(xs: &[f64], ys: &[f64], style: PlotStyle) -> Result<String, String> {
    if xs.len() != ys.len() {
        return Err("plot_points: xs and ys length mismatch".into());
    }
    if xs.is_empty() {
        return Err("plot_points: empty series".into());
    }
    let xmin = xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let xmax = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let ymin = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let ymax = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let (xmin, xmax) = pad_range(xmin, xmax);
    let (ymin, ymax) = pad_range(ymin, ymax);
    let mut dots = String::new();
    for (x, y) in xs.iter().zip(ys.iter()) {
        if !x.is_finite() || !y.is_finite() {
            continue;
        }
        let (sx, sy) = map_xy(*x, *y, xmin, xmax, ymin, ymax);
        dots.push_str(&format!(
            r#"<circle cx="{sx:.2}" cy="{sy:.2}" r="3" class="pt"/>"#
        ));
    }
    let frame = draw_frame(xmin, xmax, ymin, ymax, style);
    Ok(svg_wrap(&format!("{frame}{dots}"), "points plot"))
}

/// Standard conics via parametric sampling.
/// kind: circle | ellipse | hyperbola | parabola
pub fn plot_conic_svg(
    kind: &str,
    a: f64,
    b: f64,
    h: f64,
    k: f64,
    style: PlotStyle,
) -> Result<String, String> {
    let steps = DEFAULT_STEPS;
    let mut pts: Vec<(f64, f64)> = Vec::new();
    match kind {
        "circle" => {
            let r = if a > 0.0 { a } else { b };
            if r <= 0.0 {
                return Err("circle needs positive radius (a)".into());
            }
            for i in 0..=steps {
                let t = std::f64::consts::TAU * (i as f64) / (steps as f64);
                pts.push((h + r * t.cos(), k + r * t.sin()));
            }
        }
        "ellipse" => {
            if a <= 0.0 || b <= 0.0 {
                return Err("ellipse needs a>0, b>0".into());
            }
            for i in 0..=steps {
                let t = std::f64::consts::TAU * (i as f64) / (steps as f64);
                pts.push((h + a * t.cos(), k + b * t.sin()));
            }
        }
        "hyperbola" => {
            if a <= 0.0 || b <= 0.0 {
                return Err("hyperbola needs a>0, b>0".into());
            }
            for branch in [-1.0_f64, 1.0] {
                for i in 0..=steps {
                    let t = -2.0 + 4.0 * (i as f64) / (steps as f64);
                    pts.push((h + branch * a * t.cosh(), k + b * t.sinh()));
                }
            }
            let mid = pts.len() / 2;
            let (left, right) = pts.split_at(mid);
            let xmin = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
            let xmax = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
            let ymin = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
            let ymax = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
            let (xmin, xmax) = pad_range(xmin, xmax);
            let (ymin, ymax) = pad_range(ymin, ymax);
            let p1 = polyline_path(left, xmin, xmax, ymin, ymax);
            let p2 = polyline_path(right, xmin, xmax, ymin, ymax);
            let frame = draw_frame(xmin, xmax, ymin, ymax, style);
            return Ok(svg_wrap(
                &format!(
                    r#"{frame}<path d="{p1}" class="curve"/><path d="{p2}" class="curve"/>"#
                ),
                "hyperbola",
            ));
        }
        "parabola" => {
            let scale = if a.abs() > 1e-12 { a } else { 1.0 };
            for i in 0..=steps {
                let t = -3.0 + 6.0 * (i as f64) / (steps as f64);
                let x = h + t;
                let y = k + (t * t) / (4.0 * scale.abs()) * scale.signum();
                pts.push((x, y));
            }
        }
        other => return Err(format!("unknown conic kind `{other}`")),
    }

    let xmin = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let xmax = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let ymin = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let ymax = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    let (xmin, xmax) = pad_range(xmin, xmax);
    let (ymin, ymax) = pad_range(ymin, ymax);
    let path = polyline_path(&pts, xmin, xmax, ymin, ymax);
    let frame = draw_frame(xmin, xmax, ymin, ymax, style);
    Ok(svg_wrap(
        &format!(r#"{frame}<path d="{path}" class="curve"/>"#),
        kind,
    ))
}

fn pad_range(lo: f64, hi: f64) -> (f64, f64) {
    if (hi - lo).abs() < 1e-12 {
        (lo - 1.0, hi + 1.0)
    } else {
        let pad = 0.05 * (hi - lo);
        (lo - pad, hi + pad)
    }
}

fn polyline_path(pts: &[(f64, f64)], xmin: f64, xmax: f64, ymin: f64, ymax: f64) -> String {
    let mut path = String::new();
    for (i, (x, y)) in pts.iter().enumerate() {
        let (sx, sy) = map_xy(*x, *y, xmin, xmax, ymin, ymax);
        if i == 0 {
            path.push_str(&format!("M {sx:.2} {sy:.2}"));
        } else {
            path.push_str(&format!(" L {sx:.2} {sy:.2}"));
        }
    }
    path
}

fn map_xy(x: f64, y: f64, xmin: f64, xmax: f64, ymin: f64, ymax: f64) -> (f64, f64) {
    let w = SVG_W - 2.0 * PAD;
    let h = SVG_H - 2.0 * PAD;
    let sx = PAD + (x - xmin) / (xmax - xmin) * w;
    let sy = PAD + (ymax - y) / (ymax - ymin) * h;
    (sx, sy)
}

fn nice_step(span: f64, target: usize) -> f64 {
    let target = target.max(2) as f64;
    let raw = (span / target).abs().max(1e-12);
    let exp = raw.log10().floor();
    let mag = 10f64.powf(exp);
    let norm = raw / mag;
    let nice = if norm <= 1.5 {
        1.0
    } else if norm <= 3.0 {
        2.0
    } else if norm <= 7.0 {
        5.0
    } else {
        10.0
    };
    nice * mag
}

fn tick_values(lo: f64, hi: f64, step: f64) -> Vec<f64> {
    if step <= 0.0 || !step.is_finite() {
        return Vec::new();
    }
    let start = (lo / step).ceil() * step;
    let mut out = Vec::new();
    let mut v = start;
    let mut guard = 0;
    while v <= hi + step * 1e-9 && guard < 200 {
        if v >= lo - step * 1e-9 && v <= hi + step * 1e-9 {
            if v.abs() < step * 1e-9 {
                out.push(0.0);
            } else {
                out.push(v);
            }
        }
        v += step;
        guard += 1;
    }
    out
}

fn draw_frame(xmin: f64, xmax: f64, ymin: f64, ymax: f64, style: PlotStyle) -> String {
    let left = PAD;
    let right = SVG_W - PAD;
    let top = PAD;
    let bottom = SVG_H - PAD;

    let (ox, _) = map_xy(0.0, 0.0, xmin, xmax, ymin, ymax);
    let (_, oy) = map_xy(0.0, 0.0, xmin, xmax, ymin, ymax);
    let axis_x_y = if ymin <= 0.0 && ymax >= 0.0 {
        oy.clamp(top, bottom)
    } else if ymax < 0.0 {
        top
    } else {
        bottom
    };
    let axis_y_x = if xmin <= 0.0 && xmax >= 0.0 {
        ox.clamp(left, right)
    } else if xmax < 0.0 {
        right
    } else {
        left
    };

    let mut s = String::new();
    let x_step = nice_step(xmax - xmin, 8);
    let y_step = nice_step(ymax - ymin, 6);

    if style.grid {
        for x in tick_values(xmin, xmax, x_step) {
            let (sx, _) = map_xy(x, 0.0, xmin, xmax, ymin, ymax);
            if sx < left - 0.5 || sx > right + 0.5 {
                continue;
            }
            s.push_str(&format!(
                r#"<line x1="{sx:.2}" y1="{top}" x2="{sx:.2}" y2="{bottom}" class="grid"/>"#
            ));
        }
        for y in tick_values(ymin, ymax, y_step) {
            let (_, sy) = map_xy(0.0, y, xmin, xmax, ymin, ymax);
            if sy < top - 0.5 || sy > bottom + 0.5 {
                continue;
            }
            s.push_str(&format!(
                r#"<line x1="{left}" y1="{sy:.2}" x2="{right}" y2="{sy:.2}" class="grid"/>"#
            ));
        }
    }

    // Axes with arrowheads (positive direction).
    s.push_str(&format!(
        r#"<line x1="{left}" y1="{axis_x_y:.2}" x2="{right}" y2="{axis_x_y:.2}" class="axis" marker-end="url(#arrow)"/>"#
    ));
    s.push_str(&format!(
        r#"<line x1="{axis_y_x:.2}" y1="{bottom}" x2="{axis_y_x:.2}" y2="{top}" class="axis" marker-end="url(#arrow)"/>"#
    ));

    // Axis end labels
    s.push_str(&format!(
        r#"<text x="{x:.2}" y="{y:.2}" class="axis-label">x</text>"#,
        x = right + 4.0,
        y = axis_x_y + 4.0,
    ));
    s.push_str(&format!(
        r#"<text x="{x:.2}" y="{y:.2}" class="axis-label">y</text>"#,
        x = axis_y_x + 6.0,
        y = top + 4.0,
    ));

    // Tick labels along axes (skip 0 clutter when both axes cross).
    for x in tick_values(xmin, xmax, x_step) {
        if x.abs() < x_step * 1e-9 {
            continue;
        }
        let (sx, _) = map_xy(x, 0.0, xmin, xmax, ymin, ymax);
        if sx < left || sx > right {
            continue;
        }
        s.push_str(&format!(
            r#"<line x1="{sx:.2}" y1="{y1:.2}" x2="{sx:.2}" y2="{y2:.2}" class="tick"/>"#,
            y1 = axis_x_y - 4.0,
            y2 = axis_x_y + 4.0,
        ));
        s.push_str(&format!(
            r#"<text x="{sx:.2}" y="{ty:.2}" text-anchor="middle" class="tick-label">{label}</text>"#,
            ty = (axis_x_y + 14.0).min(bottom + 14.0),
            label = super::format_num(x),
        ));
    }
    for y in tick_values(ymin, ymax, y_step) {
        if y.abs() < y_step * 1e-9 {
            continue;
        }
        let (_, sy) = map_xy(0.0, y, xmin, xmax, ymin, ymax);
        if sy < top || sy > bottom {
            continue;
        }
        s.push_str(&format!(
            r#"<line x1="{x1:.2}" y1="{sy:.2}" x2="{x2:.2}" y2="{sy:.2}" class="tick"/>"#,
            x1 = axis_y_x - 4.0,
            x2 = axis_y_x + 4.0,
        ));
        s.push_str(&format!(
            r#"<text x="{tx:.2}" y="{sy:.2}" text-anchor="end" dominant-baseline="middle" class="tick-label">{label}</text>"#,
            tx = (axis_y_x - 8.0).max(4.0),
            label = super::format_num(y),
        ));
    }

    // Origin mark when visible
    if xmin <= 0.0 && xmax >= 0.0 && ymin <= 0.0 && ymax >= 0.0 {
        s.push_str(&format!(
            r#"<text x="{x:.2}" y="{y:.2}" class="tick-label">0</text>"#,
            x = axis_y_x - 8.0,
            y = axis_x_y + 14.0,
        ));
    }

    s
}

fn svg_wrap(body: &str, title: &str) -> String {
    // r## so fill="#…" does not terminate the raw string at "#
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img" aria-label="{title}">
<defs>
  <marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
    <path d="M 0 0 L 10 5 L 0 10 z" fill="#4b5563"/>
  </marker>
</defs>
<style>
.grid {{ stroke: #e5e7eb; stroke-width: 1; }}
.axis {{ stroke: #4b5563; stroke-width: 1.5; }}
.tick {{ stroke: #4b5563; stroke-width: 1; }}
.curve {{ fill: none; stroke: #1f6feb; stroke-width: 2; }}
.pt {{ fill: #1f6feb; }}
.axis-label {{ font: 12px ui-sans-serif, system-ui, sans-serif; fill: #374151; font-weight: 600; }}
.tick-label {{ font: 10px ui-sans-serif, system-ui, sans-serif; fill: #6b7280; }}
</style>
{body}
</svg>"##,
        w = SVG_W,
        h = SVG_H,
        title = title,
        body = body,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::parse;

    #[test]
    fn plot_has_arrows_and_grid_by_default() {
        let e = parse("x^2").unwrap();
        let svg = plot_function_svg(&e, "x", -2.0, 2.0, 40, PlotStyle::default()).unwrap();
        assert!(svg.contains("marker-end=\"url(#arrow)\""));
        assert!(svg.contains("class=\"grid\""));
        assert!(svg.contains(">x</text>"));
        assert!(svg.contains(">y</text>"));
    }

    #[test]
    fn plot_can_disable_grid() {
        let e = parse("x").unwrap();
        let svg = plot_function_svg(&e, "x", -1.0, 1.0, 20, PlotStyle { grid: false }).unwrap();
        assert!(!svg.contains("class=\"grid\""));
        assert!(svg.contains("marker-end=\"url(#arrow)\""));
    }
}
