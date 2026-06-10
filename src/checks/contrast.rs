use crate::checks::{CheckResult, Status};
use crate::color;
use crate::svg::Svg;
use usvg::{Group, Node, Paint};

pub const MIN_CONTRAST: f32 = 3.0;

pub fn check(svg: &Svg) -> CheckResult {
    let size = svg.tree.size();
    let canvas_area = size.width() * size.height();
    let bg = background_color(svg.tree.root(), canvas_area).unwrap_or((255, 255, 255));
    let bg_lum = color::relative_luminance(bg.0, bg.1, bg.2);

    let mut strokes = Vec::new();
    collect_strokes(svg.tree.root(), &mut strokes);

    let mut low = 0usize;
    let mut min_ratio = f32::INFINITY;
    for (r, g, b) in strokes {
        let ratio = color::contrast_ratio(color::relative_luminance(r, g, b), bg_lum);
        if ratio < MIN_CONTRAST {
            low += 1;
        }
        if ratio < min_ratio {
            min_ratio = ratio;
        }
    }

    let status = if low == 0 { Status::Pass } else { Status::Fail };
    let detail = if min_ratio.is_finite() {
        format!("{low} low-contrast stroke(s); min ratio {min_ratio:.2} (limit {MIN_CONTRAST})")
    } else {
        "no solid strokes".to_string()
    };
    CheckResult {
        id: "contrast",
        layer: 1,
        status,
        detail,
    }
}

fn rgb(p: &Paint) -> Option<(u8, u8, u8)> {
    match p {
        Paint::Color(c) => Some((c.red, c.green, c.blue)),
        _ => None,
    }
}

fn collect_strokes(g: &Group, out: &mut Vec<(u8, u8, u8)>) {
    for n in g.children() {
        match n {
            Node::Group(gg) => collect_strokes(gg, out),
            Node::Path(p) => {
                if let Some(c) = p.stroke().and_then(|s| rgb(s.paint())) {
                    out.push(c);
                }
            }
            _ => {}
        }
    }
}

fn background_color(g: &Group, canvas_area: f32) -> Option<(u8, u8, u8)> {
    for n in g.children() {
        match n {
            Node::Group(gg) => {
                if let Some(c) = background_color(gg, canvas_area) {
                    return Some(c);
                }
            }
            Node::Path(p) => {
                let b = p.abs_bounding_box();
                if b.width() * b.height() >= 0.95 * canvas_area {
                    if let Some(c) = p.fill().and_then(|f| rgb(f.paint())) {
                        return Some(c);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    #[test]
    fn d2_clean_passes() {
        let svg = load(Path::new("tests/fixtures/d2-clean.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn low_contrast_fails() {
        let svg = load(Path::new("tests/fixtures/low-contrast.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "contrast");
    }
}
